use arrow::record_batch::RecordBatch;

use crate::options;

pub fn me_simulation(
    import: &fmi::fmi3::import::Fmi3Import,
    options: options::CliOptions,
) -> anyhow::Result<RecordBatch> {
    let _sim_params = options::SimOptions::new(import, &options)?;

    let mut _inst = import.instantiate_me("inst1", false, true)?;

    todo!();

    // let input_state = options
    // .input_file
    // .as_ref()
    // .map(|path| InputState::new(import, path))
    // .transpose()
    // .context("Building InputState")?;
    //
    // let mut output_state = OutputState::new(import, &sim_params).context("Building
    // OutputState")?;
    //
    // set start values
    // InputState::apply_start_values(&mut inst, &options.initial_values)?;
    //
    // let mut time = sim_params.start_time;
}
