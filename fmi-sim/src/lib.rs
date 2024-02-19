use arrow::record_batch::RecordBatch;
use fmi::Import;
use sim::options::SimOptions;

pub mod options;
pub mod sim;

#[cfg(feature = "me")]
fn model_exchange(&self, options: SimOptions) -> anyhow::Result<RecordBatch> {
    match self {
        #[cfg(feature = "fmi2")]
        Import::Fmi2(import) => todo!(),
        #[cfg(feature = "fmi3")]
        Import::Fmi3(import) => sim::fmi3_me::me_simulation(import, options),
    }
}

#[cfg(feature = "cs")]
fn co_simulation(import: fmi::Import, options: SimOptions) -> anyhow::Result<RecordBatch> {
    match import {
        #[cfg(feature = "fmi2")]
        Import::Fmi2(import) => todo!(),
        #[cfg(feature = "fmi3")]
        Import::Fmi3(import) => sim::fmi3_cs::co_simulation(&import, options),

        _ => anyhow::bail!("Unsupported FMI version"),
    }
}

pub fn simulate(args: options::FmiCheckOptions) -> anyhow::Result<RecordBatch> {
    let import = fmi::Import::from_path(&args.model)?;
    match args.action {
        #[cfg(feature = "me")]
        options::Action::ME(options) => import.model_exchange(options),

        #[cfg(feature = "cs")]
        options::Action::CS(options) => co_simulation(import, options),

        _ => anyhow::bail!("Unsupported action: {:?}", args.action),
    }
}
