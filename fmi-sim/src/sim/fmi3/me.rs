use arrow::record_batch::RecordBatch;
use fmi::{
    fmi3::{import::Fmi3Import, instance::InstanceME},
    traits::FmiImport,
};

use crate::{
    options::ModelExchangeOptions,
    sim::{
        params::SimParams,
        solver,
        traits::{ImportSchemaBuilder, SimInitialize, SimMe},
        InputState, RecorderState, SimState, SimStateTrait,
    },
    Error,
};

impl<'a> SimStateTrait<'a, InstanceME<'a>> for SimState<InstanceME<'a>> {
    fn new(
        import: &'a Fmi3Import,
        sim_params: SimParams,
        input_state: InputState<InstanceME<'a>>,
        recorder_state: RecorderState<InstanceME<'a>>,
    ) -> Result<Self, Error> {
        let inst = import.instantiate_me("inst1", true, true)?;
        Ok(Self {
            sim_params,
            input_state,
            recorder_state,
            inst,
            next_event_time: None,
        })
    }
}

/// Run a model-exchange simulation
pub fn model_exchange(
    import: &Fmi3Import,
    options: &ModelExchangeOptions,
    input_data: Option<RecordBatch>,
) -> Result<RecordBatch, Error> {
    let sim_params =
        SimParams::new_from_options(&options.common, import.model_description(), true, false);

    let start_values = import.parse_start_values(&options.common.initial_values)?;
    let input_state = InputState::new(import, input_data)?;
    let recorder_state = RecorderState::new(import, &sim_params);

    let mut sim_state =
        SimState::<InstanceME>::new(import, sim_params, input_state, recorder_state)?;
    sim_state.initialize(start_values, options.common.initial_fmu_state_file.as_ref())?;
    let stats = sim_state.main_loop::<solver::Euler>(())?;

    log::info!(
        "Simulation finished at t = {:.1} after {} steps.",
        stats.end_time,
        stats.num_steps
    );

    Ok(sim_state.recorder_state.finish())
}
