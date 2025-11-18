use std::ffi::CString;

use crate::{
    Error,
    fmi2::{CallbackFunctions, Fmi2Error, Fmi2Res, Fmi2Status, import},
    traits::{FmiImport, FmiStatus},
};

use super::{CS, Instance, binding, traits};

impl Instance<CS> {
    /// Initialize a new Instance from an Import
    pub fn new(
        import: &import::Fmi2Import,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> Result<Self, Error> {
        let schema = import.model_description();

        let co_simulation = schema
            .co_simulation
            .as_ref()
            .ok_or(Error::UnsupportedFmuType("CoSimulation".to_owned()))?;

        let binding = import.binding(&co_simulation.model_identifier)?;

        let callbacks = Box::<CallbackFunctions>::default();

        let name = instance_name.to_owned();

        let instance_name = CString::new(instance_name).expect("Error building CString");
        let guid = CString::new(schema.guid.as_bytes()).expect("Error building CString");
        let resource_url =
            CString::new(import.canonical_resource_path_string()).expect("Invalid resource path");

        let component = unsafe {
            let callback_functions = &*callbacks as *const CallbackFunctions;
            binding.fmi2Instantiate(
                instance_name.as_ptr(),
                binding::fmi2Type_fmi2CoSimulation,
                guid.as_ptr(),                      // guid
                resource_url.as_ptr(),              // fmu_resource_location
                callback_functions as _,            // functions
                visible as binding::fmi2Boolean,    // visible
                logging_on as binding::fmi2Boolean, // logging_on
            )
        };
        if component.is_null() {
            return Err(Error::Instantiation);
        }
        log::trace!("Created FMI2.0 CS component {component:?}");

        Ok(Self {
            binding,
            component,
            callbacks,
            name,
            saved_states: Vec::new(),
            _tag: std::marker::PhantomData,
        })
    }
}

impl traits::CoSimulation for Instance<CS> {
    fn do_step(
        &self,
        current_communication_point: f64,
        communication_step_size: f64,
        new_step: bool,
    ) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe {
            self.binding.fmi2DoStep(
                self.component,
                current_communication_point,
                communication_step_size,
                new_step as _,
            )
        })
        .ok()
    }

    fn cancel_step(&self) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe { self.binding.fmi2CancelStep(self.component) }).ok()
    }

    fn do_step_status(&mut self) -> Result<Fmi2Status, Fmi2Error> {
        let mut ret = binding::fmi2Status_fmi2OK;
        Fmi2Status(unsafe {
            self.binding.fmi2GetStatus(
                self.component,
                binding::fmi2StatusKind_fmi2DoStepStatus,
                &mut ret,
            )
        })
        .ok()
        .map(|_| Fmi2Status(ret))
    }

    fn pending_status(&mut self) -> Result<&str, Fmi2Error> {
        let str_ret = c"";
        Fmi2Status(unsafe {
            self.binding.fmi2GetStringStatus(
                self.component,
                binding::fmi2StatusKind_fmi2PendingStatus,
                &mut str_ret.as_ptr(),
            )
        })
        .ok()
        .map(|_| str_ret.to_str().unwrap())
    }

    fn last_successful_time(&mut self) -> Result<f64, Fmi2Error> {
        let mut ret = 0.0;
        Fmi2Status(unsafe {
            self.binding.fmi2GetRealStatus(
                self.component,
                binding::fmi2StatusKind_fmi2LastSuccessfulTime,
                &mut ret,
            )
        })
        .ok()
        .map(|_| ret)
    }

    fn terminated(&mut self) -> Result<bool, Fmi2Error> {
        let mut ret = 0i32;
        Fmi2Status(unsafe {
            self.binding.fmi2GetBooleanStatus(
                self.component,
                binding::fmi2StatusKind_fmi2Terminated,
                &mut ret,
            )
        })
        .ok()
        .map(|_| ret != 0)
    }
}
