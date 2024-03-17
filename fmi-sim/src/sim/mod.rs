#[cfg(feature = "fmi2")]
pub mod fmi2;
#[cfg(feature = "fmi3")]
pub mod fmi3;
mod interpolation;
mod io;
pub mod params;
pub mod solver;
pub mod traits;
pub mod util;

use arrow::array::RecordBatch;
use fmi::traits::FmiInstance;
pub use io::{InputState, RecorderState};

use crate::{options, Error};

use self::{
    params::SimParams,
    solver::Solver,
    traits::{FmiSchemaBuilder, FmiSim},
};

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

/// Lower-level simulation function that takes an FMI import and a set of options.
pub fn simulate_with<Imp: FmiSim>(
    input_data: Option<RecordBatch>,
    interface: &options::Interface,
    import: Imp,
) -> Result<RecordBatch, Error> {
    match interface {
        #[cfg(feature = "me")]
        options::Interface::ModelExchange(options) => import.simulate_me(options, input_data),
        #[cfg(feature = "cs")]
        options::Interface::CoSimulation(options) => import.simulate_cs(options, input_data),
        #[cfg(feature = "se")]
        options::Interface::ScheduledExecution(options) => unimplemented!(),
        #[cfg(any(not(feature = "me"), not(feature = "cs")))]
        _ => Err(fmi::Error::UnsupportedInterface(format!("{}", interface)).into()),
    }
}
