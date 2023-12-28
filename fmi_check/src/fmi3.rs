use std::{fs::File, path::Path};

use fmi::{
    fmi3::{
        instance::{
            traits::{CoSimulation, Common},
            DiscreteStates, Instance,
        },
        schema::{AbstractVariableTrait, Causality, FmiModelDescription, Variability},
    },
    import::FmiImport,
};

const FIXED_SOLVER_STEP: f64 = 1e-3;

fn apply_continuous_inputs<Tag>(md: &FmiModelDescription, instance: &mut Instance<'_, Tag>) {
    for variable in
        md.model_variables.float32.iter().filter(|v| {
            v.causality() == Causality::Input && v.variability() == Variability::Continuous
        })
    {}

    //instance.set_float32(vrs, values)
}

struct CsvHeader<'a> {
    name: &'a str,
    //r#type: fmi::fmi3::schema::
}

fn csv_input<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)?;

    let mut headers = reader.headers()?;
    let time_idx = headers
        .iter()
        .enumerate()
        .find_map(|(i, &h)| (h == "time").then_some(i))
        .ok_or_else(|| anyhow::anyhow!("no time column"))?;

    let num_cols = headers.len();

    for x in reader.records() {
        let record = x?;

        let time = record
            .get(time_idx)
            .ok_or_else(|| anyhow::anyhow!("no time column"))?;

        let time = time.parse::<f64>()?;

        for (i, header) in headers.iter().enumerate() {
            if i == time_idx {
                continue;
            }

            let value = record
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("no value column"))?;

            let value = value.parse::<f64>()?;

            // set input
        }
    }

    Ok(())
}

fn co_simulation(
    import: &fmi::fmi3::import::Fmi3,
    start_time: f64,
    stop_time: f64,
) -> anyhow::Result<()> {
    let mut inst = import.instantiate_cs("inst1", true, true, true, true, &[])?;

    // set start values
    //CALL(applyStartValues(S));

    let mut time = start_time;

    // initialize the FMU
    inst.enter_initialization_mode(None, time, Some(stop_time))
        .ok()?;

    // apply continuous and discrete inputs
    //CALL(applyContinuousInputs(S, true));
    //CALL(applyDiscreteInputs(S));

    inst.exit_initialization_mode().ok()?;

    let mut states = DiscreteStates::default();

    // update discrete states
    let terminate = loop {
        inst.update_discrete_states(&mut states)?;

        if states.terminate_simulation {
            break false;
        }

        if !states.discrete_states_need_update {
            break true;
        }
    };

    inst.enter_step_mode().ok()?;

    // communication step size
    let step_size = 10 * FIXED_SOLVER_STEP;

    loop {
        //CALL(recordVariables(S, outputFile));

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
            //CALL(recordVariables(S, outputFile));

            // enter Event Mode
            inst.enter_event_mode().ok()?;

            // apply continuous and discrete inputs
            //CALL(applyContinuousInputs(S, true));
            //CALL(applyDiscreteInputs(S));

            // update discrete states
            loop {
                inst.update_discrete_states(&mut states)?;

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
