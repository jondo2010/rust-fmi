#![doc=include_str!( "../README.md")]
//! ## Feature flags
#![doc = document_features::document_features!()]
#![deny(clippy::all)]

#[cfg(feature = "fmi3")]
pub mod fmi3;

mod store;
