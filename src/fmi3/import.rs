use std::path::PathBuf;

use tempfile::TempDir;

use crate::{import::FmiImport, FmiError, FmiResult};

use self::instance::Instance;

use super::{binding, instance, model, schema};

#[derive(Debug)]
#[repr(usize)]
pub enum FmiStatus {
    Ok = binding::fmi3Status_fmi3OK as _,
    Warning = binding::fmi3Status_fmi3Warning as _,
    Discard = binding::fmi3Status_fmi3Discard as _,
    Error = binding::fmi3Status_fmi3Error as _,
    Fatal = binding::fmi3Status_fmi3Fatal as _,
}

impl From<binding::fmi3Status> for FmiStatus {
    fn from(status: binding::fmi3Status) -> Self {
        match status {
            binding::fmi3Status_fmi3OK => FmiStatus::Ok,
            binding::fmi3Status_fmi3Warning => FmiStatus::Warning,
            binding::fmi3Status_fmi3Discard => FmiStatus::Discard,
            binding::fmi3Status_fmi3Error => FmiStatus::Error,
            binding::fmi3Status_fmi3Fatal => FmiStatus::Fatal,
            _ => unreachable!("Invalid status"),
        }
    }
}

/// Callback function for logging
unsafe extern "C" fn log_callback(
    _instance_environment: binding::fmi3InstanceEnvironment,
    instance_name: binding::fmi3String,
    status: binding::fmi3Status,
    category: binding::fmi3String,
    message: binding::fmi3String,
) {
    let instance_name = std::ffi::CStr::from_ptr(instance_name)
        .to_str()
        .unwrap_or("INVALID");
    let status = FmiStatus::from(status);
    let category = std::ffi::CStr::from_ptr(category)
        .to_str()
        .unwrap_or("INVALID");
    let message = std::ffi::CStr::from_ptr(message)
        .to_str()
        .unwrap_or("INVALID");

    log::log!(
        match status {
            FmiStatus::Ok => log::Level::Info,
            FmiStatus::Warning => log::Level::Warn,
            FmiStatus::Discard => log::Level::Warn,
            FmiStatus::Error => log::Level::Error,
            FmiStatus::Fatal => log::Level::Error,
        },
        "instanceName: {}, status: {:?}, category: {}, message: {}",
        instance_name,
        status,
        category,
        message
    );
}

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
    ) -> FmiResult<Instance<'_>> {
        let binding = self.raw_bindings()?;

        let instance_ptr = unsafe {
            binding.fmi3InstantiateModelExchange(
                instance_name.as_ptr() as binding::fmi3String,
                self.model.instantiation_token.as_ptr() as binding::fmi3String,
                self.model.instantiation_token.as_ptr() as binding::fmi3String,
                visible,
                logging_on,
                std::ptr::null_mut() as binding::fmi3InstanceEnvironment,
                Some(log_callback),
            )
        };

        if instance_ptr.is_null() {
            return Err(FmiError::Instantiation);
        }

        Ok(instance::Instance::new(binding, instance_ptr, &self.model))
    }
}
