use std::path::PathBuf;

use tempfile::TempDir;

use crate::{import::FmiImport, FmiError, FmiResult};

use super::{
    binding,
    instance::{Instance, ME},
    model, schema,
};

/// FMU import for FMI 3.0
#[derive(Debug)]
pub struct Fmi3 {
    /// Path to the unzipped FMU on disk
    dir: tempfile::TempDir,
    /// Parsed raw-schema model description
    schema: schema::FmiModelDescription,
    /// Derived model description
    model: model::ModelDescription,
}

impl FmiImport for Fmi3 {
    type Schema = schema::FmiModelDescription;
    type Binding = binding::Fmi3Binding;

    /// Create a new FMI 3.0 import from a directory containing the unzipped FMU
    fn new(dir: TempDir, schema_xml: &str) -> FmiResult<Self> {
        let schema: schema::FmiModelDescription =
            yaserde::de::from_str(schema_xml).map_err(|err| FmiError::Parse(err))?;

        let model = model::ModelDescription::try_from(schema.clone()).map_err(FmiError::from)?;

        Ok(Self { dir, schema, model })
    }

    #[inline]
    fn path(&self) -> &std::path::Path {
        self.dir.path()
    }

    /// Get the path to the shared library
    fn shared_lib_path(&self) -> FmiResult<PathBuf> {
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
        let model_name = &self.model.model_name;
        let fname = format!("{model_name}{}", std::env::consts::DLL_SUFFIX);
        Ok(std::path::PathBuf::from("binaries")
            .join(platform_folder)
            .join(fname))
    }

    fn raw_schema(&self) -> &Self::Schema {
        &self.schema
    }

    /// Load the plugin shared library and return the raw bindings.
    fn raw_bindings(&self) -> FmiResult<Self::Binding> {
        let lib_path = self.dir.path().join(self.shared_lib_path()?);
        log::trace!("Loading shared library {:?}", lib_path);
        unsafe { binding::Fmi3Binding::new(lib_path).map_err(FmiError::from) }
    }
}

impl Fmi3 {
    /// Build a derived model description from the raw-schema model description
    pub fn model(&self) -> &model::ModelDescription {
        &self.model
    }

    /// Create a new instance of the FMU for Model-Exchange
    pub fn instantiate_me(
        &self,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> FmiResult<Instance<'_, ME>> {
        Instance::new(self, instance_name, visible, logging_on)
    }
}
