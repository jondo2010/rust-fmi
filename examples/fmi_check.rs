use failure::format_err;
use prettytable::{cell, color, row, table, Attr, Cell, Row, Table};
use quicli::prelude::*;
use std::rc::Rc;
use structopt::StructOpt;

/// Query/Validate/Simulate an FMU
#[derive(Debug, StructOpt)]
#[structopt(raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
struct FmiCheckOptions {
    /// Name of the CSV file name with input data.
    #[structopt(short = "i", parse(from_os_str))]
    input_file: Option<std::path::PathBuf>,

    /// Simulation result output CSV file name. Default is to use standard output.
    #[structopt(short = "o", parse(from_os_str))]
    output_file: Option<std::path::PathBuf>,

    /// Error log file name. Default is to use standard error.
    #[structopt(short = "e", parse(from_os_str))]
    error_log: Option<std::path::PathBuf>,

    /// Temporary dir to use for unpacking the FMU. Default is to use system-wide directory, e.g., C:\Temp or /tmp.
    #[structopt(short = "t", parse(from_os_str))]
    temp_dir: Option<std::path::PathBuf>,

    /// Separator to be used in CSV output.
    #[structopt(short = "c", default_value = ",")]
    separator: String,

    /// Print also left limit values at event points to the output file to investigate event behaviour. Default is to only print values after event handling.
    #[structopt(short = "d")]
    print_left_limit: bool,

    /// Print all variables to the output file. Default is to only print outputs.
    #[structopt(short = "f")]
    print_all_variables: bool,

    /// Mangle variable names to avoid quoting (needed for some CSV importing applications, but not according to the CrossCheck rules).
    #[structopt(short = "m")]
    mangle_names: bool,

    /// For ME simulation: Decides step size to use in forward Euler.
    /// For CS simulation: Decides communication step size for the stepping.
    /// Observe that if a small stepSize is used the number of saved outputs will still be limited by the number of output points. Default is to calculated a step size from the number of output points. See the -n option for how the number of outputs is set.
    #[structopt(short = "h")]
    step_size: Option<f64>,

    /// Maximum number of output points. "-n 0" means output at every step and the number of outputs are decided by the -h option. Observe that no interpolation is used, output points are taken at the steps.
    #[structopt(short = "n", default_value = "500")]
    num_steps: u32,

    /// Simulation stop time, default is to use information from 'DefaultExperiment' as specified in the model description XML.
    #[structopt(short = "s")]
    stop_time: Option<f64>,

    /// Check the XML
    #[structopt(long = "xml")]
    check_xml: bool,

    /// Perform a ModelExchange simulation (implicitly enables --xml)
    #[structopt(long = "me")]
    sim_me: bool,

    /// Perform a CoSimulation simulation (implicitly enables --xml)
    #[structopt(long = "cs")]
    sim_cs: bool,

    /// The FMU model to read
    #[structopt(name = "model.fmu", parse(from_os_str))]
    model: std::path::PathBuf,

    // Quick and easy logging setup you get for free with quicli
    #[structopt(flatten)]
    verbosity: Verbosity,
}

fn print_info(import: &Rc<fmi::Import>) {
    let mut table = table!(
        [br -> "Model name:", import.descr().model_name],
        [br -> "Model GUID:", import.descr().guid],
        [br -> "Model version:", import.descr().version],
        [br -> "Description:", import.descr().description],
        [br -> "Generation Tool:", import.descr().generation_tool],
        [br -> "Generation Date:", import.descr().generation_date_and_time]
    );
    table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
    table.printstd();

    let counts = import.descr().model_counts();
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

    /*
        table.add_row(Row::new(vec![
            Cell::new("foobar")
                .with_style(Attr::Bold)
                .with_style(Attr::ForegroundColor(color::GREEN)),
            Cell::new("bar")
                .with_style(Attr::BackgroundColor(color::RED))
                .with_style(Attr::Italic(true))
                .with_hspan(2),
            Cell::new("foo"),
        ]));
    */
}

struct FmiCheckState {
    //import: Rc<fmi::Import>,
    pub tolerance: Option<f64>,
    pub start_time: f64,
    pub stop_time: f64,
    pub step_size: f64,

    pub data_table: Table,
}

impl FmiCheckState {
    pub fn from_options_and_experiment(
        options: &FmiCheckOptions,
        default_experiment: &Option<&fmi::model_descr::DefaultExperiment>,
    ) -> Result<Self, Error> {
        let tolerance = default_experiment.and_then(|ref de| Some(de.tolerance));

        let start_time = default_experiment
            .map(|ref de| de.start_time)
            .unwrap_or(0.0);

        let stop_time = options
            .stop_time
            .or_else(|| default_experiment.and_then(|ref de| Some(de.stop_time)))
            .unwrap_or(10.0);

        let step_size = options
            .step_size
            .unwrap_or_else(|| stop_time / options.num_steps as f64);

        //TODO: better error handling
        /*
        let writer: Box<std::io::Write> = options.output_file.as_ref().map_or_else(
            || Box::new(std::io::stdout()) as Box<std::io::Write>,
            |output_path| {
                Box::new(std::fs::File::create(output_path).unwrap()) as Box<std::io::Write>
            },
        );
        let csv_out = csv::WriterBuilder::new().from_writer(writer);
        */

        let mut data_table = Table::new();
        data_table.set_format(*prettytable::format::consts::FORMAT_NO_LINESEP_WITH_TITLE);

        Ok(FmiCheckState {
            tolerance,
            start_time,
            stop_time,
            step_size,
            data_table,
        })
    }
}

fn main() -> CliResult {
    let args: FmiCheckOptions = FmiCheckOptions::from_args();

    //args.verbosity.setup_env_logger("fmi_check")?;

    let level_filter = args.verbosity.log_level().to_level_filter();
    pretty_env_logger::formatted_builder()
        .filter(Some("fmi_check"), level_filter)
        .filter(Some("fmi"), level_filter)
        .filter(Some("inst1"), level_filter)
        .filter(None, log::Level::Warn.to_level_filter())
        .try_init()?;

    let import = fmi::Import::new(std::path::Path::new(&args.model))?;

    if import.descr().fmi_version != "2.0" {
        return Err(format_err!("Unsupported FMI Version"))?;
    }

    print_info(&import);

    let default_experiment = &import.descr().default_experiment.as_ref();
    let mut fmi_check = FmiCheckState::from_options_and_experiment(&args, default_experiment)?;
    let outputs = import.descr().outputs()?;

    // Set data table headers
    fmi_check.data_table.set_titles(Row::from(
        ["t".to_owned()]
            .iter()
            .chain(outputs.iter().map(|(sv, _)| &sv.name)),
    ));

    if args.sim_me || args.sim_cs || args.check_xml {
        // Validate XML?
    }

    if !args.sim_me && !args.sim_cs {
        info!("Simulation was not requested");
        return Ok(());
    }

    if args.sim_me {
        use fmi::instance::{Common, ModelExchange};
        let instance1 = fmi::InstanceME::new(&import, "inst1", false, true)?;

        let categories = &import
            .descr()
            .log_categories
            .as_ref()
            .map(|log_categories| {
                log_categories
                    .categories
                    .iter()
                    .map(|cat| cat.name.as_ref())
                    .collect::<Vec<&str>>()
            })
            .unwrap_or(vec![]);

        instance1.set_debug_logging(false, categories)?;

        info!(
            "Preparing ME Simulation from t=[{},{}], dt={}, tol={}",
            fmi_check.start_time,
            fmi_check.stop_time,
            fmi_check.step_size,
            fmi_check.tolerance.unwrap_or(0.0)
        );

        instance1.setup_experiment(
            fmi_check.tolerance,
            fmi_check.start_time,
            Some(fmi_check.stop_time),
        )?;
        instance1.enter_initialization_mode()?;
        instance1.exit_initialization_mode()?;

        // fmiExitInitializationMode leaves FMU in event mode
        let _ = instance1.do_event_iteration()?;
        instance1.enter_continuous_time_mode()?;

        let mut states = vec![0.0; import.descr().num_states()];
        let mut states_der = vec![0.0; import.descr().num_states()];

        let mut events = vec![0.0; import.descr().num_event_indicators()];
        let mut events_prev = vec![0.0; import.descr().num_event_indicators()];

        instance1.get_continuous_states(&mut states)?;
        instance1.get_event_indicators(&mut events_prev)?;
        info!(
            "Initialized FMU for simulation starting at time {}: {:?}",
            fmi_check.start_time, states
        );

        let mut current_time = fmi_check.start_time;
        let mut terminate_simulation = false;

        // Write initial outputs
        fmi_check
            .data_table
            .add_row(Row::from([current_time].iter().cloned().chain(
                outputs.iter().map(|(sv, _)| match sv.elem {
                    fmi::model_descr::ScalarVariableElement::Real { .. } => {
                        instance1.get_real(&sv).unwrap()
                    }
                    _ => 0.0,
                }),
            )));

        while (current_time < fmi_check.stop_time) && !terminate_simulation {
            // Get derivatives
            instance1.get_derivatives(&mut states_der)?;

            // Choose time step and advance tcur
            let mut next_time = current_time + fmi_check.step_size;

            // adjust tnext step to get tend exactly
            if next_time > (fmi_check.stop_time - fmi_check.step_size / 1e16) {
                next_time = fmi_check.stop_time;
            }

            // Check for eternal events
            //fmi2_check_external_events(tcur,tnext, &eventInfo, &cdata->fmu2_inputData);

            // adjust for time events
            let time_event = false;
            /*
            if (eventInfo.nextEventTimeDefined && (tnext >= eventInfo.nextEventTime)) {
                tnext = eventInfo.nextEventTime;
                time_event = 1;
            }
            */
            let current_step = next_time - current_time;
            current_time = next_time;

            // Set time
            trace!("Simulation time: {}", current_time);
            instance1.set_time(current_time)?;

            // Set inputs
            // During continuous-time mode, only Continuous, Real, Inputs can be set.
            //if(!fmi2_status_ok_or_warning(fmistatus = fmi2_set_inputs(cdata, tcur)))

            // Integrate for next states
            for (x, dx) in states.iter_mut().zip(states_der.iter()) {
                *x += (*dx) * current_step;
            }

            // Set next states
            instance1.set_continuous_states(&states)?;

            // Check if an event indicator has triggered
            instance1.get_event_indicators(&mut events)?;

            let zero_crossing_event = events
                .iter()
                .zip(events_prev.iter())
                .find(|(e, e_prev)| (*e) * (*e_prev) < 0.0);

            // Step is completed
            let ret = instance1.completed_integrator_step(true)?;
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
                trace!("Handling a {} event", event_kind);

                /*
                if(cdata->print_all_event_vars){
                    /* print variable values before event handling*/
                if(fmi2_write_csv_data(cdata, tcur) != jm_status_success) { jmstatus = jm_status_error; }
                }
                 */

                instance1.enter_event_mode()?;
                let (nominals_changed, values_changed) = instance1.do_event_iteration()?;

                if values_changed {
                    instance1.get_continuous_states(&mut states)?;
                }

                if nominals_changed {
                    //instance1.get_nominals_of_continuous_states(&nominals);
                }

                instance1.get_event_indicators(&mut events_prev)?;
                instance1.enter_continuous_time_mode()?;
            }

            // print current variable values
            fmi_check
                .data_table
                .add_row(Row::from([current_time].iter().cloned().chain(
                    outputs.iter().map(|(sv, _)| match sv.elem {
                        fmi::model_descr::ScalarVariableElement::Real { .. } => {
                            instance1.get_real(&sv).unwrap()
                        }
                        _ => 0.0,
                    }),
                )));

            if terminate_simulation {
                info!("FMU requested simulation termination");
                break;
            }
        }

        //TODO check for discard:
        //"Simulation loop terminated at time %g since FMU returned fmiDiscard. Running with shorter time step may help.", tcur);

        //TODO check for error:
        //"Simulation loop terminated at time %g since FMU returned status: %s", tcur, fmi2_status_to_string(fmistatus));

        info!("Simulation finished successfully at time {}", current_time);

        instance1.terminate()?;

        fmi_check.data_table.printstd();
    }

    if args.sim_cs {
        let instance1 = fmi::InstanceCS::new(&import, "inst1", false, true)?;
        println!("{:#?}", instance1);
    }

    Ok(())
}

#[test]
fn tester() {
    let v = nalgebra::DVector::<f64>::zeros(10);

    v.len();
    v.norm();

    nalgebra::Matrix::angle(&v, &v);
}
