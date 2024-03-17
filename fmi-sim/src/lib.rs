use arrow::array::RecordBatch;
use fmi::schema::{traits::FmiModelDescription, MajorVersion};

pub mod options;
pub mod sim;

/// Sim error
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    FmiError(#[from] fmi::Error),

    #[error(transparent)]
    SolverError(#[from] sim::solver::SolverError),

    #[error(transparent)]
    ArrowError(#[from] arrow::error::ArrowError),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Simulate an FMI model parameterized by the given top-level options.
pub fn simulate(options: &options::FmiSimOptions) -> Result<RecordBatch, Error> {
    let mini_descr = fmi::import::peek_descr_path(&options.model)?;
    let version = mini_descr.major_version().map_err(fmi::Error::from)?;

    log::debug!("Loaded {:?}", mini_descr);

    // Read optional input data
    let input_data = options
        .input_file
        .as_ref()
        .inspect(|p| log::debug!("Reading input data from {}", p.display()))
        .map(sim::util::read_csv)
        .transpose()?;

    match version {
        MajorVersion::FMI1 => Err(fmi::Error::UnsupportedFmiVersion(version).into()),

        #[cfg(feature = "fmi2")]
        MajorVersion::FMI2 => {
            let import: fmi::fmi2::import::Fmi2Import = fmi::import::from_path(&options.model)?;
            sim::simulate_with(input_data, &options.interface, import)
        }

        #[cfg(feature = "fmi3")]
        MajorVersion::FMI3 => {
            let import: fmi::fmi3::import::Fmi3Import = fmi::import::from_path(&options.model)?;
            sim::simulate_with(input_data, &options.interface, import)
        }
    }
}
