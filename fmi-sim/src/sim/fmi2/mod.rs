use arrow::array::RecordBatch;

use fmi::fmi2::import::Fmi2Import;

use crate::{
    Error,
    options::{CoSimulationOptions, ModelExchangeOptions},
    sim::{
        InputState, RecorderState, SimState, SimStateTrait,
        traits::{ImportSchemaBuilder, SimInitialize},
    },
};

use super::{SimStats, params::SimParams, traits::FmiSim};

#[cfg(feature = "cs")]
mod cs;
mod io;
#[cfg(feature = "me")]
mod me;
mod schema;

impl FmiSim for Fmi2Import {
    #[cfg(feature = "me")]
    fn simulate_me(
        &self,
        options: &ModelExchangeOptions,
        input_data: Option<RecordBatch>,
    ) -> Result<(RecordBatch, SimStats), Error> {
        use crate::sim::{solver, traits::SimMe};
        use fmi::{fmi2::instance::InstanceME, traits::FmiImport};

        let sim_params =
            SimParams::new_from_options(&options.common, self.model_description(), true, false);

        let start_values = self.parse_start_values(&options.common.initial_values)?;
        let input_state = InputState::new(self, input_data)?;
        let output_state = RecorderState::new(self, &sim_params);

        let mut sim_state =
            SimState::<InstanceME>::new(self, sim_params, input_state, output_state)?;
        sim_state.initialize(start_values, options.common.initial_fmu_state_file.as_ref())?;
        let stats = sim_state.main_loop::<solver::Euler>(())?;

        Ok((sim_state.recorder_state.finish(), stats))
    }

    #[cfg(feature = "cs")]
    fn simulate_cs(
        &self,
        options: &CoSimulationOptions,
        input_data: Option<RecordBatch>,
    ) -> Result<(RecordBatch, SimStats), Error> {
        use fmi::{fmi2::instance::InstanceCS, traits::FmiImport};

        let sim_params = SimParams::new_from_options(
            &options.common,
            self.model_description(),
            options.event_mode_used,
            options.early_return_allowed,
        );

        let start_values = self.parse_start_values(&options.common.initial_values)?;
        let input_state = InputState::new(self, input_data)?;
        let recorder_state = RecorderState::new(self, &sim_params);

        let mut sim_state =
            SimState::<InstanceCS>::new(self, sim_params, input_state, recorder_state)?;
        sim_state.initialize(start_values, options.common.initial_fmu_state_file.as_ref())?;
        let stats = sim_state.main_loop().map_err(fmi::Error::from)?;

        Ok((sim_state.recorder_state.finish(), stats))
    }
}
