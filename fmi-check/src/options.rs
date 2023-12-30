use structopt::StructOpt;
/// Query/Validate/Simulate an FMU
#[derive(Debug, StructOpt)]
pub struct FmiCheckOptions {
    /// Name of the CSV file name with input data.
    #[structopt(short = "i", parse(from_os_str))]
    pub input_file: Option<std::path::PathBuf>,

    /// Simulation result output CSV file name. Default is to use standard output.
    #[structopt(short = "o", parse(from_os_str))]
    pub output_file: Option<std::path::PathBuf>,

    /// Error log file name. Default is to use standard error.
    #[structopt(short = "e", parse(from_os_str))]
    pub error_log: Option<std::path::PathBuf>,

    /// Temporary dir to use for unpacking the FMU. Default is to use system-wide directory, e.g.,
    /// C:\Temp or /tmp.
    #[structopt(short = "t", parse(from_os_str))]
    pub temp_dir: Option<std::path::PathBuf>,

    /// Separator to be used in CSV output.
    #[structopt(short = "c", default_value = ",")]
    pub separator: String,

    /// Print also left limit values at event points to the output file to investigate event
    /// behaviour. Default is to only print values after event handling.
    #[structopt(short = "d")]
    pub print_left_limit: bool,

    /// Print all variables to the output file. Default is to only print outputs.
    #[structopt(short = "f")]
    pub print_all_variables: bool,

    /// Mangle variable names to avoid quoting (needed for some CSV importing applications, but not
    /// according to the CrossCheck rules).
    #[structopt(short = "m")]
    pub mangle_names: bool,

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
    pub num_steps: u32,

    /// Simulation stop time, default is to use information from 'DefaultExperiment' as specified
    /// in the model description XML.
    #[structopt(short = "s")]
    pub stop_time: Option<f64>,

    /// Check the XML
    #[structopt(long = "xml")]
    pub check_xml: bool,

    /// Perform a ModelExchange simulation (implicitly enables --xml)
    #[structopt(long = "me")]
    pub sim_me: bool,

    /// Perform a CoSimulation simulation (implicitly enables --xml)
    #[structopt(long = "cs")]
    pub sim_cs: bool,

    /// The FMU model to read
    #[structopt(name = "model.fmu", parse(from_os_str))]
    pub model: std::path::PathBuf,
}
