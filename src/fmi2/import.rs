use std::path::PathBuf;

use crate::{import::FmiImport, FmiError, FmiResult};

use super::{binding, schema};

#[derive(Debug)]
pub struct Fmi2 {
    /// Path to the unzipped FMU on disk
    dir: tempfile::TempDir,
    /// Parsed raw-schema model description
    schema: schema::FmiModelDescription,
}

impl FmiImport for Fmi2 {
    type Schema = schema::FmiModelDescription;
    type Binding = binding::Fmi2Binding;

    fn new(dir: tempfile::TempDir, schema_xml: &str) -> FmiResult<Self> {
        let schema: schema::FmiModelDescription =
            yaserde::de::from_str(schema_xml).map_err(FmiError::Parse)?;
        Ok(Self { dir, schema })
    }

    #[inline]
    fn path(&self) -> &std::path::Path {
        self.dir.path()
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

    /// Get a reference to the raw-schema model description
    fn raw_schema(&self) -> &Self::Schema {
        &self.schema
    }

    /// Load the plugin shared library and return the raw bindings.
    fn raw_bindings(&self) -> FmiResult<Self::Binding> {
        let lib_path = self.dir.path().join(self.shared_lib_path()?);
        log::trace!("Loading shared library {:?}", lib_path);
        unsafe { binding::Fmi2Binding::new(lib_path).map_err(FmiError::from) }
    }
}
