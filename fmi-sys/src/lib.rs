#![doc=include_str!( "../README.md")]
//! ## Feature flags
#![doc = document_features::document_features!()]
#![deny(clippy::all)]

#[cfg(feature = "fmi2")]
pub mod fmi2;
#[cfg(feature = "fmi3")]
pub mod fmi3;
#[cfg(feature = "ls-bus")]
pub mod ls_bus;
