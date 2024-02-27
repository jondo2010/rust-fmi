use super::{Model, Solver, SolverError};

pub struct Euler {
    /// Current time
    time: f64,
    /// Continuous states
    x: Vec<f64>,
    /// Derivatives of continuous states
    dx: Vec<f64>,
    /// Event indicators
    z: Vec<f64>,
    prez: Vec<f64>,
}

impl<M: Model> Solver<M> for Euler {
    fn new(start_time: f64, _tol: f64, nx: usize, nz: usize) -> Self {
        Self {
            time: start_time,
            x: vec![0.0; nx],
            dx: vec![0.0; nx],
            z: vec![0.0; nz],
            prez: vec![0.0; nz],
        }
    }

    fn step(&mut self, model: &mut M, next_time: f64) -> Result<(f64, bool), SolverError> {
        let dt = next_time - self.time;

        if self.x.len() > 0 {
            model.get_continuous_states(&mut self.x);
            model.get_continuous_state_derivatives(&mut self.dx);

            for i in 0..self.x.len() {
                self.x[i] += self.dx[i] * dt;
            }

            model.set_continuous_states(&self.x);
        }

        let mut state_event = false;

        if self.z.len() > 0 {
            model.get_event_indicators(&mut self.z);

            for i in 0..self.z.len() {
                if self.prez[i] <= 0.0 && self.z[i] > 0.0 {
                    state_event = true; // -\+
                } else if self.prez[i] > 0.0 && self.z[i] <= 0.0 {
                    state_event = true; // +/-
                }
                self.prez[i] = self.z[i];
            }
        }
        self.time = next_time;

        Ok((self.time, state_event))
    }

    fn reset(&mut self, model: &mut M, _time: f64) -> Result<(), SolverError> {
        if self.z.len() > 0 {
            model.get_event_indicators(&mut self.z);
        }
        Ok(())
    }
}
