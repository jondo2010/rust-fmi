use anyhow::Context;
use fmi::{
    fmi3::{import::Fmi3Import, instance::InstanceCS, CoSimulation, Fmi3Model},
    traits::{FmiInstance, FmiStatus},
};

use crate::{
    sim::{
        interpolation::Linear,
        params::SimParams,
        traits::{InstRecordValues, SimHandleEvents},
        InputState, RecorderState, SimState, SimStateTrait, SimStats,
    },
    Error,
};

impl<'a> SimStateTrait<'a, InstanceCS<'a>, Fmi3Import> for SimState<InstanceCS<'a>> {
    fn new(
        import: &'a Fmi3Import,
        sim_params: SimParams,
        input_state: InputState<InstanceCS<'a>>,
        output_state: RecorderState<InstanceCS<'a>>,
    ) -> Result<Self, Error> {
        let inst = import.instantiate_cs(
            "inst1",
            true,
            true,
            sim_params.event_mode_used,
            sim_params.early_return_allowed,
            &[],
        )?;
        Ok(Self {
            sim_params,
            input_state,
            recorder_state: output_state,
            inst,
            next_event_time: None,
        })
    }
}

impl<'a> SimState<InstanceCS<'a>> {
    /// Main loop of the co-simulation
    pub fn main_loop(&mut self) -> Result<SimStats, Error> {
        let mut stats = SimStats::default();

        if self.sim_params.event_mode_used {
            self.inst.enter_step_mode().ok().map_err(fmi::Error::from)?;
        }

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
            // use `next_input_event` if it is earlier than `next_regular_point`
            let next_communication_point = next_input_event_time.min(next_regular_point);
            let input_event = next_regular_point > next_input_event_time;

            let step_size = next_communication_point - time;

            let mut event_encountered = false;
            let mut terminate_simulation = false;
            let mut early_return = false;
            let mut last_successful_time = 0.0;

            if self.sim_params.event_mode_used {
                self.input_state
                    .apply_input::<Linear>(time, &mut self.inst, false, true, false)?;
            } else {
                self.input_state
                    .apply_input::<Linear>(time, &mut self.inst, true, true, true)?;
            }

            self.inst
                .do_step(
                    time,
                    step_size,
                    true,
                    &mut event_encountered,
                    &mut terminate_simulation,
                    &mut early_return,
                    &mut last_successful_time,
                )
                .ok()
                .context("do_step")?;

            if early_return && !self.sim_params.early_return_allowed {
                panic!("Early return is not allowed.");
            }

            if terminate_simulation {
                break;
            }

            if early_return && last_successful_time < next_communication_point {
                time = last_successful_time;
            } else {
                time = next_communication_point;
            }

            if time == next_regular_point {
                stats.num_steps += 1;
            }

            if self.sim_params.event_mode_used && (input_event || event_encountered) {
                log::trace!("Event encountered at t = {time}");
                self.handle_events(time, input_event, &mut terminate_simulation)?;

                self.inst
                    .enter_step_mode()
                    .ok()
                    .context("enter_step_mode")?;
            }
        }

        self.inst.terminate().ok().context("terminate")?;

        stats.end_time = time;
        Ok(stats)
    }
}
