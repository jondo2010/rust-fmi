use structopt::StructOpt;

use crate::sim::options::SimOptions;

#[derive(Debug, StructOpt)]
pub enum Action {
    /// Perform a ModelExchange simulation
    #[cfg(feature = "me")]
    ME(SimOptions),
    /// Perform a CoSimulation simulation
    #[cfg(feature = "cs")]
    CS(SimOptions),
    /// Perform a ScheduledExecution simulation
    #[cfg(feature = "se")]
    SE(SimOptions),
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
