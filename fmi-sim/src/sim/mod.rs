#[cfg(feature = "fmi2")]
pub mod fmi2;
#[cfg(feature = "fmi3")]
pub mod fmi3;
mod interpolation;
mod io;
pub mod params;
pub mod solver;
mod traits;
pub mod util;

use fmi::traits::FmiInstance;
pub use io::{InputState, RecorderState};

use self::{params::SimParams, solver::Solver, traits::FmiSchemaBuilder};

pub struct SimState<Inst, S>
where
    Inst: FmiInstance,
    Inst::Import: FmiSchemaBuilder,
    S: Solver<Inst>,
{
    sim_params: SimParams,
    input_state: InputState<Inst>,
    recorder_state: RecorderState<Inst>,
    inst: Inst,
    next_event_time: Option<f64>,
    _phantom: std::marker::PhantomData<S>,
}

#[derive(Default, Debug)]
pub struct SimStats {
    /// End time of the simulation
    pub end_time: f64,
    /// Number of steps taken
    pub num_steps: usize,
    /// Number of events handled
    pub num_events: usize,
}
