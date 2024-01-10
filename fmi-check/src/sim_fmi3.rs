use anyhow::Context;
use arrow::record_batch::RecordBatch;
use fmi::{
    fmi3::instance::{CoSimulation, Common, DiscreteStates},
    FmiImport,
};

use crate::{
    io::{InputState, OutputState},
    options,
};

pub fn co_simulation(
    import: &fmi::fmi3::import::Fmi3Import,
    options: &options::Simulate,
) -> anyhow::Result<RecordBatch> {
    let sim_params = SimParams::new(import, options)?;

    let mut inst = import.instantiate_cs("inst1", true, true, true, true, &[])?;
    let input_state = options
        .input_file
        .as_ref()
        .map(|path| InputState::new(import, path))
        .transpose()
        .context("Building InputState")?;

    let mut output_state =
        OutputState::new(import, sim_params.num_steps).context("Building OutputState")?;

    // set start values
    InputState::apply_start_values(&mut inst, &options.initial_values)?;

    let mut time = sim_params.start_time;

    // initialize the FMU
    inst.enter_initialization_mode(None, time, Some(sim_params.stop_time))
        .ok()
        .context("enter_initialization_mode")?;

    // apply continuous and discrete inputs
    if let Some(input_state) = input_state {
        input_state.apply_continuous_inputs(time, &mut inst)?;
        input_state.apply_discrete_inputs(time, &mut inst)?;
    }

    inst.exit_initialization_mode()
        .ok()
        .context("exit_initialization_mode")?;

    let mut states = DiscreteStates::default();

    // update discrete states
    let terminate = loop {
        inst.update_discrete_states(&mut states)
            .ok()
            .context("update_discrete_states")?;

        if states.terminate_simulation {
            break false;
        }

        if !states.discrete_states_need_update {
            break true;
        }
    };

    inst.enter_step_mode().ok().context("enter_step_mode")?;

    loop {
        output_state.record_variables(&mut inst, time)?;

        if states.terminate_simulation || time >= sim_params.stop_time {
            break;
        }

        let mut event_encountered = false;
        let mut early_return = false;

        // Step only up to the stop time
        let step_size = sim_params.step_size.min(sim_params.stop_time - time);
        inst.do_step(
            time,
            step_size,
            true,
            &mut event_encountered,
            &mut states.terminate_simulation,
            &mut early_return,
            &mut time,
        )
        .ok()
        .context("do_step")?;

        if event_encountered {
            log::trace!("Event encountered at t = {}", time);
            // record variables before event update
            output_state
                .record_variables(&mut inst, time)
                .context("record_variables")?;

            // enter Event Mode
            inst.enter_event_mode().ok().context("enter_event_mode")?;

            // apply continuous and discrete inputs
            // CALL(applyContinuousInputs(S, true));
            // CALL(applyDiscreteInputs(S));

            // update discrete states
            loop {
                inst.update_discrete_states(&mut states)
                    .ok()
                    .context("update_discrete_states")?;

                if states.terminate_simulation {
                    break;
                }

                if !states.discrete_states_need_update {
                    break;
                }
            }

            // return to Step Mode
            inst.enter_step_mode().ok().context("enter_step_mode")?;
        }
    }

    Ok(output_state.finish())
}

struct SimParams {
    start_time: f64,
    stop_time: f64,
    step_size: f64,
    num_steps: usize,
}

impl SimParams {
    fn new(
        import: &fmi::fmi3::import::Fmi3Import,
        options: &options::Simulate,
    ) -> anyhow::Result<Self> {
        let start_time = options
            .start_time
            .or(import
                .model_description()
                .default_experiment
                .as_ref()
                .and_then(|de| de.start_time))
            .unwrap_or_default();

        let stop_time = options
            .stop_time
            .or(import
                .model_description()
                .default_experiment
                .as_ref()
                .and_then(|de| de.stop_time))
            .unwrap_or_default();

        let step_size = options
            .step_size
            .or(import
                .model_description()
                .default_experiment
                .as_ref()
                .and_then(|de| de.step_size))
            .unwrap_or_default();

        if step_size <= 0.0 {
            return Err(anyhow::anyhow!("`step_size` must be positive."))?;
        }

        let num_steps = {
            if options.num_steps > 0 {
                options.num_steps
            } else {
                ((stop_time - start_time) / step_size).ceil() as usize
            }
        };

        Ok(Self {
            start_time,
            stop_time,
            step_size,
            num_steps,
        })
    }
}
