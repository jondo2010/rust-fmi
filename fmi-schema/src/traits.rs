//! Common traits for FMI schema

use crate::MajorVersion;

pub trait DefaultExperiment {
    fn start_time(&self) -> Option<f64>;
    fn stop_time(&self) -> Option<f64>;
    fn tolerance(&self) -> Option<f64>;
    fn step_size(&self) -> Option<f64>;
}

/// A trait common between all FMI schema versions
pub trait FmiModelDescription: Sized {
    /// Returns the model name
    fn model_name(&self) -> &str;

    /// Returns the FMI version as a string
    fn version_string(&self) -> &str;

    /// Returns the parsed FMI version as a semver::Version
    fn version(&self) -> Result<semver::Version, crate::Error> {
        lenient_semver::parse(self.version_string()).map_err(|e| e.owned().into())
    }

    /// Returns the parsed FMI version as a MajorVersion
    fn major_version(&self) -> Result<MajorVersion, crate::Error> {
        match self.version()? {
            v if v.major == 1 => Ok(MajorVersion::FMI1),
            v if v.major == 2 => Ok(MajorVersion::FMI2),
            v if v.major == 3 => Ok(MajorVersion::FMI3),
            v => panic!("Invalid version {}", v.major),
        }
    }

    /// Deserialize the model description from XML
    fn deserialize(xml: &str) -> Result<Self, crate::Error>;

    /// Serialize the model description to XML
    fn serialize(&self) -> Result<String, crate::Error>;
}

/// A trait for FMI interface types (Model Exchange, Co-Simulation, Scheduled Execution) and versions
pub trait FmiInterfaceType: Sized {
    /// Returns the model identifier
    fn model_identifier(&self) -> &str;
    /// Returns true if the FMU needs an execution tool
    fn needs_execution_tool(&self) -> Option<bool>;
    /// Returns true if the FMU can be instantiated only once per process
    fn can_be_instantiated_only_once_per_process(&self) -> Option<bool>;
    /// Returns true if the FMU can get and set FMU state
    fn can_get_and_set_fmu_state(&self) -> Option<bool>;
    /// Returns true if the FMU can serialize FMU state
    fn can_serialize_fmu_state(&self) -> Option<bool>;
    /// Returns true if the FMU provides directional derivatives
    fn provides_directional_derivatives(&self) -> Option<bool>;
    /// Returns true if the FMU provides adjoint derivatives
    /// (only FMI 3.0)
    fn provides_adjoint_derivatives(&self) -> Option<bool>;
    /// Returns true if the FMU provides per element dependencies
    /// (only FMI 3.0)
    fn provides_per_element_dependencies(&self) -> Option<bool>;
}
