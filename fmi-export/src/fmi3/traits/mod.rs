use std::{fmt::Display, str::FromStr};

use fmi::{
    EventFlags,
    fmi3::{Fmi3Error, Fmi3Res, Fmi3Status, binding},
    schema::fmi3::AppendToModelVariables,
};

use crate::fmi3::{ModelState, instance::ModelContext};

mod model_get_set;
mod wrappers;

pub use model_get_set::{ModelGetSet, ModelGetSetStates};
pub use wrappers::{Fmi3CoSimulation, Fmi3Common, Fmi3ModelExchange};

/// Model trait. This trait should be implementing by deriving `FmuModel` on the user model struct.
///
/// It provides the necessary back-end functionality for the FMI 3.0 API, delegating user-specific
/// behavior to the `UserModel` trait.
pub trait Model: Default + UserModel {
    const MODEL_NAME: &'static str;
    const INSTANTIATION_TOKEN: &'static str;

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
        let time = fmi::schema::fmi3::FmiFloat64::new_time(None);
        AppendToModelVariables::append_to_variables(time, &mut variables);
        let mut structure = fmi::schema::fmi3::ModelStructure::default();
        let _num_vars = Self::build_metadata(&mut variables, &mut structure, 1);
        (variables, structure)
    }

    /// Set start values
    fn set_start_values(&mut self);

    /// Get the number of event indicators
    fn get_number_of_event_indicators() -> usize;

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

    fn configurate(&mut self) -> Fmi3Status {
        // Basic configuration - in a full implementation, this would:
        // - Allocate memory for event indicators if needed
        // - Allocate memory for continuous states if needed
        // - Initialize event indicator values
        // For now, just return OK since our basic implementation doesn't need these
        Fmi3Res::OK.into()
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

/// User-defined model behavior trait
///
/// This trait should be hand-implemented by the user to define the specific behavior of their model.
pub trait UserModel: Sized {
    /// The logging category type for this model
    ///
    /// This is an enum that implements `ModelLoggingCategory`
    type LoggingCategory: ModelLoggingCategory + 'static;

    /// Calculate values (derivatives, outputs, etc.)
    /// This method is called whenever the model needs to update its calculated values
    fn calculate_values(&mut self, _context: &ModelContext<Self>) -> Result<Fmi3Res, Fmi3Error> {
        Ok(Fmi3Res::OK)
    }

    /// Event update function for Model Exchange
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
        _context: &ModelContext<Self>,
        event_flags: &mut EventFlags,
    ) -> Result<Fmi3Res, Fmi3Error> {
        event_flags.reset();
        Ok(Fmi3Res::OK)
    }

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
        _context: &ModelContext<Self>,
        indicators: &mut [f64],
    ) -> Result<bool, Fmi3Error> {
        // Default implementation: no event indicators
        for indicator in indicators.iter_mut() {
            *indicator = 0.0;
        }
        Ok(true)
    }
}
