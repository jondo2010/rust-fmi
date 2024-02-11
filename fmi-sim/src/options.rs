use structopt::StructOpt;

use crate::sim::options::SimOptions;

#[derive(Debug, StructOpt)]
pub enum Action {
    /// Check the XML
    Check,
    /// Perform a ModelExchange simulation
    ME(SimOptions),
    /// Perform a CoSimulation simulation
    CS(SimOptions),
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
