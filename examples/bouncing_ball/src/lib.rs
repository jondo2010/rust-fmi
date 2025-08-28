//! Example port of the BouncingBall FMU from the Reference FMUs

use fmi::fmi3::{Fmi3Error, Fmi3Res, Fmi3Status};
use fmi_export::{
    FmuModel,
    fmi3::{Model, ModelContext, UserModel},
};

/// BouncingBall FMU model that can be exported as a complete FMU
#[derive(FmuModel, Default, Debug)]
#[model(
    model_exchange(model_identifier = "bouncing_ball"),
    logging_categories = [
        category(name = "logAll", descr = "Log all events"),
        category(name = "logEvents", descr = "Log physical events like bouncing"),
        category(name = "logError", descr = "Log error conditions"),
    ]
)]
struct BouncingBall {
    /// Height above ground (state output)
    #[variable(causality = Output, state, start = 1.0)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum BouncingBallLoggingCategory {
    /// Log all events
    #[default]
    LogAll,
    /// Log physical events like bouncing
    LogEvents,
    /// Log error conditions
    LogError,
}
impl ::std::fmt::Display for BouncingBallLoggingCategory {
    fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
        match self {
            Self::LogAll => write!(f, "logAll"),
            Self::LogEvents => write!(f, "logEvents"),
            Self::LogError => write!(f, "logError"),
        }
    }
}
impl ::std::str::FromStr for BouncingBallLoggingCategory {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "logAll" => Ok(Self::LogAll),
            "logEvents" => Ok(Self::LogEvents),
            "logError" => Ok(Self::LogError),
            _ => Err(format!("Unknown logging category: {}", s)),
        }
    }
}
impl ::fmi_export::fmi3::ModelLoggingCategory for BouncingBallLoggingCategory {
    fn all_categories() -> impl Iterator<Item = Self> {
        [Self::LogAll, Self::LogEvents, Self::LogError]
            .iter()
            .copied()
    }
}

impl UserModel for BouncingBall {
    type LoggingCategory = BouncingBallLoggingCategory;

    fn calculate_values(&mut self, context: &ModelContext<Self>) -> Fmi3Status {
        context.log(
            Fmi3Res::OK,
            BouncingBallLoggingCategory::LogAll,
            "calculate_values() called",
        );

        // Derivatives are handled by aliases: der(h) = v, der(v) = g
        Fmi3Res::OK.into()
    }

    fn event_update(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        // Handle ball bouncing off the ground
        if self.h <= 0.0 && self.v < 0.0 {
            self.h = f64::MIN_POSITIVE; // Slightly above ground
            self.v = -self.v * self.e; // Reverse velocity with energy loss

            // Stop bouncing if velocity becomes too small
            if self.v < self.v_min {
                self.v = 0.0;
                self.g = 0.0; // Disable gravity when stopped
            }
        }
        Ok(Fmi3Res::OK)
    }

    fn get_event_indicators(&mut self, indicators: &mut [f64]) -> Result<Fmi3Res, Fmi3Error> {
        if !indicators.is_empty() {
            // Event indicator for ground contact
            indicators[0] = if self.h == 0.0 && self.v == 0.0 {
                1.0 // Special case: stopped ball
            } else {
                self.h // Height as event indicator
            };
        }
        Ok(Fmi3Res::OK)
    }
}

// Export the FMU with full C API
fmi_export::export_fmu!(BouncingBall);
