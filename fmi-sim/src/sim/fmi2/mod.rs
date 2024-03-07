#[cfg(feature = "cs")]
mod cs;
mod io;
#[cfg(feature = "me")]
mod me;
mod schema;

#[cfg(feature = "cs")]
pub use cs::co_simulation;
use fmi::fmi2::instance::Common;
#[cfg(feature = "me")]
pub use me::model_exchange;

use super::{io::StartValues, solver::Solver, traits::FmiSchemaBuilder, SimState};

impl<Inst, S> SimState<Inst, S>
where
    Inst: Common,
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
                self.inst.set_array(&[(*vr)], &ary);
            }
            self.inst.exit_configuration_mode().ok()?;
        }

        start_values.variables.iter().for_each(|(vr, ary)| {
            self.inst.set_array(&[*vr], ary);
        });

        Ok(())
    }
}
