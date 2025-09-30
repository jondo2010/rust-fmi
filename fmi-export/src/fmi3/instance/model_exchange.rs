use super::ModelInstance;
use crate::fmi3::{
    Model, ModelGetSetStates, ModelState,
    traits::{ModelGetSet, ModelLoggingCategory},
};
use fmi::fmi3::{Fmi3Error, Fmi3Res, ModelExchange};

impl<M> ModelExchange for ModelInstance<M>
where
    M: Model + ModelGetSet<M> + ModelGetSetStates,
{
    fn enter_continuous_time_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK,
            M::LoggingCategory::trace_category(),
            format_args!("enter_continuous_time_mode()"),
        );
        match self.state {
            ModelState::EventMode => {
                self.state = ModelState::ContinuousTimeMode;
                Ok(Fmi3Res::OK)
            }
            _ => {
                self.context.log(
                    Fmi3Error::Error,
                    M::LoggingCategory::default(),
                    format_args!(
                        "enter_continuous_time_mode() called in invalid state {:?}",
                        self.state
                    ),
                );
                Err(Fmi3Error::Error)
            }
        }
    }

    fn completed_integrator_step(
        &mut self,
        _no_set_fmu_state_prior: bool,
        enter_event_mode: &mut bool,
        terminate_simulation: &mut bool,
    ) -> Result<Fmi3Res, Fmi3Error> {
        // Default implementation - no events, no termination
        *enter_event_mode = false;
        *terminate_simulation = false;
        Ok(Fmi3Res::OK)
    }

    fn set_time(&mut self, time: f64) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK,
            M::LoggingCategory::trace_category(),
            format_args!("set_time({})", time),
        );
        self.context.time = time;
        Ok(Fmi3Res::OK)
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK,
            M::LoggingCategory::trace_category(),
            format_args!("set_continuous_states({states:?})"),
        );
        self.model.set_continuous_states(states)?;
        self.is_dirty_values = true;
        Ok(Fmi3Res::OK)
    }

    fn get_continuous_states(
        &mut self,
        continuous_states: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        self.model.get_continuous_states(continuous_states)?;
        self.context.log(
            Fmi3Res::OK,
            M::LoggingCategory::trace_category(),
            format_args!("get_continuous_states({continuous_states:?})"),
        );
        Ok(Fmi3Res::OK)
    }

    fn get_continuous_state_derivatives(
        &mut self,
        derivatives: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        // Ensure values are up to date before computing derivatives
        if self.is_dirty_values {
            self.model.calculate_values(&mut self.context)?;
            self.is_dirty_values = false;
        }
        self.model.get_continuous_state_derivatives(derivatives)?;
        self.context.log(
            Fmi3Res::OK,
            M::LoggingCategory::trace_category(),
            format_args!("get_continuous_state_derivatives({derivatives:?})"),
        );
        Ok(Fmi3Res::OK)
    }

    fn get_event_indicators(&mut self, indicators: &mut [f64]) -> Result<bool, Fmi3Error> {
        let res = self
            .model
            .get_event_indicators(&mut self.context, indicators)?;
        self.context.log(
            Fmi3Res::OK,
            M::LoggingCategory::trace_category(),
            format_args!("get_event_indicators({indicators:?})={res}"),
        );
        Ok(res)
    }

    fn get_nominals_of_continuous_states(
        &mut self,
        nominals: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        // Default implementation: all nominals = 1.0
        for nominal in nominals.iter_mut() {
            *nominal = 1.0;
        }
        Ok(Fmi3Res::OK)
    }

    fn get_number_of_event_indicators(&mut self) -> Result<usize, Fmi3Error> {
        Ok(M::get_number_of_event_indicators())
    }

    fn get_number_of_continuous_states(&mut self) -> Result<usize, Fmi3Error> {
        Ok(M::NUM_STATES)
    }
}
