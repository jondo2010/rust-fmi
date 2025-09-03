//! Example port of the Dahlquist FMU from the Reference FMUs
//!
//! This implements a simple first-order linear ODE: der(x) = -k * x
//! where x is the state variable and k is a parameter.
#![deny(clippy::all)]

use fmi::fmi3::{Fmi3Res, Fmi3Status};
use fmi_export::{
    fmi3::{DefaultLoggingCategory, ModelContext, UserModel},
    FmuModel,
};

/// Dahlquist FMU model implementing der(x) = -k * x
///
/// This is a simple first-order linear ODE that demonstrates basic
/// Model Exchange and Co-Simulation capabilities.
#[derive(FmuModel, Default, Debug)]
#[model()]
struct Dahlquist {
    /// The state variable
    #[variable(causality = Output, variability = Continuous, state, start = 1.0, initial = Exact)]
    x: f64,

    /// The derivative of x, calculated as der(x) = -k * x
    #[variable(causality = Output, variability = Continuous, derivative = x, initial = Calculated)]
    der_x: f64,

    /// The parameter k
    #[variable(causality = Parameter, variability = Fixed, start = 1.0, initial = Exact)]
    k: f64,
}

impl UserModel for Dahlquist {
    type LoggingCategory = DefaultLoggingCategory;

    fn calculate_values(&mut self, _context: &ModelContext<Self>) -> Fmi3Status {
        // Calculate the derivative: der(x) = -k * x
        self.der_x = -self.k * self.x;
        Fmi3Res::OK.into()
    }
}

// Export the FMU with full C API
fmi_export::export_fmu!(Dahlquist);
