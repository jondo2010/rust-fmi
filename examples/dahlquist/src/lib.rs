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

#[cfg(test)]
#[allow(unused_imports)]
mod tests {
    use std::ffi::CString;
    use std::path::PathBuf;

    use super::*;
    use fmi::fmi3::{CoSimulation, Fmi3Status, GetSet, ModelExchange};
    use fmi::schema::fmi3::{AbstractVariableTrait, DependenciesKind, Fmi3Unknown};
    use fmi::traits::FmiStatus;
    use fmi_export::fmi3::{
        BasicContext, Fmi3CoSimulation, Fmi3Common, Fmi3ModelExchange, Model, ModelInstance,
    };

    #[cfg(false)]
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
        let inst = unsafe {
            <Dahlquist as Fmi3Common>::fmi3_instantiate_model_exchange(
                CString::new("test").unwrap().as_ptr(),
                CString::new(Dahlquist::INSTANTIATION_TOKEN)
                    .unwrap()
                    .as_ptr() as *mut i8,
                CString::new("path/to/fmu").unwrap().as_ptr(),
                false as _,
                true as _,
                std::ptr::null_mut(),
                None,
            )
        };

        assert_eq!(
            Fmi3Status::from(unsafe {
                <Dahlquist as Fmi3Common>::fmi3_enter_initialization_mode(
                    inst, false, 0.0, 0.0, false, 0.0,
                )
            })
            .ok(),
            Ok(Fmi3Res::OK),
        );

        assert_eq!(
            Fmi3Status::from(unsafe {
                <Dahlquist as Fmi3Common>::fmi3_exit_initialization_mode(inst)
            })
            .ok(),
            Ok(Fmi3Res::OK),
        );

        assert_eq!(
            Fmi3Status::from(unsafe {
                <Dahlquist as Fmi3ModelExchange>::fmi3_set_time(inst, 123.0)
            })
            .ok(),
            Ok(Fmi3Res::OK),
        );

        let mut f64_vals = [0.0; 4];
        assert_eq!(
            Fmi3Status::from(unsafe {
                <Dahlquist as Fmi3Common>::fmi3_get_float64(
                    inst,
                    [0, 1, 2, 3].as_ptr(),
                    4,
                    f64_vals.as_mut_ptr(),
                    4,
                )
            })
            .ok(),
            Ok(Fmi3Res::OK),
        );

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
        assert_eq!(
            Fmi3Status::from(unsafe {
                <Dahlquist as Fmi3Common>::fmi3_set_float64(
                    inst,
                    [0].as_ptr(),
                    1,
                    f64_vals[0..1].as_ptr(),
                    1,
                )
            })
            .ok(),
            Err(Fmi3Error::Error)
        );
    }

    #[test]
    fn test_model_cs_wrapper() {
        let inst = unsafe {
            <Dahlquist as Fmi3Common>::fmi3_instantiate_co_simulation(
                CString::new("test").unwrap().as_ptr(),
                CString::new(Dahlquist::INSTANTIATION_TOKEN)
                    .unwrap()
                    .as_ptr() as *mut i8,
                CString::new("path/to/fmu").unwrap().as_ptr(),
                false as _,
                true as _,
                false as _,
                false as _,
                std::ptr::null_mut(),
                0,
                std::ptr::null_mut(),
                None,
                None,
            )
        };
        assert_eq!(
            Fmi3Status::from(unsafe {
                <Dahlquist as Fmi3Common>::fmi3_enter_initialization_mode(
                    inst, false, 0.0, 0.0, false, 0.0,
                )
            })
            .ok(),
            Ok(Fmi3Res::OK),
        );

        assert_eq!(
            Fmi3Status::from(unsafe {
                <Dahlquist as Fmi3Common>::fmi3_exit_initialization_mode(inst)
            })
            .ok(),
            Ok(Fmi3Res::OK),
        );

        assert_eq!(
            Fmi3Status::from(unsafe {
                <Dahlquist as Fmi3CoSimulation>::fmi3_do_step(
                    inst,
                    0.0,
                    0.1,
                    false as _,
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                    std::ptr::null_mut(),
                )
            })
            .ok(),
            Ok(Fmi3Res::OK),
        );
    }
}
