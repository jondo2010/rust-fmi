use std::{
    io::{self, Read},
    path::Path,
    str::FromStr,
};

#[cfg(feature = "fmi2")]
use crate::fmi2;
#[cfg(feature = "fmi3")]
use crate::fmi3;
use crate::Error;

use fmi_schema::minimal::ModelDescription as MinModel;

const MODEL_DESCRIPTION: &str = "modelDescription.xml";

pub trait FmiImport: Sized {
    /// The raw parsed XML schema type
    type ModelDescription;

    /// The raw FMI bindings type
    type Binding;

    /// Create a new FMI import from a directory containing the unzipped FMU
    fn new(dir: tempfile::TempDir, schema_xml: &str) -> Result<Self, Error>;

    /// Return the path to the extracted FMU
    fn archive_path(&self) -> &std::path::Path;

    /// Get the path to the shared library
    fn shared_lib_path(&self, model_identifier: &str) -> Result<std::path::PathBuf, Error>;

    /// Return the path to the resources directory
    fn resource_url(&self) -> url::Url {
        url::Url::from_file_path(self.archive_path().join("resources"))
            .expect("Error forming resource location URL")
    }

    /// Get a reference to the raw-schema model description
    fn model_description(&self) -> &Self::ModelDescription;

    /// Load the plugin shared library and return the raw bindings.
    fn binding(&self, model_identifier: &str) -> Result<Self::Binding, Error>;
}

/// Import is responsible for extracting the FMU, parsing the modelDescription XML and loading the
/// shared library.
#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum Import {
    #[cfg(feature = "fmi2")]
    Fmi2(fmi2::import::Fmi2),
    #[cfg(feature = "fmi3")]
    Fmi3(fmi3::import::Fmi3Import),
}

impl Import {
    /// Creates a new Import by extracting the FMU and parsing the modelDescription XML
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = std::fs::File::open(path.as_ref())?;
        log::debug!("Opening FMU file {:?}", path.as_ref());
        Self::new(file)
    }

    /// Creates a new Import by extracting the FMU and parsing the modelDescription XML
    pub fn new<R: Read + io::Seek>(reader: R) -> Result<Self, Error> {
        let mut archive = zip::ZipArchive::new(reader)?;
        let temp_dir = tempfile::Builder::new().prefix("fmi-rs").tempdir()?;
        log::debug!("Extracting into {temp_dir:?}");
        archive.extract(&temp_dir)?;

        for fname in archive.file_names() {
            log::trace!("  - {}", fname);
        }

        // Open and read the modelDescription XML into a string
        let descr_file_path = temp_dir.path().join(MODEL_DESCRIPTION);
        let descr_xml = std::fs::read_to_string(descr_file_path)?;

        // Initial non-version-specific model description
        let descr = MinModel::from_str(&descr_xml)?;
        log::debug!(
            "Found FMI {} named '{}'",
            descr.fmi_version,
            descr.model_name
        );

        match descr.version()?.major {
            #[cfg(feature = "fmi2")]
            2 => fmi2::import::Fmi2::new(temp_dir, &descr_xml).map(Import::Fmi2),

            #[cfg(feature = "fmi3")]
            3 => fmi3::import::Fmi3Import::new(temp_dir, &descr_xml).map(Import::Fmi3),

            _ => Err(Error::UnsupportedFmiVersion(descr.fmi_version.to_string())),
        }
    }

    #[cfg(feature = "fmi2")]
    #[must_use]
    pub fn as_fmi2(self) -> Option<fmi2::import::Fmi2> {
        if let Self::Fmi2(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[cfg(feature = "fmi3")]
    #[must_use]
    pub fn as_fmi3(self) -> Option<fmi3::import::Fmi3Import> {
        if let Self::Fmi3(v) = self {
            Some(v)
        } else {
            None
        }
    }
}
