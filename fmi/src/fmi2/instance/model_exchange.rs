use std::ffi::CString;

use super::{CallbackFunctions, Instance, ME, binding, traits::ModelExchange};
use crate::{
    Error,
    fmi2::{Fmi2Error, Fmi2Res, Fmi2Status, import},
    traits::{FmiEventHandler, FmiImport, FmiModelExchange, FmiStatus},
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
            CString::new(import.canonical_resource_path_string()).expect("Invalid resource path");

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
        log::trace!("Created FMI2.0 ME component {component:?}");

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
    fn enter_continuous_time_mode(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe { self.binding.fmi2EnterContinuousTimeMode(self.component) }).ok()
    }

    fn enter_event_mode(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe { self.binding.fmi2EnterEventMode(self.component) }).ok()
    }

    fn new_discrete_states(
        &mut self,
        discrete_states_need_update: &mut bool,
        terminate_simulation: &mut bool,
        nominals_of_continuous_states_changed: &mut bool,
        values_of_continuous_states_changed: &mut bool,
        next_event_time: &mut Option<f64>,
    ) -> Result<Fmi2Res, Fmi2Error> {
        let mut event_info = binding::fmi2EventInfo::default();
        let result = Fmi2Status::from(unsafe {
            self.binding
                .fmi2NewDiscreteStates(self.component, &mut event_info)
        })
        .ok();
        *discrete_states_need_update = event_info.newDiscreteStatesNeeded != 0;
        *terminate_simulation = event_info.terminateSimulation != 0;
        *nominals_of_continuous_states_changed = event_info.nominalsOfContinuousStatesChanged != 0;
        *values_of_continuous_states_changed = event_info.valuesOfContinuousStatesChanged != 0;
        *next_event_time = if event_info.nextEventTimeDefined != 0 {
            Some(event_info.nextEventTime)
        } else {
            None
        };
        result
    }

    fn completed_integrator_step(
        &mut self,
        no_set_fmu_state_prior: bool,
        enter_event_mode: &mut bool,
        terminate_simulation: &mut bool,
    ) -> Result<Fmi2Res, Fmi2Error> {
        let mut _enter_event_mode = 0;
        let mut _terminate_simulation = 0;
        let result = Fmi2Status::from(unsafe {
            self.binding.fmi2CompletedIntegratorStep(
                self.component,
                no_set_fmu_state_prior as _,
                &mut _enter_event_mode,
                &mut _terminate_simulation,
            )
        })
        .ok();
        *enter_event_mode = _enter_event_mode != 0;
        *terminate_simulation = _terminate_simulation != 0;
        result
    }

    fn set_time(&mut self, time: f64) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe { self.binding.fmi2SetTime(self.component, time) }).ok()
    }

    fn get_continuous_states(
        &mut self,
        continuous_states: &mut [f64],
    ) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe {
            self.binding.fmi2GetContinuousStates(
                self.component,
                continuous_states.as_mut_ptr(),
                continuous_states.len(),
            )
        })
        .ok()
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe {
            self.binding
                .fmi2SetContinuousStates(self.component, states.as_ptr(), states.len())
        })
        .ok()
    }

    fn get_derivatives(&mut self, derivatives: &mut [f64]) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe {
            self.binding.fmi2GetDerivatives(
                self.component,
                derivatives.as_mut_ptr(),
                derivatives.len(),
            )
        })
        .ok()
    }

    fn get_nominals_of_continuous_states(
        &mut self,
        nominals: &mut [f64],
    ) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe {
            self.binding.fmi2GetNominalsOfContinuousStates(
                self.component,
                nominals.as_mut_ptr(),
                nominals.len(),
            )
        })
        .ok()
    }

    fn get_event_indicators(&mut self, event_indicators: &mut [f64]) -> Result<bool, Fmi2Error> {
        let status = unsafe {
            self.binding.fmi2GetEventIndicators(
                self.component,
                event_indicators.as_mut_ptr(),
                event_indicators.len(),
            )
        };

        // Convert status and handle Discard case
        match Fmi2Status::from(status).ok() {
            Ok(_) => Ok(true), // Successfully computed indicators
            Err(Fmi2Error::Discard) => {
                // FMU couldn't compute indicators due to numerical issues
                // but this is a recoverable condition
                Ok(false)
            }
            Err(e) => Err(e), // Other errors
        }
    }
}

impl FmiModelExchange for Instance<'_, ME> {
    fn enter_continuous_time_mode(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::enter_continuous_time_mode(self)
    }

    fn enter_event_mode(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::enter_event_mode(self)
    }

    fn update_discrete_states(
        &mut self,
        discrete_states_need_update: &mut bool,
        terminate_simulation: &mut bool,
        nominals_of_continuous_states_changed: &mut bool,
        values_of_continuous_states_changed: &mut bool,
        next_event_time: &mut Option<f64>,
    ) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::new_discrete_states(
            self,
            discrete_states_need_update,
            terminate_simulation,
            nominals_of_continuous_states_changed,
            values_of_continuous_states_changed,
            next_event_time,
        )
    }

    fn completed_integrator_step(
        &mut self,
        no_set_fmu_state_prior: bool,
        enter_event_mode: &mut bool,
        terminate_simulation: &mut bool,
    ) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::completed_integrator_step(
            self,
            no_set_fmu_state_prior,
            enter_event_mode,
            terminate_simulation,
        )
    }

    fn set_time(&mut self, time: f64) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::set_time(self, time)
    }

    fn get_continuous_states(
        &mut self,
        continuous_states: &mut [f64],
    ) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::get_continuous_states(self, continuous_states)
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::set_continuous_states(self, states)
    }

    fn get_continuous_state_derivatives(
        &mut self,
        derivatives: &mut [f64],
    ) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::get_derivatives(self, derivatives)
    }

    fn get_nominals_of_continuous_states(
        &mut self,
        nominals: &mut [f64],
    ) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::get_nominals_of_continuous_states(self, nominals)
    }

    fn get_event_indicators(
        &mut self,
        event_indicators: &mut [f64],
    ) -> Result<bool, <Self::Status as crate::traits::FmiStatus>::Err> {
        ModelExchange::get_event_indicators(self, event_indicators)
    }

    fn get_number_of_event_indicators(
        &mut self,
    ) -> Result<usize, <Self::Status as crate::traits::FmiStatus>::Err> {
        Ok(self.model_description.num_event_indicators())
    }
}

impl FmiEventHandler for Instance<'_, ME> {
    fn enter_event_mode(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::enter_event_mode(self)
    }

    fn update_discrete_states(
        &mut self,
        discrete_states_need_update: &mut bool,
        terminate_simulation: &mut bool,
        nominals_of_continuous_states_changed: &mut bool,
        values_of_continuous_states_changed: &mut bool,
        next_event_time: &mut Option<f64>,
    ) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::new_discrete_states(
            self,
            discrete_states_need_update,
            terminate_simulation,
            nominals_of_continuous_states_changed,
            values_of_continuous_states_changed,
            next_event_time,
        )
    }
}
