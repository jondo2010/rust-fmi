use std::ffi::CString;

use libc::bind;

use crate::{
    fmi2::{
        binding::{self},
        import,
        instance::traits::ModelExchange,
        CallbackFunctions, EventInfo, FmiStatus,
    },
    import::FmiImport,
    FmiError, FmiResult,
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
        let schema = import.raw_schema();

        let callbacks = Box::new(CallbackFunctions::default());
        //let me = import.container_me()?;
        //check_consistency(&import, &me.common)?;

        let instance_name = CString::new(instance_name).expect("Error building CString");
        let guid = CString::new(schema.guid.as_bytes()).expect("Error building CString");
        let resource_url =
            CString::new(import.resource_url().as_str()).expect("Error building CString");

        let component = unsafe {
            binding.fmi2Instantiate(
                instance_name.as_ptr(),
                binding::fmi2Type_fmi2ModelExchange,
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
        log::trace!("Created ME component {:?}", component);

        Ok(Self {
            binding,
            component,
            schema,
            callbacks,
            _tag: std::marker::PhantomData,
        })
    }

    /// Helper for event iteration
    /// Returned tuple is (nominals_of_continuous_states_changed,
    /// values_of_continuous_states_changed)
    pub fn do_event_iteration(&self) -> FmiResult<(bool, bool)> {
        let mut event_info = EventInfo {
            new_discrete_states_needed: true as _,
            terminate_simulation: false as _,
            nominals_of_continuous_states_changed: false as _,
            values_of_continuous_states_changed: false as _,
            next_event_time_defined: false as _,
            next_event_time: 0.0,
        };

        while (event_info.new_discrete_states_needed as _ == true)
            && (event_info.terminate_simulation as _ == true)
        {
            log::trace!("Iterating while new_discrete_states_needed=true");
            self.new_discrete_states(&mut event_info)?;
        }

        assert_eq!(
            event_info.terminate_simulation as _, false,
            "terminate_simulation in=true do_event_iteration!"
        );

        Ok((
            event_info.nominals_of_continuous_states_changed as _ == true,
            event_info.values_of_continuous_states_changed as _ == true,
        ))
    }
}

impl<'a> ModelExchange for Instance<'a, ME> {
    fn enter_event_mode(&self) -> FmiResult<FmiStatus> {
        unsafe { self.binding.fmi2EnterEventMode(self.component) }.into()
    }

    fn new_discrete_states(&self, event_info: &mut EventInfo) -> FmiResult<FmiStatus> {
        unsafe {
            self.binding
                .fmi2NewDiscreteStates(self.component, event_info)
        }
        .into()
    }

    fn enter_continuous_time_mode(&self) -> FmiResult<FmiStatus> {
        unsafe { self.binding.fmi2EnterContinuousTimeMode(self.component) }.into()
    }

    fn completed_integrator_step(
        &self,
        no_set_fmu_state_prior_to_current_point: bool,
    ) -> FmiResult<(bool, bool)> {
        // The returned tuple are the flags (enter_event_mode, terminate_simulation)
        let mut enter_event_mode = false as _;
        let mut terminate_simulation = false as _;
        let res: FmiResult<FmiStatus> = unsafe {
            self.binding.fmi2CompletedIntegratorStep(
                self.component,
                no_set_fmu_state_prior_to_current_point as binding::fmi2Boolean,
                &mut enter_event_mode,
                &mut terminate_simulation,
            )
        }
        .into();
        res.and(Ok((
            enter_event_mode as _ == true,
            terminate_simulation as _ == true,
        )))
    }

    fn set_time(&self, time: f64) -> FmiResult<FmiStatus> {
        let res: FmiStatus = unsafe {
            self.binding
                .fmi2SetTime(self.component, time as binding::fmi2Real)
        }
        .into();
    }

    fn set_continuous_states(&self, states: &[f64]) -> FmiResult<FmiStatus> {
        assert!(states.len() == self.schema.num_states());
        unsafe {
            self.binding
                .fmi2SetContinuousStates(self.component, states.as_ptr(), states.len())
        }
        .into()
    }

    fn get_derivatives(&self, dx: &mut [f64]) -> FmiResult<FmiStatus> {
        assert!(dx.len() == self.schema.num_states());
        unsafe {
            self.binding
                .fmi2GetDerivatives(self.component, dx.as_mut_ptr(), dx.len())
        }
        .into()
    }

    fn get_event_indicators(&self, events: &mut [f64]) -> FmiResult<FmiStatus> {
        assert!(events.len() == self.schema.num_event_indicators());
        unsafe {
            self.binding
                .fmi2GetEventIndicators(self.component, events.as_mut_ptr(), events.len())
        }
        .into()
    }

    fn get_continuous_states(&self, states: &mut [f64]) -> FmiResult<FmiStatus> {
        assert!(states.len() == self.scham.num_states());
        unsafe {
            self.binding
                .fmi2GetContinuousStates(self.component, states.as_mut_ptr(), states.len())
        }
        .into()
    }

    fn get_nominals_of_continuous_states(&self) -> FmiResult<&[f64]> {
        unimplemented!();
    }
}
