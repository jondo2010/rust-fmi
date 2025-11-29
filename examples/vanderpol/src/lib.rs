#![deny(clippy::all)]
//! Example port of the Van der Pol oscillator FMU from the Reference FMUs
//!
//! This implements the Van der Pol oscillator equations:
//! - der(x0) = x1
//! - der(x1) = mu * ((1 - x0²) * x1) - x0
//!
//! Where:
//! - x0 and x1 are the state variables
//! - mu is a parameter controlling the nonlinearity

use fmi::fmi3::{Fmi3Error, Fmi3Res};
use fmi_export::{
    fmi3::{Context, DefaultLoggingCategory, UserModel},
    FmuModel,
};

/// Van der Pol oscillator FMU model
///
/// The Van der Pol oscillator is a non-conservative oscillator with non-linear damping.
/// It evolves in time according to the second-order differential equation:
/// d²x/dt² - μ(1 - x²)dx/dt + x = 0
///
/// This is implemented as a system of first-order ODEs:
/// - der(x0) = x1
/// - der(x1) = μ(1 - x0²)x1 - x0
#[derive(FmuModel, Default, Debug)]
struct VanDerPol {
    #[variable(causality = Output, variability = Continuous, start = [2.0, 0.0], initial = Exact)]
    x: [f64; 2],

    #[variable(causality = Local, variability = Continuous, derivative = x, initial = Calculated)]
    der_x: [f64; 2],

    #[variable(causality = Parameter, variability = Fixed, start = 1.0, initial = Exact)]
    mu: f64,
}

impl UserModel for VanDerPol {
    type LoggingCategory = DefaultLoggingCategory;

    fn calculate_values(&mut self, _context: &dyn Context<Self>) -> Result<Fmi3Res, Fmi3Error> {
        // Calculate the derivatives according to Van der Pol equations:
        // der(x[0]) = x[1]
        self.der_x[0] = self.x[1];

        // der(x[1]) = mu * ((1 - x[0]²) * x[1]) - x[0]
        self.der_x[1] = self.mu * ((1.0 - self.x[0] * self.x[0]) * self.x[1]) - self.x[0];

        Ok(Fmi3Res::OK)
    }
}

// Export the FMU with full C API
fmi_export::export_fmu!(VanDerPol);
