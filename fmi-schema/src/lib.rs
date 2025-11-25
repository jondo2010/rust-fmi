#![doc=include_str!( "../README.md")]
//! ## Feature flags
#![doc = document_features::document_features!()]
#![deny(unsafe_code)]
#![deny(clippy::all)]

use std::fmt::Display;

use thiserror::Error;

pub mod date_time;
#[cfg(feature = "fmi2")]
pub mod fmi2;
#[cfg(feature = "fmi3")]
pub mod fmi3;
pub mod minimal;
pub mod traits;
pub mod utils;
pub mod variable_counts;

/// The major version of the FMI standard
#[derive(Debug, PartialEq, Eq)]
pub enum MajorVersion {
    FMI1,
    FMI2,
    FMI3,
}

impl Display for MajorVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MajorVersion::FMI1 => write!(f, "1.0"),
            MajorVersion::FMI2 => write!(f, "2.0"),
            MajorVersion::FMI3 => write!(f, "3.0"),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Variable {0} not found")]
    VariableNotFound(String),

    #[error(transparent)]
    Semver(#[from] lenient_semver::parser::OwnedError),

    #[error(transparent)]
    XmlParse(#[from] hard_xml::XmlError),

    #[error("Error in model: {0}")]
    Model(String),
}

/// Serialize a value to XML string. If `fragment` is true, the XML declaration is omitted.
pub fn serialize<T: hard_xml::XmlWrite>(value: &T, fragment: bool) -> Result<String, Error> {
    let xml = hard_xml::XmlWrite::to_string(value).map_err(Error::XmlParse)?;
    if fragment {
        Ok(xml)
    } else {
        Ok(format!(r#"<?xml version="1.0" encoding="UTF-8"?>{}"#, xml))
    }
}

pub fn deserialize<'a, T: hard_xml::XmlRead<'a>>(xml: &'a str) -> Result<T, Error> {
    hard_xml::XmlRead::from_str(xml).map_err(Error::XmlParse)
}
