use arrow::record_batch::RecordBatch;
use fmi::{fmi2::import::Fmi2Import, traits::FmiImport};

use crate::{options::CoSimulationOptions, sim::params::SimParams, Error};

/// Run a co-simulation simulation
pub fn co_simulation(
    import: &Fmi2Import,
    options: &CoSimulationOptions,
    input_data: Option<RecordBatch>,
) -> Result<RecordBatch, Error> {
    let sim_params = SimParams::new_from_options(
        &options.common,
        import.model_description(),
        options.event_mode_used,
        options.early_return_allowed,
    );

    /*
    let start_values = import.parse_start_values(&options.common.initial_values)?;
    let input_state = InputState::new(import, input_data)?;
    let output_state = OutputState::new(import, &sim_params);

    let mut sim_state =
        SimState::<InstanceCS, DummySolver>::new(import, sim_params, input_state, output_state)?;
    sim_state.initialize(start_values, options.common.initial_fmu_state_file.as_ref())?;
    sim_state.main_loop()?;

    Ok(sim_state.output_state.finish())
    */
    todo!();
}
