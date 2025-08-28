use std::{fmt::Display, str::FromStr};

use fmi::fmi3::{Fmi3Error, Fmi3Res, Fmi3Status, GetSet, binding};

use crate::fmi3::{ModelState, instance::ModelContext};

/// Model trait. This trait should be implementing by deriving `FmuModel` on the user model struct.
///
/// It provides the necessary back-end functionality for the FMI 3.0 API, delegating user-specific
/// behavior to the `UserModel` trait.
pub trait Model: Default + GetSet + UserModel {
    const MODEL_NAME: &'static str;
    const MODEL_DESCRIPTION: &'static str;
    const INSTANTIATION_TOKEN: &'static str;

    /// Set start values
    fn set_start_values(&mut self);

    /// Get continuous states from the model
    /// Returns the current values of all continuous state variables
    fn get_continuous_states(&self, states: &mut [f64]) -> Result<Fmi3Res, Fmi3Error>;

    /// Set continuous states in the model
    /// Sets new values for all continuous state variables
    fn set_continuous_states(&mut self, states: &[f64]) -> Result<Fmi3Res, Fmi3Error>;

    /// Get derivatives of continuous states
    /// Returns the first-order time derivatives of all continuous state variables
    fn get_continuous_state_derivatives(
        &mut self,
        derivatives: &mut [f64],
        context: &ModelContext<Self>,
    ) -> Result<Fmi3Res, Fmi3Error>;

    /// Get the number of continuous states
    fn get_number_of_continuous_states() -> usize;

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
}

/// User-defined model behavior trait
///
/// This trait should be hand-implemented by the user to define the specific behavior of their model.
pub trait UserModel: Sized {
    /// The logging category type for this model
    ///
    /// This is typically an enum that implements `ModelLoggingCategory`
    type LoggingCategory: ModelLoggingCategory + 'static;

    /// Calculate values (derivatives, outputs, etc.)
    /// This method is called whenever the model needs to update its calculated values
    fn calculate_values(&mut self, context: &ModelContext<Self>) -> Fmi3Status {
        Fmi3Res::OK.into()
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
    fn event_update(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Ok(Fmi3Res::OK)
    }

    /// Get event indicators for zero-crossing detection
    /// Returns the current values of event indicators
    fn get_event_indicators(&mut self, indicators: &mut [f64]) -> Result<Fmi3Res, Fmi3Error> {
        // Default implementation: no event indicators
        for indicator in indicators.iter_mut() {
            *indicator = 0.0;
        }
        Ok(Fmi3Res::OK)
    }
}
