use crate::fmi3::{
    Model, ModelGetSetStates, ModelState, UserModel,
    traits::{Context, ModelGetSet, ModelLoggingCategory},
};
use fmi::fmi3::{CoSimulation, Fmi3Error, Fmi3Res};

use super::ModelInstance;

impl<M, C> CoSimulation for ModelInstance<M, C>
where
    M: Model + UserModel + ModelGetSet<M> + ModelGetSetStates,
    C: Context<M>,
{
    fn enter_step_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK.into(),
            M::LoggingCategory::trace_category(),
            format_args!("enter_step_mode()"),
        );
        self.assert_instance_type(fmi::InterfaceType::CoSimulation)?;

        match self.state {
            ModelState::EventMode => {
                self.state = ModelState::StepMode;
                Ok(Fmi3Res::OK)
            }
            ModelState::StepMode => Ok(Fmi3Res::OK),
            _ => {
                self.context.log(
                    Fmi3Error::Error.into(),
                    M::LoggingCategory::default(),
                    format_args!("enter_step_mode() called in invalid state {:?}", self.state),
                );
                Err(Fmi3Error::Error)
            }
        }
    }

    fn get_output_derivatives(
        &mut self,
        vrs: &[fmi::fmi3::binding::fmi3ValueReference],
        orders: &[i32],
        _values: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Error::Error.into(),
            M::LoggingCategory::default(),
            format_args!(
                "get_output_derivatives(vrs: {:?}, orders: {:?}) not implemented",
                vrs, orders
            ),
        );
        Err(Fmi3Error::Error)
    }

    #[allow(clippy::too_many_arguments)]
    fn do_step(
        &mut self,
        current_communication_point: f64,
        communication_step_size: f64,
        no_set_fmu_state_prior_to_current_point: bool,
        event_handling_needed: &mut bool,
        terminate_simulation: &mut bool,
        early_return: &mut bool,
        last_successful_time: &mut f64,
    ) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK.into(),
            M::LoggingCategory::trace_category(),
            format_args!(
                "do_step(t: {}, h: {}, no_set_state_prior: {})",
                current_communication_point,
                communication_step_size,
                no_set_fmu_state_prior_to_current_point
            ),
        );
        self.assert_instance_type(fmi::InterfaceType::CoSimulation)?;

        match self.state {
            ModelState::StepMode => {}
            _ => {
                self.context.log(
                    Fmi3Error::Error.into(),
                    M::LoggingCategory::default(),
                    format_args!("do_step() called in invalid state {:?}", self.state),
                );
                return Err(Fmi3Error::Error);
            }
        }

        let result = self.model.do_step(
            &mut self.context,
            current_communication_point,
            communication_step_size,
            no_set_fmu_state_prior_to_current_point,
        )?;

        *event_handling_needed = result.event_handling_needed;
        *terminate_simulation = result.terminate_simulation;
        *early_return = self.context.early_return_allowed() && result.early_return;
        *last_successful_time = result.last_successful_time;

        // Mark values dirty so subsequent getters recompute at the new time
        self.is_dirty_values = true;

        // Remain in StepMode after a successful step
        Ok(Fmi3Res::OK)
    }
}
