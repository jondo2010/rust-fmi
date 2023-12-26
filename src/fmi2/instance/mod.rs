//! FMI 2.0 instance interface

use crate::fmi2::instance::traits::Common;

use super::{binding, logger, schema, CallbackFunctions, Fmi2Status};

use std::ffi::CString;

mod co_simulation;
mod model_exchange;
pub mod traits;

/// Tag for Model Exchange instances
pub struct ME;
/// Tag for Co-Simulation instances
pub struct CS;

pub struct Instance<'a, Tag> {
    /// Raw FMI 2.0 bindings
    binding: binding::Fmi2Binding,
    /// Pointer to the raw FMI 2.0 instance
    component: binding::fmi2Component,

    schema: &'a schema::FmiModelDescription,

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
#[cfg(feature = "disable")]
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
#[cfg(feature = "disable")]
pub struct FmuState<'a, Tag> {
    component: &'a Instance<'a, Tag>,
    state: binding::fmi2FMUstate,
    //container: &'a dlopen::wrapper::Container<A>,
}

#[cfg(feature = "disable")]
impl<'a, Tag> FmuState<'a, Tag> {}

#[cfg(feature = "disable")]
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

    fn version(&self) -> &str {
        unsafe { std::ffi::CStr::from_ptr(self.binding.fmi2GetVersion()) }
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

impl<'a, A> std::fmt::Debug for Instance<'a, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Instance {} {{Import {}, {:?}}}",
            self.name(),
            self.schema.model_name,
            self.component,
        )
    }
}

#[cfg(target_os = "linux")]
#[cfg(test)]
mod tests {
    use crate::{fmi2::instance::traits::CoSimulation, import::FmiImport, Import};

    use super::*;

    // TODO Make this work on other targets
    #[test]
    fn test_instance_me() {
        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap()
        .as_fmi2()
        .unwrap();

        let instance1 = Instance::<ME>::new(&import, "inst1", false, true).unwrap();
        assert_eq!(instance1.version(), "2.0");

        let categories = &import
            .model_description()
            .log_categories
            .as_ref()
            .unwrap()
            .categories
            .iter()
            .map(|cat| cat.name.as_ref())
            .collect::<Vec<&str>>();

        instance1
            .set_debug_logging(true, categories)
            .ok()
            .expect("set_debug_logging");
        instance1
            .setup_experiment(Some(1.0e-6_f64), 0.0, None)
            .ok()
            .expect("setup_experiment");
        instance1
            .enter_initialization_mode()
            .ok()
            .expect("enter_initialization_mode");
        instance1
            .exit_initialization_mode()
            .ok()
            .expect("exit_initialization_mode");
        instance1.terminate().ok().expect("terminate");
        instance1.reset().ok().expect("reset");
    }

    /// Tests on variable module requiring an instance.
    #[test]
    #[cfg(disabled)]
    fn test_variable() {
        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap()
        .as_fmi2()
        .unwrap();

        let inst = Instance::<ME>::new(&import, "inst1", false, true).unwrap();

        let mut vars = import.model_description().get_model_variables();
        let _ = Var::from_scalar_variable(&inst, vars.next().unwrap().1);

        assert!(matches!(
            Var::from_name(&inst, "false"),
            Err(FmiError::ModelDescr(
                ModelDescriptionError::VariableNotFound { .. }
            ))
        ));
    }

    #[test]
    fn test_instance_cs() {
        let import = Import::new(std::path::Path::new(
            "data/Modelica_Blocks_Sources_Sine.fmu",
        ))
        .unwrap()
        .as_fmi2()
        .unwrap();

        let instance1 = Instance::<CS>::new(&import, "inst1", false, true).unwrap();
        assert_eq!(instance1.version(), "2.0");

        instance1
            .setup_experiment(Some(1.0e-6_f64), 0.0, None)
            .ok()
            .expect("setup_experiment");

        instance1
            .enter_initialization_mode()
            .ok()
            .expect("enter_initialization_mode");

        let sv = import
            .model_description()
            .model_variable_by_name("freqHz")
            .unwrap();
        instance1
            .set_real(&[sv.value_reference], &[2.0f64])
            .ok()
            .expect("set freqHz parameter");

        instance1
            .exit_initialization_mode()
            .ok()
            .expect("exit_initialization_mode");

        let sv = import
            .model_description()
            .model_variable_by_name("y")
            .unwrap();

        let mut y = [0.0];

        instance1
            .get_real(&[sv.value_reference], &mut y)
            .ok()
            .unwrap();

        assert_eq!(y, [1.0e-6]);

        instance1.do_step(0.0, 0.125, false).ok().expect("do_step");

        instance1
            .get_real(&[sv.value_reference], &mut y)
            .ok()
            .unwrap();

        assert_eq!(y, [1.0]);
    }
}
