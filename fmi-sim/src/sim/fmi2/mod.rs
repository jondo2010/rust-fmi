#[cfg(feature = "cs")]
mod cs;
mod io;
#[cfg(feature = "me")]
mod me;
mod schema;

use std::path::Path;

use arrow::array::RecordBatch;
#[cfg(feature = "cs")]
pub use cs::co_simulation;
use fmi::{
    fmi2::{import::Fmi2Import, Fmi2Error},
    traits::FmiInstance,
};
#[cfg(feature = "me")]
pub use me::model_exchange;

use crate::{
    options::{CoSimulationOptions, ModelExchangeOptions},
    Error,
};

use super::{
    io::StartValues,
    params::SimParams,
    traits::{FmiSim, InstanceSetValues},
    InputState, RecorderState, SimStateTrait,
};

trait Fmi2Sim<'a, Inst: FmiInstance + InstanceSetValues>: SimStateTrait<Inst> {
    fn new(
        import: &'a Fmi2Import,
        sim_params: SimParams,
        input_state: InputState<Inst>,
        recorder_state: RecorderState<Inst>,
    ) -> Result<Self, fmi::Error>
    where
        Self: Sized;

    fn apply_start_values(
        &mut self,
        start_values: &StartValues<Inst::ValueReference>,
    ) -> Result<(), Fmi2Error> {
        start_values.variables.iter().for_each(|(vr, ary)| {
            self.inst().set_array(&[*vr], ary);
        });
        Ok(())
    }

    fn initialize<P: AsRef<Path>>(
        &mut self,
        start_values: StartValues<Inst::ValueReference>,
        initial_fmu_state_file: Option<P>,
    ) -> Result<(), Fmi2Error> {
        log::trace!("Initializing FMI model");

        if let Some(_initial_state_file) = &initial_fmu_state_file {
            unimplemented!("initial_fmu_state_file");
            // self.inst.restore_fmu_state_from_file(initial_state_file)?;
        }

        self.apply_start_values(&start_values)?;

        /*
        self.input_state
            .apply_input::<Linear>(
                self.sim_params.start_time,
                &mut self.inst,
                true,
                true,
                false,
            )
            .unwrap();
        */

        // Default initialization
        if initial_fmu_state_file.is_none() {
            self.default_initialize()?;
        }

        Ok(())
    }

    fn default_initialize(&mut self) -> Result<(), Fmi2Error>;
}

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
