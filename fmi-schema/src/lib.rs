#![doc=include_str!( "../README.md")]
//! ## Feature flags
#![doc = document_features::document_features!()]
#![deny(unsafe_code)]
#![deny(clippy::all)]

use std::fmt::Display;

use thiserror::Error;
use yaserde::{YaDeserialize, YaSerialize};

pub mod date_time;
#[cfg(feature = "fmi2")]
pub mod fmi2;
#[cfg(feature = "fmi3")]
pub mod fmi3;
pub mod minimal;
pub mod traits;
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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error {
    #[error("Variable {0} not found")]
    VariableNotFound(String),

    #[error(transparent)]
    Semver(#[from] lenient_semver::parser::OwnedError),

    #[error("Error parsing XML: {0}")]
    XmlParse(String),

    #[error("Error in model: {0}")]
    Model(String),
}

/// A helper function to provide a default value for types that implement `Default`.
/// This is used in the schema definitions to provide default values for fields.
#[inline]
fn default_wrapper<T: Default>() -> T {
    T::default()
}

pub fn serialize<T: YaSerialize>(value: &T) -> Result<String, Error> {
    yaserde::ser::to_string(value).map_err(Error::XmlParse)
}

pub fn deserialize<T: YaDeserialize>(xml: &str) -> Result<T, Error> {
    yaserde::de::from_str(xml).map_err(|e| Error::XmlParse(e.to_string()))
}
