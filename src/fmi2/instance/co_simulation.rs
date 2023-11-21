use crate::{fmi2::import, import::FmiImport};

use super::{Instance, CS};

impl Instance<CS> {
    /// Initialize a new Instance from an Import
    pub fn new(
        import: &import::Fmi2,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> FmiResult<Self> {
        let binding = import.raw_bindings()?;

        let callbacks = Box::new(CallbackFunctions::default());
        //let cs = import.container_cs()?;
        //check_consistency(&import, &cs.common)?;

        let instance_name = CString::new(instance_name).expect("Error building CString");
        let guid = CString::new(import.descr().guid.as_bytes()).expect("Error building CString");
        let resource_url =
            CString::new(import.resource_url().as_str()).expect("Error building CString");

        let comp = unsafe {
            binding.fmi2Instantiate(
                instance_name.as_ptr(),
                fmi2Type::CoSimulation,
                guid.as_ptr(),             // guid
                resource_url.as_ptr(),     // fmu_resource_location
                &*callbacks,               // functions
                visible as fmi2Boolean,    // visible
                logging_on as fmi2Boolean, // logging_on
            )
        };
        if comp.is_null() {
            return Err(FmiError::Instantiation);
        }
        trace!("Created CS component {:?}", comp);

        Ok(Self {
            name: instance_name.to_owned(),
            binding,
            callbacks,
            component: comp,
            num_states: import.descr().num_states(),
            num_event_indicators: import.descr().num_event_indicators(),
        })
    }
}
