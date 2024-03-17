//! Common traits for FMI schema

use crate::MajorVersion;

pub trait DefaultExperiment {
    fn start_time(&self) -> Option<f64>;
    fn stop_time(&self) -> Option<f64>;
    fn tolerance(&self) -> Option<f64>;
    fn step_size(&self) -> Option<f64>;
}

pub trait FmiModelDescription {
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
}
