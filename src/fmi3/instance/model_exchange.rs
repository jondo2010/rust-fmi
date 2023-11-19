use libc::ENOTRECOVERABLE;

use crate::{
    fmi3::{binding, import, logger::callback_log, FmiStatus},
    FmiError, FmiResult,
};

use super::{traits, Instance, ME};

impl<'a> Instance<'a, ME> {
    pub fn new(
        import: &'a import::Fmi3,
        instance_name: &str,
        visible: bool,
        logging_on: bool,
    ) -> FmiResult<Self> {
        let binding = import.raw_bindings()?;
        let model = import.model();

        let instance = unsafe {
            binding.fmi3InstantiateModelExchange(
                instance_name.as_ptr() as binding::fmi3String,
                model.instantiation_token.as_ptr() as binding::fmi3String,
                model.instantiation_token.as_ptr() as binding::fmi3String,
                visible,
                logging_on,
                std::ptr::null_mut() as binding::fmi3InstanceEnvironment,
                Some(callback_log),
            )
        };
        dbg!(instance);

        if instance.is_null() {
            return Err(FmiError::Instantiation);
        }

        Ok(Self {
            binding,
            instance,
            model,
            _tag: std::marker::PhantomData,
        })
    }
}

impl<'a> traits::ModelExchange for Instance<'a, ME> {
    fn enter_continuous_time_mode(&mut self) -> FmiResult<()> {
        let res: FmiStatus =
            unsafe { self.binding.fmi3EnterContinuousTimeMode(self.instance) }.into();
        res.into()
    }

    fn completed_integrator_step(
        &mut self,
        no_set_fmu_state_prior: bool,
    ) -> FmiResult<(bool, bool)> {
        let mut enter_event_mode = false;
        let mut terminate_simulation = false;
        let res: FmiStatus = unsafe {
            self.binding.fmi3CompletedIntegratorStep(
                self.instance,
                no_set_fmu_state_prior,
                &mut enter_event_mode,
                &mut terminate_simulation,
            )
        }
        .into();
        FmiResult::from(res).map(|_| (enter_event_mode, terminate_simulation))
    }

    fn set_time(&mut self, time: f64) -> FmiResult<()> {
        let res: FmiStatus = unsafe { self.binding.fmi3SetTime(self.instance, time) }.into();
        res.into()
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> FmiResult<()> {
        let res: FmiStatus = unsafe {
            self.binding
                .fmi3SetContinuousStates(self.instance, states.as_ptr(), states.len())
        }
        .into();
        res.into()
    }

    fn get_continuous_state_derivatives(&mut self) -> FmiResult<Vec<f64>> {
        todo!()
    }
}
