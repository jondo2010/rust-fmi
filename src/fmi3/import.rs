use std::{path::PathBuf, str::FromStr};

use tempfile::TempDir;

use crate::{import::FmiImport, Error};

use super::{
    binding,
    instance::{Instance, CS, ME},
    schema,
};

/// FMU import for FMI 3.0
#[derive(Debug)]
pub struct Fmi3 {
    /// Path to the unzipped FMU on disk
    dir: tempfile::TempDir,
    /// Parsed raw-schema model description
    schema: schema::FmiModelDescription,
}

impl FmiImport for Fmi3 {
    type Schema = schema::FmiModelDescription;
    type Binding = binding::Fmi3Binding;

    /// Create a new FMI 3.0 import from a directory containing the unzipped FMU
    fn new(dir: TempDir, schema_xml: &str) -> Result<Self, Error> {
        let schema = schema::FmiModelDescription::from_str(schema_xml)?;
        Ok(Self { dir, schema })
    }

    #[inline]
    fn archive_path(&self) -> &std::path::Path {
        self.dir.path()
    }

    /// Get the path to the shared library
    fn shared_lib_path(&self, model_identifier: &str) -> Result<PathBuf, Error> {
        use std::env::consts::{ARCH, OS};
        let platform_folder = match (OS, ARCH) {
            ("windows", "x86_64") => "x86_64-windows",
            ("windows", "x86") => "x86-windows",
            ("linux", "x86_64") => "x86_64-linux",
            ("linux", "x86") => "x86-linux",
            ("macos", "x86_64") => "x86-64-darwin",
            ("macos", "x86") => "x86-darwin",
            ("macos", "aarch64") => "aarch64-darwin",
            _ => panic!("Unsupported platform: {OS} {ARCH}"),
        };
        let fname = format!("{model_identifier}{}", std::env::consts::DLL_SUFFIX);
        Ok(std::path::PathBuf::from("binaries")
            .join(platform_folder)
            .join(fname))
    }

    /// Get the parsed raw-schema model description
    fn model_description(&self) -> &Self::Schema {
        &self.schema
    }

    /// Load the plugin shared library and return the raw bindings.
    fn binding(&self, model_identifier: &str) -> Result<Self::Binding, Error> {
        let lib_path = self
            .dir
            .path()
            .join(self.shared_lib_path(model_identifier)?);
        log::debug!("Loading shared library {:?}", lib_path);
        unsafe { binding::Fmi3Binding::new(lib_path).map_err(Error::from) }
    }
}

impl Fmi3 {
    /// Build a derived model description from the raw-schema model description
    #[cfg(feature = "disabled")]
    pub fn model(&self) -> &model::ModelDescription {
        &self.model
    }

    /// Create a new instance of the FMU for Model-Exchange
    ///
    /// See [`Instance::<ME>::new()`] for more information.
    pub fn instantiate_me(
        &self,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> Result<Instance<'_, ME>, Error> {
        Instance::<'_, ME>::new(self, instance_name, visible, logging_on)
    }

    /// Create a new instance of the FMU for Co-Simulation
    ///
    /// See [`Instance::<CS>::new()`] for more information.
    pub fn instantiate_cs(
        &self,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
        event_mode_used: bool,
        early_return_allowed: bool,
        required_intermediate_variables: &[binding::fmi3ValueReference],
    ) -> Result<Instance<'_, CS>, Error> {
        Instance::<'_, CS>::new(
            self,
            instance_name,
            visible,
            logging_on,
            event_mode_used,
            early_return_allowed,
            required_intermediate_variables,
        )
    }
}
