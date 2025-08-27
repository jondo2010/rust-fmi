use arrow::array::RecordBatch;

use fmi::{
    fmi3::{Common, import::Fmi3Import},
    traits::{FmiImport, FmiInstance},
};

use crate::{
    Error,
    options::{CoSimulationOptions, ModelExchangeOptions},
    sim::{
        InputState, RecorderState, SimState, SimStateTrait,
        params::SimParams,
        traits::{ImportSchemaBuilder, SimInitialize},
    },
};

use super::{
    SimStats,
    io::StartValues,
    traits::{FmiSim, InstSetValues},
};

#[cfg(feature = "cs")]
mod cs;
mod io;
#[cfg(feature = "me")]
mod me;
mod schema;

macro_rules! impl_sim_apply_start_values {
    ($inst:ty) => {
        impl super::traits::SimApplyStartValues<$inst> for super::SimState<$inst> {
            fn apply_start_values(
                &mut self,
                start_values: &StartValues<<$inst as FmiInstance>::ValueRef>,
            ) -> Result<(), Error> {
                if !start_values.structural_parameters.is_empty() {
                    self.inst
                        .enter_configuration_mode()
                        .map_err(fmi::Error::from)?;
                    for (vr, ary) in &start_values.structural_parameters {
                        //log::trace!("Setting structural parameter `{}`", (*vr).into());
                        self.inst.set_array(&[(*vr)], ary);
                    }
                    self.inst
                        .exit_configuration_mode()
                        .map_err(fmi::Error::from)?;
                }

                start_values.variables.iter().for_each(|(vr, ary)| {
                    self.inst.set_array(&[*vr], ary);
                });

                Ok(())
            }
        }
    };
}

#[cfg(feature = "me")]
impl_sim_apply_start_values!(fmi::fmi3::instance::InstanceME<'_>);
#[cfg(feature = "cs")]
impl_sim_apply_start_values!(fmi::fmi3::instance::InstanceCS<'_>);

impl FmiSim for Fmi3Import {
    #[cfg(feature = "me")]
    fn simulate_me(
        &self,
        options: &ModelExchangeOptions,
        input_data: Option<RecordBatch>,
    ) -> Result<(RecordBatch, SimStats), Error> {
        use crate::sim::{solver, traits::SimMe};
        use fmi::fmi3::instance::InstanceME;

        let sim_params =
            SimParams::new_from_options(&options.common, self.model_description(), true, false);

        let start_values = self.parse_start_values(&options.common.initial_values)?;
        let input_state = InputState::new(self, input_data)?;
        let recorder_state = RecorderState::new(self, &sim_params);

        let mut sim_state =
            SimState::<InstanceME>::new(self, sim_params, input_state, recorder_state)?;
        log::debug!("Initializing with start values");
        sim_state.initialize(start_values, options.common.initial_fmu_state_file.as_ref())?;
        log::debug!("Starting main loop");
        let stats = sim_state.main_loop::<solver::Euler>(())?;

        Ok((sim_state.recorder_state.finish(), stats))
    }

    #[cfg(feature = "cs")]
    fn simulate_cs(
        &self,
        options: &CoSimulationOptions,
        input_data: Option<RecordBatch>,
    ) -> Result<(RecordBatch, SimStats), Error> {
        use fmi::fmi3::instance::InstanceCS;

        let sim_params = SimParams::new_from_options(
            &options.common,
            self.model_description(),
            options.event_mode_used,
            options.early_return_allowed,
        );

        let start_values = self.parse_start_values(&options.common.initial_values)?;
        let input_state = InputState::new(self, input_data)?;
        let output_state = RecorderState::new(self, &sim_params);

        let mut sim_state =
            SimState::<InstanceCS>::new(self, sim_params, input_state, output_state)?;
        sim_state.initialize(start_values, options.common.initial_fmu_state_file.as_ref())?;
        let stats = sim_state.main_loop()?;

        Ok((sim_state.recorder_state.finish(), stats))
    }
}
