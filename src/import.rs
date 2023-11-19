use std::{io::Read, path::Path};

use yaserde_derive::YaDeserialize;

use crate::{fmi2, fmi3};

use super::{FmiError, FmiResult};
use log::trace;

const MODEL_DESCRIPTION: &str = "modelDescription.xml";
const FMI_MAJOR_VERSION_2: u32 = 2;
const FMI_MAJOR_VERSION_3: u32 = 3;

/// A minimal model description that only contains the FMI version
/// This is used to determine the FMI version of the FMU
#[derive(Debug, Default, PartialEq, YaDeserialize)]
#[yaserde(root = "fmiModelDescription")]
struct ModelDescription {
    #[yaserde(attribute, rename = "fmiVersion")]
    fmi_version: String,
    #[yaserde(attribute, rename = "modelName")]
    model_name: String,
}

impl ModelDescription {
    /// Get the major version from the version string
    fn major_version(&self) -> FmiResult<u32> {
        self.fmi_version
            .split_once(".")
            .and_then(|(major, _)| major.parse().ok())
            .ok_or(FmiError::Parse(format!(
                "Unable to parse major version from FMI version string {}",
                self.fmi_version
            )))
    }
}

fn extract_archive(archive: impl AsRef<Path>, outdir: impl AsRef<Path>) -> FmiResult<()> {
    let archive = archive.as_ref();
    let outdir = outdir.as_ref();
    trace!("Extracting {} into {}", archive.display(), outdir.display());
    let file = std::fs::File::open(archive)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = outdir.join(file.name());
        if file.is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

pub trait FmiImport: Sized {
    fn new(dir: tempfile::TempDir, schema_xml: &str) -> FmiResult<Self>;

    /// Return the path to the extracted FMU
    fn path(&self) -> &std::path::Path;

    /// Return the path to the resources directory
    fn resource_url(&self) -> url::Url {
        url::Url::from_file_path(self.path().join("resources"))
            .expect("Error forming resource location URL")
    }
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
    pub fn new(path: impl Into<std::path::PathBuf>) -> FmiResult<Import> {
        // First create a temp directory
        let temp_dir = tempfile::Builder::new().prefix("fmi-rs").tempdir()?;
        extract_archive(path.into(), temp_dir.path())?;

        // Open and read the modelDescription XML into a string
        let descr_file_path = temp_dir.path().join(MODEL_DESCRIPTION);
        let mut descr_file = std::fs::File::open(descr_file_path)?;
        let mut descr_buf = String::new();
        let descr_size = descr_file.read_to_string(&mut descr_buf)?;
        trace!("Read {} bytes from {MODEL_DESCRIPTION}", descr_size,);

        // Initial non-version-specific model description
        let descr: ModelDescription =
            yaserde::de::from_str(&descr_buf).map_err(|err| FmiError::Parse(err))?;

        trace!(
            "Found FMI {} named '{}",
            descr.fmi_version,
            descr.model_name
        );

        match descr.major_version()? {
            #[cfg(feature = "fmi2")]
            FMI_MAJOR_VERSION_2 => {
                fmi2::import::Fmi2::new(temp_dir, &descr_buf).map(|import| Import::Fmi2(import))
            }
            #[cfg(feature = "fmi3")]
            FMI_MAJOR_VERSION_3 => {
                fmi3::import::Fmi3::new(temp_dir, &descr_buf).map(|import| Import::Fmi3(import))
            }
            _ => {
                return Err(FmiError::UnsupportedFmiVersion(descr.fmi_version));
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
#[cfg(target_os = "linux")]
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
        assert_eq!(import.model().fmi_version, "3.0");
        assert_eq!(import.model().model_name, "BouncingBall");
        let binding = import.raw_bindings().unwrap();
        let ver = unsafe {
            std::ffi::CStr::from_ptr(binding.fmi3GetVersion())
                .to_str()
                .unwrap()
        };
        assert_eq!(ver, "3.0");

        let mut inst1 = import.instantiate_me("inst1", true, true).unwrap();
        inst1
            .set_debug_logging(true, import.model().log_categories.keys())
            .unwrap();
        inst1.enter_initialization_mode(None, 0.0, None).unwrap();
        inst1.exit_initialization_mode().unwrap();
        inst1.set_time(1234.0).unwrap();

        inst1.enter_continuous_time_mode().unwrap();

        let states = (0..import
            .model()
            .model_structure
            .continuous_state_derivatives
            .len())
            .map(|x| x as f64)
            .collect::<Vec<_>>();
        dbg!(&states);

        inst1.set_continuous_states(&states).unwrap();
        let ret = inst1.completed_integrator_step(false).unwrap();
        dbg!(ret);
    }

    #[test]
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
