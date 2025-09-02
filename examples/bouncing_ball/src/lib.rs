//! Example port of the BouncingBall FMU from the Reference FMUs
#![deny(clippy::all)]

use fmi::{
    EventFlags,
    fmi3::{Fmi3Error, Fmi3Res, Fmi3Status},
};
use fmi_export::{
    FmuModel,
    fmi3::{DefaultLoggingCategory, ModelContext, UserModel},
};

/// BouncingBall FMU model that can be exported as a complete FMU
#[derive(FmuModel, Default, Debug)]
#[model()]
struct BouncingBall {
    /// Height above ground (state output)
    #[variable(causality = Output, state, event_indicator, start = 1.0)]
    h: f64,

    /// Velocity of the ball
    #[variable(causality = Output, state, start = 0.0)]
    #[alias(name="der(h)", causality = Local, derivative = h)]
    v: f64,

    /// Gravitational acceleration
    #[variable(causality = Parameter, start = -9.81)]
    #[alias(name = "der(v)", causality = Local, derivative = v)]
    g: f64,

    /// Coefficient of restitution (parameter)
    #[variable(causality = Parameter, start = 0.7)]
    e: f64,

    /// Minimum velocity threshold (constant)
    #[variable(causality = Local, start = 0.1)]
    v_min: f64,
}

impl UserModel for BouncingBall {
    type LoggingCategory = DefaultLoggingCategory;

    fn calculate_values(&mut self, _context: &ModelContext<Self>) -> Fmi3Status {
        // nothing to do
        Fmi3Res::OK.into()
    }

    fn event_update(
        &mut self,
        context: &ModelContext<Self>,
        event_flags: &mut EventFlags,
    ) -> Result<Fmi3Res, Fmi3Error> {
        // Handle ball bouncing off the ground
        if self.h <= 0.0 && self.v < 0.0 {
            context.log(
                Fmi3Res::OK,
                Self::LoggingCategory::default(),
                format_args!("Ball bounced! h={:.3}, v={:.3}", self.h, self.v),
            );

            self.h = f64::MIN_POSITIVE; // Slightly above ground
            self.v = -self.v * self.e; // Reverse velocity with energy loss

            // Stop bouncing if velocity becomes too small
            if self.v < self.v_min {
                context.log(
                    Fmi3Res::OK,
                    Self::LoggingCategory::default(),
                    format_args!("Ball stopped bouncing"),
                );
                self.v = 0.0;
                self.g = 0.0; // Disable gravity when stopped
            }

            event_flags.values_of_continuous_states_changed = true;
        } else {
            event_flags.values_of_continuous_states_changed = false;
        }

        Ok(Fmi3Res::OK)
    }

    fn get_event_indicators(
        &mut self,
        _context: &ModelContext<Self>,
        indicators: &mut [f64],
    ) -> Result<bool, Fmi3Error> {
        assert!(!indicators.is_empty());
        // Event indicator for ground contact
        indicators[0] = if self.h == 0.0 && self.v == 0.0 {
            1.0 // Special case: stopped ball
        } else {
            self.h // Height as event indicator
        };
        Ok(true)
    }
}

// Export the FMU with full C API
fmi_export::export_fmu!(BouncingBall);
