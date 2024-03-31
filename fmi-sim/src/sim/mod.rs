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
    interpolation::Linear,
    params::SimParams,
    traits::{FmiSchemaBuilder, FmiSim, InstanceSetValues},
};

pub struct SimState<Inst>
where
    Inst: FmiInstance,
    Inst::Import: FmiSchemaBuilder,
{
    sim_params: SimParams,
    input_state: InputState<Inst>,
    recorder_state: RecorderState<Inst>,
    inst: Inst,
    next_event_time: Option<f64>,
}

pub trait SimStateTrait<Inst: FmiInstance> {
    fn params(&mut self) -> &mut SimParams;

    fn inst(&mut self) -> &mut Inst;

    fn apply_input(
        &mut self,
        time: f64,
        discrete: bool,
        continuous: bool,
        input_event: bool,
    ) -> Result<(), Error>;
}

impl<Inst> SimStateTrait<Inst> for SimState<Inst>
where
    Inst: FmiInstance + InstanceSetValues,
    Inst::Import: FmiSchemaBuilder,
{
    fn params(&mut self) -> &mut SimParams {
        &mut self.sim_params
    }

    fn inst(&mut self) -> &mut Inst {
        &mut self.inst
    }

    fn apply_input(
        &mut self,
        time: f64,
        discrete: bool,
        continuous: bool,
        input_event: bool,
    ) -> Result<(), Error> {
        self.input_state.apply_input::<Linear>(
            time,
            &mut self.inst,
            discrete,
            continuous,
            input_event,
        )
    }
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
