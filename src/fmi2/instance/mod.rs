use crate::{FmiError, FmiResult, FmiStatus, Import};

use super::*;
use std::ffi::CString;

mod co_simulation;
mod model_exchange;
pub mod traits;

/// Tag for Model Exchange instances
pub struct ME;
/// Tag for Co-Simulation instances
pub struct CS;
/// Tag for Scheduled Execution instances

pub struct Instance<'a, Tag> {
    /// Raw FMI 2.0 bindings
    binding: binding::Fmi2Binding,
    /// Pointer to the raw FMI 2.0 instance
    component: binding::fmi2Component,

    schema: &'a meta::ModelDescription,

    /// Callbacks struct
    #[allow(dead_code)]
    callbacks: Box<CallbackFunctions>,

    _tag: std::marker::PhantomData<Tag>,
}

impl<'a, A> Drop for Instance<'a, A> {
    fn drop(&mut self) {
        log::trace!("Freeing component {:?}", self.component);
        unsafe { self.binding.fmi2FreeInstance(self.component) };
    }
}

impl Default for CallbackFunctions {
    fn default() -> Self {
        CallbackFunctions {
            logger: Some(logger::callback_logger_handler),
            allocate_memory: Some(libc::calloc),
            free_memory: Some(libc::free),
            step_finished: None,
            component_environment: std::ptr::null_mut::<std::os::raw::c_void>(),
        }
    }
}

/// Check the internal consistency of the FMU by comparing the TypesPlatform and FMI versions
/// from the library and the Model Description XML
fn check_consistency(import: &Import, common: &FmiCommon) -> FmiResult<()> {
    let types_platform =
        unsafe { std::ffi::CStr::from_ptr(common.get_types_platform()) }.to_bytes_with_nul();

    if types_platform != binding::fmi2TypesPlatform {
        return Err(FmiError::TypesPlatformMismatch {
            found: types_platform.into(),
        });
    }

    let fmi_version = unsafe { std::ffi::CStr::from_ptr(common.get_version()) }.to_bytes();
    if fmi_version != import.descr().fmi_version.as_bytes() {
        return Err(FmiError::FmiVersionMismatch {
            found: fmi_version.into(),
            expected: import.descr().fmi_version.as_bytes().into(),
        });
    }

    Ok(())
}

// We assume here that the exported FMUs are thread-safe (true for OpenModelica)
//unsafe impl<A: FmiApi> Send for Instance<A> {}
//unsafe impl<A: FmiApi> Sync for Instance<A> {}

/// FmuState wraps the FMUstate pointer and is used for managing FMU state
pub struct FmuState<'a, Tag> {
    component: &'a Instance<'a, Tag>,
    state: binding::fmi2FMUstate,
    //container: &'a dlopen::wrapper::Container<A>,
}

impl<'a, Tag> FmuState<'a, Tag> {}

impl<'a, Tag> Drop for FmuState<'a, Tag> {
    fn drop(&mut self) {
        log::trace!("Freeing FmuState");
        let ret = unsafe {
            self.component
                .binding
                .fmi2FreeFMUstate(self.component.component, self.state)
        };
        todo!();
    }
}

impl<'a, A> traits::Common for Instance<'a, A> {
    fn name(&self) -> &str {
        &self.schema.model_name
    }

    fn version(&self) -> FmiResult<&str> {
        unsafe { std::ffi::CStr::from_ptr(self.binding.fmi2GetVersion()) }
            .to_str()
            .map_err(FmiError::from)
    }

    fn set_debug_logging(&self, logging_on: bool, categories: &[&str]) -> FmiResult<FmiStatus> {
        let category_cstr = categories
            .iter()
            .map(|c| CString::new(*c).unwrap())
            .collect::<Vec<_>>();

        let category_ptrs: Vec<_> = category_cstr.iter().map(|c| c.as_ptr()).collect();

        let ret: FmiStatus = unsafe {
            self.binding.fmi2SetDebugLogging(
                self.component,
                logging_on as binding::fmi2Boolean,
                category_ptrs.len(),
                category_ptrs.as_ptr(),
            )
        }
        .into();
        ret.into()
    }

    fn setup_experiment(
        &self,
        tolerance: Option<f64>,
        start_time: f64,
        stop_time: Option<f64>,
    ) -> FmiResult<FmiStatus> {
        unsafe {
            self.container.common().setup_experiment(
                self.component,
                tolerance.is_some() as binding::fmi2Boolean,
                tolerance.unwrap_or(0.0),
                start_time,
                stop_time.is_some() as binding::fmi2Boolean,
                stop_time.unwrap_or(0.0),
            )
        }
        .into()
    }

    fn enter_initialization_mode(&self) -> FmiResult<FmiStatus> {
        unsafe {
            self.container
                .common()
                .enter_initialization_mode(self.component)
        }
        .into()
    }

    fn exit_initialization_mode(&self) -> FmiResult<FmiStatus> {
        unsafe {
            self.container
                .common()
                .exit_initialization_mode(self.component)
        }
        .into()
    }

    fn terminate(&self) -> FmiResult<FmiStatus> {
        unsafe { self.container.common().terminate(self.component) }.into()
    }

    fn reset(&self) -> FmiResult<FmiStatus> {
        unsafe { self.container.common().reset(self.component) }.into()
    }

    fn get_real(&self, sv: &meta::ScalarVariable) -> FmiResult<binding::fmi2Real> {
        let mut ret: binding::fmi2Real = 0.0;
        let res: FmiResult<FmiStatus> = unsafe {
            self.container
                .common()
                .get_real(self.component, &sv.value_reference.0, 1, &mut ret)
        }
        .into();
        res.and(Ok(ret as f64))
    }

    fn get_integer(&self, sv: &meta::ScalarVariable) -> FmiResult<binding::fmi2Integer> {
        let mut ret: binding::fmi2Integer = 0;
        let res: FmiResult<FmiStatus> = unsafe {
            self.container
                .common()
                .get_integer(self.component, &sv.value_reference.0, 1, &mut ret)
        }
        .into();
        res.and(Ok(ret))
    }

    fn get_boolean(&self, sv: &meta::ScalarVariable) -> FmiResult<binding::fmi2Boolean> {
        let mut ret: binding::fmi2Boolean = 0;
        let res: FmiResult<FmiStatus> = unsafe {
            self.container
                .common()
                .get_boolean(self.component, &sv.value_reference.0, 1, &mut ret)
        }
        .into();
        res.and(Ok(ret))
    }

    fn get_string(&self, _sv: &meta::ScalarVariable) -> FmiResult<binding::fmi2String> {
        unimplemented!()
    }

    fn set_real(
        &self,
        vrs: &[meta::ValueReference],
        values: &[binding::fmi2Real],
    ) -> FmiResult<FmiStatus> {
        assert!(vrs.len() == values.len());
        unsafe {
            self.container.common().set_real(
                self.component,
                vrs.as_ptr() as *const u32,
                values.len(),
                values.as_ptr(),
            )
        }
        .into()
    }

    // fn set_real(&self, sv: &model_descr::ScalarVariable, value: f64) -> Result<()> {
    // let vr = sv.value_reference as fmi::fmi2ValueReference;
    // let vr = &vr as *const fmi::fmi2ValueReference;
    // handle_status_u32(unsafe {
    // self.container
    // .common()
    // .set_real(self.component, vr, 1, &value as *const fmi::fmi2Real)
    // })
    // }

    fn set_integer(
        &self,
        vrs: &[meta::ValueReference],
        values: &[binding::fmi2Integer],
    ) -> FmiResult<FmiStatus> {
        unsafe {
            self.container.common().set_integer(
                self.component,
                vrs.as_ptr() as *const u32,
                values.len(),
                values.as_ptr(),
            )
        }
        .into()
    }

    fn set_boolean(
        &self,
        vrs: &[binding::fmi2ValueReference],
        values: &[binding::fmi2Boolean],
    ) -> FmiResult<FmiStatus> {
        unsafe {
            self.container.common().set_boolean(
                self.component,
                vrs.as_ptr(),
                values.len(),
                values.as_ptr(),
            )
        }
        .into()
    }

    fn set_string(
        &self,
        _vrs: &[binding::fmi2ValueReference],
        _values: &[binding::fmi2String],
    ) -> FmiResult<FmiStatus> {
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
    ) -> FmiResult<FmiStatus> {
        assert!(unknown_vrs.len() == dv_unknown_values.len());
        assert!(known_vrs.len() == dv_unknown_values.len());
        unsafe {
            self.container.common().get_directional_derivative(
                self.component,
                unknown_vrs.as_ptr(),
                unknown_vrs.len(),
                known_vrs.as_ptr(),
                known_vrs.len(),
                dv_known_values.as_ptr(),
                dv_unknown_values.as_mut_ptr(),
            )
        }
        .into()
    }
}

impl<'a, A> std::fmt::Debug for Instance<'a, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Instance {} {{Import {}, {:?}}}",
            self.name(),
            self.model.model_name,
            self.component,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO Make this work on other targets
    #[cfg(target_os = "linux")]
    #[test]
    fn test_instance_me() {
        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap();

        let instance1 = InstanceME::new(&import, "inst1", false, true).unwrap();
        assert_eq!(instance1.version().unwrap(), "2.0");

        let categories = &import
            .descr()
            .log_categories
            .as_ref()
            .unwrap()
            .categories
            .iter()
            .map(|cat| cat.name.as_ref())
            .collect::<Vec<&str>>();

        instance1
            .set_debug_logging(true, categories)
            .expect("set_debug_logging");
        instance1
            .setup_experiment(Some(1.0e-6_f64), 0.0, None)
            .expect("setup_experiment");
        instance1
            .enter_initialization_mode()
            .expect("enter_initialization_mode");
        instance1
            .exit_initialization_mode()
            .expect("exit_initialization_mode");
        instance1.terminate().expect("terminate");
        instance1.reset().expect("reset");
    }

    /// Tests on variable module requiring an instance.
    #[cfg(target_os = "linux")]
    #[cfg(feature = "disable")]
    #[test]
    fn test_variable() {
        use crate::{model_descr::ModelDescriptionError, Var};
        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap();

        let inst = InstanceME::new(&import, "inst1", false, true).unwrap();

        let mut vars = import.descr().get_model_variables();
        let _ = Var::from_scalar_variable(&inst, vars.next().unwrap().1);

        assert!(matches!(
            Var::from_name(&inst, "false"),
            Err(FmiError::ModelDescr(
                ModelDescriptionError::VariableNotFound { .. }
            ))
        ));
    }

    #[cfg(target_os = "linux")]
    #[cfg(feature = "disable")]
    #[test]
    fn test_instance_cs() {
        use crate::{Value, Var};
        use assert_approx_eq::assert_approx_eq;

        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap();

        let instance1 = InstanceCS::new(&import, "inst1", false, true).unwrap();
        assert_eq!(instance1.version().unwrap(), "2.0");

        instance1
            .setup_experiment(Some(1.0e-6_f64), 0.0, None)
            .expect("setup_experiment");

        instance1
            .enter_initialization_mode()
            .expect("enter_initialization_mode");

        let param = Var::from_name(&instance1, "freqHz").expect("freqHz parameter from_name");
        param
            .set(&Value::Real(2.0f64))
            .expect("set freqHz parameter");

        instance1
            .exit_initialization_mode()
            .expect("exit_initialization_mode");

        let y = Var::from_name(&instance1, "y").expect("get y");

        if let Value::Real(y_val) = y.get().expect("get y value") {
            assert_approx_eq!(y_val, 0.0, 1.0e-6);
        }

        instance1.do_step(0.0, 0.125, false).expect("do_step");

        if let Value::Real(y_val) = y.get().expect("get y value") {
            assert_approx_eq!(y_val, 1.0, 1.0e-6);
        }
    }
}
