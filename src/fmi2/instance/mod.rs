//! FMI 2.0 instance interface

use crate::{fmi2::instance::traits::Common, Error};

use super::{binding, logger, schema, CallbackFunctions, Fmi2Status};

mod co_simulation;
mod common;
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

impl<'a, Tag> Instance<'a, Tag> {
    /// Check the internal consistency of the FMU by comparing the TypesPlatform and FMI versions
    /// from the library and the Model Description XML
    pub fn check_consistency(&self) -> Result<(), Error> {
        // TODO: Fix
        // let types_platform = self.get_types_platform();
        // if types_platform != binding::fmi2TypesPlatform {
        //    return Err(FmiError::TypesPlatformMismatch {
        //        found: types_platform.into(),
        //    });
        //}

        let fmi_version = self.version();
        if fmi_version != self.schema.fmi_version {
            return Err(Error::FmiVersionMismatch {
                found: fmi_version.to_owned(),
                expected: self.schema.fmi_version.to_owned(),
            });
        }

        Ok(())
    }
}

/// FmuState wraps the FMUstate pointer and is used for managing FMU state
#[cfg(feature = "disable")]
pub struct FmuState<'a, Tag> {
    component: &'a Instance<'a, Tag>,
    state: binding::fmi2FMUstate,
    // container: &'a dlopen::wrapper::Container<A>,
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

        assert_eq!(y, [0.0]);

        instance1.do_step(0.0, 0.125, false).ok().expect("do_step");

        instance1
            .get_real(&[sv.value_reference], &mut y)
            .ok()
            .unwrap();

        assert_eq!(y, [1.0]);
    }
}
