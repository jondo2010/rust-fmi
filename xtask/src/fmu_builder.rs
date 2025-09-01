use anyhow::{bail, Context, Result};
use cargo_metadata::MetadataCommand;
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use fmi::fmi3::schema::{BuildConfiguration, Fmi3BuildDescription, SourceFile, SourceFileSet};

use crate::model_description_extractor;
use crate::platform::PlatformMapping;

/// Builder for creating FMU packages
pub struct FmuBuilder {
    crate_path: PathBuf,
    target_name: String,
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
        // Find the workspace root and package directory
        let workspace_root = std::env::current_dir()?;
        let package_path = Self::find_package_path(&workspace_root, &package_name)?;

        // Check if this package can be built as a cdylib
        Self::validate_package_for_fmu(&package_path)?;

        Ok(Self {
            crate_path: package_path,
            target_name: package_name.clone(),
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
    fn validate_package_for_fmu(package_path: &Path) -> Result<()> {
        let cargo_toml_path = package_path.join("Cargo.toml");
        if !cargo_toml_path.exists() {
            bail!("No Cargo.toml found at {}", cargo_toml_path.display());
        }

        let cargo_toml_content = std::fs::read_to_string(&cargo_toml_path)?;

        // Check if the package has lib crate-type = ["cdylib"]
        if cargo_toml_content.contains("crate-type") && cargo_toml_content.contains("cdylib") {
            return Ok(());
        }

        bail!(
            "Package '{}' is not configured to build as a dynamic library (cdylib). \
            Add 'crate-type = [\"cdylib\"]' to [lib] section in Cargo.toml",
            package_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
        );
    }

    /// Find the path to a package in the workspace
    fn find_package_path(workspace_root: &Path, package_name: &str) -> Result<PathBuf> {
        let metadata = MetadataCommand::new()
            .current_dir(workspace_root)
            .exec()
            .context("Failed to execute cargo metadata")?;

        for package in metadata.packages {
            if package.name.as_str() == package_name {
                return Ok(package.manifest_path.parent().unwrap().into());
            }
        }

        bail!("Package '{}' not found in workspace", package_name);
    }

    /// Get the cargo target directory using cargo metadata
    fn get_target_directory(&self) -> Result<PathBuf> {
        let metadata = MetadataCommand::new()
            .current_dir(&self.crate_path)
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

        let mut cmd = Command::new("cargo");
        cmd.current_dir(&self.crate_path)
            .args(["build", "--lib"])
            .args(["--target", target]);

        if self.release {
            cmd.arg("--release");
        }

        debug!("Executing cargo build command: {:?}", cmd);
        info!("Running cargo build for target: {}", target);

        // Use spawn() instead of output() to preserve colors and real-time output
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
            format!("{}.{}", self.target_name.replace('-', "_"), extension)
        } else {
            format!("lib{}.{}", self.target_name.replace('-', "_"), extension)
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

        // Extract model description from the first dylib
        let first_dylib = &dylib_paths[0].1;
        self.extract_model_description(&fmu_temp.join("modelDescription.xml"), first_dylib)?;

        // Add sources and documentation
        self.add_source_files(&fmu_temp.join("sources"))?;
        self.add_documentation(&fmu_temp.join("documentation"))?;

        // Create ZIP file
        self.create_zip(fmu_temp, fmu_path)?;

        Ok(())
    }

    /// Extract the modelDescription.xml file from the dylib
    /// Extract model description XML from the built dylib
    fn extract_model_description(&self, xml_path: &Path, dylib_path: &Path) -> Result<()> {
        info!(
            "Extracting model description from dylib: {}",
            self.to_relative_path(dylib_path)
        );

        // Check if the dylib file exists and log its size
        if !dylib_path.exists() {
            return Err(anyhow::anyhow!(
                "Dylib does not exist: {}",
                self.to_relative_path(dylib_path)
            ));
        }

        let metadata = fs::metadata(dylib_path)?;
        debug!("Dylib file size: {} bytes", metadata.len());

        let xml_content = model_description_extractor::extract_model_description(dylib_path)
            .context("Failed to extract model description from dylib")?;

        debug!("Extracted XML content ({} bytes)", xml_content.len());
        fs::write(xml_path, xml_content)?;
        debug!("Model description written to: {}", xml_path.display());

        Ok(())
    }

    /// Add source files to the FMU
    fn add_source_files(&self, sources_dir: &Path) -> Result<()> {
        debug!("Adding source files to: {}", sources_dir.display());

        // Create a proper FMI 3.0 build description using the new schema structures
        let build_description = Fmi3BuildDescription {
            fmi_version: "3.0".to_string(),
            build_configurations: vec![BuildConfiguration {
                model_identifier: self.model_identifier.clone(),
                platform: Some("generic".to_string()),
                description: Some(format!(
                    "Build configuration for {} FMU generated from Rust",
                    self.model_identifier
                )),
                source_file_sets: vec![SourceFileSet {
                    name: Some("documentation".to_string()),
                    language: Some("Markdown".to_string()),
                    source_files: vec![SourceFile {
                        name: "README.md".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        };

        // Serialize the build description to XML using yaserde
        let build_description_xml = fmi::schema::serialize(&build_description)
            .map_err(|e| anyhow::anyhow!("Failed to serialize build description to XML: {}", e))?;

        fs::write(
            sources_dir.join("buildDescription.xml"),
            format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n{}",
                build_description_xml
            ),
        )?;

        // Add a README
        let build_command = format!("cargo xtask bundle {}", self.target_name);

        let readme = format!(
            r#"# {} FMU Sources

This FMU was generated from Rust code using rust-fmi.

## Building

This FMU was built using the rust-fmi xtask build system:

```
{}
```

## Model: {}

Generated on: {}
"#,
            self.model_identifier,
            build_command,
            self.model_identifier,
            chrono::Utc::now().to_rfc3339()
        );

        fs::write(sources_dir.join("README.md"), readme)?;
        debug!("Source files added successfully");

        Ok(())
    }

    /// Add documentation files to the FMU
    fn add_documentation(&self, documentation_dir: &Path) -> Result<()> {
        debug!("Adding documentation to: {}", documentation_dir.display());

        let target_info = format!("Package: {}", self.target_name);

        let doc_content = format!(
            r#"# {} Documentation

This FMU was generated from Rust code using the rust-fmi library.

## Model Information

- **Model Identifier**: {}
- **Target**: {}
- **FMI Version**: 3.0
- **Generated**: {}

## Usage

This FMU can be used in any FMI-compatible simulation environment that supports FMI 3.0.

## Source

The source code for this FMU is available in the sources/ directory of this FMU archive.
"#,
            self.model_identifier,
            self.model_identifier,
            target_info,
            chrono::Utc::now().to_rfc3339()
        );

        fs::write(documentation_dir.join("README.md"), doc_content)?;
        debug!("Documentation added successfully");

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
