use std::ffi::{CStr, CString};

use crate::{
    fmi2::{import, CallbackFunctions, Fmi2Status},
    import::FmiImport,
    Error,
};

use super::{binding, traits, Instance, CS};

impl<'a> Instance<'a, CS> {
    /// Initialize a new Instance from an Import
    pub fn new(
        import: &'a import::Fmi2,
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
        //.check_consistency(&import, &cs.common)?;

        let name = instance_name.to_owned();

        let instance_name = CString::new(instance_name).expect("Error building CString");
        let guid = CString::new(schema.guid.as_bytes()).expect("Error building CString");
        let resource_url =
            CString::new(import.resource_url().as_str()).expect("Error building CString");

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
        log::trace!("Created CS component {:?}", component);

        Ok(Self {
            binding,
            component,
            model_description: schema,
            callbacks,
            name,
            _tag: std::marker::PhantomData,
        })
    }
}

impl<'a> traits::CoSimulation for Instance<'a, CS> {
    fn do_step(
        &self,
        current_communication_point: f64,
        communication_step_size: f64,
        new_step: bool,
    ) -> Fmi2Status {
        Fmi2Status(unsafe {
            self.binding.fmi2DoStep(
                self.component,
                current_communication_point,
                communication_step_size,
                new_step as _,
            )
        })
    }

    fn cancel_step(&self) -> Fmi2Status {
        Fmi2Status(unsafe { self.binding.fmi2CancelStep(self.component) })
    }

    fn do_step_status(&mut self) -> Result<Fmi2Status, Error> {
        let mut ret = binding::fmi2Status_fmi2OK;
        Fmi2Status(unsafe {
            self.binding.fmi2GetStatus(
                self.component,
                binding::fmi2StatusKind_fmi2DoStepStatus,
                &mut ret,
            )
        })
        .ok()?;
        Ok(Fmi2Status(ret))
    }

    fn pending_status(&mut self) -> Result<&str, Error> {
        let str_ret = CStr::from_bytes_with_nul(b"\0").unwrap();
        Fmi2Status(unsafe {
            self.binding.fmi2GetStringStatus(
                self.component,
                binding::fmi2StatusKind_fmi2PendingStatus,
                &mut str_ret.as_ptr(),
            )
        })
        .ok()?;
        Ok(str_ret.to_str()?)
    }

    fn last_successful_time(&mut self) -> Result<f64, Error> {
        let mut ret = 0.0;
        Fmi2Status(unsafe {
            self.binding.fmi2GetRealStatus(
                self.component,
                binding::fmi2StatusKind_fmi2LastSuccessfulTime,
                &mut ret,
            )
        })
        .ok()?;
        Ok(ret)
    }

    fn terminated(&mut self) -> Result<bool, Error> {
        let mut ret = 0i32;
        Fmi2Status(unsafe {
            self.binding.fmi2GetBooleanStatus(
                self.component,
                binding::fmi2StatusKind_fmi2Terminated,
                &mut ret,
            )
        })
        .ok()?;
        Ok(ret != 0)
    }
}
