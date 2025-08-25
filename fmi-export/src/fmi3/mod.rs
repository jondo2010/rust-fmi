//! ## Design
//!
//! The [`crate::export_fmu`] macro generates the necessary C-API bindings for the exported FMU.
//! Many of these bindings operate on a [`binding::fmi3Instance`], which is an opaque pointer to an
//! instance of [`ModelInstance`].
//!
//! [`ModelInstance`] implements the [`Common`] trait, which provides the actual implementation of
//! the FMI 3.0 API. All user-model-specific functions are delegated to the [`Model`] trait,
//! which the user model must implement.

use std::{collections::BTreeMap, ffi::CString, path::PathBuf};

use fmi::fmi3::{Common, Fmi3Error, Fmi3Res, Fmi3Status, GetSet, ModelExchange, binding, schema};

mod macros;

/// User-defined model behavior trait
/// This trait should be implemented by users to define their model-specific behavior
pub trait UserModel {
    /// Calculate values (derivatives, outputs, etc.)
    /// This method is called whenever the model needs to update its calculated values
    fn calculate_values(&mut self) -> Fmi3Status {
        Fmi3Res::OK.into()
    }

    /// Event update function for Model Exchange
    /// Called to update discrete states and check for events
    fn event_update(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        Ok(Fmi3Res::OK)
    }

    /// Get event indicators for zero-crossing detection
    /// Returns the current values of event indicators
    fn get_event_indicators(&mut self, indicators: &mut [f64]) -> Result<Fmi3Res, Fmi3Error> {
        // Default implementation: no event indicators
        for indicator in indicators.iter_mut() {
            *indicator = 0.0;
        }
        Ok(Fmi3Res::OK)
    }
}

enum ModelState {
    StartAndEnd,
    ConfigurationMode,
    Instantiated,
    InitializationMode,
    EventMode,
    ContinuousTimeMode,
    StepMode,
    ClockActivationMode,
    StepDiscarded,
    ReconfigurationMode,
    IntermediateUpdateMode,
    Terminated,
}

/// Model trait, to be implemented by the user model
pub trait Model: Default + GetSet + UserModel {
    const MODEL_NAME: &'static str;
    const MODEL_DESCRIPTION: &'static str;
    const INSTANTIATION_TOKEN: &'static str;

    /// Set start values
    fn set_start_values(&mut self);

    /// Get continuous states from the model
    /// Returns the current values of all continuous state variables
    fn get_continuous_states(&self, states: &mut [f64]) -> Result<Fmi3Res, Fmi3Error>;

    /// Set continuous states in the model
    /// Sets new values for all continuous state variables
    fn set_continuous_states(&mut self, states: &[f64]) -> Result<Fmi3Res, Fmi3Error>;

    /// Get derivatives of continuous states
    /// Returns the first-order time derivatives of all continuous state variables
    fn get_continuous_state_derivatives(
        &mut self,
        derivatives: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error>;

    /// Get the number of continuous states
    fn get_number_of_continuous_states() -> usize {
        0
    }

    /// Get the number of event indicators
    fn get_number_of_event_indicators() -> usize {
        0
    }

    fn configurate(&mut self) -> Fmi3Status {
        // Basic configuration - in a full implementation, this would:
        // - Allocate memory for event indicators if needed
        // - Allocate memory for continuous states if needed
        // - Initialize event indicator values
        // For now, just return OK since our basic implementation doesn't need these
        Fmi3Res::OK.into()
    }

    /// Describe the model variables for this model
    fn model_variables() -> schema::ModelVariables {
        // should be implemented by the user model
        schema::ModelVariables {
            ..Default::default()
        }
    }

    /// Describe the model structure for this model
    fn model_structure() -> schema::ModelStructure {
        // should be implemented by the user model
        schema::ModelStructure {
            ..Default::default()
        }
    }

    /// Build a model description for this model
    fn model_description() -> schema::Fmi3ModelDescription {
        schema::Fmi3ModelDescription {
            fmi_version: unsafe { str::from_utf8_unchecked(binding::fmi3Version).to_owned() },
            model_name: Self::MODEL_NAME.to_owned(),
            instantiation_token: Self::INSTANTIATION_TOKEN.to_owned(),
            description: Some(Self::MODEL_DESCRIPTION.to_owned()),
            generation_tool: Some("rust-fmi".to_owned()),
            generation_date_and_time: Some(chrono::Utc::now().to_rfc3339()),
            model_variables: Self::model_variables(),
            model_structure: Self::model_structure(),
            ..Default::default()
        }
    }
}

/// An exportable FMU instance
pub struct ModelInstance<M: Model> {
    start_time: f64,
    stop_time: f64,
    time: f64,
    instance_name: String,
    resource_path: PathBuf,
    /// Map of logging categories to their enabled state.
    /// This is used to track which categories are enabled for logging.
    logging_on: BTreeMap<log::Level, bool>,
    /// Callback for logging messages.
    log_message: binding::fmi3LogMessageCallback,
    state: ModelState,

    // event info
    new_discrete_states_needed: bool,
    terminate_simulation: bool,
    nominals_of_continuous_states_changed: bool,
    values_of_continuous_states_changed: bool,
    next_event_time_defined: bool,
    next_event_time: f64,
    clocks_ticked: bool,

    // event indicators
    event_indicators: Vec<f64>,

    // internal solver steps
    n_steps: usize,

    is_dirty_values: bool,
    model: M,
    _marker: std::marker::PhantomData<M>,
}

impl<F: Model> ModelInstance<F> {
    pub fn new(
        name: String,
        resource_path: PathBuf,
        logging_on: bool,
        log_message: binding::fmi3LogMessageCallback,
        instantiation_token: &str,
    ) -> Result<Self, Fmi3Error> {
        // Validate the instantiation token using the compile-time constant
        if instantiation_token != F::INSTANTIATION_TOKEN {
            log::error!(
                "Instantiation token mismatch. Expected: '{}', got: '{}'",
                F::INSTANTIATION_TOKEN,
                instantiation_token
            );
            return Err(Fmi3Error::Error);
        }

        let logging_on = log::Level::iter()
            .map(|level| (level, logging_on))
            .collect();

        let num_event_indicators = F::get_number_of_event_indicators();

        let mut instance = Self {
            start_time: 0.0,
            stop_time: 1.0,
            time: 0.0,
            instance_name: name,
            resource_path,
            logging_on,
            log_message,
            state: ModelState::Instantiated,
            new_discrete_states_needed: false,
            terminate_simulation: false,
            nominals_of_continuous_states_changed: false,
            values_of_continuous_states_changed: false,
            next_event_time_defined: false,
            next_event_time: 0.0,
            clocks_ticked: false,
            event_indicators: vec![0.0; num_event_indicators],
            n_steps: 0,
            is_dirty_values: false,
            model: F::default(),
            _marker: std::marker::PhantomData,
        };

        // Set start values for the model
        instance.model.set_start_values();

        Ok(instance)
    }

    pub fn log(&self, level: log::Level, message: &str) {
        if let Some(enabled) = self.logging_on.get(&level) {
            if *enabled {
                let status: Fmi3Status = Fmi3Res::OK.into();
                let category = CString::new(level.to_string()).expect("Invalid category name");
                let message = CString::new(message).expect("Invalid message");

                unsafe {
                    (self.log_message.unwrap())(
                        std::ptr::null_mut() as binding::fmi3InstanceEnvironment,
                        status.into(),
                        category.as_ptr() as binding::fmi3String,
                        message.as_ptr() as binding::fmi3String,
                    )
                };
            }
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
                .parse::<log::Level>()
                .ok()
                .and_then(|level| self.logging_on.get_mut(&level))
            {
                *cat = logging_on;
            } else {
                log::warn!("Unknown logging category: {cat}");
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
        self.new_discrete_states_needed = false;
        self.terminate_simulation = false;
        self.nominals_of_continuous_states_changed = false;
        self.values_of_continuous_states_changed = false;
        self.next_event_time_defined = false;
        self.next_event_time = 0.0;
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
        self.values_of_continuous_states_changed = true;
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

impl<F> ModelInstance<F>
where
    F: Model<ValueRef = binding::fmi3ValueReference>,
{
    /// Update discrete states after an event has been detected
    pub fn event_update(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        // Reset event flags
        self.new_discrete_states_needed = false;
        self.nominals_of_continuous_states_changed = false;
        self.values_of_continuous_states_changed = false;
        self.next_event_time_defined = false;

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
