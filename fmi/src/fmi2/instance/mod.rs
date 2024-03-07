//! FMI 2.0 instance interface

use crate::{
    traits::{FmiImport, FmiInstance},
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

pub struct Instance<'a, Tag> {
    /// Raw FMI 2.0 bindings
    binding: binding::Fmi2Binding,
    /// Pointer to the raw FMI 2.0 instance
    component: binding::fmi2Component,
    /// Model description
    model_description: &'a schema::FmiModelDescription,
    /// Callbacks struct
    #[allow(dead_code)]
    callbacks: Box<CallbackFunctions>,
    /// Copy of the instance name
    name: String,
    _tag: std::marker::PhantomData<Tag>,
}

impl<'a, Tag> Drop for Instance<'a, Tag> {
    fn drop(&mut self) {
        log::trace!("Freeing component {:?}", self.component);
        unsafe { self.binding.fmi2FreeInstance(self.component) };
    }
}

impl<'a, Tag> FmiInstance for Instance<'a, Tag> {
    type ModelDescription = schema::FmiModelDescription;

    type Import = Fmi2Import;

    type ValueReference = <Fmi2Import as FmiImport>::ValueReference;

    fn name(&self) -> &str {
        &self.name
    }

    fn get_version(&self) -> &str {
        <Self as Common>::get_version(self)
    }

    fn model_description(&self) -> &Self::ModelDescription {
        self.model_description
    }

    fn get_number_of_continuous_state_values(&mut self) -> usize {
        self.model_description.num_states()
    }

    fn get_number_of_event_indicator_values(&mut self) -> usize {
        self.model_description.num_event_indicators()
    }
}

impl Default for CallbackFunctions {
    fn default() -> Self {
        CallbackFunctions {
            logger: Some(super::binding::logger::callback_logger_handler),
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
        let types_platform = self.get_types_platform();
        if types_platform != "default" {
            return Err(Fmi2Error::TypesPlatformMismatch(types_platform.to_owned()).into());
        }

        let fmi_version = <Self as Common>::get_version(self);
        if fmi_version != self.model_description.fmi_version {
            return Err(Error::FmiVersionMismatch {
                found: fmi_version.to_owned(),
                expected: self.model_description.fmi_version.to_owned(),
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
            self.name, self.model_description.model_name, self.component,
        )
    }
}
