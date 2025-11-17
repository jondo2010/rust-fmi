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

use super::{ModelInstance, context::BasicContext};

/// Extended context implementing Co-Simulation by wrapping Model-Exchange methods
struct WrapperContext<M: UserModel> {
    basic: BasicContext<M>,
    /// Internal step count
    num_steps: usize,
    /// Whether early return from a step is allowed.
    early_return_allowed: bool,
    /// Whether event mode is used.
    event_mode_used: bool,
    /// Next communication point for co-simulation.
    next_communication_point: f64,
    /// Event indicators' current values
    cur_z: Vec<f64>,
    /// Event indicators' last values
    pre_z: Vec<f64>,
    /// Current state vector
    x: Vec<f64>,
    /// Derivative of the state vector
    dx: Vec<f64>,
}

impl<M: Model + UserModel + ModelGetSetStates> WrapperContext<M> {
    pub fn new(
        logging_on: bool,
        log_message: LogMessageClosure,
        resource_path: PathBuf,
        early_return_allowed: bool,
    ) -> Self {
        Self {
            basic: BasicContext::new(logging_on, log_message, resource_path),
            num_steps: 0,
            early_return_allowed,
            event_mode_used: false,
            next_communication_point: 0.0,
            cur_z: vec![0.0; <M as Model>::MAX_EVENT_INDICATORS],
            pre_z: vec![0.0; <M as Model>::MAX_EVENT_INDICATORS],
            x: vec![0.0; <M as ModelGetSetStates>::NUM_STATES],
            dx: vec![0.0; <M as ModelGetSetStates>::NUM_STATES],
        }
    }
}

impl<M> Context<M> for WrapperContext<M>
where
    M: UserModel + 'static,
{
    fn logging_on(&self, category: <M as UserModel>::LoggingCategory) -> bool {
        self.basic.logging_on(category)
    }

    fn set_logging(&mut self, category: <M as UserModel>::LoggingCategory, enabled: bool) {
        self.basic.set_logging(category, enabled);
    }

    /// Log a message if the specified logging category is enabled.
    fn log(&self, status: Fmi3Status, category: M::LoggingCategory, args: std::fmt::Arguments<'_>) {
        self.basic.log(status, category, args);
    }

    /// Get the path to the resources directory.
    fn resource_path(&self) -> &PathBuf {
        &self.basic.resource_path()
    }

    fn initialize(&mut self, start_time: f64, stop_time: Option<f64>) {
        self.basic.initialize(start_time, stop_time);
    }

    fn time(&self) -> f64 {
        self.basic.time()
    }

    fn set_time(&mut self, time: f64) {
        self.basic.set_time(time);
    }

    fn stop_time(&self) -> Option<f64> {
        self.basic.stop_time()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl<M> CoSimulation for ModelInstance<M>
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
        vrs: &[fmi::fmi3::binding::fmi3ValueReference],
        orders: &[i32],
        values: &mut [f64],
    ) -> Result<Fmi3Res, Fmi3Error> {
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
    ) -> Result<Fmi3Res, Fmi3Error> {
        todo!()
    }
}

impl<M> CoSimulation for ModelInstance<M>
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
        vrs: &[fmi::fmi3::binding::fmi3ValueReference],
        orders: &[i32],
        values: &mut [f64],
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
