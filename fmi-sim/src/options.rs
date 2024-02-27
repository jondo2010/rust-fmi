#[derive(Default, Debug, Clone, clap::ValueEnum)]
pub enum SolverArg {
    /// Euler solver
    #[clap(name = "euler")]
    #[default]
    Euler,
}

#[derive(Default, Debug, clap::Args)]
/// Perform a ModelExchange simulation
pub struct ModelExchangeOptions {
    #[command(flatten)]
    pub common: CommonOptions,

    /// The solver to use
    #[arg(long, default_value = "euler")]
    pub solver: SolverArg,
}

#[derive(Default, Debug, clap::Args)]
/// Perform a CoSimulation simulation
pub struct CoSimulationOptions {
    #[command(flatten)]
    pub common: CommonOptions,

    /// Use event mode
    #[arg(long)]
    pub event_mode_used: bool,

    /// Support early-return in Co-Simulation.
    #[arg(long)]
    pub early_return_allowed: bool,
}

#[derive(Debug, clap::Subcommand)]
pub enum Interface {
    #[cfg(feature = "me")]
    #[command(alias = "me")]
    ModelExchange(ModelExchangeOptions),

    #[cfg(feature = "cs")]
    #[command(alias = "cs")]
    CoSimulation(CoSimulationOptions),

    /// Perform a ScheduledExecution simulation
    #[cfg(feature = "se")]
    ScheduledExecution(CommonOptions),
}

#[derive(Default, Debug, clap::Args)]
pub struct CommonOptions {
    /// Name of the CSV file name with input data.
    #[arg(short = 'i', long)]
    pub input_file: Option<std::path::PathBuf>,

    /// Simulation result output CSV file name. Default is to use standard output.
    #[arg(short = 'o', long)]
    pub output_file: Option<std::path::PathBuf>,

    /// File containing initial serialized FMU state.
    #[arg(long)]
    pub initial_fmu_state_file: Option<std::path::PathBuf>,

    /// File to write final serialized FMU state.
    #[arg(long)]
    pub final_fmu_state_file: Option<std::path::PathBuf>,

    /// Separator to be used in CSV input/output.
    #[arg(short = 'c', default_value = ",")]
    pub separator: String,

    /// Mangle variable names to avoid quoting (needed for some CSV importing applications, but not
    /// according to the CrossCheck rules).
    #[arg(short = 'm')]
    pub mangle_names: bool,

    /// List of initial values to set before simulation starts. The format is
    /// "variableName=value", where variableName is the name of the variable and value is the
    /// value to set. The value must be of the same type as the variable. The variable name must
    /// be a valid FMI variable name, i.e. it must be a valid identifier and it must be unique.
    #[arg(short = 'v')]
    pub initial_values: Vec<String>,

    /// Print also left limit values at event points to the output file to investigate event
    /// behaviour. Default is to only print values after event handling.
    #[arg(short = 'd')]
    pub print_left_limit: bool,

    /// Print all variables to the output file. Default is to only print outputs.
    #[arg(long = "print-all")]
    pub print_all_variables: bool,

    /// For ME simulation: Decides step size to use in forward Euler.
    /// For CS simulation: Decides communication step size for the stepping.
    /// Observe that if a small stepSize is used the number of saved outputs will still be limited
    /// by the number of output points. Default is to calculated a step size from the number of
    /// output points. See the -n option for how the number of outputs is set.
    #[arg(long = "ss")]
    pub step_size: Option<f64>,

    #[arg(long = "output-interval")]
    pub output_interval: Option<f64>,

    /// Maximum number of output points. "-n 0" means output at every step and the number of
    /// outputs are decided by the -h option. Observe that no interpolation is used, output points
    /// are taken at the steps.
    #[arg(short = 'n', default_value = "500")]
    pub num_steps: usize,

    /// Simulation start time, default is to use information from 'DefaultExperiment' as specified
    /// in the model description XML.
    #[arg(short = 's')]
    pub start_time: Option<f64>,

    /// Simulation stop time, default is to use information from 'DefaultExperiment' as specified
    /// in the model description XML.
    #[arg(short = 'f')]
    pub stop_time: Option<f64>,

    /// Relative tolerance
    #[arg(long)]
    pub tolerance: Option<f64>,
}

/// Simulate an FMU
#[derive(Debug, clap::Parser)]
#[command(version, about)]
pub struct FmiCheckOptions {
    /// Which FMI interface to use
    #[command(subcommand)]
    pub interface: Interface,
    /// The FMU model to read
    #[arg(long)]
    pub model: std::path::PathBuf,
}
