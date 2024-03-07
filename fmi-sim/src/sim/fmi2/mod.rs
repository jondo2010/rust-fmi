#[cfg(feature = "cs")]
mod cs;
mod io;
#[cfg(feature = "me")]
mod me;
mod schema;

use std::path::Path;

#[cfg(feature = "cs")]
pub use cs::co_simulation;
use fmi::{fmi2::instance::Common, traits::FmiInstance};
#[cfg(feature = "me")]
pub use me::model_exchange;

use super::{io::StartValues, solver::Solver, traits::FmiSchemaBuilder, SimState};

trait Fmi2Sim<Inst: FmiInstance> {
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
        input_event: bool,
        terminate_simulation: &mut bool,
    ) -> Result<bool, anyhow::Error>;
}

impl<Inst, S> Fmi2Sim<Inst> for SimState<Inst, S>
where
    Inst: Common,
    Inst::Import: FmiSchemaBuilder,
    S: Solver<Inst>,
{
    fn apply_start_values(
        &mut self,
        start_values: &StartValues<Inst::ValueReference>,
    ) -> anyhow::Result<()> {
        /*
        if !start_values.structural_parameters.is_empty() {
            self.inst.enter_configuration_mode().ok()?;
            for (vr, ary) in &start_values.structural_parameters {
                log::trace!("Setting structural parameter `{}`", (*vr).into());
                self.inst.set_array(&[(*vr)], &ary);
            }
            self.inst.exit_configuration_mode().ok()?;
        }
        */

        start_values.variables.iter().for_each(|(vr, ary)| {
            //self.inst.set_array(&[*vr], ary);
        });

        Ok(())
    }

    fn initialize<P: AsRef<std::path::Path>>(
        &mut self,
        start_values: StartValues<<Inst as fmi::traits::FmiInstance>::ValueReference>,
        initial_fmu_state_file: Option<P>,
    ) -> anyhow::Result<()> {
        todo!()
    }

    fn default_initialize(&mut self) -> anyhow::Result<()> {
        todo!()
    }

    fn handle_events(
        &mut self,
        input_event: bool,
        terminate_simulation: &mut bool,
    ) -> Result<bool, anyhow::Error> {
        todo!()
    }
}
