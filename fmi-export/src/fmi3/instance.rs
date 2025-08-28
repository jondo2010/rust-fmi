use std::{collections::BTreeMap, path::PathBuf};

use fmi::fmi3::{Common, Fmi3Error, Fmi3Res, Fmi3Status, GetSet, ModelExchange, binding};

use crate::fmi3::{
    ModelState,
    traits::{Model, ModelLoggingCategory},
};

/// An exportable FMU instance
pub struct ModelInstance<M: Model> {
    start_time: f64,
    stop_time: f64,
    time: f64,
    instance_name: String,
    resource_path: PathBuf,
    /// Map of logging categories to their enabled state.
    /// This is used to track which categories are enabled for logging.
    logging_on: BTreeMap<M::LoggingCategory, bool>,

    /// Callback for logging messages.
    log_message: Box<dyn Fn(Fmi3Status, &str, &str) + Send + Sync>,

    state: ModelState,

    /// event info
    event_flags: EventFlags,

    clocks_ticked: bool,

    // event indicators
    event_indicators: Vec<f64>,

    // internal solver steps
    n_steps: usize,

    is_dirty_values: bool,
    model: M,
    _marker: std::marker::PhantomData<M>,
}

/// Event flags information
#[derive(Default, Debug, Copy, Clone)]
struct EventFlags {
    /// Indicates if discrete states need to be updated.
    pub discrete_states_need_update: bool,
    /// Indicates if the simulation should be terminated.
    pub terminate_simulation: bool,
    /// Indicates if the nominal values of the continuous states have changed.
    pub nominals_of_continuous_states_changed: bool,
    /// Indicates if the values of the continuous states have changed.
    pub values_of_continuous_states_changed: bool,
    /// Indicates the time of the next event.
    pub next_event_time: Option<f64>,
}

impl<M: Model> ModelInstance<M> {
    pub fn new(
        name: String,
        resource_path: PathBuf,
        logging_on: bool,
        log_message: Box<dyn Fn(Fmi3Status, &str, &str) + Send + Sync>,
        instantiation_token: &str,
    ) -> Result<Self, Fmi3Error> {
        // Validate the instantiation token using the compile-time constant
        if instantiation_token != M::INSTANTIATION_TOKEN {
            log::error!(
                "Instantiation token mismatch. Expected: '{}', got: '{}'",
                M::INSTANTIATION_TOKEN,
                instantiation_token
            );
            return Err(Fmi3Error::Error);
        }

        let logging_on = M::LoggingCategory::all_categories()
            .map(|category| (category, logging_on))
            .collect();

        let num_event_indicators = M::get_number_of_event_indicators();

        let mut instance = Self {
            start_time: 0.0,
            stop_time: 1.0,
            time: 0.0,
            instance_name: name,
            resource_path,
            logging_on,
            log_message,
            state: ModelState::Instantiated,
            event_flags: EventFlags::default(),
            clocks_ticked: false,
            event_indicators: vec![0.0; num_event_indicators],
            n_steps: 0,
            is_dirty_values: false,
            model: M::default(),
            _marker: std::marker::PhantomData,
        };

        // Set start values for the model
        instance.model.set_start_values();

        Ok(instance)
    }

    pub fn log(&self, status: Fmi3Status, category: M::LoggingCategory, message: &str) {
        if matches!(self.logging_on.get(&category), Some(true)) {
            // Call the logging callback
            (self.log_message)(status, &category.to_string(), message);
        }
    }

    /// Validate that a variable can be set in the current model state
    fn validate_variable_setting(&self, vr: binding::fmi3ValueReference) -> Result<(), Fmi3Error> {
        match M::validate_variable_setting(vr, &self.state) {
            Ok(()) => Ok(()),
            Err(message) => {
                self.log(
                    Fmi3Error::Error.into(),
                    M::LoggingCategory::default(),
                    &format!("Variable setting error for VR {}: {}", vr, message),
                );
                Err(Fmi3Error::Error)
            }
        }
    }

    /// Update discrete states after an event has been detected
    pub fn event_update(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        // Reset event flags
        self.event_flags.discrete_states_need_update = false;
        self.event_flags.nominals_of_continuous_states_changed = false;
        self.event_flags.values_of_continuous_states_changed = false;
        self.event_flags.next_event_time = None;

        // Delegate to the model's event update
        match self.model.event_update() {
            Ok(_) => {
                // Mark that values may have changed
                self.is_dirty_values = true;
                Ok(Fmi3Res::OK)
            }
            Err(e) => Err(e),
        }
    }
}

impl<F> GetSet for ModelInstance<F>
where
    F: Model<ValueRef = binding::fmi3ValueReference>,
{
    type ValueRef = binding::fmi3ValueReference;

    fn get_float32(
        &mut self,
        vrs: &[Self::ValueRef],
        values: &mut [f32],
    ) -> Result<Fmi3Res, Fmi3Error> {
        self.model.get_float32(vrs, values)
    }

    fn get_float64(
        &mut self,
        vrs: &[Self::ValueRef],
        values: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        self.model.get_float64(vrs, values)
    }

    fn set_float64(
        &mut self,
        vrs: &[Self::ValueRef],
        values: &[f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        // Validate variable setting restrictions before setting values
        for &vr in vrs {
            self.validate_variable_setting(vr)?;
        }

        self.model.set_float64(vrs, values)?;
        self.is_dirty_values = true;
        Ok(Fmi3Res::OK)
    }
}

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
                .and_then(|level| self.logging_on.get_mut(&level))
            {
                *cat = logging_on;
            } else {
                self.log(
                    Fmi3Error::Error.into(),
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
            self.model.calculate_values();
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

impl<F> ModelExchange for ModelInstance<F>
where
    F: Model<ValueRef = binding::fmi3ValueReference>,
{
    fn enter_continuous_time_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        match self.state {
            ModelState::EventMode => {
                self.state = ModelState::ContinuousTimeMode;
                Ok(Fmi3Res::OK)
            }
            _ => Err(Fmi3Error::Error),
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
            self.model.calculate_values();
            self.is_dirty_values = false;
        }
        self.model.get_continuous_state_derivatives(derivatives)
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
            .get_event_indicators(&mut self.event_indicators)?;

        // Copy to the output array
        let copy_len = indicators.len().min(self.event_indicators.len());
        indicators[..copy_len].copy_from_slice(&self.event_indicators[..copy_len]);

        // Check for zero crossings by comparing with previous values (simplified)
        // In a full implementation, this would detect actual zero crossings
        Ok(false) // Return false for now, indicating no state events
    }
}

impl<F> ModelInstance<F> where F: Model<ValueRef = binding::fmi3ValueReference> {}
