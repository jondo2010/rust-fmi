use std::path::PathBuf;

use crate::{import::FmiImport, FmiError, FmiResult};

use super::{binding, meta};

#[derive(Debug)]
pub struct Fmi2 {
    /// Path to the unzipped FMU on disk
    dir: tempfile::TempDir,
    /// Parsed raw-schema model description
    schema: meta::ModelDescription,
}

impl FmiImport for Fmi2 {
    #[inline]
    fn path(&self) -> &std::path::Path {
        self.dir.path()
    }

    fn new(dir: tempfile::TempDir, schema_xml: &str) -> FmiResult<Self> {
        let schema: meta::ModelDescription =
            serde_xml_rs::from_str(schema_xml).map_err(FmiError::from)?;
        Ok(Self { dir, schema })
    }
}

#[cfg(feature = "fmi2")]
impl Fmi2 {
    /// Get a reference to the raw-schema model description
    pub fn raw_schema(&self) -> &meta::ModelDescription {
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
    pub fn raw_bindings(&self) -> FmiResult<binding::Fmi2Binding> {
        let lib_path = self.dir.path().join(self.shared_lib_path()?);
        log::trace!("Loading shared library {:?}", lib_path);
        unsafe { binding::Fmi2Binding::new(lib_path).map_err(FmiError::from) }
    }
}
