use anyhow::anyhow;
use prettytable::{row, table, Row, Table};

mod options;

fn print_info(import: &fmi::Import) {
    let mut table = table!(
        [br -> "Model name:", import.descr.model_name],
        [br -> "Model GUID:", import.descr.guid],
        [br -> "Model version:", import.descr.version],
        [br -> "Description:", import.descr.description],
        [br -> "Generation Tool:", import.descr.generation_tool],
        [br -> "Generation Date:", import.descr.generation_date_and_time]
    );
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.printstd();

    let counts = import.descr.model_counts();
    let mut table = Table::new();
    table.set_titles(row!["By Variability", "By Causality", "By DataType"]);
    table.add_row(row![
        format!("{} constants\n{} continuous\n{} discrete", counts.num_constants, counts.num_continuous, counts.num_discrete),
        format!(
            "{} inputs\n{} outputs\n{} locals\n{} independents\n{} parameters\n{} calculated parameters", 
            counts.num_inputs,
            counts.num_outputs,
            counts.num_local,
            counts.num_independent,
            counts.num_parameters,
            counts.num_calculated_parameters,
        ),
        format!("{} real variables\n{} integer variables\n{} enumeration variables\n{} boolean variables\n{} string variables",
            counts.num_real_vars,
            counts.num_integer_vars,
            counts.num_enum_vars,
            counts.num_bool_vars,
            counts.num_string_vars
        ),
    ]);
    table.printstd();
}

struct FmiCheckState {
    // import: Rc<fmi::Import>,
    pub tolerance: Option<f64>,
    pub start_time: f64,
    pub stop_time: f64,
    pub step_size: f64,
}

impl FmiCheckState {
    pub fn from_options_and_experiment(
        options: &options::FmiCheckOptions,
        default_experiment: &Option<&fmi::model_descr::DefaultExperiment>,
    ) -> anyhow::Result<Self> {
        let tolerance = default_experiment.and_then(|de| Some(de.tolerance));

        let start_time = default_experiment.map(|de| de.start_time).unwrap_or(0.0);

        let stop_time = options
            .stop_time
            .or_else(|| default_experiment.and_then(|de| Some(de.stop_time)))
            .unwrap_or(10.0);

        let step_size = options
            .step_size
            .unwrap_or(stop_time / options.num_steps as f64);

        // TODO: better error handling
        // let writer: Box<std::io::Write> = options.output_file.as_ref().map_or_else(
        // || Box::new(std::io::stdout()) as Box<std::io::Write>,
        // |output_path| {
        // Box::new(std::fs::File::create(output_path).unwrap()) as Box<std::io::Write>
        // },
        // );
        // let csv_out = csv::WriterBuilder::new().from_writer(writer);

        Ok(FmiCheckState {
            tolerance,
            start_time,
            stop_time,
            step_size,
        })
    }
}

fn setup_debug_logging<I: fmi::Common>(
    import: &fmi::Import,
    instance: &I,
    logging_on: bool,
) -> fmi::Result<()> {
    let categories = &import
        .descr
        .log_categories
        .as_ref()
        .map(|log_categories| {
            log_categories
                .categories
                .iter()
                .map(|cat| cat.name.as_ref())
                .collect::<Vec<&str>>()
        })
        .unwrap_or_default();

    instance
        .set_debug_logging(logging_on, categories)
        .and(Ok(()))
}

fn sim_prelude<'a, I: fmi::Common>(
    import: &'a fmi::Import,
    instance: &'a I,
    fmi_check: &FmiCheckState,
) -> fmi::Result<(Vec<UnknownsTuple<'a>>, Table)> {
    let outputs = import.descr.outputs()?;

    // Set data table headers
    let mut data_table = Table::new();
    data_table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    data_table.set_titles(Row::from(
        ["t".to_owned()]
            .iter()
            .chain(outputs.iter().map(|(sv, _)| &sv.name)),
    ));

    setup_debug_logging(import, instance, true)?;

    log::info!(
        "Preparing simulation from t=[{},{}], dt={}, tol={}",
        fmi_check.start_time,
        fmi_check.stop_time,
        fmi_check.step_size,
        fmi_check.tolerance.unwrap_or(0.0)
    );

    instance.setup_experiment(
        fmi_check.tolerance,
        fmi_check.start_time,
        Some(fmi_check.stop_time),
    )?;
    instance.enter_initialization_mode()?;
    instance.exit_initialization_mode()?;

    // Write initial outputs
    data_table.add_row(Row::from([fmi_check.start_time].iter().cloned().chain(
        outputs.iter().map(|(sv, _)| match sv.elem {
            fmi::model_descr::ScalarVariableElement::Real { .. } => instance.get_real(sv).unwrap(),
            _ => 0.0,
        }),
    )));

    Ok((outputs, data_table))
}

fn sim_cs(import: &fmi::Import, fmi_check: &mut FmiCheckState) -> fmi::Result<Table> {
    use fmi::instance::{CoSimulation, Common};
    let instance = fmi::InstanceCS::new(import, "inst1", false, true)?;
    let (outputs, mut data_table) = sim_prelude(import, &instance, fmi_check)?;

    log::info!(
        "Initialized FMU for CS simulation starting at time {}.",
        fmi_check.start_time
    );

    let mut current_time = fmi_check.start_time;

    while current_time < fmi_check.stop_time {
        instance.do_step(current_time, fmi_check.step_size, true)?;

        data_table.add_row(Row::from([current_time].iter().cloned().chain(
            outputs.iter().map(|(sv, _)| match sv.elem {
                fmi::model_descr::ScalarVariableElement::Real { .. } => {
                    instance.get_real(sv).unwrap()
                }
                _ => 0.0,
            }),
        )));

        current_time += fmi_check.step_size;
    }

    Ok(data_table)
}

/// Run a simple ModelExchange simulation
fn sim_me(import: &fmi::Import, fmi_check: &mut FmiCheckState) -> fmi::Result<Table> {
    use fmi::instance::{Common, ModelExchange};
    let instance = fmi::InstanceME::new(import, "inst1", false, true)?;
    let (outputs, mut data_table) = sim_prelude(import, &instance, fmi_check)?;

    // fmiExitInitializationMode leaves FMU in event mode
    let _ = instance.do_event_iteration()?;
    instance.enter_continuous_time_mode()?;

    let mut states = vec![0.0; import.descr.num_states()];
    let mut states_der = vec![0.0; import.descr.num_states()];
    let mut events = vec![0.0; import.descr.num_event_indicators()];
    let mut events_prev = vec![0.0; import.descr.num_event_indicators()];

    instance.get_continuous_states(&mut states)?;
    instance.get_event_indicators(&mut events_prev)?;
    log::info!(
        "Initialized FMU for ME simulation starting at time {}: {:?}",
        fmi_check.start_time,
        states
    );

    let mut current_time = fmi_check.start_time;
    let mut terminate_simulation = false;

    while (current_time < fmi_check.stop_time) && !terminate_simulation {
        // Get derivatives
        instance.get_derivatives(&mut states_der)?;

        // Choose time step and advance tcur
        let mut next_time = current_time + fmi_check.step_size;

        // adjust tnext step to get tend exactly
        if next_time > (fmi_check.stop_time - fmi_check.step_size / 1e16) {
            next_time = fmi_check.stop_time;
        }

        // Check for eternal events
        // fmi2_check_external_events(tcur,tnext, &eventInfo, &cdata->fmu2_inputData);

        // adjust for time events
        let time_event = false;
        // if (eventInfo.nextEventTimeDefined && (tnext >= eventInfo.nextEventTime)) {
        // tnext = eventInfo.nextEventTime;
        // time_event = 1;
        // }
        let current_step = next_time - current_time;
        current_time = next_time;

        // Set time
        log::trace!("Simulation time: {}", current_time);
        instance.set_time(current_time)?;

        // Set inputs
        // During continuous-time mode, only Continuous, Real, Inputs can be set.
        // if(!fmi2_status_ok_or_warning(fmistatus = fmi2_set_inputs(cdata, tcur)))

        // Integrate for next states
        for (x, dx) in states.iter_mut().zip(states_der.iter()) {
            *x += (*dx) * current_step;
        }

        // Set next states
        instance.set_continuous_states(&states)?;

        // Check if an event indicator has triggered
        instance.get_event_indicators(&mut events)?;

        let zero_crossing_event = events
            .iter()
            .zip(events_prev.iter())
            .find(|(e, e_prev)| (*e) * (*e_prev) < 0.0);

        // Step is completed
        let ret = instance.completed_integrator_step(true)?;
        let enter_event_mode = ret.0;
        terminate_simulation = ret.1;

        // Handle events
        if enter_event_mode || zero_crossing_event.is_some() || time_event {
            let event_kind = if enter_event_mode {
                "step"
            } else if zero_crossing_event.is_some() {
                "state"
            } else {
                "time"
            };
            log::trace!("Handling a {} event", event_kind);

            /*
            if(cdata->print_all_event_vars){
                /* print variable values before event handling*/
            if(fmi2_write_csv_data(cdata, tcur) != jm_status_success) { jmstatus = jm_status_error; }
            }
             */

            instance.enter_event_mode()?;
            let (nominals_changed, values_changed) = instance.do_event_iteration()?;

            if values_changed {
                instance.get_continuous_states(&mut states)?;
            }

            if nominals_changed {
                //instance1.get_nominals_of_continuous_states(&nominals);
            }

            instance.get_event_indicators(&mut events_prev)?;
            instance.enter_continuous_time_mode()?;
        }

        // print current variable values
        data_table.add_row(Row::from([current_time].iter().cloned().chain(
            outputs.iter().map(|(sv, _)| match sv.elem {
                fmi::model_descr::ScalarVariableElement::Real { .. } => {
                    instance.get_real(sv).unwrap()
                }
                _ => 0.0,
            }),
        )));

        if terminate_simulation {
            log::info!("FMU requested simulation termination");
            break;
        }
    }

    // TODO check for discard:
    //"Simulation loop terminated at time %g since FMU returned fmiDiscard. Running with shorter
    //"Simulation time step may help.", tcur);

    // TODO check for error:
    //"Simulation loop terminated at time %g since FMU returned status: %s", tcur,
    //"Simulation fmi2_status_to_string(fmistatus));

    log::info!("Simulation finished successfully at time {}", current_time);
    instance.terminate()?;
    Ok(data_table)
}

fn main() -> anyhow::Result<()> {
    let args: FmiCheckOptions = FmiCheckOptions::from_args();

    sensible_env_logger::init!();

    let import = fmi::Import::new(std::path::Path::new(&args.model))?;

    if import.descr.fmi_version != "2.0" {
        return Err(anyhow!("Unsupported FMI Version"));
    }

    print_info(&import);

    let default_experiment = &import.descr.default_experiment.as_ref();
    let mut fmi_check = FmiCheckState::from_options_and_experiment(&args, default_experiment)?;

    if args.sim_me || args.sim_cs || args.check_xml {
        // Validate XML?
    }

    if !args.sim_me && !args.sim_cs {
        log::info!("Simulation was not requested");
        return Ok(());
    }

    if args.sim_me {
        let data_table = sim_me(&import, &mut fmi_check)?;
        data_table.printstd();
    }

    if args.sim_cs {
        let data_table = sim_cs(&import, &mut fmi_check)?;
        data_table.printstd();
    }

    Ok(())
}
