use arrow::{array::StringArray, datatypes::DataType};
use fmi::{
    fmi3::instance::{
        traits::{CoSimulation, Common},
        DiscreteStates, Instance,
    },
    import::FmiImport,
    FmiInstance,
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
    let start_time = start_time(import, options);
    let stop_time = stop_time(import, options);
    let num_steps = num_steps(import, options, start_time, stop_time)?;

    let mut inst = import.instantiate_cs("inst1", true, true, true, true, &[])?;
    let input_state = options
        .input_file
        .as_ref()
        .map(|path| InputState::new(import, path))
        .transpose()?;

    let output_state = OutputState::new(import, num_steps)?;

    // set start values
    apply_start_values(&mut inst, &options.initial_values)?;

    let mut time = start_time;

    // initialize the FMU
    inst.enter_initialization_mode(None, time, Some(stop_time))
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

    // communication step size
    let step_size = 10.0 * FIXED_SOLVER_STEP;

    loop {
        output_state.record_variables(&mut inst, time)?;

        if (states.terminate_simulation || time >= stop_time) {
            break;
        }

        let mut event_encountered = false;
        let mut early_return = false;

        inst.do_step(
            time,
            step_size,
            true,
            &mut event_encountered,
            &mut states.terminate_simulation,
            &mut early_return,
            &mut time,
        )
        .ok()?;

        if event_encountered {
            // record variables before event update
            // CALL(recordVariables(S, outputFile));

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

fn num_steps(
    import: &fmi::fmi3::import::Fmi3Import,
    options: &options::Simulate,
    start_time: f64,
    stop_time: f64,
) -> anyhow::Result<usize> {
    if options.num_steps > 0 {
        Ok(options.num_steps)
    } else {
        options
            .step_size
            .ok_or(anyhow::anyhow!(
                "`num_steps > 0` or `step_size` must be specified."
            ))
            .and_then(|step_size| {
                if step_size > 0.0 {
                    Ok(step_size)
                } else {
                    Err(anyhow::anyhow!("`step_size` must be positive."))
                }
            })
            .map(|step_size| ((stop_time - start_time) / step_size).ceil() as usize)
    }
}

fn stop_time(import: &fmi::fmi3::import::Fmi3Import, options: &options::Simulate) -> f64 {
    options
        .stop_time
        .or(import
            .model_description()
            .default_experiment
            .as_ref()
            .and_then(|de| de.stop_time))
        .unwrap_or_default()
}

fn start_time(import: &fmi::fmi3::import::Fmi3Import, options: &options::Simulate) -> f64 {
    let mut time = options
        .start_time
        .or(import
            .model_description()
            .default_experiment
            .as_ref()
            .and_then(|de| de.start_time))
        .unwrap_or_default();
    time
}
