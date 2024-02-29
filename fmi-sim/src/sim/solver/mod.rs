use fmi::fmi3::instance::ModelExchange;

mod euler;

pub use euler::Euler;

pub trait Model {
    fn get_continuous_states(&mut self, x: &mut [f64]);
    fn set_continuous_states(&mut self, states: &[f64]);
    fn get_continuous_state_derivatives(&mut self, dx: &mut [f64]);
    fn get_event_indicators(&mut self, z: &mut [f64]);
}

impl<Inst: ModelExchange> Model for Inst {
    fn get_continuous_states(&mut self, x: &mut [f64]) {
        ModelExchange::get_continuous_states(self, x);
    }

    fn set_continuous_states(&mut self, states: &[f64]) {
        ModelExchange::set_continuous_states(self, states);
    }

    fn get_continuous_state_derivatives(&mut self, dx: &mut [f64]) {
        ModelExchange::get_continuous_state_derivatives(self, dx);
    }

    fn get_event_indicators(&mut self, z: &mut [f64]) {
        ModelExchange::get_event_indicators(self, z);
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SolverError {
    #[error("Step error")]
    StepError,
}

pub trait Solver<M> {
    /// Create a new Solver instance.
    /// # Arguments
    /// * `nx` - The number of continuous states.
    /// * `nz` - The number of event indicators.
    fn new(start_time: f64, tolerance: f64, nx: usize, nz: usize) -> Self;

    /// Perform a single step of the solver.
    ///
    /// # Arguments
    /// * `model` - The model to be simulated.
    /// * `next_time` - The time at which the simulation should stop.
    ///
    /// # Returns
    /// A tuple of (`time_reached`, `state_event`)
    fn step(&mut self, model: &mut M, next_time: f64) -> Result<(f64, bool), SolverError>;

    /// Reset the solver
    fn reset(&mut self, model: &mut M, time: f64) -> Result<(), SolverError>;
}

/// A dummy solver that does nothing.
pub struct DummySolver;

impl<M> Solver<M> for DummySolver {
    fn new(_start_time: f64, _tolerance: f64, _nx: usize, _nz: usize) -> Self {
        Self
    }

    fn step(&mut self, _model: &mut M, _next_time: f64) -> Result<(f64, bool), SolverError> {
        Ok((0.0, false))
    }

    fn reset(&mut self, _model: &mut M, _time: f64) -> Result<(), SolverError> {
        Ok(())
    }
}
