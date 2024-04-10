use arrow::array::RecordBatch;

use fmi::fmi2::import::Fmi2Import;

use crate::{
    options::{CoSimulationOptions, ModelExchangeOptions},
    Error,
};

use super::traits::FmiSim;

#[cfg(feature = "cs")]
mod cs;
mod io;
#[cfg(feature = "me")]
mod me;
mod schema;

#[cfg(feature = "cs")]
pub use cs::co_simulation;
#[cfg(feature = "me")]
pub use me::model_exchange;

impl FmiSim for Fmi2Import {
    fn simulate_me(
        &self,
        options: &ModelExchangeOptions,
        input_data: Option<RecordBatch>,
    ) -> Result<RecordBatch, Error> {
        model_exchange(self, options, input_data)
    }

    fn simulate_cs(
        &self,
        options: &CoSimulationOptions,
        input_data: Option<RecordBatch>,
    ) -> Result<RecordBatch, Error> {
        co_simulation(self, options, input_data)
    }
}
