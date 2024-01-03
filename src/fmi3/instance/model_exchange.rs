use std::ffi::CString;

use crate::{
    fmi3::{binding, import, logger, Fmi3Error, Fmi3Status},
    import::FmiImport as _,
    Error,
};

use super::{traits, Instance, ME};

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

        let binding = import.binding(&model_exchange.model_identifier)?;

        let instance_name = CString::new(instance_name).expect("Invalid instance name");
        let instantiation_token = CString::new(schema.instantiation_token.as_bytes())
            .expect("Invalid instantiation token");
        let resource_path =
            CString::new(import.resource_url().as_str()).expect("Invalid resource path");

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
            instance,
            model_description: schema,
            name,
            _tag: std::marker::PhantomData,
        })
    }
}

impl<'a> traits::ModelExchange for Instance<'a, ME> {
    fn enter_continuous_time_mode(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3EnterContinuousTimeMode(self.instance) }.into()
    }

    fn enter_event_mode(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3EnterEventMode(self.instance) }.into()
    }

    fn enter_configuration_mode(&mut self) -> Fmi3Status {
        unsafe { self.binding.fmi3EnterConfigurationMode(self.instance) }.into()
    }

    fn completed_integrator_step(
        &mut self,
        no_set_fmu_state_prior: bool,
    ) -> Result<(bool, bool), Fmi3Error> {
        let mut enter_event_mode = false;
        let mut terminate_simulation = false;
        let res: Fmi3Status = unsafe {
            self.binding.fmi3CompletedIntegratorStep(
                self.instance,
                no_set_fmu_state_prior,
                &mut enter_event_mode,
                &mut terminate_simulation,
            )
        }
        .into();
        res.ok().map(|_| (enter_event_mode, terminate_simulation))
    }

    fn set_time(&mut self, time: f64) -> Fmi3Status {
        unsafe { self.binding.fmi3SetTime(self.instance, time) }.into()
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> Fmi3Status {
        assert_eq!(
            states.len(),
            self.model_description
                .model_structure
                .continuous_state_derivative
                .len(),
            "Invalid length of continuous_states array, must match the ModelDescription."
        );

        unsafe {
            self.binding
                .fmi3SetContinuousStates(self.instance, states.as_ptr(), states.len())
        }
        .into()
    }

    fn get_continuous_states(&mut self, continuous_states: &mut [f64]) -> Fmi3Status {
        assert_eq!(
            continuous_states.len(),
            self.model_description
                .model_structure
                .continuous_state_derivative
                .len(),
            "Invalid length of continuous_states array, must match the ModelDescription."
        );

        unsafe {
            self.binding.fmi3GetContinuousStates(
                self.instance,
                continuous_states.as_mut_ptr(),
                continuous_states.len(),
            )
        }
        .into()
    }

    fn get_continuous_state_derivatives(&mut self, derivatives: &mut [f64]) -> Fmi3Status {
        assert_eq!(
            derivatives.len(),
            self.model_description
                .model_structure
                .continuous_state_derivative
                .len(),
            "Invalid length of derivatives array, must match the ModelDescription."
        );

        unsafe {
            self.binding.fmi3GetContinuousStateDerivatives(
                self.instance,
                derivatives.as_mut_ptr(),
                derivatives.len(),
            )
        }
        .into()
    }

    fn get_nominals_of_continuous_states(&mut self, nominals: &mut [f64]) -> Fmi3Status {
        assert_eq!(
            nominals.len(),
            self.model_description
                .model_structure
                .continuous_state_derivative
                .len(),
            "Invalid length of nominals array, must match the ModelDescription."
        );

        unsafe {
            self.binding.fmi3GetNominalsOfContinuousStates(
                self.instance,
                nominals.as_mut_ptr(),
                nominals.len(),
            )
        }
        .into()
    }

    fn get_event_indicators(&mut self, event_indicators: &mut [f64]) -> Fmi3Status {
        assert_eq!(
            event_indicators.len(),
            self.model_description.model_structure.event_indicator.len(),
            "Invalid length of event_indicators array, must match the ModelDescription."
        );

        unsafe {
            self.binding.fmi3GetEventIndicators(
                self.instance,
                event_indicators.as_mut_ptr(),
                event_indicators.len(),
            )
        }
        .into()
    }
}
