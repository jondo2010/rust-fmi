//! Model-Exchange simulation generic across FMI versions.

use fmi::traits::{FmiEventHandler, FmiInstance, FmiModelExchange, FmiStatus};

use crate::Error;

use super::{
    interpolation::Linear,
    solver::Solver,
    traits::{InstRecordValues, InstSetValues, SimHandleEvents, SimMe},
    SimState, SimStats,
};

impl<Inst> SimMe<Inst> for SimState<Inst>
where
    Inst: FmiInstance + FmiModelExchange + InstSetValues + InstRecordValues + FmiEventHandler,
{
    fn main_loop<S>(&mut self, solver_params: S::Params) -> Result<SimStats, Error>
    where
        S: Solver<Inst>,
    {
        let mut stats = SimStats::default();
        self.inst
            .enter_continuous_time_mode()
            .ok()
            .map_err(Into::into)?;

        let nx = self.inst.get_number_of_continuous_state_values();
        let nz = self.inst.get_number_of_event_indicator_values();

        let mut solver = <S as Solver<Inst>>::new(
            self.sim_params.start_time,
            self.sim_params.tolerance.unwrap_or_default(),
            nx,
            nz,
            solver_params,
        );

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
            let time_event = next_regular_point >= self.next_event_time.unwrap_or(f64::INFINITY);

            // Use the earliest of [next_input_event, next_event_time, and next_regular_point]
            let next_communication_point = if input_event || time_event {
                next_input_event_time.min(self.next_event_time.unwrap_or(f64::INFINITY))
            } else {
                next_regular_point
            };

            let (time_reached, state_event) =
                solver.step(&mut self.inst, next_communication_point)?;
            time = time_reached;

            self.inst.set_time(time).ok().map_err(Into::into)?;

            self.input_state
                .apply_input::<Linear>(time, &mut self.inst, false, true, false)?;

            if time == next_regular_point {
                stats.num_steps += 1;
            }

            let mut step_event = false;
            let mut terminate = false;

            self.inst
                .completed_integrator_step(true, &mut step_event, &mut terminate)
                .ok()
                .map_err(Into::into)?;

            if terminate {
                log::info!("Termination requested by FMU");
                break;
            }

            if input_event || time_event || state_event || step_event {
                log::trace!("Event encountered at t = {time}. [INPUT/TIME/STATE/STEP] = [{input_event}/{time_event}/{state_event}/{step_event}]");
                stats.num_events += 1;
                let mut terminate = false;
                let reset_solver = self.handle_events(time, input_event, &mut terminate)?;

                if terminate {
                    break;
                }

                self.inst
                    .enter_continuous_time_mode()
                    .ok()
                    .map_err(Into::into)?;

                if reset_solver {
                    solver.reset(&mut self.inst, time)?;
                }
            }
        }

        self.inst.terminate().ok().map_err(Into::into)?;

        Ok(stats)
    }
}
