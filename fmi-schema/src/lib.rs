#![doc=include_str!( "../README.md")]
//!
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
pub mod variable_counts;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Variable {0} not found")]
    VariableNotFound(String),

    #[error(transparent)]
    Semver(#[from] lenient_semver::parser::OwnedError),

    #[error("Error parsing XML: {0}")]
    XmlParse(String),
}
