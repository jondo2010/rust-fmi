use std::{path::PathBuf, str::FromStr};

use super::{
    binding,
    instance::{Instance, CS, ME},
};
use crate::{traits::FmiImport, Error};

use fmi_schema::{fmi2 as schema, MajorVersion};

#[derive(Debug)]
pub struct Fmi2Import {
    /// Path to the unzipped FMU on disk
    dir: tempfile::TempDir,
    /// Parsed raw-schema model description
    model_description: schema::Fmi2ModelDescription,
}

impl FmiImport for Fmi2Import {
    const MAJOR_VERSION: MajorVersion = MajorVersion::FMI2;
    type ModelDescription = schema::Fmi2ModelDescription;
    type Binding = binding::Fmi2Binding;
    type ValueReference = binding::fmi2ValueReference;

    fn new(dir: tempfile::TempDir, schema_xml: &str) -> Result<Self, Error> {
        let schema = schema::Fmi2ModelDescription::from_str(schema_xml)?;
        Ok(Self {
            dir,
            model_description: schema,
        })
    }

    #[inline]
    fn archive_path(&self) -> &std::path::Path {
        self.dir.path()
    }

    /// Get the path to the shared library
    fn shared_lib_path(&self, model_identifier: &str) -> Result<PathBuf, Error> {
        let platform_folder = match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => "win64",
            ("windows", "x86") => "win32",
            ("linux", "x86_64") => "linux64",
            ("linux", "x86") => "linux32",
            ("macos", "x86_64") => "darwin64",
            ("macos", "x86") => "darwin32",
            _ => {
                return Err(Error::UnsupportedPlatform {
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                });
            }
        };
        let fname = format!("{model_identifier}{}", std::env::consts::DLL_SUFFIX);
        Ok(std::path::PathBuf::from("binaries")
            .join(platform_folder)
            .join(fname))
    }

    fn model_description(&self) -> &Self::ModelDescription {
        &self.model_description
    }

    /// Load the plugin shared library and return the raw bindings.
    fn binding(&self, model_identifier: &str) -> Result<Self::Binding, Error> {
        let lib_path = self
            .dir
            .path()
            .join(self.shared_lib_path(model_identifier)?);
        log::trace!("Loading shared library {:?}", lib_path);
        unsafe { binding::Fmi2Binding::new(lib_path).map_err(Error::from) }
    }
}

impl Fmi2Import {
    /// Create a new instance of the FMU for Model-Exchange
    pub fn instantiate_me(
        &self,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> Result<Instance<'_, ME>, Error> {
        Instance::<'_, ME>::new(self, instance_name, visible, logging_on)
    }

    /// Create a new instance of the FMU for Co-Simulation
    pub fn instantiate_cs(
        &self,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> Result<Instance<'_, CS>, Error> {
        Instance::<'_, CS>::new(self, instance_name, visible, logging_on)
    }
}
