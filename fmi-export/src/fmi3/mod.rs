//! ## Architecture
//!
//! The [`crate::export_fmu`] macro generates the necessary C-API bindings for the exported FMU.
//! Many of these bindings operate on a [`binding::fmi3Instance`], which is an opaque pointer to an
//! instance of [`instance::ModelInstance`].
//!
//! [`instance::ModelInstance`] implements the [`Common`] trait, which provides the actual implementation of
//! the FMI 3.0 API. All user-model-specific functions are delegated to the [`Model`] trait,
//! which the user model must implement.

// Disable coverage for the export module. Due to some bug in llvm-cov, the generated C functions
// are never covered. This is ok since they are just thin wrappers around the Rust functions.
#[cfg_attr(coverage_nightly, coverage(off))]
mod export;

mod instance;
mod traits;
mod types;
mod variable_builder;

use std::{fmt::Display, str::FromStr};

// Re-exports
pub use instance::{context::BasicContext, ModelInstance};
pub use traits::{
    Context, CSDoStepResult, Fmi3CoSimulation, Fmi3Common, Fmi3ModelExchange,
    Fmi3ScheduledExecution, Model, ModelGetSet, ModelGetSetStates, ModelLoggingCategory, UserModel,
    UserModelCS, UserModelME, UserModelSE,
};
pub use types::{Binary, Clock, InitializeFromStart};
pub use variable_builder::{FmiVariableBuilder, VariableBuilder};

/// Specifies how Co-Simulation is implemented for a model
/// Represents the current state of the model instance
#[derive(Debug)]
pub enum ModelState {
    StartAndEnd,
    ConfigurationMode,
    /// In the state `Instantiated` the FMU can do one-time initializations and allocate memory.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#Instantiated>
    Instantiated,
    /// The `InitializationMode` is used by the simulation algorithm to compute consistent initial
    /// conditions for the overall system. Equations are active to determine the initial FMU state,
    /// as well as all outputs (and optionally other variables exposed by the exporting tool).
    /// Artificial or real algebraic loops over connected FMUs in Initialization Mode may be handled
    /// by using appropriate numerical algorithms.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#InitializationMode>
    InitializationMode,
    /// In `EventMode` all continuous-time, discrete-time equations and active model partitions are
    /// evaluated. Algebraic loops active during Event Mode are solved by event iteration.
    ///
    /// See <https://fmi-standard.org/docs/3.0.1/#EventMode>
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
    LogAll,
    /// Trace FMI API calls
    Trace,
}

impl Display for DefaultLoggingCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DefaultLoggingCategory::LogAll => f.write_str("logAll"),
            DefaultLoggingCategory::Trace => f.write_str("trace"),
        }
    }
}

impl FromStr for DefaultLoggingCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "logAll" => Ok(Self::LogAll),
            "trace" => Ok(Self::Trace),
            _ => Err(format!("Unknown logging category: {}", s)),
        }
    }
}

impl ModelLoggingCategory for DefaultLoggingCategory {
    fn all_categories() -> impl Iterator<Item = Self> {
        [Self::LogAll, Self::Trace].into_iter()
    }
    fn trace_category() -> Self {
        Self::Trace
    }
    fn error_category() -> Self {
        Self::LogAll
    }
}
