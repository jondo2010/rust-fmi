use std::{collections::BTreeMap, path::PathBuf};

use fmi::fmi3::Fmi3Status;

use crate::fmi3::{Model, ModelGetSetStates, ModelLoggingCategory, UserModel, instance::LogMessageClosure, traits::Context};

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
}

impl<M: UserModel> BasicContext<M> {
    pub fn new(logging_on: bool, log_message: LogMessageClosure, resource_path: PathBuf) -> Self {
        let logging_on = <M as UserModel>::LoggingCategory::all_categories()
            .map(|category| (category, logging_on))
            .collect();
        Self {
            logging_on,
            log_message,
            resource_path,
            stop_time: None,
            time: 0.0,
        }
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

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

/// Extended context implementing Co-Simulation by wrapping Model-Exchange methods
pub struct WrapperContext<M: UserModel> {
    pub basic: BasicContext<M>,
    /// Internal step count
    pub num_steps: usize,
    /// Whether early return from a step is allowed.
    pub early_return_allowed: bool,
    /// Whether event mode is used.
    pub event_mode_used: bool,
    /// Next communication point for co-simulation.
    pub next_communication_point: f64,
    /// Event indicators' current values
    pub cur_z: Vec<f64>,
    /// Event indicators' last values
    pub pre_z: Vec<f64>,
    /// Current state vector
    pub x: Vec<f64>,
    /// Derivative of the state vector
    pub dx: Vec<f64>,
}

impl<M> WrapperContext<M> 
where 
    M: Model + UserModel + ModelGetSetStates
{
    pub fn new(
        logging_on: bool,
        log_message: LogMessageClosure,
        resource_path: PathBuf,
        early_return_allowed: bool,
    ) -> Self {
        Self {
            basic: BasicContext::new(logging_on, log_message, resource_path),
            num_steps: 0,
            early_return_allowed,
            event_mode_used: false,
            next_communication_point: 0.0,
            cur_z: vec![0.0; M::MAX_EVENT_INDICATORS],
            pre_z: vec![0.0; M::MAX_EVENT_INDICATORS],
            x: vec![0.0; M::NUM_STATES],
            dx: vec![0.0; M::NUM_STATES],
        }
    }
}

impl<M> Context<M> for WrapperContext<M>
where
    M: UserModel + 'static,
{
    fn logging_on(&self, category: M::LoggingCategory) -> bool {
        self.basic.logging_on(category)
    }

    fn set_logging(&mut self, category: M::LoggingCategory, enabled: bool) {
        self.basic.set_logging(category, enabled);
    }

    fn log(&self, status: Fmi3Status, category: M::LoggingCategory, args: std::fmt::Arguments<'_>) {
        self.basic.log(status, category, args);
    }

    fn resource_path(&self) -> &PathBuf {
        self.basic.resource_path()
    }

    fn initialize(&mut self, start_time: f64, stop_time: Option<f64>) {
        self.basic.initialize(start_time, stop_time);
    }

    fn time(&self) -> f64 {
        self.basic.time()
    }

    fn set_time(&mut self, time: f64) {
        self.basic.set_time(time);
    }

    fn stop_time(&self) -> Option<f64> {
        self.basic.stop_time()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
