use std::{collections::BTreeMap, path::PathBuf};

use fmi::{
    InterfaceType,
    fmi3::{Fmi3Error, Fmi3Status, binding},
};

use crate::fmi3::{
    ModelState, UserModel,
    traits::{Model, ModelLoggingCategory},
};

mod common;
mod get_set;
mod model_exchange;

/// An exportable FMU instance
pub struct ModelInstance<M: Model> {
    /// The name of this instance
    instance_name: String,
    /// Context for the model instance
    context: ModelContext<M>,
    /// Current state of the model instance
    state: ModelState,
    /// Do we need to re-evaluate the model equations?
    is_dirty_values: bool,
    /// The user-defined model
    model: M,
}

type LogMessageClosure = Box<dyn Fn(Fmi3Status, &str, std::fmt::Arguments<'_>) + Send + Sync>;

pub struct ModelContext<M: UserModel> {
    /// Map of logging categories to their enabled state.
    /// This is used to track which categories are enabled for logging.
    logging_on: BTreeMap<M::LoggingCategory, bool>,
    /// Callback for logging messages.
    log_message: LogMessageClosure,
    /// Path to the resources directory.
    resource_path: PathBuf,
    /// Current simulation time.
    time: f64,
}

impl<M: UserModel> ModelContext<M> {
    /// Log a message if the specified logging category is enabled.
    pub fn log(
        &self,
        status: impl Into<Fmi3Status>,
        category: M::LoggingCategory,
        args: std::fmt::Arguments<'_>,
    ) {
        if matches!(self.logging_on.get(&category), Some(true)) {
            // Call the logging callback
            (self.log_message)(status.into(), &category.to_string(), args);
        } else {
            eprintln!("Logging disabled for category: {}", category);
        }
    }

    /// Get the path to the resources directory.
    pub fn resource_path(&self) -> &PathBuf {
        &self.resource_path
    }

    /// Get the current simulation time.
    pub fn time(&self) -> f64 {
        self.time
    }
}

impl<M: Model> ModelInstance<M> {
    pub fn new(
        name: String,
        resource_path: PathBuf,
        logging_on: bool,
        log_message: LogMessageClosure,
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

        let context = ModelContext {
            logging_on,
            log_message,
            resource_path,
            time: 0.0,
        };

        let mut instance = Self {
            instance_name: name,
            context,
            state: ModelState::Instantiated,
            is_dirty_values: true,
            model: M::default(),
        };

        // Set start values for the model
        instance.model.set_start_values();

        Ok(instance)
    }

    pub fn interface_type(&self) -> InterfaceType {
        fmi::InterfaceType::ModelExchange
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
                    format_args!("Variable setting error for VR {vr}: {message}"),
                );
                Err(Fmi3Error::Error)
            }
        }
    }
}
