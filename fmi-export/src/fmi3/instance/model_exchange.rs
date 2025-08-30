use super::ModelInstance;
use crate::fmi3::{Model, ModelState};
use fmi::fmi3::{Fmi3Error, Fmi3Res, ModelExchange, binding};

impl<F> ModelExchange for ModelInstance<F>
where
    F: Model<ValueRef = binding::fmi3ValueReference>,
{
    fn enter_continuous_time_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK,
            Default::default(),
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
                    F::LoggingCategory::default(),
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
        self.time = time;
        Ok(Fmi3Res::OK)
    }

    fn set_continuous_states(&mut self, states: &[f64]) -> Result<Fmi3Res, Fmi3Error> {
        self.model.set_continuous_states(states)?;
        self.is_dirty_values = true;
        self.event_flags.values_of_continuous_states_changed = true;
        Ok(Fmi3Res::OK)
    }

    fn get_continuous_states(
        &mut self,
        continuous_states: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        self.model.get_continuous_states(continuous_states)
    }

    fn get_continuous_state_derivatives(
        &mut self,
        derivatives: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        // Ensure values are up to date before computing derivatives
        if self.is_dirty_values {
            self.model.calculate_values(&self.context);
            self.is_dirty_values = false;
        }
        self.model
            .get_continuous_state_derivatives(derivatives, &self.context)
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
        Ok(F::get_number_of_event_indicators())
    }

    fn get_event_indicators(&mut self, indicators: &mut [f64]) -> Result<bool, Fmi3Error> {
        // Update the internal event indicators from the model
        self.model
            .get_event_indicators(&mut self.event_indicators, &self.context)?;

        // Copy to the output array
        let copy_len = indicators.len().min(self.event_indicators.len());
        indicators[..copy_len].copy_from_slice(&self.event_indicators[..copy_len]);

        // Check for zero crossings by comparing with previous values (simplified)
        // In a full implementation, this would detect actual zero crossings
        Ok(false) // Return false for now, indicating no state events
    }
}
