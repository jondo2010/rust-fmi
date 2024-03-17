use std::path::Path;

use anyhow::Context;
use arrow::array::RecordBatch;
use fmi::{
    fmi3::{import::Fmi3Import, instance::Common},
    traits::FmiInstance,
};

use crate::{
    options::{CoSimulationOptions, ModelExchangeOptions},
    Error,
};

use super::{
    interpolation::Linear,
    io::StartValues,
    solver::Solver,
    traits::{FmiSchemaBuilder, FmiSim, InstanceRecordValues, InstanceSetValues},
    SimState,
};

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

trait Fmi3Sim<Inst: FmiInstance> {
    fn apply_start_values(
        &mut self,
        start_values: &StartValues<Inst::ValueReference>,
    ) -> anyhow::Result<()>;

    fn initialize<P: AsRef<Path>>(
        &mut self,
        start_values: StartValues<Inst::ValueReference>,
        initial_fmu_state_file: Option<P>,
    ) -> anyhow::Result<()>;

    fn default_initialize(&mut self) -> anyhow::Result<()>;

    fn handle_events(
        &mut self,
        time: f64,
        input_event: bool,
        terminate_simulation: &mut bool,
    ) -> Result<bool, anyhow::Error>;
}

impl<Inst, S> Fmi3Sim<Inst> for SimState<Inst, S>
where
    Inst: Common + InstanceSetValues + InstanceRecordValues,
    Inst::Import: FmiSchemaBuilder,
    S: Solver<Inst>,
{
    fn apply_start_values(
        &mut self,
        start_values: &StartValues<Inst::ValueReference>,
    ) -> anyhow::Result<()> {
        if !start_values.structural_parameters.is_empty() {
            self.inst.enter_configuration_mode().ok()?;
            for (vr, ary) in &start_values.structural_parameters {
                log::trace!("Setting structural parameter `{}`", (*vr).into());
                self.inst.set_array(&[(*vr)], ary);
            }
            self.inst.exit_configuration_mode().ok()?;
        }

        start_values.variables.iter().for_each(|(vr, ary)| {
            self.inst.set_array(&[*vr], ary);
        });

        Ok(())
    }

    fn initialize<P>(
        &mut self,
        start_values: StartValues<Inst::ValueReference>,
        initial_fmu_state_file: Option<P>,
    ) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        if let Some(_initial_state_file) = &initial_fmu_state_file {
            unimplemented!("initial_fmu_state_file");
            // self.inst.restore_fmu_state_from_file(initial_state_file)?;
        }

        // set start values
        self.apply_start_values(&start_values)?;

        self.input_state.apply_input::<Linear>(
            self.sim_params.start_time,
            &mut self.inst,
            true,
            true,
            false,
        )?;

        // Default initialization
        if initial_fmu_state_file.is_none() {
            self.default_initialize()?;
        }

        Ok(())
    }

    fn default_initialize(&mut self) -> anyhow::Result<()> {
        self.inst
            .enter_initialization_mode(
                self.sim_params.tolerance,
                self.sim_params.start_time,
                Some(self.sim_params.stop_time),
            )
            .ok()
            .context("enter_initialization_mode")?;

        self.inst
            .exit_initialization_mode()
            .ok()
            .context("exit_initialization_mode")?;

        if self.sim_params.event_mode_used {
            // update discrete states
            let mut discrete_states_need_update = true;
            let mut nominals_of_continuous_states_changed = false;
            let mut values_of_continuous_states_changed = false;
            while discrete_states_need_update {
                let mut terminate_simulation = false;

                self.inst
                    .update_discrete_states(
                        &mut discrete_states_need_update,
                        &mut terminate_simulation,
                        &mut nominals_of_continuous_states_changed,
                        &mut values_of_continuous_states_changed,
                        &mut self.next_event_time,
                    )
                    .ok()
                    .context("update_discrete_states")?;

                if terminate_simulation {
                    self.inst.terminate().ok().context("terminate")?;
                    anyhow::bail!("update_discrete_states() requested termination.");
                }
            }
        }

        Ok(())
    }

    fn handle_events(
        &mut self,
        time: f64,
        input_event: bool,
        terminate_simulation: &mut bool,
    ) -> Result<bool, anyhow::Error> {
        self.inst.record_outputs(time, &mut self.recorder_state)?;
        self.inst
            .enter_event_mode()
            .ok()
            .context("enter_event_mode")?;
        if input_event {
            self.input_state
                .apply_input::<Linear>(time, &mut self.inst, true, true, true)?;
        }
        let mut reset_solver = false;
        let mut discrete_states_need_update = true;
        let mut nominals_of_continuous_states_changed = false;
        let mut values_of_continuous_states_changed = false;
        while discrete_states_need_update {
            self.inst
                .update_discrete_states(
                    &mut discrete_states_need_update,
                    terminate_simulation,
                    &mut nominals_of_continuous_states_changed,
                    &mut values_of_continuous_states_changed,
                    &mut self.next_event_time,
                )
                .ok()
                .context("update_discrete_states")?;

            if *terminate_simulation {
                break;
            }
            reset_solver |=
                nominals_of_continuous_states_changed || values_of_continuous_states_changed;
        }
        Ok(reset_solver)
    }
}

impl FmiSim for Fmi3Import {
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
