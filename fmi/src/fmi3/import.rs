use std::{path::PathBuf, str::FromStr};

use fmi_schema::MajorVersion;
use tempfile::TempDir;

use crate::{
    fmi3::{binding, instance, schema, Fmi3Model},
    traits::FmiImport,
    Error,
};

/// FMU import for FMI 3.0
#[derive(Debug)]
pub struct Fmi3Import {
    /// Path to the unzipped FMU on disk
    dir: tempfile::TempDir,
    /// Parsed raw-schema model description
    model_description: schema::Fmi3ModelDescription,
}

impl FmiImport for Fmi3Import {
    const MAJOR_VERSION: MajorVersion = MajorVersion::FMI3;
    type ModelDescription = schema::Fmi3ModelDescription;
    type Binding = binding::Fmi3Binding;
    type ValueRef = binding::fmi3ValueReference;

    /// Create a new FMI 3.0 import from a directory containing the unzipped FMU
    fn new(dir: TempDir, schema_xml: &str) -> Result<Self, Error> {
        let model_description = schema::Fmi3ModelDescription::from_str(schema_xml)?;
        Ok(Self {
            dir,
            model_description,
        })
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
    fn model_description(&self) -> &Self::ModelDescription {
        &self.model_description
    }

    /// Load the plugin shared library and return the raw bindings.
    fn binding(&self, model_identifier: &str) -> Result<Self::Binding, Error> {
        let lib_path = self
            .dir
            .path()
            .join(self.shared_lib_path(model_identifier)?);
        log::debug!("Loading shared library {lib_path:?}");
        unsafe { binding::Fmi3Binding::new(lib_path).map_err(Error::from) }
    }

    /// Get a `String` representation of the resources path for this FMU
    ///
    /// As per the FMI3.0 standard, `resourcePath` is the absolute file path (with a trailing file separator) of the
    /// resources directory of the extracted FMU archive.
    fn canonical_resource_path_string(&self) -> String {
        std::path::absolute(self.resource_path())
            .expect("Invalid resource path")
            .to_str()
            .expect("Invalid resource path")
            .to_owned()
    }
}

impl<'a> Fmi3Model<'a> for Fmi3Import {
    type InstanceCS = instance::InstanceCS<'a>;
    type InstanceME = instance::InstanceME<'a>;
    type InstanceSE = instance::InstanceSE<'a>;

    /// Build a derived model description from the raw-schema model description
    #[cfg(false)]
    pub fn model(&self) -> &model::ModelDescription {
        &self.model
    }

    /// Create a new instance of the FMU for Model-Exchange
    ///
    /// See [`Instance::<ME>::new()`] for more information.
    fn instantiate_me(
        &'a self,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> Result<Self::InstanceME, Error> {
        instance::InstanceME::new(self, instance_name, visible, logging_on)
    }

    /// Create a new instance of the FMU for Co-Simulation
    ///
    /// See [`Instance::<CS>::new()`] for more information.
    fn instantiate_cs(
        &'a self,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
        event_mode_used: bool,
        early_return_allowed: bool,
        required_intermediate_variables: &[binding::fmi3ValueReference],
    ) -> Result<Self::InstanceCS, Error> {
        instance::InstanceCS::new(
            self,
            instance_name,
            visible,
            logging_on,
            event_mode_used,
            early_return_allowed,
            required_intermediate_variables,
        )
    }

    /// Create a new instance of the FMU for Scheduled Execution
    ///
    /// See [`Instance::<SE>::new()`] for more information.
    fn instantiate_se(
        &'a self,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> Result<Self::InstanceSE, Error> {
        instance::InstanceSE::new(self, instance_name, visible, logging_on)
    }
}
