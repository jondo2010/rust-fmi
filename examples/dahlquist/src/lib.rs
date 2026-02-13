#![allow(unexpected_cfgs)]
#![deny(clippy::all)]
//! Example port of the Dahlquist FMU from the Reference FMUs
//!
//! This implements a simple first-order linear ODE: der(x) = -k * x
//! where x is the state variable and k is a parameter.

use fmi::fmi3::{Fmi3Error, Fmi3Res};
use fmi_export::{
    fmi3::{CSDoStepResult, Context, DefaultLoggingCategory, UserModel},
    FmuModel,
};

/// Dahlquist FMU model implementing der(x) = -k * x
///
/// This is a simple first-order linear ODE that demonstrates basic
/// Model Exchange and Co-Simulation capabilities.
#[derive(FmuModel, Default, Debug)]
#[model(model_exchange = true, co_simulation = true, user_model = false)]
struct Dahlquist {
    /// The state variable
    #[variable(causality = Output, variability = Continuous, start = 1.0, initial = Exact)]
    x: f64,

    /// The derivative of x, calculated as der(x) = -k * x
    #[variable(causality = Local, variability = Continuous, derivative = x, initial = Calculated)]
    der_x: f64,

    /// The parameter k
    #[variable(causality = Parameter, variability = Fixed, start = 1.0, initial = Exact)]
    k: f64,
}

impl UserModel for Dahlquist {
    type LoggingCategory = DefaultLoggingCategory;

    fn calculate_values(&mut self, _context: &dyn Context<Self>) -> Result<Fmi3Res, Fmi3Error> {
        // Calculate the derivative: der(x) = -k * x
        self.der_x = -self.k * self.x;
        Ok(Fmi3Res::OK)
    }

    fn do_step(
        &mut self,
        context: &mut dyn Context<Self>,
        current_communication_point: f64,
        communication_step_size: f64,
        _no_set_fmu_state_prior_to_current_point: bool,
    ) -> Result<CSDoStepResult, Fmi3Error> {
        // Align context time with the current communication point
        context.set_time(current_communication_point);

        // Compute derivatives at the current point
        self.calculate_values(context)?;

        // Forward Euler step for the single state
        self.x += self.der_x * communication_step_size;

        let last_time = current_communication_point + communication_step_size;
        context.set_time(last_time);

        Ok(CSDoStepResult::completed(last_time))
    }
}

// Export the FMU with full C API
fmi_export::export_fmu!(Dahlquist);
