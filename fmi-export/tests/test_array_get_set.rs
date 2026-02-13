use std::ffi::CString;

use fmi::fmi3::{Fmi3Error, Fmi3Res, Fmi3Status};
use fmi::traits::FmiStatus;
use fmi_export::fmi3::{Context, DefaultLoggingCategory, Fmi3Common, Fmi3ModelExchange, UserModel};
use fmi_export::{FmuModel, fmi3::Model};

#[derive(FmuModel, Default, Debug)]
#[model(user_model = false)]
struct ArrayModel {
    #[variable(causality = Output, variability = Continuous, start = [2.0, 0.0], initial = Exact)]
    x: [f64; 2],

    #[variable(causality = Local, variability = Continuous, derivative = x, initial = Calculated)]
    der_x: [f64; 2],

    #[variable(causality = Parameter, variability = Fixed, start = 1.0, initial = Exact)]
    mu: f64,
}

impl UserModel for ArrayModel {
    type LoggingCategory = DefaultLoggingCategory;

    fn calculate_values(&mut self, _context: &dyn Context<Self>) -> Result<Fmi3Res, Fmi3Error> {
        self.der_x[0] = self.x[1];
        self.der_x[1] = self.mu * ((1.0 - self.x[0] * self.x[0]) * self.x[1]) - self.x[0];
        Ok(Fmi3Res::OK)
    }
}

#[test]
fn test_array_get_set_vrs_are_scalar() {
    let inst = unsafe {
        <ArrayModel as Fmi3Common>::fmi3_instantiate_model_exchange(
            CString::new("test").unwrap().as_ptr(),
            CString::new(ArrayModel::INSTANTIATION_TOKEN)
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
            <ArrayModel as Fmi3Common>::fmi3_enter_initialization_mode(
                inst, false, 0.0, 0.0, false, 0.0,
            )
        })
        .ok(),
        Ok(Fmi3Res::OK),
    );

    assert_eq!(
        Fmi3Status::from(unsafe {
            <ArrayModel as Fmi3Common>::fmi3_exit_initialization_mode(inst)
        })
        .ok(),
        Ok(Fmi3Res::OK),
    );

    assert_eq!(
        Fmi3Status::from(unsafe { <ArrayModel as Fmi3ModelExchange>::fmi3_set_time(inst, 0.0) })
            .ok(),
        Ok(Fmi3Res::OK),
    );

    let set_values = [1.0, 2.0, 3.0, 4.0, 5.0];
    assert_eq!(
        Fmi3Status::from(unsafe {
            <ArrayModel as Fmi3Common>::fmi3_set_float64(
                inst,
                [1, 2, 3].as_ptr(),
                3,
                set_values.as_ptr(),
                set_values.len(),
            )
        })
        .ok(),
        Ok(Fmi3Res::OK),
    );

    let mut get_values = [0.0; 5];
    assert_eq!(
        Fmi3Status::from(unsafe {
            <ArrayModel as Fmi3Common>::fmi3_get_float64(
                inst,
                [1, 2, 3].as_ptr(),
                3,
                get_values.as_mut_ptr(),
                get_values.len(),
            )
        })
        .ok(),
        Ok(Fmi3Res::OK),
    );

    assert_eq!(get_values, [1.0, 2.0, 2.0, -1.0, 5.0]);
}
