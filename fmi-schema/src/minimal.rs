//! Minimal FMI definitions for determining FMI version.
//!
//! ```rust
//! # use fmi_schema::minimal::ModelDescription;
//! # use std::str::FromStr;
//! let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
//!     <fmiModelDescription fmiVersion="2.0" modelName="BouncingBall">
//!     </fmiModelDescription>"#;
//! let md = ModelDescription::from_str(xml).unwrap();
//! let version = md.version().unwrap();
//! assert_eq!(version, semver::Version::new(2, 0, 0));
//! ```

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
