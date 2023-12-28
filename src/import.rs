use std::{path::Path, str::FromStr};

#[cfg(feature = "fmi2")]
use crate::fmi2;
#[cfg(feature = "fmi3")]
use crate::fmi3;
use crate::Error;

use fmi_schema::minimal::ModelDescription as MinModel;

const MODEL_DESCRIPTION: &str = "modelDescription.xml";

pub trait FmiImport: Sized {
    /// The raw parsed XML schema type
    type Schema;

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
    fn model_description(&self) -> &Self::Schema;

    /// Load the plugin shared library and return the raw bindings.
    fn binding(&self, model_identifier: &str) -> Result<Self::Binding, Error>;
}

/// Import is responsible for extracting the FMU, parsing the modelDescription XML and loading the
/// shared library.
#[derive(Debug)]
pub enum Import {
    #[cfg(feature = "fmi2")]
    Fmi2(fmi2::import::Fmi2),
    #[cfg(feature = "fmi3")]
    Fmi3(fmi3::import::Fmi3),
}

impl Import {
    /// Creates a new Import by extracting the FMU and parsing the modelDescription XML
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let file = std::fs::File::open(path.as_ref())?;
        let mut archive = zip::ZipArchive::new(file)?;
        let temp_dir = tempfile::Builder::new().prefix("fmi-rs").tempdir()?;
        log::debug!("Extracting {:?} into {temp_dir:?}", path.as_ref());
        archive.extract(&temp_dir)?;

        for fname in archive.file_names() {
            log::trace!("  - {}", fname);
        }

        // Open and read the modelDescription XML into a string
        let descr_file_path = temp_dir.path().join(MODEL_DESCRIPTION);
        let descr_xml = std::fs::read_to_string(&descr_file_path)?;

        // Initial non-version-specific model description
        let descr = MinModel::from_str(&descr_xml)?;
        log::debug!(
            "Found FMI {} named '{}",
            descr.fmi_version,
            descr.model_name
        );

        match descr.version()?.major {
            #[cfg(feature = "fmi2")]
            2 => fmi2::import::Fmi2::new(temp_dir, &descr_xml).map(|import| Import::Fmi2(import)),

            #[cfg(feature = "fmi3")]
            3 => fmi3::import::Fmi3::new(temp_dir, &descr_xml).map(|import| Import::Fmi3(import)),

            _ => {
                return Err(Error::UnsupportedFmiVersion(descr.fmi_version.to_string()));
            }
        }
    }

    #[cfg(feature = "fmi2")]
    pub fn as_fmi2(self) -> Option<fmi2::import::Fmi2> {
        if let Self::Fmi2(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[cfg(feature = "fmi3")]
    pub fn as_fmi3(self) -> Option<fmi3::import::Fmi3> {
        if let Self::Fmi3(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

// TODO Make this work on other targets
#[cfg(test)]
#[cfg(target_os = "linux")]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "fmi2")]
    fn test_import_fmi2() {
        let import = Import::new("data/reference_fmus/2.0/BouncingBall.fmu")
            .unwrap()
            .as_fmi2()
            .unwrap();
        assert_eq!(import.model_description().fmi_version, "2.0");
        assert_eq!(import.model_description().model_name, "BouncingBall");
        let me = import.model_description().model_exchange.as_ref().unwrap();
        assert_eq!(me.model_identifier, "BouncingBall");
        let binding = import.binding(&me.model_identifier).unwrap();
        let ver = unsafe {
            std::ffi::CStr::from_ptr(binding.fmi2GetVersion())
                .to_str()
                .unwrap()
        };
        assert_eq!(ver, "2.0");
    }

    #[test_log::test]
    #[cfg(feature = "fmi3")]
    fn test_import_fmi3() {
        let import = Import::new("data/reference_fmus/3.0/BouncingBall.fmu")
            .unwrap()
            .as_fmi3()
            .unwrap();
        assert_eq!(import.model_description().fmi_version, "3.0");
        assert_eq!(import.model_description().model_name, "BouncingBall");
        let me = import.model_description().model_exchange.as_ref().unwrap();
        let binding = import.binding(&me.model_identifier).unwrap();
        let ver = unsafe {
            std::ffi::CStr::from_ptr(binding.fmi3GetVersion())
                .to_str()
                .unwrap()
        };
        assert_eq!(ver, "3.0");
    }

    #[test_log::test]
    #[cfg(feature = "fmi2")]
    fn test_import_me() {
        let import = Import::new("data/Modelica_Blocks_Sources_Sine.fmu")
            .unwrap()
            .as_fmi2()
            .unwrap();
        assert_eq!(import.model_description().fmi_version, "2.0");
        let me = import.instantiate_me("inst1", false, true).unwrap();
        assert_eq!(fmi2::instance::traits::Common::version(&me), "2.0");
    }

    #[test_log::test]
    #[cfg(feature = "fmi2")]
    fn test_import_cs() {
        let import = Import::new("data/Modelica_Blocks_Sources_Sine.fmu")
            .unwrap()
            .as_fmi2()
            .unwrap();
        assert_eq!(import.model_description().fmi_version, "2.0");
        let cs = import.instantiate_cs("inst1", false, true).unwrap();
        assert_eq!(fmi2::instance::traits::Common::version(&cs), "2.0");
    }
}
