use arrow::array::RecordBatch;

use fmi::{
    fmi3::{import::Fmi3Import, instance::Common},
    traits::{FmiInstance, FmiStatus},
};

use crate::{
    options::{CoSimulationOptions, ModelExchangeOptions},
    Error,
};

use super::{
    io::StartValues,
    traits::{FmiSim, InstanceSetValues},
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
                        .ok()
                        .map_err(fmi::Error::from)?;
                    for (vr, ary) in &start_values.structural_parameters {
                        //log::trace!("Setting structural parameter `{}`", (*vr).into());
                        self.inst.set_array(&[(*vr)], ary);
                    }
                    self.inst
                        .exit_configuration_mode()
                        .ok()
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
