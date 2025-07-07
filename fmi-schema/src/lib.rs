#![doc=include_str!( "../README.md")]
//! ## Feature flags
#![doc = document_features::document_features!()]
#![deny(unsafe_code)]
#![deny(clippy::all)]

use thiserror::Error;

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

impl ToString for MajorVersion {
    fn to_string(&self) -> String {
        match self {
            MajorVersion::FMI1 => "1.0".to_string(),
            MajorVersion::FMI2 => "2.0".to_string(),
            MajorVersion::FMI3 => "3.0".to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Variable {0} not found")]
    VariableNotFound(String),

    #[error(transparent)]
    Semver(#[from] lenient_semver::parser::OwnedError),

    #[error("Error parsing XML: {0}")]
    XmlParse(String),
}

/// A helper function to provide a default value for types that implement `Default`.
/// This is used in the schema definitions to provide default values for fields.
#[inline]
fn default_wrapper<T: Default>() -> T {
    T::default()
}
