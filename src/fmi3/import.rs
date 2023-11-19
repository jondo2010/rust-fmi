use std::path::PathBuf;

use tempfile::TempDir;

use crate::{import::FmiImport, FmiError, FmiResult};

use self::instance::Instance;

use super::{
    binding,
    instance::{self, ME},
    model, schema,
};

/// FMU import for FMI 3.0
///
///
#[derive(Debug)]
pub struct Fmi3 {
    /// Path to the unzipped FMU on disk
    dir: tempfile::TempDir,
    /// Derived model description
    model: model::ModelDescription,
}

impl FmiImport for Fmi3 {
    /// Create a new FMI 3.0 import from a directory containing the unzipped FMU
    fn new(dir: TempDir, schema_xml: &str) -> FmiResult<Self> {
        // Parsed raw-schema model description
        let schema: schema::FmiModelDescription =
            yaserde::de::from_str(schema_xml).map_err(|err| FmiError::Parse(err))?;

        let model = model::ModelDescription::try_from(schema).map_err(FmiError::from)?;

        Ok(Self { dir, model })
    }

    #[inline]
    fn path(&self) -> &std::path::Path {
        self.dir.path()
    }
}

impl Fmi3 {
    /// Get a reference to the raw-schema model description
    //pub fn raw_schema(&self) -> &schema::FmiModelDescription {
    //    &self.schema
    //}

    /// Build a derived model description from the raw-schema model description
    pub fn model(&self) -> &model::ModelDescription {
        &self.model
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
        let model_name = &self.model.model_name;
        let fname = format!("{model_name}{}", std::env::consts::DLL_SUFFIX);
        Ok(std::path::PathBuf::from("binaries")
            .join(platform_folder)
            .join(fname))
    }

    /// Load the plugin shared library and return the raw bindings.
    pub fn raw_bindings(&self) -> FmiResult<binding::Fmi3Binding> {
        let lib_path = self.dir.path().join(self.shared_lib_path()?);
        log::trace!("Loading shared library {:?}", lib_path);
        unsafe { binding::Fmi3Binding::new(lib_path).map_err(FmiError::from) }
    }

    /// Create a new instance of the FMU for Model-Exchange
    pub fn instantiate_me(
        &self,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> FmiResult<Instance<'_, ME>> {
        instance::Instance::new(self, instance_name, visible, logging_on)
    }
}
