use arrow::record_batch::RecordBatch;
use fmi::fmi3::{import::Fmi3Import, instance::InstanceME};

use crate::sim::{
    options::SimOptions,
    params::SimParams,
    traits::{FmiSchemaBuilder, SimTrait},
    InputState, OutputState, SimState,
};

impl<'a> SimTrait<'a> for SimState<InstanceME<'a>> {
    type Import = Fmi3Import;
    type InputState = InputState<InstanceME<'a>>;
    type OutputState = OutputState<InstanceME<'a>>;

    fn new(
        import: &'a Self::Import,
        sim_params: SimParams,
        input_state: Self::InputState,
        output_state: Self::OutputState,
    ) -> anyhow::Result<Self> {
        let inst = import.instantiate_me("inst1", true, true)?;
        let time = sim_params.start_time;
        Ok(Self {
            sim_params,
            input_state,
            output_state,
            inst,
            time,
            next_event_time: None,
        })
    }

    fn main_loop(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}

/// Run a model-exchange simulation
pub fn model_exchange(import: &Fmi3Import, options: SimOptions) -> anyhow::Result<RecordBatch> {
    let start_values = import.parse_start_values(&options.initial_values)?;

    let mut sim_state = SimState::<InstanceME>::new_from_options(import, &options)?;
    sim_state.initialize(start_values, options.initial_fmu_state_file)?;
    sim_state.main_loop()?;

    Ok(sim_state.output_state.finish())
}
