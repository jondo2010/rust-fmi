#![deny(clippy::all)]
//! Example port of the Dahlquist FMU from the Reference FMUs
//!
//! This implements a simple first-order linear ODE: der(x) = -k * x
//! where x is the state variable and k is a parameter.

use fmi::fmi3::{Fmi3Error, Fmi3Res};
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

    fn calculate_values(&mut self, _context: &ModelContext<Self>) -> Result<Fmi3Res, Fmi3Error> {
        // Calculate the derivative: der(x) = -k * x
        self.der_x = -self.k * self.x;
        Ok(Fmi3Res::OK)
    }
}

// Export the FMU with full C API
fmi_export::export_fmu!(Dahlquist);

pub trait ModelVariablesBuilder {
    fn iter_model_variables() -> impl Iterator<Item = ()>;
}

use fmi_export::fmi3::FmiVariableBuilder;

impl ModelVariablesBuilder for Dahlquist {
    fn iter_model_variables() -> impl Iterator<Item = ()> {
        let x_var = <f64 as FmiVariableBuilder>::variable("x")
            .with_causality(fmi::fmi3::schema::Causality::Output)
            .with_variability(fmi::fmi3::schema::Variability::Continuous)
            .with_start(1.0)
            .with_initial(fmi::fmi3::schema::Initial::Exact)
            .finish();
        let der_x_var = <f64 as FmiVariableBuilder>::variable("der_x")
            .with_causality(fmi::fmi3::schema::Causality::Local)
            .with_variability(fmi::fmi3::schema::Variability::Continuous)
            .with_derivative("x")
            .with_initial(fmi::fmi3::schema::Initial::Calculated)
            .finish();
        let k_var = <f64 as ModelVariablesBuilder>::variable("k")
            .with_causality(fmi::fmi3::schema::Causality::Parameter)
            .with_variability(fmi::fmi3::schema::Variability::Fixed)
            .with_start(1.0)
            .with_initial(fmi::fmi3::schema::Initial::Exact)
            .finish();
        vec![x_var, der_x_var, k_var].into_iter()
    }
}
