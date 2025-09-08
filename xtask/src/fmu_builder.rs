use anyhow::{bail, Context, Result};
use cargo_metadata::{MetadataCommand, Package};
use chrono::Utc;
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use fmi::fmi3::{binding, schema};

use crate::extractor;
use crate::platform::PlatformMapping;

/// Builder for creating FMU packages
pub struct FmuBuilder {
    package: Package,
    workspace_metadata: cargo_metadata::Metadata,
    model_identifier: String,
    output_dir: PathBuf,
    release: bool,
    platform_mapping: PlatformMapping,
}

impl FmuBuilder {
    pub fn new_for_package(
        package_name: String,
        model_identifier: String,
        output_dir: PathBuf,
        release: bool,
    ) -> Result<Self> {
        // Get the workspace metadata
        let workspace_root = std::env::current_dir()?;
        let workspace_metadata = MetadataCommand::new()
            .current_dir(&workspace_root)
            .exec()
            .context("Failed to execute cargo metadata")?;

        // Find the package
        let package = workspace_metadata
            .packages
            .iter()
            .find(|p| p.name.as_str() == package_name)
            .ok_or_else(|| anyhow::anyhow!("Package '{}' not found in workspace", package_name))?
            .clone();

        // Check if this package can be built as a cdylib
        Self::validate_package_for_fmu(&package)?;

        Ok(Self {
            package,
            workspace_metadata,
            model_identifier,
            output_dir,
            release,
            platform_mapping: PlatformMapping::new(),
        })
    }

    /// Convert an absolute path to a relative path from the current working directory
    fn to_relative_path(&self, path: &Path) -> String {
        std::env::current_dir()
            .ok()
            .and_then(|cwd| path.strip_prefix(&cwd).ok())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string())
    }

    /// Validate that a package can be built as an FMU
    fn validate_package_for_fmu(package: &Package) -> Result<()> {
        let cargo_toml_path = package.manifest_path.parent().unwrap().join("Cargo.toml");
        let cargo_toml_content = std::fs::read_to_string(&cargo_toml_path)?;

        // Check if the package has lib crate-type = ["cdylib"]
        if cargo_toml_content.contains("crate-type") && cargo_toml_content.contains("cdylib") {
            return Ok(());
        }

        bail!(
            "Package '{}' is not configured to build as a dynamic library (cdylib). \
            Add 'crate-type = [\"cdylib\"]' to [lib] section in Cargo.toml",
            package.name
        );
    }

    /// Get the cargo target directory using cargo metadata
    fn get_target_directory(&self) -> Result<PathBuf> {
        let metadata = MetadataCommand::new()
            .current_dir(&self.package.manifest_path.parent().unwrap())
            .no_deps()
            .exec()
            .context("Failed to execute cargo metadata")?;

        Ok(metadata.target_directory.into_std_path_buf())
    }

    /// Build FMU for the native platform
    pub fn build_native(&self) -> Result<()> {
        let output = Command::new("rustc")
            .args(["-vV"])
            .output()
            .context("Failed to get rustc info")?;

        let rustc_info = String::from_utf8(output.stdout)?;
        let host_target = rustc_info
            .lines()
            .find(|line| line.starts_with("host: "))
            .and_then(|line| line.strip_prefix("host: "))
            .context("Could not determine host target")?;

        self.build_for_target(host_target)
    }

    /// Build FMU for a specific target platform
    pub fn build_for_target(&self, target: &str) -> Result<()> {
        info!("Building FMU for target: {}", target);

        // Build the dynamic library
        let dylib_path = self.build_dylib(target)?;

        // Create FMU package
        let fmu_path = self
            .output_dir
            .join(format!("{}.fmu", self.model_identifier));
        self.create_fmu_package(&fmu_path, &[(target, dylib_path)])?;

        info!("Created FMU: {}", fmu_path.display());
        Ok(())
    }

    /// Build the dynamic library for a specific target
    fn build_dylib(&self, target: &str) -> Result<PathBuf> {
        debug!("Building dylib for target: {}", target);

        let package_dir = self.package.manifest_path.parent().unwrap();
        let mut cmd = Command::new("cargo");
        cmd.current_dir(&package_dir)
            .args(["build", "--lib"])
            .args(["--target", target]);

        if self.release {
            cmd.arg("--release");
        }

        debug!("Executing cargo build command: {:?}", cmd);
        info!("Running cargo build for target: {}", target);

        let status = cmd.status().context("Failed to execute cargo build")?;

        if !status.success() {
            bail!("Cargo build failed with exit code: {:?}", status.code());
        }

        // Get the actual target directory from cargo metadata
        let target_dir = self
            .get_target_directory()
            .context("Failed to get target directory from cargo metadata")?;

        let profile = if self.release { "release" } else { "debug" };
        let extension = self.platform_mapping.get_library_extension(target);

        let lib_name = if target.contains("windows") {
            format!(
                "{}.{}",
                self.package.name.as_str().replace('-', "_"),
                extension
            )
        } else {
            format!(
                "lib{}.{}",
                self.package.name.as_str().replace('-', "_"),
                extension
            )
        };

        // Construct the path using the target directory from cargo metadata
        let dylib_path = target_dir.join(target).join(profile).join(&lib_name);

        debug!(
            "Looking for dylib at: {}",
            self.to_relative_path(&dylib_path)
        );
        if !dylib_path.exists() {
            bail!(
                "Built library not found at: {}",
                self.to_relative_path(&dylib_path)
            );
        }

        info!(
            "Successfully built dylib: {}",
            self.to_relative_path(&dylib_path)
        );
        Ok(dylib_path)
    }

    /// Create the FMU ZIP package
    fn create_fmu_package(&self, fmu_path: &Path, dylib_paths: &[(&str, PathBuf)]) -> Result<()> {
        debug!("Creating FMU package at: {}", fmu_path.display());

        // Ensure output directory exists
        if let Some(parent) = fmu_path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Create temporary directory for FMU structure
        let temp_dir = TempDir::new()?;
        let fmu_temp = temp_dir.path();

        debug!("Using temporary directory: {}", fmu_temp.display());

        // Create FMU directory structure
        fs::create_dir_all(fmu_temp.join("binaries"))?;
        fs::create_dir_all(fmu_temp.join("sources"))?;
        fs::create_dir_all(fmu_temp.join("documentation"))?;

        // Copy binaries for each platform
        for (target, dylib_path) in dylib_paths {
            if let Some(fmi_platform) = self.platform_mapping.get_fmi_platform(target) {
                let platform_dir = fmu_temp.join("binaries").join(fmi_platform);
                fs::create_dir_all(&platform_dir)?;

                let extension = self.platform_mapping.get_library_extension(target);
                let lib_filename = format!("{}.{}", self.model_identifier, extension);
                let dest_path = platform_dir.join(&lib_filename);

                fs::copy(dylib_path, &dest_path).with_context(|| {
                    format!(
                        "Failed to copy {} to {}",
                        dylib_path.display(),
                        dest_path.display()
                    )
                })?;

                debug!("Copied binary: {}", dest_path.display());
            } else {
                warn!("Unknown FMI platform mapping for target: {}", target);
            }
        }

        // Generate model description from dylib and package metadata
        let first_dylib = &dylib_paths[0].1;
        self.create_model_description(&fmu_temp.join("modelDescription.xml"), first_dylib)?;

        // Add buildDescription.xml and README.md to root
        self.add_build_description(&fmu_temp)?;
        self.add_readme(&fmu_temp)?;

        // Add sources and documentation
        self.add_source_files(&fmu_temp.join("sources"))?;

        // Create ZIP file
        self.create_zip(fmu_temp, fmu_path)?;

        Ok(())
    }

    /// Parse FMU metadata from package.metadata.fmu and convert to DefaultExperiment
    fn parse_fmu_metadata(&self) -> Result<Option<schema::DefaultExperiment>> {
        // Check if the package has FMU metadata
        if let Some(fmu_metadata_value) = self.package.metadata.get("fmu") {
            debug!("Found FMU metadata in package: {:?}", fmu_metadata_value);

            // Look for default_experiment in the metadata
            if let Some(default_exp_value) = fmu_metadata_value.get("default_experiment") {
                debug!(
                    "Parsing DefaultExperiment from metadata: {:?}",
                    default_exp_value
                );

                // Parse directly into DefaultExperiment using serde
                let experiment: schema::DefaultExperiment = serde_json::from_value(default_exp_value.clone())
                    .context("Failed to parse default_experiment metadata. Please check the format of your Cargo.toml [package.metadata.fmu.default_experiment] section.")?;

                info!("Successfully parsed DefaultExperiment: start_time={:?}, stop_time={:?}, tolerance={:?}, step_size={:?}",
                      experiment.start_time, experiment.stop_time, experiment.tolerance, experiment.step_size);

                return Ok(Some(experiment));
            }
        }

        debug!("No FMU metadata found in package");
        Ok(None)
    }

    /// Create the modelDescription.xml file from package metadata and extracted dylib data
    fn create_model_description(&self, xml_path: &Path, dylib_path: &Path) -> Result<()> {
        info!(
            "Creating model description from package metadata and dylib: {}",
            self.to_relative_path(dylib_path)
        );

        // Extract model variables, structure, and instantiation token from dylib
        let model_data = extractor::extract_model_data(dylib_path)
            .context("Failed to extract model data from dylib")?;

        // Parse FMU metadata for DefaultExperiment and other configurations
        let default_experiment = self
            .parse_fmu_metadata()
            .context("Failed to parse FMU metadata from Cargo.toml")?;

        let fmi_version = unsafe { std::ffi::CStr::from_ptr(binding::fmi3Version.as_ptr() as _) }
            .to_string_lossy();

        // Build model description from package metadata
        let model_description = schema::Fmi3ModelDescription {
            fmi_version: fmi_version.to_string(),
            model_name: self.package.name.as_str().to_string(),
            instantiation_token: model_data.instantiation_token,
            description: self.package.description.clone(),
            author: self.package.authors.first().map(|s| s.to_string()),
            version: Some(self.package.version.to_string()),
            generation_tool: Some("rust-fmi xtask".to_string()),
            generation_date_and_time: Some(Utc::now().to_rfc3339()),
            // Set the extracted model variables and structure
            model_variables: model_data.model_variables,
            model_structure: model_data.model_structure,
            // Set the DefaultExperiment from metadata if present
            default_experiment,
            // For now we only support model-exchange
            model_exchange: Some(schema::Fmi3ModelExchange {
                common: schema::Fmi3InterfaceType {
                    model_identifier: self.model_identifier.clone(),
                    can_get_and_set_fmu_state: Some(false),
                    can_serialize_fmu_state: Some(false),
                    ..Default::default()
                },
                ..Default::default()
            }),
            ..Default::default()
        };

        // Serialize to XML
        let xml_content = fmi::schema::serialize(&model_description, false)
            .context("Failed to serialize model description to XML")?;

        fs::write(xml_path, xml_content)?;
        debug!("Model description written to: {}", xml_path.display());

        Ok(())
    }

    /// Add buildDescription.xml to the root of the FMU
    fn add_build_description(&self, fmu_root: &Path) -> Result<()> {
        debug!("Adding buildDescription.xml to: {}", fmu_root.display());

        // Create build description
        let build_description = schema::Fmi3BuildDescription {
            fmi_version: "3.0".to_string(),
            build_configurations: vec![schema::BuildConfiguration {
                model_identifier: self.model_identifier.clone(),
                platform: Some("generic".to_string()),
                description: Some(format!(
                    "Build configuration for {} FMU generated from Rust",
                    self.model_identifier
                )),
                source_file_sets: vec![schema::SourceFileSet {
                    name: Some("documentation".to_string()),
                    language: Some("Markdown".to_string()),
                    source_files: vec![schema::SourceFile {
                        name: "README.md".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        // Serialize the build description to XML
        let build_description_xml = fmi::schema::serialize(&build_description, false)
            .context("Failed to serialize build description to XML")?;

        fs::write(fmu_root.join("buildDescription.xml"), build_description_xml)?;

        debug!("buildDescription.xml added successfully");
        Ok(())
    }

    /// Add README.md to the root of the FMU
    fn add_readme(&self, fmu_root: &Path) -> Result<()> {
        debug!("Adding README.md to: {}", fmu_root.display());

        let readme_content = format!(
            r#"# {} FMU

This FMU (Functional Mock-up Unit) was generated from Rust code using the rust-fmi library.

## Model Information
- Model Identifier: {}
- FMI Version: 3.0
- Generated: {}

## Description
This FMU implements a functional mock-up unit for use in simulation environments
supporting the FMI (Functional Mock-up Interface) standard.

## Usage
Load this FMU in any FMI-compatible simulation environment to use the model.

## Technical Details
- Generated using rust-fmi
- Implements FMI 3.0 standard
- Platform: Cross-platform compatible
"#,
            self.model_identifier,
            self.model_identifier,
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );

        fs::write(fmu_root.join("README.md"), readme_content)?;

        debug!("README.md added successfully");
        Ok(())
    }

    /// Add source files to the FMU
    fn add_source_files(&self, sources_dir: &Path) -> Result<()> {
        debug!("Adding source files to: {}", sources_dir.display());

        // Note: buildDescription.xml and README.md are now added to the root of the FMU
        // instead of the sources directory

        debug!("Source files added successfully");
        Ok(())
    }

    /// Create the final ZIP file
    fn create_zip(&self, source_dir: &Path, zip_path: &Path) -> Result<()> {
        let file = fs::File::create(zip_path)?;
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        self.add_dir_to_zip(&mut zip, source_dir, source_dir, &options)?;

        zip.finish()?;
        Ok(())
    }

    /// Recursively add directory contents to ZIP
    fn add_dir_to_zip<W: std::io::Write + std::io::Seek>(
        &self,
        zip: &mut ZipWriter<W>,
        dir: &Path,
        base_dir: &Path,
        options: &SimpleFileOptions,
    ) -> Result<()> {
        debug!("Adding directory to ZIP: {}", dir.display());

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            let relative_path = path.strip_prefix(base_dir)?;

            if path.is_dir() {
                // Add directory
                let dir_name = format!("{}/", relative_path.to_string_lossy());
                debug!("Adding directory: {}", dir_name);
                zip.add_directory(dir_name, *options)?;

                // Recursively add directory contents
                self.add_dir_to_zip(zip, &path, base_dir, options)?;
            } else {
                // Add file
                let file_name = relative_path.to_string_lossy().to_string();
                debug!("Adding file: {}", file_name);
                zip.start_file(file_name, *options)?;

                let file_contents = fs::read(&path)?;
                std::io::Write::write_all(&mut *zip, &file_contents)?;
            }
        }

        Ok(())
    }
}
