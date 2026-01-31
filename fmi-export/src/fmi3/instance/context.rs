use std::{collections::BTreeMap, path::PathBuf};

use fmi::fmi3::Fmi3Status;

use crate::fmi3::{
    UserModel,
    instance::{IntermediateUpdateClosure, LogMessageClosure},
    traits::{Context, ModelLoggingCategory},
};

/// Basic context for Model-Exchange FMU instances
pub struct BasicContext<M: UserModel> {
    /// Map of logging categories to their enabled state.
    /// This is used to track which categories are enabled for logging.
    logging_on: BTreeMap<M::LoggingCategory, bool>,
    /// Callback for logging messages.
    log_message: LogMessageClosure,
    /// Path to the resources directory.
    resource_path: PathBuf,
    /// Simulation stop time.
    stop_time: Option<f64>,
    /// Current simulation time.
    time: f64,
    /// Whether early return is allowed for CS steps.
    early_return_allowed: bool,
    /// Optional FMI intermediate update callback (CS).
    intermediate_update: Option<IntermediateUpdateClosure>,
}

impl<M: UserModel> BasicContext<M> {
    pub fn new(
        logging_on: bool,
        log_message: LogMessageClosure,
        resource_path: PathBuf,
        early_return_allowed: bool,
        intermediate_update: Option<IntermediateUpdateClosure>,
    ) -> Self {
        let logging_on = <M as UserModel>::LoggingCategory::all_categories()
            .map(|category| (category, logging_on))
            .collect();
        Self {
            logging_on,
            log_message,
            resource_path,
            stop_time: None,
            time: 0.0,
            early_return_allowed,
            intermediate_update,
        }
    }

    pub fn intermediate_update(&self) -> Option<&IntermediateUpdateClosure> {
        self.intermediate_update.as_ref()
    }
}

impl<M> Context<M> for BasicContext<M>
where
    M: UserModel + 'static,
{
    fn logging_on(&self, category: <M as UserModel>::LoggingCategory) -> bool {
        matches!(self.logging_on.get(&category), Some(true))
    }

    fn set_logging(&mut self, category: <M as UserModel>::LoggingCategory, enabled: bool) {
        self.logging_on.insert(category, enabled);
    }

    /// Log a message if the specified logging category is enabled.
    fn log(&self, status: Fmi3Status, category: M::LoggingCategory, args: std::fmt::Arguments<'_>) {
        if self.logging_on(category) {
            // Call the logging callback
            (self.log_message)(status, &category.to_string(), args);
        } else {
            eprintln!("Logging disabled for category: {}", category);
        }
    }

    /// Get the path to the resources directory.
    fn resource_path(&self) -> &PathBuf {
        &self.resource_path
    }

    fn initialize(&mut self, start_time: f64, stop_time: Option<f64>) {
        self.time = start_time;
        self.stop_time = stop_time;
    }

    fn time(&self) -> f64 {
        self.time
    }

    fn set_time(&mut self, time: f64) {
        self.time = time;
    }

    fn stop_time(&self) -> Option<f64> {
        self.stop_time
    }

    fn early_return_allowed(&self) -> bool {
        self.early_return_allowed
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
