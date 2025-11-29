#![deny(clippy::all)]
//! Example port of the Dahlquist FMU from the Reference FMUs
//!
//! This implements a simple first-order linear ODE: der(x) = -k * x
//! where x is the state variable and k is a parameter.

use fmi::fmi3::{Fmi3Error, Fmi3Res};
use fmi_export::{
    fmi3::{Context, DefaultLoggingCategory, UserModel, UserModelCSWrapper, UserModelME},
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

    fn calculate_values(&mut self, _context: &dyn Context<Self>) -> Result<Fmi3Res, Fmi3Error> {
        // Calculate the derivative: der(x) = -k * x
        self.der_x = -self.k * self.x;
        Ok(Fmi3Res::OK)
    }
}

impl UserModelME for Dahlquist {}

impl UserModelCSWrapper for Dahlquist {
    const FIXED_SOLVER_STEP: f64 = 0.1;
}

// Export the FMU with full C API
//fmi_export::export_fmu!(Dahlquist);

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use fmi::fmi3::{CoSimulation, GetSet, ModelExchange};
    use fmi::schema::fmi3::{AbstractVariableTrait, DependenciesKind, Fmi3Unknown};
    use fmi_export::fmi3::{Model, ModelInstance};

    #[test]
    fn test_metadata() {
        let (vars, structure) = Dahlquist::build_toplevel_metadata();

        // Check the generated variables and their VRs
        assert_eq!(vars.float64[0].name(), "time");
        assert_eq!(vars.float64[0].value_reference(), 0);
        assert_eq!(vars.float64[1].name(), "x");
        assert_eq!(vars.float64[1].value_reference(), 1);
        assert_eq!(vars.float64[2].name(), "der_x");
        assert_eq!(vars.float64[2].value_reference(), 2);
        assert_eq!(vars.float64[3].name(), "k");
        assert_eq!(vars.float64[3].value_reference(), 3);

        // Check the model structure

        assert_eq!(
            structure.outputs,
            // 'x' is the only output, and depends on 'der_x'
            vec![Fmi3Unknown {
                value_reference: 1,
                dependencies: Some(vec![2]),
                dependencies_kind: Some(vec![DependenciesKind::Dependent]),
                ..Default::default()
            }]
        );

        assert_eq!(
            structure.continuous_state_derivative,
            // 'der_x' is the derivative of 'x'
            vec![Fmi3Unknown {
                value_reference: 2,
                ..Default::default()
            }]
        );

        assert_eq!(
            structure.initial_unknown,
            // 'x' is initial exact, 'der_x' is initial calculated
            vec![Fmi3Unknown {
                value_reference: 2,
                ..Default::default()
            }]
        );

        let xml = fmi::schema::serialize(&structure, true).unwrap();
        println!("{xml}");
    }

    #[test]
    fn test_model_get_set() {
        let mut inst = ModelInstance::<Dahlquist>::new(
            "Dahlquist".to_string(),
            Dahlquist::INSTANTIATION_TOKEN,
            BasicContext::new(true, Box::new(|_, _, _| {}), PathBuf::new()),
        )
        .unwrap();

        inst.set_time(123.0).unwrap();

        let mut f64_vals = [0.0; 4];
        inst.get_float64(&[0, 1, 2, 3], &mut f64_vals).unwrap();
        assert_eq!(
            f64_vals,
            [
                123.0, // time
                1.0,   // x (start value)
                -1.0,  // der_x (calculated later)
                1.0    // k (parameter)
            ]
        );

        // Test the time VR=0
        assert_eq!(inst.set_float64(&[0], &[0.0]), Err(Fmi3Error::Error));
    }

    #[test]
    fn test_model_cs_wrapper() {
        let mut inst = ModelInstance::<Dahlquist>::new(
            "Dahlquist".to_string(),
            Dahlquist::INSTANTIATION_TOKEN,
            MEWrapperContext::new(true, Box::new(|_, _, _| {}), PathBuf::new(), false),
        )
        .unwrap();

        //inst.do_step(0.0, 0.1, true).unwrap();
    }
}
