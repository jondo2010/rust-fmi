//! This crate provides a Rust representation of the FMI schema.
//!
//! The refernce XSI can be found at https://fmi-standard.org/downloads.
//!
//! # Determining the FMI version
//!
//! FMI2.0 and 3.0 have different XML schemas. The FMI version can initially be determined in a non-specific way by
//! using [`minimal::ModelDescription`].
//!
//! # Example
//!
//! ```rust
//!
//! ```

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
