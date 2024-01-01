use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub enum Action {
    /// Check the XML
    Check,
    /// Perform a ModelExchange simulation
    ME(Simulate),
    /// Perform a CoSimulation simulation
    CS(Simulate),
}

#[derive(Debug, StructOpt)]
pub struct Simulate {
    /// Name of the CSV file name with input data.
    #[structopt(short = "i", parse(from_os_str))]
    pub input_file: Option<std::path::PathBuf>,

    /// Simulation result output CSV file name. Default is to use standard output.
    #[structopt(short = "o", parse(from_os_str))]
    pub output_file: Option<std::path::PathBuf>,

    /// Separator to be used in CSV input/output.
    #[structopt(short = "c", default_value = ",")]
    pub separator: String,

    /// Mangle variable names to avoid quoting (needed for some CSV importing applications, but not
    /// according to the CrossCheck rules).
    #[structopt(short = "m")]
    pub mangle_names: bool,

    /// List of initial values to set before simulation starts. The format is
    /// "variableName=value", where variableName is the name of the variable and value is the
    /// value to set. The value must be of the same type as the variable. The variable name must
    /// be a valid FMI variable name, i.e. it must be a valid identifier and it must be unique.
    #[structopt(short = "v")]
    pub initial_values: Vec<String>,

    /// Print also left limit values at event points to the output file to investigate event
    /// behaviour. Default is to only print values after event handling.
    #[structopt(short = "d")]
    pub print_left_limit: bool,

    /// Print all variables to the output file. Default is to only print outputs.
    #[structopt(long = "print-all")]
    pub print_all_variables: bool,

    /// For ME simulation: Decides step size to use in forward Euler.
    /// For CS simulation: Decides communication step size for the stepping.
    /// Observe that if a small stepSize is used the number of saved outputs will still be limited
    /// by the number of output points. Default is to calculated a step size from the number of
    /// output points. See the -n option for how the number of outputs is set.
    #[structopt(short = "h")]
    pub step_size: Option<f64>,

    /// Maximum number of output points. "-n 0" means output at every step and the number of
    /// outputs are decided by the -h option. Observe that no interpolation is used, output points
    /// are taken at the steps.
    #[structopt(short = "n", default_value = "500")]
    pub num_steps: usize,

    /// Simulation start time, default is to use information from 'DefaultExperiment' as specified
    /// in the model description XML.
    #[structopt(short = "s")]
    pub start_time: Option<f64>,

    /// Simulation stop time, default is to use information from 'DefaultExperiment' as specified
    /// in the model description XML.
    #[structopt(short = "f")]
    pub stop_time: Option<f64>,
}

/// Query/Validate/Simulate an FMU
#[derive(Debug, StructOpt)]
pub struct FmiCheckOptions {
    /// The FMU model to read
    #[structopt(name = "model.fmu", parse(from_os_str))]
    pub model: std::path::PathBuf,

    #[structopt(subcommand)]
    pub action: Action,
}
