use std::ffi::CString;

use super::{CallbackFunctions, Instance, binding, traits::ModelExchange};
use crate::{
    Error, EventFlags, ME,
    fmi2::{Fmi2Error, Fmi2Res, Fmi2Status, import},
    traits::{FmiEventHandler, FmiImport, FmiModelExchange, FmiStatus},
};

impl Instance<ME> {
    /// Initialize a new Instance from an Import
    pub fn new(
        import: &import::Fmi2Import,
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

        // Cache values from model description
        let num_states = schema.num_states();
        let num_event_indicators = schema.num_event_indicators();
        let fmi_version = schema.fmi_version.clone();
        let model_name = schema.model_name.clone();

        Ok(Self {
            binding,
            component,
            callbacks,
            name,
            saved_states: Vec::new(),
            num_states,
            num_event_indicators,
            fmi_version,
            model_name,
            _tag: std::marker::PhantomData,
        })
    }
}

impl ModelExchange for Instance<ME> {
    fn enter_continuous_time_mode(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe { self.binding.fmi2EnterContinuousTimeMode(self.component) }).ok()
    }

    fn enter_event_mode(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        Fmi2Status::from(unsafe { self.binding.fmi2EnterEventMode(self.component) }).ok()
    }

    fn new_discrete_states(&mut self, event_flags: &mut EventFlags) -> Result<Fmi2Res, Fmi2Error> {
        let mut event_info = binding::fmi2EventInfo::default();
        let result = Fmi2Status::from(unsafe {
            self.binding
                .fmi2NewDiscreteStates(self.component, &mut event_info)
        })
        .ok()?;
        event_flags.update_from_fmi2_event_info(event_info);
        Ok(result)
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

impl FmiModelExchange for Instance<ME> {
    fn enter_continuous_time_mode(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::enter_continuous_time_mode(self)
    }

    fn enter_event_mode(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::enter_event_mode(self)
    }

    fn update_discrete_states(
        &mut self,
        event_flags: &mut EventFlags,
    ) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::new_discrete_states(self, event_flags)
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
        Ok(self.num_event_indicators)
    }
}

impl FmiEventHandler for Instance<ME> {
    fn enter_event_mode(&mut self) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::enter_event_mode(self)
    }

    fn update_discrete_states(
        &mut self,
        event_flags: &mut EventFlags,
    ) -> Result<Fmi2Res, Fmi2Error> {
        ModelExchange::new_discrete_states(self, event_flags)
    }
}
