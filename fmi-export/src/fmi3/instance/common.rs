use super::ModelInstance;
use crate::fmi3::{Model, ModelState, instance::EventFlags};
use fmi::fmi3::{Common, Fmi3Error, Fmi3Res, Fmi3Status, ModelExchange, binding};

impl<F> Common for ModelInstance<F>
where
    F: Model<ValueRef = binding::fmi3ValueReference>,
{
    fn get_version(&self) -> &str {
        // Safety: binding::fmi3Version is a null-terminated byte array representing the version string
        unsafe { str::from_utf8_unchecked(binding::fmi3Version) }
    }

    fn set_debug_logging(
        &mut self,
        logging_on: bool,
        categories: &[&str],
    ) -> Result<Fmi3Res, Fmi3Error> {
        for &cat in categories.iter() {
            if let Some(cat) = cat
                .parse::<F::LoggingCategory>()
                .ok()
                .and_then(|level| self.context.logging_on.get_mut(&level))
            {
                *cat = logging_on;
            } else {
                self.context.log(
                    Fmi3Error::Error,
                    F::LoggingCategory::default(),
                    &format!("Unknown logging category {cat}"),
                );
                return Err(Fmi3Error::Error);
            }
        }
        Ok(Fmi3Res::OK)
    }

    fn enter_initialization_mode(
        &mut self,
        _tolerance: Option<f64>,
        _start_time: f64,
        _stop_time: Option<f64>,
    ) -> Result<Fmi3Res, Fmi3Error> {
        match self.state {
            ModelState::Instantiated => {
                // Transition to INITIALIZATION_MODE
                self.state = ModelState::InitializationMode;
                //self.log("info", "Entering initialization mode");
                Ok(Fmi3Res::OK)
            }
            _ => {
                //this.log( "error", "Cannot enter initialization mode from current state",);
                Err(Fmi3Error::Error)
            }
        }
    }

    fn exit_initialization_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        // if values were set and no fmi3GetXXX triggered update before,
        // ensure calculated values are updated now
        if self.is_dirty_values {
            self.model.calculate_values(&self.context);
            self.is_dirty_values = false;
        }

        /*
        switch (S->type) {
            case ModelExchange:
                S->state = EventMode;
                break;
            case CoSimulation:
                S->state = S->eventModeUsed ? EventMode : StepMode;
                break;
            case ScheduledExecution:
                S->state = ClockActivationMode;
                break;
        }
        */

        self.model.configurate();
        Ok(Fmi3Res::OK)
    }

    fn terminate(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.state = ModelState::Terminated;
        Ok(Fmi3Res::OK)
    }

    fn reset(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.state = ModelState::Instantiated;
        self.start_time = 0.0;
        self.time = 0.0;
        self.n_steps = 0;

        // Reset event info
        self.event_flags = EventFlags::default();
        self.clocks_ticked = false;

        // Reset event indicators
        for indicator in &mut self.event_indicators {
            *indicator = 0.0;
        }

        self.model.set_start_values();
        Ok(Fmi3Res::OK)
    }

    fn enter_configuration_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        todo!()
    }

    fn exit_configuration_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        todo!()
    }

    fn enter_event_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.state = ModelState::EventMode;
        Ok(Fmi3Res::OK)
    }

    fn update_discrete_states(
        &mut self,
        discrete_states_need_update: &mut bool,
        terminate_simulation: &mut bool,
        nominals_of_continuous_states_changed: &mut bool,
        values_of_continuous_states_changed: &mut bool,
        next_event_time: &mut Option<f64>,
    ) -> Result<Fmi3Res, Fmi3Error> {
        *discrete_states_need_update = self.event_flags.discrete_states_need_update;
        *terminate_simulation = self.event_flags.terminate_simulation;
        *nominals_of_continuous_states_changed =
            self.event_flags.nominals_of_continuous_states_changed;
        *values_of_continuous_states_changed = self.event_flags.values_of_continuous_states_changed;
        *next_event_time = self.event_flags.next_event_time;
        Ok(Fmi3Res::OK)
    }
}
