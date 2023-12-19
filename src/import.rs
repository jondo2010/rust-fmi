use std::{io::Read, path::Path, str::FromStr};

#[cfg(feature = "fmi2")]
use crate::fmi2;
#[cfg(feature = "fmi3")]
use crate::fmi3;
use crate::Error;

use fmi_schema::minimal::ModelDescription as MinModel;

const MODEL_DESCRIPTION: &str = "modelDescription.xml";

pub trait FmiImport<'a>: Sized {
    /// The raw parsed XML schema type
    type Schema;

    /// The raw FMI bindings type
    type Binding;

    /// Create a new FMI import from a directory containing the unzipped FMU
    fn new(dir: tempfile::TempDir, schema_xml: String) -> Result<Self, Error>;

    /// Return the path to the extracted FMU
    fn archive_path(&self) -> &std::path::Path;

    /// Get the path to the shared library
    fn shared_lib_path(&self) -> Result<std::path::PathBuf, Error>;

    /// Return the path to the resources directory
    fn resource_url(&self) -> url::Url {
        url::Url::from_file_path(self.archive_path().join("resources"))
            .expect("Error forming resource location URL")
    }

    /// Get a reference to the raw-schema model description
    fn model_description(&self) -> &Self::Schema;

    /// Load the plugin shared library and return the raw bindings.
    fn binding(&self) -> Result<Self::Binding, Error>;
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
        let file = std::fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let temp_dir = tempfile::Builder::new().prefix("fmi-rs").tempdir()?;
        log::trace!("Extracting {archive:?} into {temp_dir:?}");
        archive.extract(&temp_dir)?;

        // Open and read the modelDescription XML into a string
        let descr_file_path = temp_dir.path().join(MODEL_DESCRIPTION);
        let mut descr_file = std::fs::File::open(descr_file_path)?;
        let mut descr_buf = String::new();
        let descr_size = descr_file.read_to_string(&mut descr_buf)?;
        log::trace!("Read {descr_size} bytes from {MODEL_DESCRIPTION}");

        // Initial non-version-specific model description
        let descr = MinModel::from_str(&descr_buf)?;
        log::trace!(
            "Found FMI {} named '{}",
            descr.fmi_version,
            descr.model_name
        );

        match descr.version()?.major {
            #[cfg(feature = "fmi2")]
            2 => fmi2::import::Fmi2::new(temp_dir, &descr_buf).map(|import| Import::Fmi2(import)),

            #[cfg(feature = "fmi3")]
            3 => Ok(Self::Fmi3(fmi3::import::Fmi3::new(temp_dir, descr_buf)?)),

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
//#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(feature = "fmi2")]
    fn test_import_fmi2() {
        let import = Import::new("data/reference_fmus/2.0/BouncingBall.fmu")
            .unwrap()
            .as_fmi2()
            .unwrap();
        assert_eq!(import.raw_schema().fmi_version, "2.0");
        assert_eq!(import.raw_schema().model_name, "BouncingBall");
        let binding = import.raw_bindings().unwrap();
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
        use crate::fmi3::instance::traits::{Common, ModelExchange};

        let import = Import::new("data/reference_fmus/3.0/BouncingBall.fmu")
            .unwrap()
            .as_fmi3()
            .unwrap();
        assert_eq!(import.model_description().fmi_version, "3.0");
        assert_eq!(import.model_description().model_name, "BouncingBall");
        let binding = import.binding().unwrap();
        let ver = unsafe {
            std::ffi::CStr::from_ptr(binding.fmi3GetVersion())
                .to_str()
                .unwrap()
        };
        assert_eq!(ver, "3.0");

        let mut inst1 = import.instantiate_me("inst1", true, true).unwrap();
        //inst1 .set_debug_logging(true, import.model().log_categories.keys()) .unwrap();
        inst1.enter_initialization_mode(None, 0.0, None).unwrap();
        inst1.exit_initialization_mode().unwrap();
        inst1.set_time(1234.0).unwrap();

        inst1.enter_continuous_time_mode().unwrap();

        let states = (0..import
            .model_description()
            .model_structure
            .continuous_state_derivative
            .len())
            .map(|x| x as f64)
            .collect::<Vec<_>>();
        dbg!(&states);

        inst1.set_continuous_states(&states).unwrap();
        let ret = inst1.completed_integrator_step(false).unwrap();
        dbg!(ret);

        let mut ders = vec![0.0; states.len()];
        inst1
            .get_continuous_state_derivatives(ders.as_mut_slice())
            .unwrap();
        dbg!(ders);
    }

    #[test]
    #[cfg(feature = "fmi2")]
    fn test_import_me() {
        let import = Import::new("data/Modelica_Blocks_Sources_Sine.fmu")
            .unwrap()
            .as_fmi2()
            .unwrap();
        assert_eq!(import.raw_schema().fmi_version, "2.0");
        //let _me = import.container_me().unwrap();
    }

    #[test]
    #[cfg(feature = "disabled")]
    fn test_import_cs() {
        let import = Import::new("data/Modelica_Blocks_Sources_Sine.fmu").unwrap();
        assert_eq!(import.descr.fmi_version, "2.0");
        let _cs = import.container_cs().unwrap();
    }
}
