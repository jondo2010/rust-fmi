//! Minimal FMI definitions for determining FMI version.
//!
//! ```rust
//! # use fmi_schema::{minimal::MinModelDescription, traits::FmiModelDescription};
//! # use std::str::FromStr;
//! let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
//!     <fmiModelDescription fmiVersion="2.0" modelName="BouncingBall">
//!     </fmiModelDescription>"#;
//! let md = MinModelDescription::from_str(xml).unwrap();
//! let version = md.version().unwrap();
//! assert_eq!(version, semver::Version::new(2, 0, 0));
//! ```

use std::str::FromStr;

use yaserde_derive::YaDeserialize;

use crate::traits::FmiModelDescription;

/// A minimal model description that only contains the FMI version
/// This is used to determine the FMI version of the FMU
#[derive(Default, PartialEq, Debug, YaDeserialize)]
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
}

impl FromStr for MinModelDescription {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        yaserde::de::from_str(s).map_err(crate::Error::XmlParse)
    }
}
