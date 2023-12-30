use std::ffi::{CStr, CString};

use crate::fmi2::{binding, Fmi2Status};

use super::{traits, Instance};

impl<'a, Tag> traits::Common for Instance<'a, Tag> {
    fn name(&self) -> &str {
        &self.schema.model_name
    }

    fn version(&self) -> &str {
        unsafe { CStr::from_ptr(self.binding.fmi2GetVersion()) }
            .to_str()
            .expect("Error converting string")
    }

    fn get_types_platform(&self) -> &str {
        unsafe { CStr::from_ptr(self.binding.fmi2GetTypesPlatform()) }
            .to_str()
            .expect("Error converting string")
    }

    fn set_debug_logging(&self, logging_on: bool, categories: &[&str]) -> Fmi2Status {
        let category_cstr = categories
            .iter()
            .map(|c| CString::new(*c).unwrap())
            .collect::<Vec<_>>();

        let category_ptrs: Vec<_> = category_cstr.iter().map(|c| c.as_ptr()).collect();

        Fmi2Status(unsafe {
            self.binding.fmi2SetDebugLogging(
                self.component,
                logging_on as binding::fmi2Boolean,
                category_ptrs.len(),
                category_ptrs.as_ptr(),
            )
        })
    }

    fn setup_experiment(
        &self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> Fmi2Status {
        Fmi2Status(unsafe {
            self.binding.fmi2SetupExperiment(
                self.component,
                tolerance.is_some() as binding::fmi2Boolean,
                tolerance.unwrap_or(0.0),
                start_time,
                stop_time.is_some() as binding::fmi2Boolean,
                stop_time.unwrap_or(0.0),
            )
        })
    }

    fn enter_initialization_mode(&self) -> Fmi2Status {
        Fmi2Status(unsafe { self.binding.fmi2EnterInitializationMode(self.component) })
    }

    fn exit_initialization_mode(&self) -> Fmi2Status {
        Fmi2Status(unsafe { self.binding.fmi2ExitInitializationMode(self.component) })
    }

    fn terminate(&self) -> Fmi2Status {
        Fmi2Status(unsafe { self.binding.fmi2Terminate(self.component) })
    }

    fn reset(&self) -> Fmi2Status {
        Fmi2Status(unsafe { self.binding.fmi2Reset(self.component) })
    }

    fn get_real(
        &self,
        vrs: &[binding::fmi2ValueReference],
        values: &mut [binding::fmi2Real],
    ) -> Fmi2Status {
        assert_eq!(vrs.len(), values.len());
        Fmi2Status(unsafe {
            self.binding
                .fmi2GetReal(self.component, vrs.as_ptr(), vrs.len(), values.as_mut_ptr())
        })
    }

    fn get_integer(
        &self,
        vrs: &[binding::fmi2ValueReference],
        values: &mut [binding::fmi2Integer],
    ) -> Fmi2Status {
        Fmi2Status(unsafe {
            self.binding.fmi2GetInteger(
                self.component,
                vrs.as_ptr(),
                vrs.len(),
                values.as_mut_ptr(),
            )
        })
    }

    fn get_boolean(
        &self,
        sv: &[binding::fmi2ValueReference],
        v: &mut [binding::fmi2Boolean],
    ) -> Fmi2Status {
        Fmi2Status(unsafe {
            self.binding
                .fmi2GetBoolean(self.component, sv.as_ptr(), sv.len(), v.as_mut_ptr())
        })
    }

    fn get_string(
        &self,
        sv: &[binding::fmi2ValueReference],
        v: &mut [binding::fmi2String],
    ) -> Fmi2Status {
        Fmi2Status(unsafe {
            self.binding
                .fmi2GetString(self.component, sv.as_ptr(), sv.len(), v.as_mut_ptr())
        })
    }

    fn set_real(
        &self,
        vrs: &[binding::fmi2ValueReference],
        values: &[binding::fmi2Real],
    ) -> Fmi2Status {
        assert_eq!(vrs.len(), values.len());
        Fmi2Status(unsafe {
            self.binding.fmi2SetReal(
                self.component,
                vrs.as_ptr() as *const u32,
                values.len(),
                values.as_ptr(),
            )
        })
    }

    fn set_integer(
        &self,
        vrs: &[binding::fmi2ValueReference],
        values: &[binding::fmi2Integer],
    ) -> Fmi2Status {
        assert_eq!(vrs.len(), values.len());
        Fmi2Status(unsafe {
            self.binding
                .fmi2SetInteger(self.component, vrs.as_ptr(), values.len(), values.as_ptr())
        })
    }

    fn set_boolean(
        &self,
        vrs: &[binding::fmi2ValueReference],
        values: &mut [binding::fmi2Boolean],
    ) -> Fmi2Status {
        assert_eq!(vrs.len(), values.len());
        Fmi2Status(unsafe {
            self.binding
                .fmi2SetBoolean(self.component, vrs.as_ptr(), values.len(), values.as_ptr())
        })
    }

    fn set_string(
        &self,
        _vrs: &[binding::fmi2ValueReference],
        _values: &[binding::fmi2String],
    ) -> Fmi2Status {
        unimplemented!()
    }

    // fn get_fmu_state(&self, state: *mut fmi2FMUstate) -> FmiResult<()> {}

    // fn set_fmu_state(&self, state: &[u8]) -> FmiResult<()> {}

    fn get_directional_derivative(
        &self,
        unknown_vrs: &[binding::fmi2ValueReference],
        known_vrs: &[binding::fmi2ValueReference],
        dv_known_values: &[binding::fmi2Real],
        dv_unknown_values: &mut [binding::fmi2Real],
    ) -> Fmi2Status {
        assert!(unknown_vrs.len() == dv_unknown_values.len());
        assert!(known_vrs.len() == dv_unknown_values.len());
        Fmi2Status(unsafe {
            self.binding.fmi2GetDirectionalDerivative(
                self.component,
                unknown_vrs.as_ptr(),
                unknown_vrs.len(),
                known_vrs.as_ptr(),
                known_vrs.len(),
                dv_known_values.as_ptr(),
                dv_unknown_values.as_mut_ptr(),
            )
        })
    }
}
