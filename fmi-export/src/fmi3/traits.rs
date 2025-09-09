use std::{fmt::Display, str::FromStr};

use fmi::{
    EventFlags,
    fmi3::{Fmi3Error, Fmi3Res, Fmi3Status, binding},
};

use crate::fmi3::{ModelState, instance::ModelContext};

/// Macro to generate getter and setter method declarations for the Model trait
macro_rules! model_getter_setter {
    ($name:ident, $ty:ty) => {
        paste::paste! {
            /// Get [<$name>] values from the model
            fn [<get_ $name>](
                &mut self,
                vrs: &[Self::ValueRef],
                values: &mut [$ty],
                context: &ModelContext<Self>,
            ) -> Result<Fmi3Res, Fmi3Error> {
                let _ = (vrs, values, context);
                Err(Fmi3Error::Error)
            }

            /// Set [<$name>] values in the model
            fn [<set_ $name>](
                &mut self,
                vrs: &[Self::ValueRef],
                values: &[$ty],
                context: &ModelContext<Self>,
            ) -> Result<Fmi3Res, Fmi3Error> {
                let _ = (vrs, values, context);
                Err(Fmi3Error::Error)
            }
        }
    };
}

/// Macro for special getter/setter pairs that have different return types
macro_rules! model_getter_setter_special {
    (string) => {
        /// Get string values from the model
        fn get_string(
            &mut self,
            vrs: &[Self::ValueRef],
            values: &mut [std::ffi::CString],
            context: &ModelContext<Self>,
        ) -> Result<(), Fmi3Error> {
            let _ = (vrs, values, context);
            Err(Fmi3Error::Error)
        }

        /// Set string values in the model
        fn set_string(
            &mut self,
            vrs: &[Self::ValueRef],
            values: &[std::ffi::CString],
            context: &ModelContext<Self>,
        ) -> Result<(), Fmi3Error> {
            let _ = (vrs, values, context);
            Err(Fmi3Error::Error)
        }
    };
    (binary) => {
        /// Get binary values from the model
        fn get_binary(
            &mut self,
            vrs: &[Self::ValueRef],
            values: &mut [&mut [u8]],
            context: &ModelContext<Self>,
        ) -> Result<Vec<usize>, Fmi3Error> {
            let _ = (vrs, values, context);
            Err(Fmi3Error::Error)
        }

        /// Set binary values in the model
        fn set_binary(
            &mut self,
            vrs: &[Self::ValueRef],
            values: &[&[u8]],
            context: &ModelContext<Self>,
        ) -> Result<(), Fmi3Error> {
            let _ = (vrs, values, context);
            Err(Fmi3Error::Error)
        }
    };
}

/// Model trait. This trait should be implementing by deriving `FmuModel` on the user model struct.
///
/// It provides the necessary back-end functionality for the FMI 3.0 API, delegating user-specific
/// behavior to the `UserModel` trait.
pub trait Model: Default + UserModel {
    type ValueRef: Copy + From<binding::fmi3ValueReference> + Into<binding::fmi3ValueReference>;

    const MODEL_NAME: &'static str;
    const MODEL_VARIABLES_XML: &'static str;
    const MODEL_STRUCTURE_XML: &'static str;
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

    // GetSet methods now absorbed into Model trait using macro

    model_getter_setter!(boolean, bool);
    model_getter_setter!(float32, f32);
    model_getter_setter!(float64, f64);
    model_getter_setter!(int8, i8);
    model_getter_setter!(int16, i16);
    model_getter_setter!(int32, i32);
    model_getter_setter!(int64, i64);
    model_getter_setter!(uint8, u8);
    model_getter_setter!(uint16, u16);
    model_getter_setter!(uint32, u32);
    model_getter_setter!(uint64, u64);

    // Special getter/setter methods with different signatures
    model_getter_setter_special!(string);
    model_getter_setter_special!(binary);
}

pub trait ModelLoggingCategory: Display + FromStr + Ord + Copy + Default {
    /// Return an iterator over all possible logging categories
    fn all_categories() -> impl Iterator<Item = Self>;
    /// Get the category for tracing FMI API calls
    fn trace_category() -> Self;
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
