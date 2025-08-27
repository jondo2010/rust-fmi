use anyhow::{bail, Context, Result};
use cargo_metadata::MetadataCommand;
use log::{debug, info, warn};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::model_description_extractor;
use crate::platform::PlatformMapping;

/// Builder for creating FMU packages
pub struct FmuBuilder {
    crate_path: PathBuf,
    example_name: String,
    model_identifier: String,
    output_dir: PathBuf,
    release: bool,
    platform_mapping: PlatformMapping,
}

impl FmuBuilder {
    pub fn new(
        crate_path: PathBuf,
        example_name: String,
        model_identifier: String,
        output_dir: PathBuf,
        release: bool,
    ) -> Result<Self> {
        if !crate_path.exists() {
            bail!("Crate path does not exist: {}", crate_path.display());
        }

        Ok(Self {
            crate_path,
            example_name,
            model_identifier,
            output_dir,
            release,
            platform_mapping: PlatformMapping::new(),
        })
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

    /// Build FMU for multiple platforms
    pub fn build_multi_platform(&self, targets: &[String]) -> Result<()> {
        info!("Building multi-platform FMU for targets: {:?}", targets);

        let mut dylib_paths = Vec::new();

        // Build for each target
        for target in targets {
            debug!("Building for target: {}", target);
            let dylib_path = self.build_dylib(target)?;
            dylib_paths.push((target.as_str(), dylib_path));
        }

        // Create multi-platform FMU package
        let fmu_path = self
            .output_dir
            .join(format!("{}.fmu", self.model_identifier));
        self.create_fmu_package(&fmu_path, &dylib_paths)?;

        info!("Created multi-platform FMU: {}", fmu_path.display());
        Ok(())
    }

    /// Build the dynamic library for a specific target
    fn build_dylib(&self, target: &str) -> Result<PathBuf> {
        debug!("Building dylib for target: {}", target);

        let mut cmd = Command::new("cargo");
        cmd.current_dir(&self.crate_path)
            .args(["build", "--example", &self.example_name])
            .args(["--target", target]);

        if self.release {
            cmd.arg("--release");
        }

        debug!("Executing cargo build command: {:?}", cmd);
        let output = cmd.output().context("Failed to execute cargo build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("Cargo build failed:\n{}", stderr);
        }

        // Get the actual target directory from cargo metadata
        let target_dir = self
            .get_target_directory()
            .context("Failed to get target directory from cargo metadata")?;

        let profile = if self.release { "release" } else { "debug" };
        let extension = self.platform_mapping.get_library_extension(target);
        let lib_name = if target.contains("windows") {
            format!("{}.{}", self.example_name, extension)
        } else {
            format!("lib{}.{}", self.example_name, extension)
        };

        // Construct the path using the target directory from cargo metadata
        let dylib_path = target_dir
            .join(target)
            .join(profile)
            .join("examples")
            .join(&lib_name);

        debug!("Looking for dylib at: {}", dylib_path.display());
        if !dylib_path.exists() {
            bail!("Built library not found at: {}", dylib_path.display());
        }

        info!("Successfully built dylib: {}", dylib_path.display());
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
            dylib_path.display()
        );

        // Check if the dylib file exists and log its size
        if !dylib_path.exists() {
            return Err(anyhow::anyhow!(
                "Dylib does not exist: {}",
                dylib_path.display()
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

        // Add a build description
        let build_description = r#"<?xml version="1.0" encoding="UTF-8"?>
<BuildDescription fmiVersion="3.0">
    <SourceFileSet>
        <SourceFile name="README.md"/>
    </SourceFileSet>
</BuildDescription>"#;

        fs::write(sources_dir.join("buildDescription.xml"), build_description)?;

        // Add a README
        let readme = format!(
            r#"# {} FMU Sources

This FMU was generated from Rust code using rust-fmi.

## Building

This FMU was built using the rust-fmi xtask build system:

```
cargo xtask build-fmu --crate-path . --example {}
```

## Model: {}

Generated on: {}
"#,
            self.model_identifier,
            self.example_name,
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

        let doc_content = format!(
            r#"# {} Documentation

This FMU was generated from Rust code using the rust-fmi library.

## Model Information

- **Model Identifier**: {}
- **Example Name**: {}
- **FMI Version**: 3.0
- **Generated**: {}

## Usage

This FMU can be used in any FMI-compatible simulation environment that supports FMI 3.0.

## Source

The source code for this FMU is available in the sources/ directory of this FMU archive.
"#,
            self.model_identifier,
            self.model_identifier,
            self.example_name,
            chrono::Utc::now().to_rfc3339()
        );

        fs::write(documentation_dir.join("README.md"), doc_content)?;
        debug!("Documentation added successfully");

        Ok(())
    }

    /// Create the final ZIP file
    fn create_zip(&self, source_dir: &Path, zip_path: &Path) -> Result<()> {
        info!("Creating FMU archive: {}", zip_path.display());

        let file = fs::File::create(zip_path)?;
        let mut zip = ZipWriter::new(file);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        self.add_dir_to_zip(&mut zip, source_dir, source_dir, &options)?;

        zip.finish()?;
        info!("FMU archive created successfully: {}", zip_path.display());
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
