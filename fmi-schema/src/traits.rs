//! Common traits for FMI schema

pub trait DefaultExperiment {
    fn start_time(&self) -> Option<f64>;
    fn stop_time(&self) -> Option<f64>;
    fn tolerance(&self) -> Option<f64>;
    fn step_size(&self) -> Option<f64>;
}
