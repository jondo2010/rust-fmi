use std::ffi::CString;

use super::{binding, traits::ModelExchange, CallbackFunctions, Instance, ME};
use crate::{
    fmi2::import,
    traits::{FmiImport, FmiModelExchange},
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
        log::trace!("Created FMI2.0 ME component {:?}", component);

        Ok(Self {
            binding,
            component,
            model_description: schema,
            callbacks,
            name,
            saved_states: Vec::new(),
            _tag: std::marker::PhantomData,
        })
    }
}

impl ModelExchange for Instance<'_, ME> {
    fn enter_continuous_time_mode(&mut self) -> Self::Status {
        unsafe { self.binding.fmi2EnterContinuousTimeMode(self.component) }.into()
    }

    fn enter_event_mode(&mut self) -> Self::Status {
        unsafe { self.binding.fmi2EnterEventMode(self.component) }.into()
    }

    fn new_discrete_states(&mut self, event_info: &mut binding::fmi2EventInfo) -> Self::Status {
        unsafe {
            self.binding
                .fmi2NewDiscreteStates(self.component, event_info)
        }
        .into()
    }

    fn completed_integrator_step(
        &mut self,
        no_set_fmu_state_prior: bool,
        enter_event_mode: &mut bool,
        terminate_simulation: &mut bool,
    ) -> Self::Status {
        let mut _enter_event_mode = 0;
        let mut _terminate_simulation = 0;
        let status = unsafe {
            self.binding.fmi2CompletedIntegratorStep(
                self.component,
                no_set_fmu_state_prior as _,
                &mut _enter_event_mode,
                &mut _terminate_simulation,
            )
        }
        .into();
        *enter_event_mode = _enter_event_mode != 0;
        *terminate_simulation = _terminate_simulation != 0;
        status
    }

    fn set_time(&mut self, time: f64) -> Self::Status {
        unsafe { self.binding.fmi2SetTime(self.component, time) }.into()
    }

    fn get_continuous_states(&mut self, continuous_states: &mut [f64]) -> Self::Status {
        unsafe {
            self.binding.fmi2GetContinuousStates(
                self.component,
                continuous_states.as_mut_ptr(),
                continuous_states.len(),
            )
        }
        .into()
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> Self::Status {
        unsafe {
            self.binding
                .fmi2SetContinuousStates(self.component, states.as_ptr(), states.len())
        }
        .into()
    }

    fn get_derivatives(&mut self, derivatives: &mut [f64]) -> Self::Status {
        unsafe {
            self.binding.fmi2GetDerivatives(
                self.component,
                derivatives.as_mut_ptr(),
                derivatives.len(),
            )
        }
        .into()
    }

    fn get_nominals_of_continuous_states(&mut self, nominals: &mut [f64]) -> Self::Status {
        unsafe {
            self.binding.fmi2GetNominalsOfContinuousStates(
                self.component,
                nominals.as_mut_ptr(),
                nominals.len(),
            )
        }
        .into()
    }

    fn get_event_indicators(&mut self, event_indicators: &mut [f64]) -> Self::Status {
        unsafe {
            self.binding.fmi2GetEventIndicators(
                self.component,
                event_indicators.as_mut_ptr(),
                event_indicators.len(),
            )
        }
        .into()
    }
}

impl FmiModelExchange for Instance<'_, ME> {
    fn enter_continuous_time_mode(&mut self) -> Self::Status {
        ModelExchange::enter_continuous_time_mode(self)
    }

    fn enter_event_mode(&mut self) -> Self::Status {
        ModelExchange::enter_event_mode(self)
    }

    fn update_discrete_states(
        &mut self,
        discrete_states_need_update: &mut bool,
        terminate_simulation: &mut bool,
        nominals_of_continuous_states_changed: &mut bool,
        values_of_continuous_states_changed: &mut bool,
        next_event_time: &mut Option<f64>,
    ) -> Self::Status {
        let mut event_info = binding::fmi2EventInfo {
            newDiscreteStatesNeeded: 0,
            terminateSimulation: 0,
            nominalsOfContinuousStatesChanged: 0,
            valuesOfContinuousStatesChanged: 0,
            nextEventTimeDefined: 0,
            nextEventTime: 0.0,
        };
        let status = ModelExchange::new_discrete_states(self, &mut event_info);
        *discrete_states_need_update = event_info.newDiscreteStatesNeeded != 0;
        *terminate_simulation = event_info.terminateSimulation != 0;
        *nominals_of_continuous_states_changed = event_info.nominalsOfContinuousStatesChanged != 0;
        *values_of_continuous_states_changed = event_info.valuesOfContinuousStatesChanged != 0;
        *next_event_time = if event_info.nextEventTimeDefined != 0 {
            Some(event_info.nextEventTime)
        } else {
            None
        };
        status
    }

    fn completed_integrator_step(
        &mut self,
        no_set_fmu_state_prior: bool,
        enter_event_mode: &mut bool,
        terminate_simulation: &mut bool,
    ) -> Self::Status {
        ModelExchange::completed_integrator_step(
            self,
            no_set_fmu_state_prior,
            enter_event_mode,
            terminate_simulation,
        )
    }

    fn set_time(&mut self, time: f64) -> Self::Status {
        ModelExchange::set_time(self, time)
    }

    fn get_continuous_states(&mut self, continuous_states: &mut [f64]) -> Self::Status {
        ModelExchange::get_continuous_states(self, continuous_states)
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> Self::Status {
        ModelExchange::set_continuous_states(self, states)
    }

    fn get_continuous_state_derivatives(&mut self, derivatives: &mut [f64]) -> Self::Status {
        ModelExchange::get_derivatives(self, derivatives)
    }

    fn get_nominals_of_continuous_states(&mut self, nominals: &mut [f64]) -> Self::Status {
        ModelExchange::get_nominals_of_continuous_states(self, nominals)
    }

    fn get_event_indicators(&mut self, event_indicators: &mut [f64]) -> Self::Status {
        ModelExchange::get_event_indicators(self, event_indicators)
    }

    fn get_number_of_event_indicators(
        &self,
        number_of_event_indicators: &mut usize,
    ) -> Self::Status {
        todo!()
    }
}
