//! Minimal FMI definitions for determining FMI version.

use std::str::FromStr;

use yaserde_derive::YaDeserialize;

/// A minimal model description that only contains the FMI version
/// This is used to determine the FMI version of the FMU
#[derive(Default, PartialEq, Debug, YaDeserialize)]
#[yaserde(rename = "fmiModelDescription")]
pub struct ModelDescription {
    #[yaserde(attribute, rename = "fmiVersion")]
    pub fmi_version: String,
    #[yaserde(attribute, rename = "modelName")]
    pub model_name: String,
}

impl ModelDescription {
    /// Returns the parsed FMI version as a semver::Version
    pub fn version(&self) -> Result<semver::Version, crate::Error> {
        lenient_semver::parse(&self.fmi_version).map_err(|e| e.owned().into())
    }
}

impl FromStr for ModelDescription {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        yaserde::de::from_str(s).map_err(|e| crate::Error::XmlParse(e))
    }
}
