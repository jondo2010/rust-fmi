use arrow::record_batch::RecordBatch;

pub mod options;
pub mod sim;

pub fn simulate(args: options::FmiCheckOptions) -> anyhow::Result<RecordBatch> {
    let mini_descr = fmi::import::peek_descr_path(&args.model)?;
    let version = mini_descr.version()?;

    match version.major {
        #[cfg(feature = "fmi2")]
        2 => {
            let import: fmi::fmi2::import::Fmi2Import = fmi::import::from_path(&args.model)?;
            match args.action {
                #[cfg(feature = "me")]
                options::Interface::ME(options) => todo!(),
                #[cfg(feature = "cs")]
                options::Interface::CS(options) => todo!(),
            }
        }
        #[cfg(feature = "fmi3")]
        3 => {
            let import: fmi::fmi3::import::Fmi3Import = fmi::import::from_path(&args.model)?;
            match args.action {
                #[cfg(feature = "me")]
                options::Interface::ModelExchange { common } => {
                    sim::fmi3::model_exchange(&import, common)
                }
                #[cfg(feature = "cs")]
                options::Interface::CoSimulation {
                    common,
                    event_mode_used,
                    early_return_allowed,
                } => {
                    sim::fmi3::co_simulation(&import, common, event_mode_used, early_return_allowed)
                }
                #[cfg(feature = "se")]
                options::Interface::ScheduledExecution(options) => unimplemented!(),
            }
        }

        _ => anyhow::bail!("Unsupported FMI version: {version:?}"),
    }
}
