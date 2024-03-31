//! FMI 2.0 instance interface

use crate::{
    traits::{FmiImport, FmiInstance, FmiStatus},
    Error,
};

use super::{binding, import::Fmi2Import, schema, CallbackFunctions, Fmi2Error, Fmi2Status};

mod co_simulation;
mod common;
mod model_exchange;
mod traits;

pub use traits::{CoSimulation, Common, ModelExchange};

/// Tag for Model Exchange instances
pub struct ME;
/// Tag for Co-Simulation instances
pub struct CS;

pub type InstanceME<'a> = Instance<'a, ME>;
pub type InstanceCS<'a> = Instance<'a, CS>;

pub struct FmuState(usize);

pub struct Instance<'a, Tag> {
    /// Copy of the instance name
    name: String,
    /// Raw FMI 2.0 bindings
    binding: binding::Fmi2Binding,
    /// Pointer to the raw FMI 2.0 instance
    component: binding::fmi2Component,
    /// Model description
    model_description: &'a schema::Fmi2ModelDescription,
    /// Callbacks struct
    #[allow(dead_code)]
    callbacks: Box<CallbackFunctions>,
    /// Allocated FMU states
    saved_states: Vec<binding::fmi2FMUstate>,
    _tag: std::marker::PhantomData<Tag>,
}

impl<'a, Tag> Drop for Instance<'a, Tag> {
    fn drop(&mut self) {
        log::trace!("Freeing component {:?}", self.component);
        unsafe {
            for state in &mut self.saved_states {
                self.binding.fmi2FreeFMUstate(self.component, state);
            }
            self.binding.fmi2FreeInstance(self.component)
        };
    }
}

impl<'a, Tag> FmiInstance for Instance<'a, Tag> {
    type ModelDescription = schema::Fmi2ModelDescription;
    type Import = Fmi2Import;
    type ValueRef = <Fmi2Import as FmiImport>::ValueRef;
    type Status = Fmi2Status;

    fn name(&self) -> &str {
        &self.name
    }

    /// The FMI-standard version string
    fn get_version(&self) -> &str {
        Common::get_version(self)
    }

    fn model_description(&self) -> &Self::ModelDescription {
        self.model_description
    }

    fn set_debug_logging(&mut self, logging_on: bool, categories: &[&str]) -> Self::Status {
        Common::set_debug_logging(self, logging_on, categories)
    }

    fn get_number_of_continuous_state_values(&mut self) -> usize {
        self.model_description.num_states()
    }

    fn get_number_of_event_indicator_values(&mut self) -> usize {
        self.model_description.num_event_indicators()
    }

    fn terminate(&mut self) -> Fmi2Status {
        Common::terminate(self)
    }

    fn reset(&mut self) -> Fmi2Status {
        Common::reset(self)
    }
}

impl Default for CallbackFunctions {
    fn default() -> Self {
        CallbackFunctions {
            logger: Some(super::binding::logger::callback_logger_handler as _),
            allocate_memory: Some(libc::calloc),
            free_memory: Some(libc::free),
            step_finished: None,
            component_environment: std::ptr::null_mut::<std::os::raw::c_void>(),
        }
    }
}

impl<'a, Tag> Instance<'a, Tag> {
    fn get_fmu_state(&mut self) -> Result<FmuState, Fmi2Error> {
        let mut state = std::ptr::null_mut();
        Fmi2Status(unsafe { self.binding.fmi2GetFMUstate(self.component, &mut state) }).ok()?;

        if state.is_null() {
            log::error!("FMU returned a null state");
            Err(Fmi2Error::Fatal)
        } else {
            self.saved_states.push(state);
            Ok(FmuState(self.saved_states.len() - 1))
        }
    }

    fn set_fmu_state(&mut self, state: FmuState) -> Fmi2Status {
        let state = self.saved_states.get(state.0).unwrap();
        unsafe { self.binding.fmi2SetFMUstate(self.component, *state) }.into()
    }

    fn update_fmu_state(&mut self, state: FmuState) -> Fmi2Status {
        let state = self.saved_states.get_mut(state.0).unwrap();
        unsafe { self.binding.fmi2GetFMUstate(self.component, state) }.into()
    }

    /// Check the internal consistency of the FMU by comparing the TypesPlatform and FMI versions
    /// from the library and the Model Description XML
    pub fn check_consistency(&self) -> Result<(), Error> {
        let types_platform = self.get_types_platform();
        if types_platform != "default" {
            return Err(Fmi2Error::TypesPlatformMismatch(types_platform.to_owned()).into());
        }

        let fmi_version = Common::get_version(self);
        if fmi_version != self.model_description.fmi_version {
            return Err(Error::FmiVersionMismatch {
                found: fmi_version.to_owned(),
                expected: self.model_description.fmi_version.to_owned(),
            });
        }

        Ok(())
    }
}

impl<'a, A> std::fmt::Debug for Instance<'a, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Instance {} {{Import {}, {:?}}}",
            self.name, self.model_description.model_name, self.component,
        )
    }
}
