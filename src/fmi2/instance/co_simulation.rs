use std::ffi::CString;

use crate::{
    fmi2::{import, FmiStatus},
    import::FmiImport,
    FmiError, FmiResult,
};

use super::{binding, traits, Instance, CS};

impl<'a> Instance<'a, CS> {
    /// Initialize a new Instance from an Import
    pub fn new(
        import: &import::Fmi2,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> FmiResult<Self> {
        let binding = import.raw_bindings()?;
        let schema = import.raw_schema();

        let callbacks = Box::new(CallbackFunctions::default());
        //let cs = import.container_cs()?;
        //check_consistency(&import, &cs.common)?;

        let instance_name = CString::new(instance_name).expect("Error building CString");
        let guid = CString::new(schema.guid.as_bytes()).expect("Error building CString");
        let resource_url =
            CString::new(import.resource_url().as_str()).expect("Error building CString");

        let component = unsafe {
            binding.fmi2Instantiate(
                instance_name.as_ptr(),
                binding::fmi2Type_fmi2CoSimulation,
                guid.as_ptr(),                      // guid
                resource_url.as_ptr(),              // fmu_resource_location
                &*callbacks,                        // functions
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
    ) -> FmiResult<FmiStatus> {
        unsafe {
            self.container.cs.do_step(
                self.component,
                current_communication_point,
                communication_step_size,
                new_step as fmi2Boolean,
            )
        }
        .into()
    }

    fn cancel_step(&self) -> FmiResult<FmiStatus> {
        unsafe { self.container.cs.cancel_step(self.component) }.into()
    }

    fn get_status(&self, kind: fmi2StatusKind) -> FmiResult<FmiStatus> {
        let mut ret = fmi2Status::OK;
        let _ = FmiResult::<FmiStatus>::from(unsafe {
            self.container.cs.get_status(self.component, kind, &mut ret)
        })?;
        ret.into()
    }
}
