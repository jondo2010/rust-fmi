use std::ffi::CString;

use crate::{
    Error,
    fmi3::{Common, Fmi3Error, Fmi3Res, Fmi3Status, ModelExchange, binding, import, logger},
    traits::{FmiEventHandler, FmiImport, FmiModelExchange, FmiStatus},
};

use super::{Instance, ME};

impl<'a> Instance<'a, ME> {
    pub fn new(
        import: &'a import::Fmi3Import,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> Result<Self, Error> {
        let schema = import.model_description();

        let name = instance_name.to_owned();

        let model_exchange = schema
            .model_exchange
            .as_ref()
            .ok_or(Error::UnsupportedFmuType("ModelExchange".to_owned()))?;

        log::debug!(
            "Instantiating ME: {} '{name}'",
            model_exchange.model_identifier
        );

        let binding = import.binding(&model_exchange.model_identifier)?;

        let instance_name = CString::new(instance_name).expect("Invalid instance name");
        let instantiation_token = CString::new(schema.instantiation_token.as_bytes())
            .expect("Invalid instantiation token");
        let resource_path =
            CString::new(import.canonical_resource_path_string()).expect("Invalid resource path");

        let instance = unsafe {
            binding.fmi3InstantiateModelExchange(
                instance_name.as_ptr() as binding::fmi3String,
                instantiation_token.as_ptr() as binding::fmi3String,
                resource_path.as_ptr() as binding::fmi3String,
                visible,
                logging_on,
                std::ptr::null_mut() as binding::fmi3InstanceEnvironment,
                Some(logger::callback_log),
            )
        };

        if instance.is_null() {
            return Err(Error::Instantiation);
        }

        Ok(Self {
            binding,
            ptr: instance,
            model_description: schema,
            name,
            _tag: std::marker::PhantomData,
        })
    }
}

impl ModelExchange for Instance<'_, ME> {
    /// This function must be called to change from Event Mode into Continuous-Time Mode in Model
    /// Exchange.
    fn enter_continuous_time_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe { self.binding.fmi3EnterContinuousTimeMode(self.ptr) }).ok()
    }

    fn completed_integrator_step(
        &mut self,
        no_set_fmu_state_prior: bool,
        enter_event_mode: &mut bool,
        terminate_simulation: &mut bool,
    ) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe {
            self.binding.fmi3CompletedIntegratorStep(
                self.ptr,
                no_set_fmu_state_prior as _,
                enter_event_mode as *mut _,
                terminate_simulation as *mut _,
            )
        })
        .ok()
    }

    fn set_time(&mut self, time: f64) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe { self.binding.fmi3SetTime(self.ptr, time) }).ok()
    }

    fn get_continuous_states(
        &mut self,
        continuous_states: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe {
            self.binding.fmi3GetContinuousStates(
                self.ptr,
                continuous_states.as_mut_ptr(),
                continuous_states.len(),
            )
        })
        .ok()
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe {
            self.binding
                .fmi3SetContinuousStates(self.ptr, states.as_ptr(), states.len())
        })
        .ok()
    }

    fn get_continuous_state_derivatives(
        &mut self,
        derivatives: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe {
            self.binding.fmi3GetContinuousStateDerivatives(
                self.ptr,
                derivatives.as_mut_ptr(),
                derivatives.len(),
            )
        })
        .ok()
    }

    fn get_nominals_of_continuous_states(
        &mut self,
        nominals: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        Fmi3Status::from(unsafe {
            self.binding.fmi3GetNominalsOfContinuousStates(
                self.ptr,
                nominals.as_mut_ptr(),
                nominals.len(),
            )
        })
        .ok()
    }

    fn get_event_indicators(&mut self, event_indicators: &mut [f64]) -> Result<bool, Fmi3Error> {
        let status = unsafe {
            self.binding.fmi3GetEventIndicators(
                self.ptr,
                event_indicators.as_mut_ptr(),
                event_indicators.len(),
            )
        };

        // Convert status and handle Discard case
        match Fmi3Status::from(status).ok() {
            Ok(_) => Ok(true), // Successfully computed indicators
            Err(Fmi3Error::Discard) => {
                // FMU couldn't compute indicators due to numerical issues
                // but this is a recoverable condition
                Ok(false)
            }
            Err(e) => Err(e), // Other errors
        }
    }

    fn get_number_of_event_indicators(&mut self) -> Result<usize, Fmi3Error> {
        let mut number_of_event_indicators = 0usize;
        Fmi3Status::from(unsafe {
            self.binding
                .fmi3GetNumberOfEventIndicators(self.ptr, &mut number_of_event_indicators)
        })
        .ok()?;
        Ok(number_of_event_indicators)
    }
}

impl FmiModelExchange for Instance<'_, ME> {
    fn enter_continuous_time_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        ModelExchange::enter_continuous_time_mode(self)
    }

    fn enter_event_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Common::enter_event_mode(self)
    }

    fn update_discrete_states(
        &mut self,
        discrete_states_need_update: &mut bool,
        terminate_simulation: &mut bool,
        nominals_of_continuous_states_changed: &mut bool,
        values_of_continuous_states_changed: &mut bool,
        next_event_time: &mut Option<f64>,
    ) -> Result<Fmi3Res, Fmi3Error> {
        Common::update_discrete_states(
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
    ) -> Result<Fmi3Res, Fmi3Error> {
        ModelExchange::completed_integrator_step(
            self,
            no_set_fmu_state_prior,
            enter_event_mode,
            terminate_simulation,
        )
    }

    fn set_time(&mut self, time: f64) -> Result<Fmi3Res, Fmi3Error> {
        ModelExchange::set_time(self, time)
    }

    fn get_continuous_states(
        &mut self,
        continuous_states: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        ModelExchange::get_continuous_states(self, continuous_states)
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> Result<Fmi3Res, Fmi3Error> {
        ModelExchange::set_continuous_states(self, states)
    }

    fn get_continuous_state_derivatives(
        &mut self,
        derivatives: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        ModelExchange::get_continuous_state_derivatives(self, derivatives)
    }

    fn get_nominals_of_continuous_states(
        &mut self,
        nominals: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        ModelExchange::get_nominals_of_continuous_states(self, nominals)
    }

    fn get_event_indicators(
        &mut self,
        event_indicators: &mut [f64],
    ) -> Result<bool, <Self::Status as crate::traits::FmiStatus>::Err> {
        ModelExchange::get_event_indicators(self, event_indicators)
    }

    fn get_number_of_event_indicators(&mut self) -> Result<usize, Fmi3Error> {
        ModelExchange::get_number_of_event_indicators(self)
    }
}

impl FmiEventHandler for Instance<'_, ME> {
    #[inline]
    fn enter_event_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Common::enter_event_mode(self)
    }

    #[inline]
    fn update_discrete_states(
        &mut self,
        discrete_states_need_update: &mut bool,
        terminate_simulation: &mut bool,
        nominals_of_continuous_states_changed: &mut bool,
        values_of_continuous_states_changed: &mut bool,
        next_event_time: &mut Option<f64>,
    ) -> Result<Fmi3Res, Fmi3Error> {
        Common::update_discrete_states(
            self,
            discrete_states_need_update,
            terminate_simulation,
            nominals_of_continuous_states_changed,
            values_of_continuous_states_changed,
            next_event_time,
        )
    }
}
