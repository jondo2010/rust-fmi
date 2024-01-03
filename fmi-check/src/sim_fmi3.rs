use arrow::{array::StringArray, datatypes::DataType};
use fmi::{
    fmi3::instance::{CoSimulation, Common, DiscreteStates, Instance},
    FmiImport, FmiInstance,
};

use crate::{
    io::{InputState, OutputState},
    options,
};

const FIXED_SOLVER_STEP: f64 = 1e-3;

/// Parse the start values from the command line and set them in the FMU.
pub fn apply_start_values<Tag>(
    instance: &mut Instance<'_, Tag>,
    start_values: &[String],
) -> anyhow::Result<()> {
    for start_value in start_values.into_iter() {
        let (name, value) = start_value
            .split_once('=')
            .ok_or_else(|| anyhow::anyhow!("Invalid start value"))?;

        let var = instance
            .model_description()
            .model_variables
            .iter_abstract()
            .find(|var| var.name() == name)
            .ok_or_else(|| anyhow::anyhow!("Invalid variable name"))?;

        let ary = StringArray::from(vec![value.to_string()]);
        let ary = arrow::compute::cast(&ary, &var.data_type().into())
            .map_err(|_| anyhow::anyhow!("Error casting type"))?;
        instance.set_values(&[var.value_reference()], &ary);
    }

    Ok(())
}

pub fn co_simulation(
    import: &fmi::fmi3::import::Fmi3Import,
    options: &options::Simulate,
) -> anyhow::Result<()> {
    let sim_params = SimParams::new(import, options)?;

    let mut inst = import.instantiate_cs("inst1", true, true, true, true, &[])?;
    let input_state = options
        .input_file
        .as_ref()
        .map(|path| InputState::new(import, path))
        .transpose()?;

    let mut output_state = OutputState::new(import, sim_params.num_steps)?;

    // set start values
    apply_start_values(&mut inst, &options.initial_values)?;

    let mut time = sim_params.start_time;

    // initialize the FMU
    inst.enter_initialization_mode(None, time, Some(sim_params.stop_time))
        .ok()?;

    // apply continuous and discrete inputs
    if let Some(input_state) = input_state {
        input_state.apply_continuous_inputs(time, &mut inst);
        input_state.apply_discrete_inputs(time, &mut inst);
    }

    inst.exit_initialization_mode().ok()?;

    let mut states = DiscreteStates::default();

    // update discrete states
    let terminate = loop {
        inst.update_discrete_states(&mut states).ok()?;

        if states.terminate_simulation {
            break false;
        }

        if !states.discrete_states_need_update {
            break true;
        }
    };

    inst.enter_step_mode().ok()?;

    loop {
        output_state.record_variables(&mut inst, time)?;

        if states.terminate_simulation || time >= sim_params.stop_time {
            break;
        }

        let mut event_encountered = false;
        let mut early_return = false;

        inst.do_step(
            time,
            sim_params.step_size,
            true,
            &mut event_encountered,
            &mut states.terminate_simulation,
            &mut early_return,
            &mut time,
        )
        .ok()?;

        if event_encountered {
            log::trace!("Event encountered at t = {}", time);
            // record variables before event update
            output_state.record_variables(&mut inst, time)?;

            // enter Event Mode
            inst.enter_event_mode().ok()?;

            // apply continuous and discrete inputs
            // CALL(applyContinuousInputs(S, true));
            // CALL(applyDiscreteInputs(S));

            // update discrete states
            loop {
                inst.update_discrete_states(&mut states).ok()?;

                if states.terminate_simulation {
                    break;
                }

                if !states.discrete_states_need_update {
                    break;
                }
            }

            // return to Step Mode
            inst.enter_step_mode().ok()?;
        }
    }

    Ok(())
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
