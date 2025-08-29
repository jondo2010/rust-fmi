//! ## Architecture
//!
//! The [`crate::export_fmu`] macro generates the necessary C-API bindings for the exported FMU.
//! Many of these bindings operate on a [`binding::fmi3Instance`], which is an opaque pointer to an
//! instance of [`instance::ModelInstance`].
//!
//! [`instance::ModelInstance`] implements the [`Common`] trait, which provides the actual implementation of
//! the FMI 3.0 API. All user-model-specific functions are delegated to the [`Model`] trait,
//! which the user model must implement.

mod instance;
mod macros;
mod traits;

use std::{fmt::Display, str::FromStr};

pub use instance::{ModelContext, ModelInstance};
pub use traits::{Model, ModelLoggingCategory, UserModel};

/// Represents the current state of the model instance
pub enum ModelState {
    StartAndEnd,
    ConfigurationMode,
    Instantiated,
    InitializationMode,
    EventMode,
    ContinuousTimeMode,
    StepMode,
    ClockActivationMode,
    StepDiscarded,
    ReconfigurationMode,
    IntermediateUpdateMode,
    Terminated,
}

/// Simple default logging category for models that don't need custom logging
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum DefaultLoggingCategory {
    #[default]
    Default,
}

impl Display for DefaultLoggingCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
        }
    }
}

impl FromStr for DefaultLoggingCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "default" => Ok(Self::Default),
            _ => Err(format!("Unknown logging category: {}", s)),
        }
    }
}

impl ModelLoggingCategory for DefaultLoggingCategory {
    fn all_categories() -> impl Iterator<Item = Self> {
        [Self::Default].iter().copied()
    }
}
