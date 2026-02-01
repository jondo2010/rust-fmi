use std::ffi::CString;

use crate::schema::traits::FmiInterfaceType;

use crate::{
    Error,
    fmi3::{Fmi3Error, Fmi3Res, Fmi3Status, ScheduledExecution, binding, import, logger},
    traits::{FmiImport, FmiStatus},
};

use super::{Instance, SE};

unsafe extern "C" fn clock_update(_instance_environment: binding::fmi3InstanceEnvironment) {
    todo!();
}
unsafe extern "C" fn lock_preemption() {
    todo!();
}
unsafe extern "C" fn unlock_preemption() {
    todo!();
}

impl Instance<SE> {
    pub fn new(
        import: &import::Fmi3Import,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> Result<Self, Error> {
        let schema = import.model_description();

        let name = instance_name.to_owned();

        let scheduled_execution = schema
            .scheduled_execution
            .as_ref()
            .ok_or(Error::UnsupportedFmuType("ScheduledExecution".to_owned()))?;

        log::debug!(
            "Instantiating ME: {} '{name}'",
            scheduled_execution.model_identifier()
        );

        let binding = import.binding(scheduled_execution.model_identifier())?;

        let instance_name = CString::new(instance_name).expect("Invalid instance name");
        let instantiation_token = CString::new(schema.instantiation_token.as_bytes())
            .expect("Invalid instantiation token");
        let resource_path =
            CString::new(import.canonical_resource_path_string()).expect("Invalid resource path");

        let instance = unsafe {
            binding.fmi3InstantiateScheduledExecution(
                instance_name.as_ptr(),
                instantiation_token.as_ptr(),
                resource_path.as_ptr() as binding::fmi3String,
                visible,
                logging_on,
                std::ptr::null_mut() as binding::fmi3InstanceEnvironment,
                Some(logger::callback_log),
                Some(clock_update),
                Some(lock_preemption),
                Some(unlock_preemption),
            )
        };

        if instance.is_null() {
            return Err(Error::Instantiation);
        }

        Ok(Self {
            binding,
            ptr: instance,
            name,
            _tag: std::marker::PhantomData,
        })
    }
}

impl ScheduledExecution for Instance<SE> {
    fn activate_model_partition(
        &mut self,
        clock_reference: binding::fmi3ValueReference,
        activation_time: f64,
    ) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe {
            self.binding
                .fmi3ActivateModelPartition(self.ptr, clock_reference, activation_time)
        })
        .ok()
    }
}
