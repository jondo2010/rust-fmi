use arrow::array::RecordBatch;

pub mod options;
pub mod sim;

/// Sim error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    //return Err(anyhow::anyhow!("`output_interval` must be positive."))?;
    #[error(transparent)]
    FmiError(#[from] fmi::Error),

    #[error(transparent)]
    SolverError(#[from] sim::solver::SolverError),

    #[error(transparent)]
    ArrowError(#[from] arrow::error::ArrowError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub fn simulate(options: &options::FmiSimOptions) -> Result<RecordBatch, Error> {
    let mini_descr = fmi::import::peek_descr_path(&options.model)?;
    let version = mini_descr.version().map_err(fmi::Error::from)?;

    // Read optional input data
    let input_data = options
        .input_file
        .as_ref()
        .map(sim::util::read_csv)
        .transpose()?;

    let outputs = match version.major {
        #[cfg(feature = "fmi2")]
        2 => {
            let import: fmi::fmi2::import::Fmi2Import = fmi::import::from_path(&options.model)?;
            match &options.interface {
                #[cfg(feature = "me")]
                options::Interface::ME(options) => todo!(),
                #[cfg(feature = "cs")]
                options::Interface::CS(options) => todo!(),
            }
        }
        #[cfg(feature = "fmi3")]
        3 => {
            let import: fmi::fmi3::import::Fmi3Import = fmi::import::from_path(&options.model)?;

            match &options.interface {
                #[cfg(feature = "me")]
                options::Interface::ModelExchange(options) => {
                    sim::fmi3::model_exchange(&import, options, input_data)
                }
                #[cfg(feature = "cs")]
                options::Interface::CoSimulation(options) => {
                    sim::fmi3::co_simulation(&import, options, input_data)
                }
                #[cfg(feature = "se")]
                options::Interface::ScheduledExecution(options) => unimplemented!(),
            }
        }

        _ => Err(fmi::Error::UnsupportedFmiVersion(version.to_string()).into()),
    }?;

    Ok(outputs)
}
