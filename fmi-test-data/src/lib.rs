#![doc=include_str!( "../README.md")]
#![deny(unsafe_code)]
#![deny(clippy::all)]

use anyhow::Context;
use fetch_data::{FetchData, ctor};
use fmi::{schema::MajorVersion, traits::FmiImport};
use std::{
    fs::File,
    io::{Cursor, Read},
};
use tempfile::NamedTempFile;

/// Version of the Reference FMUs to download
pub const REF_FMU_VERSION: &str = "0.0.39";

/// Computed archive filename
pub const REF_ARCHIVE: &str = const_format::concatcp!("Reference-FMUs-", REF_FMU_VERSION, ".zip");

/// Base URL for downloading Reference FMUs
pub const REF_URL: &str = const_format::concatcp!(
    "https://github.com/modelica/Reference-FMUs/releases/download/v",
    REF_FMU_VERSION,
    "/"
);

#[ctor]
static STATIC_FETCH_DATA: FetchData = FetchData::new(
    include_str!("registry.txt"),
    REF_URL,
    "FMU_DATA_DIR",
    "org",
    "modelica",
    "reference-fmus",
);

/// A Rust interface to the Modelica Reference-FMUs, downloaded as an archive using `fetch_data`
///
/// This struct provides access to the official Modelica Reference FMUs for testing and validation
/// purposes. It automatically downloads the FMU archive from the official GitHub repository
/// and provides methods to access individual FMUs.
///
/// # Examples
///
/// ```no_run
/// use fmi_test_data::ReferenceFmus;
/// use fmi::traits::FmiImport;
///
/// let mut reference_fmus = ReferenceFmus::new()?;
///
/// // Load a specific FMU
/// let fmu: fmi::fmi3::import::Fmi3Import = reference_fmus.get_reference_fmu("BouncingBall")?;
///
/// // List all available FMUs
/// let available_fmus = reference_fmus.list_available_fmus()?;
/// println!("Available FMUs: {:?}", available_fmus);
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct ReferenceFmus {
    archive: zip::ZipArchive<File>,
}

impl std::fmt::Debug for ReferenceFmus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReferenceFmus")
            .field("archive", &self.archive.comment())
            .finish()
    }
}

impl ReferenceFmus {
    /// Fetch the released Modelica Reference-FMUs file
    ///
    /// Downloads and caches the Reference FMUs archive from the official GitHub repository.
    /// The archive is automatically verified using SHA256 checksums.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The download fails
    /// - The file cannot be opened
    /// - The ZIP archive is corrupted
    pub fn new() -> anyhow::Result<Self> {
        let path = STATIC_FETCH_DATA
            .fetch_file(REF_ARCHIVE)
            .context(format!("Fetch {REF_ARCHIVE}"))?;
        let f = std::fs::File::open(&path).context(format!("Open {path:?}"))?;
        let archive = zip::ZipArchive::new(f)?;
        Ok(Self { archive })
    }

    /// Get a reference FMU as an import instance
    ///
    /// Loads a specific FMU from the archive and returns it as the requested import type.
    /// The FMU version is automatically determined by the import type.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the FMU to load (e.g., "BouncingBall")
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fmi_test_data::ReferenceFmus;
    /// # use fmi::traits::FmiImport;
    /// let mut reference_fmus = ReferenceFmus::new()?;
    ///
    /// // Load FMI 2.0 version
    /// let fmu: fmi::fmi2::import::Fmi2Import = reference_fmus.get_reference_fmu("BouncingBall")?;
    ///
    /// // Load FMI 3.0 version  
    /// let fmu: fmi::fmi3::import::Fmi3Import = reference_fmus.get_reference_fmu("BouncingBall")?;
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_reference_fmu<Imp: FmiImport>(&mut self, name: &str) -> anyhow::Result<Imp> {
        let version = Imp::MAJOR_VERSION.to_string();
        let mut f = self.archive.by_name(&format!("{version}/{name}.fmu"))?;
        // Read f into a Vec<u8> that can be used to create a new Import
        let mut buf = Vec::new();
        f.read_to_end(buf.as_mut())?;
        Ok(fmi::import::new(Cursor::new(buf))?)
    }

    /// Extract a reference FMU from the reference archive into a temporary file
    ///
    /// This method extracts an FMU to a temporary file on disk, which can be useful
    /// when you need to pass a file path to other tools or libraries.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the FMU to extract
    /// * `version` - The FMI major version to extract
    ///
    /// # Returns
    ///
    /// A `NamedTempFile` containing the extracted FMU. The file will be automatically
    /// deleted when the returned value is dropped.
    pub fn extract_reference_fmu(
        &mut self,
        name: &str,
        version: MajorVersion,
    ) -> anyhow::Result<NamedTempFile> {
        let version = version.to_string();
        let filename = format!("{version}/{name}.fmu");
        let mut fin = self
            .archive
            .by_name(&filename)
            .context(format!("Open {filename}"))?;
        let mut fout = tempfile::NamedTempFile::new()?;
        std::io::copy(fin.by_ref(), fout.as_file_mut())
            .context(format!("Extracting {filename} to tempfile"))?;
        Ok(fout)
    }

    /// Get a list of all available FMU files in the archive
    ///
    /// Returns a sorted list of all FMU names available in the Reference FMUs archive.
    /// These names can be used with `get_reference_fmu()` or `extract_reference_fmu()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use fmi_test_data::ReferenceFmus;
    /// let mut reference_fmus = ReferenceFmus::new()?;
    /// let fmus = reference_fmus.list_available_fmus()?;
    ///
    /// for fmu_name in &fmus {
    ///     println!("Available: {}", fmu_name);
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn list_available_fmus(&mut self) -> anyhow::Result<Vec<String>> {
        let mut fmus = Vec::new();
        for i in 0..self.archive.len() {
            let file = self.archive.by_index(i)?;
            let name = file.name();
            if name.ends_with(".fmu") {
                // Extract just the filename without path and extension
                if let Some(filename) = name.rsplit('/').next()
                    && let Some(base_name) = filename.strip_suffix(".fmu")
                {
                    fmus.push(base_name.to_string());
                }
            }
        }
        fmus.sort();
        fmus.dedup();
        Ok(fmus)
    }

    /// Get the Reference FMU version being used
    ///
    /// Returns the version string of the Reference FMUs package that this crate is configured to use.
    pub fn version() -> &'static str {
        REF_FMU_VERSION
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fmi::traits::FmiImport;

    #[test]
    fn test_reference_fmus_basic() {
        let mut reference_fmus = ReferenceFmus::new().unwrap();

        // Test FMI 2.0 BouncingBall
        let fmu: fmi::fmi2::import::Fmi2Import =
            reference_fmus.get_reference_fmu("BouncingBall").unwrap();
        assert_eq!(fmu.model_description().fmi_version, "2.0");
        assert_eq!(fmu.model_description().model_name, "BouncingBall");

        // Test FMI 3.0 BouncingBall
        let fmu: fmi::fmi3::import::Fmi3Import =
            reference_fmus.get_reference_fmu("BouncingBall").unwrap();
        assert_eq!(fmu.model_description().fmi_version, "3.0");
        assert_eq!(fmu.model_description().model_name, "BouncingBall");
    }

    #[test]
    fn test_version_constant() {
        assert_eq!(ReferenceFmus::version(), "0.0.39");
        assert!(REF_ARCHIVE.contains("0.0.39"));
        assert!(REF_URL.contains("v0.0.39"));
    }

    #[test]
    fn test_list_available_fmus() {
        let mut reference_fmus = ReferenceFmus::new().unwrap();
        let fmus = reference_fmus.list_available_fmus().unwrap();

        // Check that we have some common FMUs
        assert!(fmus.contains(&"BouncingBall".to_string()));
        assert!(fmus.contains(&"Dahlquist".to_string()));
        assert!(fmus.contains(&"VanDerPol".to_string()));

        // Ensure the list is sorted and contains no duplicates
        let mut sorted_fmus = fmus.clone();
        sorted_fmus.sort();
        assert_eq!(fmus, sorted_fmus);
    }

    #[test]
    fn test_extract_reference_fmu() {
        let mut reference_fmus = ReferenceFmus::new().unwrap();

        // Test extraction to temporary file
        let temp_file = reference_fmus
            .extract_reference_fmu("BouncingBall", MajorVersion::FMI3)
            .unwrap();

        // Verify the temporary file exists and has content
        assert!(temp_file.path().exists());
        let metadata = std::fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.len() > 0);
    }

    #[test]
    fn test_feedthrough_fmu() {
        let mut reference_fmus = ReferenceFmus::new().unwrap();

        // Test the Feedthrough FMU which should exist in both versions
        let fmu_v2: fmi::fmi2::import::Fmi2Import =
            reference_fmus.get_reference_fmu("Feedthrough").unwrap();
        assert_eq!(fmu_v2.model_description().model_name, "Feedthrough");

        let fmu_v3: fmi::fmi3::import::Fmi3Import =
            reference_fmus.get_reference_fmu("Feedthrough").unwrap();
        assert_eq!(fmu_v3.model_description().model_name, "Feedthrough");
    }

    #[test]
    fn test_nonexistent_fmu() {
        let mut reference_fmus = ReferenceFmus::new().unwrap();

        // This should fail gracefully
        let result: anyhow::Result<fmi::fmi3::import::Fmi3Import> =
            reference_fmus.get_reference_fmu("NonExistentFMU");
        assert!(result.is_err());
    }

    #[cfg(false)]
    #[test]
    fn print_registry_contents() {
        let registry_contents = STATIC_FETCH_DATA
            .gen_registry_contents([REF_ARCHIVE])
            .unwrap();
        println!("{registry_contents}");
    }

    #[cfg(false)]
    #[test]
    fn print_all_available_fmus() {
        let mut reference_fmus = ReferenceFmus::new().unwrap();
        let fmus = reference_fmus.list_available_fmus().unwrap();
        println!("Available FMUs ({} total):", fmus.len());
        for fmu in fmus {
            println!("  - {}", fmu);
        }
    }
}
