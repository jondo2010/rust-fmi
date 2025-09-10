#![deny(clippy::all)]
//! Example port of the Stair FMU from the Reference FMUs
//!
//! This implements a discrete event model that increments a counter
//! every second, demonstrating event handling and discrete variables.

use fmi::{
    fmi3::{Fmi3Error, Fmi3Res},
    EventFlags,
};
use fmi_export::{
    fmi3::{DefaultLoggingCategory, ModelContext, UserModel},
    FmuModel,
};

/// Stair FMU model implementing a discrete counter that increments every second
///
/// This model demonstrates:
/// - Discrete event handling with time events
/// - Integer variables
/// - Model termination conditions
/// - Fixed step size for Co-Simulation
#[derive(FmuModel, Default, Debug)]
#[model()]
struct Stair {
    /// Counter that increments every second
    #[variable(causality = Output, variability = Discrete, start = 1, initial = Exact)]
    counter: i32,
}

impl UserModel for Stair {
    type LoggingCategory = DefaultLoggingCategory;

    fn event_update(
        &mut self,
        context: &ModelContext<Self>,
        event_flags: &mut EventFlags,
    ) -> Result<Fmi3Res, Fmi3Error> {
        let epsilon = (1.0 + context.time().abs()) * f64::EPSILON;

        if let Some(next_event) = &mut event_flags.next_event_time {
            if (context.time() + epsilon) >= *next_event {
                self.counter += 1;
                *next_event += 1.0; // Schedule next event in 1 second
            }
        } else {
            // First call to event_update, schedule the first event
            event_flags.next_event_time = Some(1.0);
        }

        event_flags.values_of_continuous_states_changed = false;
        event_flags.nominals_of_continuous_states_changed = false;
        event_flags.terminate_simulation = self.counter >= 10; // Terminate after counter reaches 10

        Ok(Fmi3Res::OK)
    }
}

// Export the FMU with full C API
fmi_export::export_fmu!(Stair);
