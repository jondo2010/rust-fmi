#[cfg(feature = "cs")]
mod cs;
mod io;
#[cfg(feature = "me")]
mod me;
mod schema;

use std::path::Path;

#[cfg(feature = "cs")]
pub use cs::co_simulation;
use fmi::{
    fmi2::{instance::Common, Fmi2Error},
    traits::FmiInstance,
};
#[cfg(feature = "me")]
pub use me::model_exchange;

use crate::Error;

use super::{
    interpolation::Linear,
    io::StartValues,
    solver::Solver,
    traits::{FmiSchemaBuilder, InstanceRecordValues, InstanceSetValues},
    SimState,
};

trait Fmi2Sim<Inst: FmiInstance> {
    fn apply_start_values(
        &mut self,
        start_values: &StartValues<Inst::ValueReference>,
    ) -> Result<(), Fmi2Error>;

    fn initialize<P: AsRef<Path>>(
        &mut self,
        start_values: StartValues<Inst::ValueReference>,
        initial_fmu_state_file: Option<P>,
    ) -> Result<(), Fmi2Error>;

    fn default_initialize(&mut self) -> Result<(), Fmi2Error>;

    fn handle_events(
        &mut self,
        input_event: bool,
        terminate_simulation: &mut bool,
    ) -> Result<bool, Error>;
}

impl<Inst, S> Fmi2Sim<Inst> for SimState<Inst, S>
where
    Inst: Common + InstanceSetValues + InstanceRecordValues,
    Inst::Import: FmiSchemaBuilder,
    S: Solver<Inst>,
{
    fn apply_start_values(
        &mut self,
        start_values: &StartValues<Inst::ValueReference>,
    ) -> Result<(), Fmi2Error> {
        #[cfg(feature = "disable")]
        if !start_values.structural_parameters.is_empty() {
            for (vr, ary) in &start_values.structural_parameters {
                log::trace!("Setting structural parameter `{}`", (*vr).into());
                self.inst.set_array(&[(*vr)], &ary);
            }
        }

        start_values.variables.iter().for_each(|(vr, ary)| {
            self.inst.set_array(&[*vr], ary);
        });

        Ok(())
    }

    fn initialize<P: AsRef<std::path::Path>>(
        &mut self,
        start_values: StartValues<<Inst as fmi::traits::FmiInstance>::ValueReference>,
        initial_fmu_state_file: Option<P>,
    ) -> Result<(), Fmi2Error> {
        if let Some(_initial_state_file) = &initial_fmu_state_file {
            unimplemented!("initial_fmu_state_file");
            // self.inst.restore_fmu_state_from_file(initial_state_file)?;
        }

        self.apply_start_values(&start_values)?;

        self.input_state
            .apply_input::<Linear>(
                self.sim_params.start_time,
                &mut self.inst,
                true,
                true,
                false,
            )
            .unwrap();

        // Default initialization
        if initial_fmu_state_file.is_none() {
            self.default_initialize()?;
        }

        Ok(())
    }

    fn default_initialize(&mut self) -> Result<(), Fmi2Error> {
        self.inst
            .setup_experiment(
                self.sim_params.tolerance,
                self.sim_params.start_time,
                Some(self.sim_params.stop_time),
            )
            .ok()?;
        self.inst.enter_initialization_mode().ok()?;
        self.inst.exit_initialization_mode().ok()?;

        Ok(())
    }

    fn handle_events(
        &mut self,
        input_event: bool,
        terminate_simulation: &mut bool,
    ) -> Result<bool, Error> {
        todo!()
    }
}
