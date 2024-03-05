#[cfg(feature = "fmi2")]
pub mod fmi2;
#[cfg(feature = "fmi3")]
pub mod fmi3;
mod interpolation;
mod io;
pub mod params;
mod schema_builder;
pub mod set_values;
pub mod solver;
mod traits;
pub mod util;

use fmi::traits::FmiInstance;
pub use io::{InputState, OutputState};

use self::{params::SimParams, solver::Solver, traits::FmiSchemaBuilder};

pub struct SimState<Inst, S>
where
    Inst: FmiInstance,
    Inst::Import: FmiSchemaBuilder,
    S: Solver<Inst>,
{
    sim_params: SimParams,
    input_state: InputState<Inst>,
    output_state: OutputState<Inst>,
    inst: Inst,
    time: f64,
    next_event_time: Option<f64>,
    _phantom: std::marker::PhantomData<S>,
}
