use std::{fmt::Display, path::PathBuf, str::FromStr};

use fmi::{
    EventFlags,
    fmi3::{Fmi3Error, Fmi3Res, Fmi3Status, binding},
    schema::fmi3::AppendToModelVariables,
};

use crate::fmi3::ModelState;

mod model_get_set;
mod wrappers;

pub use model_get_set::{ModelGetSet, ModelGetSetStates};
pub use wrappers::{Fmi3CoSimulation, Fmi3Common, Fmi3ModelExchange, Fmi3ScheduledExecution};

/// Context trait for FMU instances
pub trait Context<M: UserModel> {
    /// Check if logging is enabled for the specified category.
    fn logging_on(&self, category: M::LoggingCategory) -> bool;

    /// Enable or disable logging for the specified category.
    fn set_logging(&mut self, category: M::LoggingCategory, enabled: bool);

    /// Log a message if the specified logging category is enabled.
    fn log(&self, status: Fmi3Status, category: M::LoggingCategory, args: std::fmt::Arguments<'_>);

    /// Get the path to the resources directory.
    fn resource_path(&self) -> &PathBuf;

    fn initialize(&mut self, start_time: f64, stop_time: Option<f64>);

    /// Get the current simulation time.
    fn time(&self) -> f64;

    /// Set the current simulation time.
    fn set_time(&mut self, time: f64);

    /// Get the simulation stop time, if any.
    fn stop_time(&self) -> Option<f64>;

    /// Whether early return is allowed for this instance (relevant for CS).
    fn early_return_allowed(&self) -> bool {
        false
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}

/// Model trait. This trait should be implementing by deriving `FmuModel` on the user model struct.
///
/// It provides the necessary back-end functionality for the FMI 3.0 API, delegating user-specific
/// behavior to the `UserModel` trait.
pub trait Model: Default {
    const MODEL_NAME: &'static str;
    const INSTANTIATION_TOKEN: &'static str;

    /// Number of event indicators
    const MAX_EVENT_INDICATORS: usize;

    /// Whether this model supports Model Exchange interface
    const SUPPORTS_MODEL_EXCHANGE: bool;

    /// Whether this model supports Co-Simulation interface
    const SUPPORTS_CO_SIMULATION: bool;

    /// Whether this model supports Scheduled Execution interface
    const SUPPORTS_SCHEDULED_EXECUTION: bool;

    /// Recursively build the model variables and structure by appending to the provided
    /// `ModelVariables` and `ModelStructure` instances.
    ///
    /// Returns the number of variables that were added.
    fn build_metadata(
        variables: &mut fmi::schema::fmi3::ModelVariables,
        model_structure: &mut fmi::schema::fmi3::ModelStructure,
        vr_offset: u32,
    ) -> u32;

    /// Build the top-level model variables and structure, including the 'time' variable.
    fn build_toplevel_metadata() -> (
        fmi::schema::fmi3::ModelVariables,
        fmi::schema::fmi3::ModelStructure,
    ) {
        let mut variables = fmi::schema::fmi3::ModelVariables::default();
        let time = fmi::schema::fmi3::FmiFloat64::new(
            "time".to_string(),
            0,
            None,
            fmi::schema::fmi3::Causality::Independent,
            fmi::schema::fmi3::Variability::Continuous,
            None,
            None,
        );
        AppendToModelVariables::append_to_variables(time, &mut variables);
        let mut structure = fmi::schema::fmi3::ModelStructure::default();
        let _num_vars = Self::build_metadata(&mut variables, &mut structure, 1);
        (variables, structure)
    }

    /// Set start values
    fn set_start_values(&mut self);

    /// Validate that a variable can be set in the current model state
    /// This method should be implemented by the generated code to check
    /// causality and variability restrictions for each variable
    fn validate_variable_setting(
        vr: binding::fmi3ValueReference,
        state: &ModelState,
    ) -> Result<(), &'static str> {
        // Default implementation allows all variable setting
        // Generated implementations will provide specific validation rules
        let _ = (vr, state);
        Ok(())
    }
}

pub trait ModelLoggingCategory: Display + FromStr + Ord + Copy + Default {
    /// Return an iterator over all possible logging categories
    fn all_categories() -> impl Iterator<Item = Self>;
    /// Get the category for tracing FMI API calls
    fn trace_category() -> Self;
    /// Get the category for logging errors
    fn error_category() -> Self;
}

/// Result payload for a Co-Simulation `do_step` implementation.
#[derive(Debug, Clone, Copy, Default)]
pub struct CSDoStepResult {
    pub event_handling_needed: bool,
    pub terminate_simulation: bool,
    pub early_return: bool,
    pub last_successful_time: f64,
}

impl CSDoStepResult {
    pub fn completed(last_successful_time: f64) -> Self {
        Self {
            event_handling_needed: false,
            terminate_simulation: false,
            early_return: false,
            last_successful_time,
        }
    }
}

/// User-defined model behavior trait
///
/// This trait should be hand-implemented by the user to define the specific behavior of their model.
pub trait UserModel: Sized {
    /// The logging category type for this model
    ///
    /// This is an enum that implements `ModelLoggingCategory`
    type LoggingCategory: ModelLoggingCategory + 'static;

    /// Configure the model (allocate memory, initialize states, etc.)
    /// This method is called upon exiting initialization mode
    fn configurate(&mut self, _context: &dyn Context<Self>) -> Result<(), Fmi3Error> {
        Ok(())
    }

    /// Calculate values (derivatives, outputs, etc.)
    /// This method is called whenever the model needs to update its calculated values
    fn calculate_values(&mut self, _context: &dyn Context<Self>) -> Result<Fmi3Res, Fmi3Error> {
        Ok(Fmi3Res::OK)
    }

    /// Called to update discrete states and check for events
    ///
    /// This method should:
    /// - Update any discrete state variables
    /// - Check for state events and time events
    /// - Set appropriate flags to indicate what has changed
    ///
    /// Returns Ok with the appropriate Fmi3Res status, or Err if an error occurs
    fn event_update(
        &mut self,
        _context: &dyn Context<Self>,
        event_flags: &mut EventFlags,
    ) -> Result<Fmi3Res, Fmi3Error> {
        event_flags.reset();
        Ok(Fmi3Res::OK)
    }
}

pub trait UserModelME: UserModel {
    /// Get event indicators for zero-crossing detection
    ///
    /// # Returns
    /// - `Ok(true)` if event indicators were successfully computed
    /// - `Ok(false)` if the FMU was not able to compute the event indicators because, for example,
    ///     a numerical issue such as division by zero occurred (corresponding to the C API
    ///     returning fmi3Discard)
    /// - `Err(Fmi3Error)` for other error conditions
    fn get_event_indicators(
        &mut self,
        _context: &dyn Context<Self>,
        indicators: &mut [f64],
    ) -> Result<bool, Fmi3Error> {
        // Default implementation: no event indicators
        for indicator in indicators.iter_mut() {
            *indicator = 0.0;
        }
        Ok(true)
    }
}

/// Implement this trait on your model for Co-Simulation support.
pub trait UserModelCS: UserModel {
    fn do_step(
        &mut self,
        context: &mut dyn Context<Self>,
        current_communication_point: f64,
        communication_step_size: f64,
        no_set_fmu_state_prior_to_current_point: bool,
    ) -> Result<CSDoStepResult, Fmi3Error>;
}

/// Implement this trait on your model to enable an FMU wrapper for Co-Simulation.
///
/// A fixed-step solver will be used to advance the simulation time.
/// Implement this trait on your model for Scheduled Execution support.
pub trait UserModelSE: UserModel {}
