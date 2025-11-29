//! Implementation of the Co-Simulation interface by wrapping Model-Exchange methods.
//!
//! This allows a Model-Exchange FMU to be used as a Co-Simulation FMU by delegating
//! Co-Simulation calls to the underlying Model-Exchange implementation.
//!
//!

use std::path::PathBuf;

use crate::fmi3::{
    Context, Model, ModelGetSet, ModelGetSetStates, ModelState, UserModel,
    instance::LogMessageClosure,
    traits::{ModelLoggingCategory, UserModelCS, UserModelCSWrapper, UserModelME},
};
use fmi::{
    EventFlags,
    fmi3::{CoSimulation, Fmi3Error, Fmi3Res, Fmi3Status},
};

use super::{ModelInstance, context::{BasicContext, WrapperContext}};

/// Co-Simulation implementation using embedded solver to wrap Model Exchange

// TODO: Co-Simulation implementation for direct CS (UserModelCS)
// This will be implemented when we have a model that directly implements UserModelCS
/*
impl<M> CoSimulation for ModelInstance<M, CSContext<M>>
where
    M: Model + ModelGetSet<M> + ModelGetSetStates + UserModel + UserModelCS + UserModelME + 'static,
{
    fn enter_step_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK.into(),
            M::LoggingCategory::trace_category(),
            format_args!("enter_step_mode()"),
        );
        self.assert_instance_type(fmi::InterfaceType::CoSimulation)?;
        match self.state {
            ModelState::InitializationMode | ModelState::EventMode => {
                self.state = ModelState::StepMode;
                Ok(Fmi3Res::OK)
            }
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
        _vrs: &[fmi::fmi3::binding::fmi3ValueReference],
        _orders: &[i32],
        _values: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        todo!()
    }

    fn do_step(
        &mut self,
        _current_communication_point: f64,
        _communication_step_size: f64,
        _no_set_fmu_state_prior_to_current_point: bool,
        _event_handling_needed: &mut bool,
        _terminate_simulation: &mut bool,
        _early_return: &mut bool,
        _last_successful_time: &mut f64,
    ) -> Result<Fmi3Res, Fmi3Error> {
        todo!()
    }
}
*/

/// Co-Simulation implementation using embedded solver to wrap Model Exchange
impl<M> CoSimulation for ModelInstance<M, WrapperContext<M>>
where
    M: Model
        + ModelGetSet<M>
        + ModelGetSetStates
        + UserModel
        + UserModelCSWrapper
        + UserModelME
        + 'static,
{
    fn enter_step_mode(&mut self) -> Result<Fmi3Res, Fmi3Error> {
        todo!()
    }

    fn do_step(
        &mut self,
        current_communication_point: f64,
        communication_step_size: f64,
        _no_set_fmu_state_prior_to_current_point: bool,
        event_handling_needed: &mut bool,
        terminate_simulation: &mut bool,
        early_return: &mut bool,
        last_successful_time: &mut f64,
    ) -> Result<Fmi3Res, Fmi3Error> {
        self.context.log(
            Fmi3Res::OK.into(),
            M::LoggingCategory::trace_category(),
            format_args!("do_step({current_communication_point}, {communication_step_size}, ..)"),
        );
        self.assert_instance_type(fmi::InterfaceType::CoSimulation)?;

        let context = {
            if let Some(context) = self
                .context
                .as_any_mut()
                .downcast_mut::<WrapperContext<M>>()
            {
                context
            } else {
                self.context.log(
                    Fmi3Error::Error.into(),
                    M::LoggingCategory::error_category(),
                    format_args!("Context is not of type MEWrapperContext"),
                );
                return Err(Fmi3Error::Error);
            }
        };

        if is_close(
            context.next_communication_point,
            current_communication_point,
        ) {
            context.log(
                Fmi3Error::Error.into(),
                M::LoggingCategory::error_category(),
                format_args!(
                    "Expected currentCommunicationPoint = {:.16} but was {:.16}.",
                    context.next_communication_point, current_communication_point
                ),
            );
            return Err(Fmi3Error::Error);
        }

        if communication_step_size <= 0.0 {
            self.context.log(
                Fmi3Error::Error.into(),
                M::LoggingCategory::error_category(),
                format_args!(
                    "Communication step size must be > 0 but was {communication_step_size}."
                ),
            );
            return Err(Fmi3Error::Error);
        }

        let next_communication_point = current_communication_point + communication_step_size;

        if let Some(stop_time) = context.stop_time()
            && next_communication_point > stop_time
            && is_close(next_communication_point, stop_time)
        {
            self.context.log(
                Fmi3Error::Error.into(),
                M::LoggingCategory::error_category(),
                format_args!(
                    "At communication point {current_communication_point:.16} a step size of \
                     {communication_step_size:.16} was requested but stop time is {stop_time:.16}."
                ),
            );
            return Err(Fmi3Error::Error);
        }

        *event_handling_needed = false;
        *terminate_simulation = false;
        *early_return = false;

        let mut event_flags = EventFlags::default();
        let mut next_communication_point_reached;

        loop {
            let next_solver_step_time = context.time() + M::FIXED_SOLVER_STEP;

            next_communication_point_reached = next_solver_step_time > next_communication_point
                && !is_close(next_solver_step_time, next_communication_point);

            if next_communication_point_reached
                || (*event_handling_needed && context.early_return_allowed)
            {
                break;
            }

            if *event_handling_needed {
                self.model.event_update(context, &mut event_flags)?;
            }

            let (state_event, time_event) = do_fixed_step(&mut self.model, context)?;

            if state_event || time_event {
                if context.event_mode_used {
                    *event_handling_needed = true
                } else {
                    self.model.event_update(context, &mut event_flags)?;
                    if M::MAX_EVENT_INDICATORS > 0 {
                        let _ret = self
                            .model
                            .get_event_indicators(&context.basic, &mut context.pre_z)?;
                    }
                }

                if context.early_return_allowed {
                    break;
                }
            }

            if event_flags.terminate_simulation {
                break;
            }
        }

        *terminate_simulation = event_flags.terminate_simulation;
        *early_return = context.early_return_allowed && !next_communication_point_reached;
        *last_successful_time = context.time();

        context.next_communication_point = if next_communication_point_reached {
            current_communication_point + communication_step_size
        } else {
            context.time()
        };

        Ok(Fmi3Res::OK)
    }

    fn get_output_derivatives(
        &mut self,
        _vrs: &[fmi::fmi3::binding::fmi3ValueReference],
        _orders: &[i32],
        _values: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
        todo!()
    }
}

fn is_close(a: f64, b: f64) -> bool {
    (a - b).abs() < f64::EPSILON
}

fn do_fixed_step<M>(
    model: &mut M,
    context: &mut WrapperContext<M>,
) -> Result<(bool, bool), Fmi3Error>
where
    M: Model + UserModel + UserModelCSWrapper + ModelGetSetStates + UserModelME + 'static,
{
    model.get_continuous_states(&mut context.x)?;
    model.get_continuous_state_derivatives(&mut context.dx)?;

    // forward Euler integration
    for (x_i, dx_i) in context.x.iter_mut().zip(context.dx.iter()) {
        *x_i += dx_i * M::FIXED_SOLVER_STEP;
    }

    model.set_continuous_states(&context.x)?;

    context.num_steps += 1;
    //context.set_time(context.start_time() + context.num_steps as f64 * M::FIXED_SOLVER_STEP);

    model.get_event_indicators(&context.basic, &mut context.cur_z)?;

    // check for zero-crossings
    let state_event = context
        .pre_z
        .iter()
        .zip(context.cur_z.iter())
        .any(|(&pre, &cur)| (pre <= 0.0 && cur > 0.0) || (pre > 0.0 && cur <= 0.0));

    // update previous event indicators
    context.pre_z.copy_from_slice(&context.cur_z);

    let time_event = false;

    // optional intermediate update
    //TODO

    Ok((state_event, time_event))
}
