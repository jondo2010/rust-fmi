use std::ffi::CString;

use crate::{
    fmi2::{binding, import, FmiStatus},
    FmiResult,
};

use super::{traits, Instance, ME};

impl<'a> Instance<'a, ME> {
    /// Initialize a new Instance from an Import
    pub fn new(
        import: &import::Fmi2,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> FmiResult<Self> {
        let binding = import.raw_bindings()?;

        let callbacks = Box::new(CallbackFunctions::default());
        //let me = import.container_me()?;
        //check_consistency(&import, &me.common)?;

        let instance_name = CString::new(instance_name).expect("Error building CString");
        let guid = CString::new(import.descr().guid.as_bytes()).expect("Error building CString");
        let resource_url =
            CString::new(import.resource_url().as_str()).expect("Error building CString");

        let comp = unsafe {
            binding.fmi2Instantiate(
                instance_name.as_ptr(),
                ModelExchange,
                guid.as_ptr(),                      // guid
                resource_url.as_ptr(),              // fmu_resource_location
                &*callbacks,                        // functions
                visible as binding::fmi2Boolean,    // visible
                logging_on as binding::fmi2Boolean, // logging_on
            )
        };
        if comp.is_null() {
            return Err(FmiError::Instantiation);
        }
        trace!("Created ME component {:?}", comp);

        Ok(Self {
            name: instance_name.to_owned(),
            model_name: import.descr().model_name.clone(),
            container: me,
            callbacks,
            component: comp,
            num_states: import.descr().num_states(),
            num_event_indicators: import.descr().num_event_indicators(),
        })
    }

    /// Helper for event iteration
    /// Returned tuple is (nominals_of_continuous_states_changed,
    /// values_of_continuous_states_changed)
    pub fn do_event_iteration(&self) -> FmiResult<(bool, bool)> {
        let mut event_info = EventInfo {
            new_discrete_states_needed: true,
            terminate_simulation: false,
            nominals_of_continuous_states_changed: false,
            values_of_continuous_states_changed: false,
            next_event_time_defined: false,
            next_event_time: 0.0,
        };

        while (event_info.new_discrete_states_needed == true)
            && (event_info.terminate_simulation == true)
        {
            log::trace!("Iterating while new_discrete_states_needed=true");
            self.new_discrete_states(&mut event_info)?;
        }

        assert_eq!(
            event_info.terminate_simulation, false,
            "terminate_simulation in=true do_event_iteration!"
        );

        Ok((
            event_info.nominals_of_continuous_states_changed == true,
            event_info.values_of_continuous_states_changed == true,
        ))
    }
}

impl<'a> traits::ModelExchange for Instance<'a, ME> {
    fn enter_event_mode(&self) -> FmiResult<FmiStatus> {
        unsafe { self.container.me.enter_event_mode(self.component) }.into()
    }

    fn new_discrete_states(&self, event_info: &mut EventInfo) -> FmiResult<FmiStatus> {
        unsafe {
            self.container
                .me
                .new_discrete_states(self.component, event_info)
        }
        .into()
    }

    fn enter_continuous_time_mode(&self) -> FmiResult<FmiStatus> {
        unsafe { self.container.me.enter_continuous_time_mode(self.component) }.into()
    }

    fn completed_integrator_step(
        &self,
        no_set_fmu_state_prior_to_current_point: bool,
    ) -> FmiResult<(bool, bool)> {
        // The returned tuple are the flags (enter_event_mode, terminate_simulation)
        let mut enter_event_mode = false;
        let mut terminate_simulation = false;
        let res: FmiResult<FmiStatus> = unsafe {
            self.container.me.completed_integrator_step(
                self.component,
                no_set_fmu_state_prior_to_current_point as binding::fmi2Boolean,
                &mut enter_event_mode,
                &mut terminate_simulation,
            )
        }
        .into();
        res.and(Ok((enter_event_mode == true, terminate_simulation == true)))
    }

    fn set_time(&self, time: f64) -> FmiResult<FmiStatus> {
        unsafe {
            self.container
                .me
                .set_time(self.component, time as binding::fmi2Real)
        }
        .into()
    }

    fn set_continuous_states(&self, states: &[f64]) -> FmiResult<FmiStatus> {
        assert!(states.len() == self.num_states);
        unsafe {
            self.container
                .me
                .set_continuous_states(self.component, states.as_ptr(), states.len())
        }
        .into()
    }

    fn get_derivatives(&self, dx: &mut [f64]) -> FmiResult<FmiStatus> {
        assert!(dx.len() == self.num_states);
        unsafe {
            self.container
                .me
                .get_derivatives(self.component, dx.as_mut_ptr(), dx.len())
        }
        .into()
    }

    fn get_event_indicators(&self, events: &mut [f64]) -> FmiResult<FmiStatus> {
        assert!(events.len() == self.num_event_indicators);
        unsafe {
            self.container.me.get_event_indicators(
                self.component,
                events.as_mut_ptr(),
                events.len(),
            )
        }
        .into()
    }

    fn get_continuous_states(&self, states: &mut [f64]) -> FmiResult<FmiStatus> {
        assert!(states.len() == self.num_states);
        unsafe {
            self.container.me.get_continuous_states(
                self.component,
                states.as_mut_ptr(),
                states.len(),
            )
        }
        .into()
    }

    fn get_nominals_of_continuous_states(&self) -> FmiResult<&[f64]> {
        unimplemented!();
    }
}
