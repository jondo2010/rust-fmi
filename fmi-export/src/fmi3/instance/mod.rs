use std::{collections::BTreeMap, path::PathBuf};

use fmi::fmi3::{Fmi3Error, Fmi3Res, Fmi3Status, binding};

use crate::fmi3::{
    ModelState, UserModel,
    traits::{Model, ModelLoggingCategory},
};

mod common;
mod get_set;
mod model_exchange;

/// An exportable FMU instance
pub struct ModelInstance<M: Model> {
    start_time: f64,
    stop_time: f64,
    time: f64,
    instance_name: String,
    resource_path: PathBuf,

    context: ModelContext<M>,

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

pub struct ModelContext<M: UserModel> {
    /// Map of logging categories to their enabled state.
    /// This is used to track which categories are enabled for logging.
    logging_on: BTreeMap<M::LoggingCategory, bool>,
    /// Callback for logging messages.
    log_message: Box<dyn Fn(Fmi3Status, &str, &str) + Send + Sync>,
}

impl<M: UserModel> ModelContext<M> {
    /// Create a new ModelContext for testing purposes
    pub fn new_for_test() -> Self {
        let logging_on = M::LoggingCategory::all_categories()
            .map(|category| (category, false))
            .collect();

        Self {
            logging_on,
            log_message: Box::new(|_status, _category, _message| {
                // Mock logger - does nothing in tests
            }),
        }
    }

    /// Log a message if the specified logging category is enabled.
    pub fn log(&self, status: impl Into<Fmi3Status>, category: M::LoggingCategory, message: &str) {
        if matches!(self.logging_on.get(&category), Some(true)) {
            // Call the logging callback
            (self.log_message)(status.into(), &category.to_string(), message);
        } else {
            eprintln!("Logging disabled for category: {}", category);
        }
    }
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
            eprintln!(
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

        let context = ModelContext {
            logging_on,
            log_message,
        };

        let mut instance = Self {
            start_time: 0.0,
            stop_time: 1.0,
            time: 0.0,
            instance_name: name,
            resource_path,
            context,
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

    pub fn instance_name(&self) -> &str {
        &self.instance_name
    }

    pub fn context(&self) -> &ModelContext<M> {
        &self.context
    }

    /// Validate that a variable can be set in the current model state
    fn validate_variable_setting(&self, vr: binding::fmi3ValueReference) -> Result<(), Fmi3Error> {
        match M::validate_variable_setting(vr, &self.state) {
            Ok(()) => Ok(()),
            Err(message) => {
                self.context.log(
                    Fmi3Error::Error,
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
        match self.model.event_update(&self.context) {
            Ok(_) => {
                // Mark that values may have changed
                self.is_dirty_values = true;
                Ok(Fmi3Res::OK)
            }
            Err(e) => Err(e),
        }
    }
}
