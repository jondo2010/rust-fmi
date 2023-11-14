use std::{
    io::Read,
    path::{Path, PathBuf},
};

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

#[cfg(feature = "fmi2")]
#[derive(Debug)]
pub struct Fmi2 {
    /// Path to the unzipped FMU on disk
    dir: tempfile::TempDir,
    /// Parsed raw-schema model description
    schema: fmi2::meta::ModelDescription,
}

#[cfg(feature = "fmi2")]
impl Fmi2 {
    /// Get a reference to the raw-schema model description
    pub fn raw_schema(&self) -> &fmi2::meta::ModelDescription {
        &self.schema
    }

    /// Get the path to the shared library
    fn shared_lib_path(&self) -> FmiResult<PathBuf> {
        let platform_folder = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => "win64",
            ("windows", "x86") => "win32",
            ("linux", "x86_64") => "linux64",
            ("linux", "x86") => "linux32",
            ("macos", "x86_64") => "darwin64",
            ("macos", "x86") => "darwin32",
            _ => panic!("Unsupported platform"),
        };
        let model_identifier = &self.schema.model_name;
        let fname = format!("{model_identifier}{}", std::env::consts::DLL_SUFFIX);
        Ok(std::path::PathBuf::from("binaries")
            .join(platform_folder)
            .join(fname))
    }

    /// Load the plugin shared library and return the raw bindings.
    pub fn raw_bindings(&self) -> FmiResult<fmi2::binding::Fmi2Binding> {
        let lib_path = self.dir.path().join(self.shared_lib_path()?);
        trace!("Loading shared library {:?}", lib_path);
        unsafe { fmi2::binding::Fmi2Binding::new(lib_path).map_err(FmiError::from) }
    }
}

#[cfg(feature = "fmi3")]
#[derive(Debug)]
pub struct Fmi3 {
    /// Path to the unzipped FMU on disk
    dir: tempfile::TempDir,
    /// Parsed raw-schema model description
    schema: fmi3::schema::FmiModelDescription,
}

#[cfg(feature = "fmi3")]
impl Fmi3 {
    /// Get a reference to the raw-schema model description
    pub fn raw_schema(&self) -> &fmi3::schema::FmiModelDescription {
        &self.schema
    }

    /// Build a derived model description from the raw-schema model description
    pub fn model(&self) -> FmiResult<fmi3::model::ModelDescription> {
        fmi3::model::ModelDescription::try_from(&self.schema).map_err(FmiError::from)
    }

    /// Get the path to the shared library
    fn shared_lib_path(&self) -> FmiResult<PathBuf> {
        let platform_folder = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => "x86_64-windows",
            ("windows", "x86") => "x86-windows",
            ("linux", "x86_64") => "x86_64-linux",
            ("linux", "x86") => "x86-linux",
            ("macos", "x86_64") => "x86-64-darwin",
            ("macos", "x86") => "x86-darwin",
            _ => panic!("Unsupported platform"),
        };
        let model_identifier = &self.schema.model_name;
        let fname = format!("{model_identifier}{}", std::env::consts::DLL_SUFFIX);
        Ok(std::path::PathBuf::from("binaries")
            .join(platform_folder)
            .join(fname))
    }

    /// Load the plugin shared library and return the raw bindings.
    pub fn raw_bindings(&self) -> FmiResult<fmi3::binding::Fmi3Binding> {
        let lib_path = self.dir.path().join(self.shared_lib_path()?);
        trace!("Loading shared library {:?}", lib_path);
        unsafe { fmi3::binding::Fmi3Binding::new(lib_path).map_err(FmiError::from) }
    }
}

/// Import is responsible for extracting the FMU, parsing the modelDescription XML and loading the
/// shared library.
#[derive(Debug)]
pub enum Import {
    #[cfg(feature = "fmi2")]
    Fmi2(Fmi2),
    #[cfg(feature = "fmi3")]
    Fmi3(Fmi3),
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
                let schema: fmi2::meta::ModelDescription =
                    serde_xml_rs::from_str(&descr_buf).map_err(FmiError::from)?;

                Ok(Import::Fmi2(Fmi2 {
                    dir: temp_dir,
                    schema,
                }))
            }

            #[cfg(feature = "fmi3")]
            FMI_MAJOR_VERSION_3 => {
                let schema: fmi3::schema::FmiModelDescription =
                    yaserde::de::from_str(&descr_buf).map_err(|err| FmiError::Parse(err))?;

                Ok(Import::Fmi3(Fmi3 {
                    dir: temp_dir,
                    schema,
                }))
            }

            _ => {
                return Err(FmiError::UnsupportedFmiVersion(descr.fmi_version));
            }
        }
    }

    /// Create a ModelExchange API container if supported
    #[cfg(feature = "disabled")]
    pub fn container_me(&self) -> FmiResult<Container<fmi2::Fmi2ME>> {
        let me = self
            .descr
            .model_exchange
            .as_ref()
            .ok_or(FmiError::UnsupportedFmuType(fmi2::fmi2Type::ModelExchange))?;
        trace!("Found ModelExchange model \"{}\"", me.model_identifier);

        let lib_path = self
            .dir
            .path()
            .join(construct_so_path(&me.model_identifier)?);
        trace!("Loading shared library {:?}", lib_path);

        unsafe { Container::load(lib_path) }.map_err(FmiError::from)
    }

    /// Create a CoSimulation API container if supported
    #[cfg(feature = "disabled")]
    pub fn container_cs(&self) -> FmiResult<Container<fmi2::Fmi2CS>> {
        let cs = self
            .descr
            .co_simulation
            .as_ref()
            .ok_or(FmiError::UnsupportedFmuType(fmi2::fmi2Type::CoSimulation))?;
        trace!("Found CoSimulation model \"{}\"", cs.model_identifier);

        let lib_path = self
            .dir
            .path()
            .join(construct_so_path(&cs.model_identifier)?);
        trace!("Loading shared library {:?}", lib_path);

        unsafe { Container::load(lib_path) }.map_err(FmiError::from)
    }

    /// Return the path to the extracted FMU
    pub fn path(&self) -> &std::path::Path {
        match self {
            #[cfg(feature = "fmi2")]
            Import::Fmi2(Fmi2 { dir, .. }) => dir.path(),
            #[cfg(feature = "fmi3")]
            Import::Fmi3(Fmi3 { dir, .. }) => dir.path(),
        }
    }

    /// Return the path to the resources directory
    pub fn resource_url(&self) -> url::Url {
        url::Url::from_file_path(self.path().join("resources"))
            .expect("Error forming resource location URL")
    }

    #[cfg(feature = "fmi2")]
    pub fn as_fmi2(self) -> Option<Fmi2> {
        if let Self::Fmi2(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[cfg(feature = "fmi3")]
    pub fn as_fmi3(self) -> Option<Fmi3> {
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
    fn test_initial_import() {
        // Test an import of every FMU file in the data/reference_fmus directory
        for entry in std::fs::read_dir("data/reference_fmus/3.0").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            println!("Importing {:?}", path);
            let import = Import::new(path).unwrap();
            //assert_eq!(import.descr.fmi_version, "2.0");

            println!("Imported {:?}", import.path());
        }
    }

    #[test]
    #[cfg(feature = "fmi2")]
    fn test_import_fmi2() {
        let import = Import::new("data/reference_fmus/2.0/BouncingBall.fmu")
            .unwrap()
            .as_fmi2()
            .unwrap();
        assert_eq!(import.schema.fmi_version, "2.0");
        assert_eq!(import.schema.model_name, "BouncingBall");
        let binding = import.raw_bindings().unwrap();
        let ver = unsafe {
            std::ffi::CStr::from_ptr(binding.fmi2GetVersion())
                .to_str()
                .unwrap()
        };
        assert_eq!(ver, "2.0");
    }

    #[test]
    #[cfg(feature = "fmi3")]
    fn test_import_fmi3() {
        let import = Import::new("data/reference_fmus/3.0/BouncingBall.fmu")
            .unwrap()
            .as_fmi3()
            .unwrap();
        assert_eq!(import.schema.fmi_version, "3.0");
        assert_eq!(import.schema.model_name, "BouncingBall");
        let binding = import.raw_bindings().unwrap();
        let ver = unsafe {
            std::ffi::CStr::from_ptr(binding.fmi3GetVersion())
                .to_str()
                .unwrap()
        };
        assert_eq!(ver, "3.0");
    }

    #[test]
    #[cfg(feature = "disabled")]
    fn test_import_me() {
        let import = Import::new("data/Modelica_Blocks_Sources_Sine.fmu").unwrap();
        assert_eq!(import.descr.fmi_version, "2.0");
        let _me = import.container_me().unwrap();
    }

    #[test]
    #[cfg(feature = "disabled")]
    fn test_import_cs() {
        let import = Import::new("data/Modelica_Blocks_Sources_Sine.fmu").unwrap();
        assert_eq!(import.descr.fmi_version, "2.0");
        let _cs = import.container_cs().unwrap();
    }
}
