//! Model-Exchange simulation generic across FMI versions.

use fmi::traits::{FmiEventHandler, FmiInstance, FmiModelExchange};

use crate::Error;

use super::{
    SimState, SimStats,
    interpolation::Linear,
    solver::Solver,
    traits::{InstRecordValues, InstSetValues, SimHandleEvents, SimMe},
};

impl<Inst> SimMe<Inst> for SimState<Inst>
where
    Inst: FmiInstance + FmiModelExchange + InstSetValues + InstRecordValues + FmiEventHandler,
{
    fn main_loop<S>(&mut self, mut solver: S) -> Result<SimStats, Error>
    where
        S: Solver<Inst>,
    {
        let mut stats = SimStats::default();
        self.inst.enter_continuous_time_mode().map_err(Into::into)?;

        let mut time = self.sim_params.start_time;

        loop {
            self.inst.record_outputs(time, &mut self.recorder_state)?;

            if time >= self.sim_params.stop_time {
                break;
            }

            // calculate next time point
            let next_regular_point = self.sim_params.start_time
                + (stats.num_steps + 1) as f64 * self.sim_params.output_interval;
            let next_input_event_time = self.input_state.next_input_event(time);

            let input_event = next_regular_point >= next_input_event_time;
            let time_event = next_regular_point >= self.next_event_time();

            // Use the earliest of [next_input_event, next_event_time, and next_regular_point]
            let next_communication_point = if input_event || time_event {
                next_input_event_time.min(self.next_event_time())
            } else {
                next_regular_point
            };

            let (time_reached, state_event) =
                solver.step(&mut self.inst, next_communication_point)?;
            time = time_reached;

            self.inst.set_time(time).map_err(Into::into)?;

            self.input_state
                .apply_input::<Linear>(time, &mut self.inst, false, true, false)?;

            if time == next_regular_point {
                stats.num_steps += 1;
            }

            let mut step_event = false;
            let mut terminate = false;

            self.inst
                .completed_integrator_step(true, &mut step_event, &mut terminate)
                .map_err(Into::into)?;

            if terminate {
                log::info!("Termination requested by FMU");
                break;
            }

            if input_event || time_event || state_event || step_event {
                log::trace!(
                    "Event encountered at t = {time}. [Input: {input_event}, Time: {time_event}, State: {state_event}, Step: {step_event}]"
                );
                stats.num_events += 1;
                let (reset_solver, terminate) = self.handle_events(time, input_event)?;

                if terminate {
                    break;
                }

                self.inst.enter_continuous_time_mode().map_err(Into::into)?;

                if reset_solver {
                    solver.reset(&mut self.inst, time)?;
                }
            }
        }

        self.inst.terminate().map_err(Into::into)?;

        Ok(stats)
    }
}
