#![deny(clippy::all)]
//! Example port of the Stair FMU from the Reference FMUs
//!
//! This implements a discrete event model that increments a counter
//! every second, demonstrating event handling and discrete variables.

use fmi::{
    EventFlags,
    fmi3::{Fmi3Error, Fmi3Res},
};
use fmi_export::{
    FmuModel,
    fmi3::{DefaultLoggingCategory, ModelContext, UserModel},
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

    fn calculate_values(&mut self, _context: &ModelContext<Self>) -> Result<Fmi3Res, Fmi3Error> {
        // Nothing to calculate in this discrete model
        Ok(Fmi3Res::OK)
    }

    fn event_update(
        &mut self,
        context: &ModelContext<Self>,
        event_flags: &mut EventFlags,
    ) -> Result<Fmi3Res, Fmi3Error> {
        let epsilon = (1.0 + context.time().abs()) * f64::EPSILON;

        let next_event_time = event_flags.next_event_time.unwrap_or(1.0);

        if dbg!(context.time() + epsilon >= next_event_time) {
            self.counter += 1;
            event_flags.next_event_time = Some(next_event_time + 1.0); // Schedule next event in 1 second
        }

        event_flags.values_of_continuous_states_changed = false;
        event_flags.nominals_of_continuous_states_changed = false;
        event_flags.terminate_simulation = self.counter >= 10; // Terminate after counter reaches 10

        Ok(Fmi3Res::OK)
    }
}

// Export the FMU with full C API
fmi_export::export_fmu!(Stair);
