#![doc=include_str!( "../README.md")]
//! ## Feature flags
#![doc = document_features::document_features!()]
#![deny(clippy::all)]
#![deny(deref_nullptr)]
#![deny(integer_to_ptr_transmutes)]
#![deny(invalid_value)]
#![deny(invalid_from_utf8)]
#![deny(never_type_fallback_flowing_into_unsafe)]
#![deny(ptr_to_integer_transmute_in_consts)]
#![deny(static_mut_refs)]

#[cfg(feature = "fmi3")]
pub mod fmi3;

// Re-export the derive macro
#[doc = include_str!("fmu_model_docs.md")]
pub use fmi_export_derive::FmuModel;

// Re-export paste for use in macros
#[doc(hidden)]
pub use paste;
