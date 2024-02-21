use std::ffi::CString;

use super::{binding, traits::ModelExchange, CallbackFunctions, Fmi2Status, Instance, ME};
use crate::{
    fmi2::{import, EventInfo},
    traits::FmiImport,
    Error,
};

impl<'a> Instance<'a, ME> {
    /// Initialize a new Instance from an Import
    pub fn new(
        import: &'a import::Fmi2Import,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> Result<Self, Error> {
        let schema = import.model_description();

        let model_exchange = schema
            .model_exchange
            .as_ref()
            .ok_or(Error::UnsupportedFmuType("ModelExchange".to_owned()))?;

        let binding = import.binding(&model_exchange.model_identifier)?;

        let callbacks = Box::<CallbackFunctions>::default();
        // check_consistency(&import, &me.common)?;

        let name = instance_name.to_owned();

        let instance_name = CString::new(instance_name).expect("Error building CString");
        let guid = CString::new(schema.guid.as_bytes()).expect("Error building CString");
        let resource_url =
            CString::new(import.resource_url().as_str()).expect("Error building CString");

        let component = unsafe {
            let callback_functions = &*callbacks as *const CallbackFunctions;
            binding.fmi2Instantiate(
                instance_name.as_ptr(),
                binding::fmi2Type_fmi2ModelExchange,
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
        log::trace!("Created ME component {:?}", component);

        Ok(Self {
            binding,
            component,
            model_description: schema,
            callbacks,
            name,
            _tag: std::marker::PhantomData,
        })
    }

    /// Helper for event iteration
    /// Returned tuple is (nominals_of_continuous_states_changed,
    /// values_of_continuous_states_changed)
    pub fn do_event_iteration(&self) -> Result<(bool, bool), Error> {
        let mut event_info = EventInfo {
            new_discrete_states_needed: true as _,
            terminate_simulation: false as _,
            nominals_of_continuous_states_changed: false as _,
            values_of_continuous_states_changed: false as _,
            next_event_time_defined: false as _,
            next_event_time: 0.0,
        };

        while (event_info.new_discrete_states_needed != 0) && (event_info.terminate_simulation != 0)
        {
            log::trace!("Iterating while new_discrete_states_needed=true");
            self.new_discrete_states(&mut event_info).ok()?;
        }

        assert!(
            event_info.terminate_simulation != 0,
            "terminate_simulation in=true do_event_iteration!"
        );

        Ok((
            event_info.nominals_of_continuous_states_changed != 0,
            event_info.values_of_continuous_states_changed != 0,
        ))
    }
}

impl<'a> ModelExchange for Instance<'a, ME> {
    fn enter_event_mode(&self) -> Fmi2Status {
        Fmi2Status(unsafe { self.binding.fmi2EnterEventMode(self.component) })
    }

    fn new_discrete_states(&self, event_info: &mut EventInfo) -> Fmi2Status {
        let event_info: *mut binding::fmi2EventInfo = &*event_info as *const _ as _;
        Fmi2Status(unsafe {
            self.binding
                .fmi2NewDiscreteStates(self.component, event_info)
        })
    }

    fn enter_continuous_time_mode(&self) -> Fmi2Status {
        Fmi2Status(unsafe { self.binding.fmi2EnterContinuousTimeMode(self.component) })
    }

    fn completed_integrator_step(
        &self,
        no_set_fmu_state_prior_to_current_point: bool,
    ) -> (Fmi2Status, bool, bool) {
        // The returned tuple are the flags (enter_event_mode, terminate_simulation)
        let enter_event_mode = false;
        let terminate_simulation = false;
        let res = Fmi2Status(unsafe {
            self.binding.fmi2CompletedIntegratorStep(
                self.component,
                no_set_fmu_state_prior_to_current_point as binding::fmi2Boolean,
                &mut (enter_event_mode as _),
                &mut (terminate_simulation as _),
            )
        });
        (res, enter_event_mode, terminate_simulation)
    }

    fn set_time(&self, time: f64) -> Fmi2Status {
        Fmi2Status(unsafe {
            self.binding
                .fmi2SetTime(self.component, time as binding::fmi2Real)
        })
    }

    fn set_continuous_states(&self, states: &[f64]) -> Fmi2Status {
        assert!(states.len() == self.model_description.num_states());
        Fmi2Status(unsafe {
            self.binding
                .fmi2SetContinuousStates(self.component, states.as_ptr(), states.len())
        })
    }

    fn get_derivatives(&self, dx: &mut [f64]) -> Fmi2Status {
        assert!(dx.len() == self.model_description.num_states());
        Fmi2Status(unsafe {
            self.binding
                .fmi2GetDerivatives(self.component, dx.as_mut_ptr(), dx.len())
        })
    }

    fn get_event_indicators(&self, events: &mut [f64]) -> Fmi2Status {
        assert_eq!(events.len(), self.model_description.num_event_indicators());
        Fmi2Status(unsafe {
            self.binding
                .fmi2GetEventIndicators(self.component, events.as_mut_ptr(), events.len())
        })
    }

    fn get_continuous_states(&self, states: &mut [f64]) -> Fmi2Status {
        assert_eq!(states.len(), self.model_description.num_states());
        Fmi2Status(unsafe {
            self.binding
                .fmi2GetContinuousStates(self.component, states.as_mut_ptr(), states.len())
        })
    }

    fn get_nominals_of_continuous_states(&self, nominals: &mut [f64]) -> Fmi2Status {
        assert_eq!(nominals.len(), self.model_description.num_states());
        Fmi2Status(unsafe {
            self.binding.fmi2GetNominalsOfContinuousStates(
                self.component,
                nominals.as_mut_ptr() as _,
                nominals.len(),
            )
        })
    }
}
