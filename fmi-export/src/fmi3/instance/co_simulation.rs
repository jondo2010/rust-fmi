use crate::fmi3::{Model, ModelGetSet, ModelGetSetStates};

use super::ModelInstance;
use fmi::fmi3::CoSimulation;

impl<M> CoSimulation for ModelInstance<M>
where
    M: Model + ModelGetSet<M> + ModelGetSetStates,
{
    fn enter_step_mode(&mut self) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
        todo!()
    }

    fn do_step(
        &mut self,
        current_communication_point: f64,
        communication_step_size: f64,
        no_set_fmu_state_prior_to_current_point: bool,
        event_handling_needed: &mut bool,
        terminate_simulation: &mut bool,
        early_return: &mut bool,
        last_successful_time: &mut f64,
    ) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
        todo!()
    }
}
