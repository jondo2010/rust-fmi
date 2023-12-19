use std::ffi::CString;

use crate::{
    fmi2::{import, CallbackFunctions, FmiStatus, StatusKind},
    import::FmiImport,
    FmiError, FmiResult,
};

use super::{binding, traits, Instance, CS};

impl<'a> Instance<'a, CS> {
    /// Initialize a new Instance from an Import
    pub fn new(
        import: &'a import::Fmi2,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> FmiResult<Self> {
        let binding = import.raw_bindings()?;
        let schema = import.raw_schema();

        let callbacks = Box::new(CallbackFunctions::default());
        //check_consistency(&import, &cs.common)?;

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
            return Err(FmiError::Instantiation);
        }
        log::trace!("Created CS component {:?}", component);

        Ok(Self {
            binding,
            component,
            schema,
            callbacks,
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
    ) -> FmiStatus {
        FmiStatus(unsafe {
            self.binding.fmi2DoStep(
                self.component,
                current_communication_point,
                communication_step_size,
                new_step as _,
            )
        })
    }

    fn cancel_step(&self) -> FmiStatus {
        FmiStatus(unsafe { self.binding.fmi2CancelStep(self.component) })
    }

    fn get_status(&self, kind: StatusKind) -> FmiStatus {
        let mut ret: binding::fmi2Status = binding::fmi2Status_fmi2OK;
        FmiStatus(unsafe {
            self.binding
                .fmi2GetStatus(self.component, kind as _, &mut ret)
        });
        todo!();
    }
}
