//! Minimal FMI definitions for determining FMI version.
//!
//! ```rust
//! # use fmi_schema::{minimal::MinModelDescription, traits::FmiModelDescription};
//! let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
//!     <fmiModelDescription fmiVersion="2.0" modelName="BouncingBall">
//!     </fmiModelDescription>"#;
//! let md: MinModelDescription = fmi_schema::deserialize(xml).unwrap();
//! let version = md.version().unwrap();
//! assert_eq!(version, semver::Version::new(2, 0, 0));
//! ```

use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::traits::FmiModelDescription;

/// A minimal model description that only contains the FMI version
/// This is used to determine the FMI version of the FMU
#[derive(Default, PartialEq, Debug, YaDeserialize, YaSerialize)]
#[yaserde(rename = "fmiModelDescription")]
pub struct MinModelDescription {
    #[yaserde(attribute = true, rename = "fmiVersion")]
    pub fmi_version: String,
    #[yaserde(attribute = true, rename = "modelName")]
    pub model_name: String,
}

impl FmiModelDescription for MinModelDescription {
    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn version_string(&self) -> &str {
        &self.fmi_version
    }

    fn deserialize(xml: &str) -> Result<Self, crate::Error> {
        yaserde::de::from_str(xml).map_err(crate::Error::XmlParse)
    }

    fn serialize(&self) -> Result<String, crate::Error> {
        yaserde::ser::to_string(self).map_err(crate::Error::XmlParse)
    }
}
