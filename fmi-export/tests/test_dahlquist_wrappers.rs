use std::ffi::CString;

use fmi::{
    fmi3::{Fmi3Error, Fmi3Res, Fmi3Status},
    traits::FmiStatus,
};
use fmi_export::{
    FmuModel,
    fmi3::{
        CSDoStepResult, Context, DefaultLoggingCategory, Fmi3CoSimulation, Fmi3Common,
        Fmi3ModelExchange, Model, UserModel,
    },
};

#[derive(FmuModel, Default, Debug)]
#[model(model_exchange = true, co_simulation = true, user_model = false)]
struct Dahlquist {
    #[variable(causality = Output, variability = Continuous, start = 1.0, initial = Exact)]
    x: f64,

    #[variable(causality = Local, variability = Continuous, derivative = x, initial = Calculated)]
    der_x: f64,

    #[variable(causality = Parameter, variability = Fixed, start = 1.0, initial = Exact)]
    k: f64,
}

impl UserModel for Dahlquist {
    type LoggingCategory = DefaultLoggingCategory;

    fn calculate_values(&mut self, _context: &dyn Context<Self>) -> Result<Fmi3Res, Fmi3Error> {
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
        context.set_time(current_communication_point);
        self.calculate_values(context)?;
        self.x += self.der_x * communication_step_size;
        let last_time = current_communication_point + communication_step_size;
        context.set_time(last_time);
        Ok(CSDoStepResult::completed(last_time))
    }
}

#[test]
fn test_model_get_set_scalar_vrs() {
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
        Fmi3Status::from(unsafe { <Dahlquist as Fmi3Common>::fmi3_exit_initialization_mode(inst) })
            .ok(),
        Ok(Fmi3Res::OK),
    );

    assert_eq!(
        Fmi3Status::from(unsafe { <Dahlquist as Fmi3ModelExchange>::fmi3_set_time(inst, 123.0) })
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

    assert_eq!(f64_vals, [123.0, 1.0, -1.0, 1.0]);

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
        Err(Fmi3Error::Error),
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
        Fmi3Status::from(unsafe { <Dahlquist as Fmi3Common>::fmi3_exit_initialization_mode(inst) })
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
