use anyhow::Context;
use arrow::record_batch::RecordBatch;
use fmi::fmi3::{
    import::Fmi3Import,
    instance::{CoSimulation, Common, InstanceCS},
};

use crate::sim::{
    options::SimOptions,
    params::SimParams,
    traits::{FmiSchemaBuilder, SimInput, SimOutput, SimTrait},
    InputState, OutputState, SimState,
};

impl<'a> SimTrait<'a> for SimState<InstanceCS<'a>> {
    type Import = Fmi3Import;
    type InputState = InputState<InstanceCS<'a>>;
    type OutputState = OutputState<InstanceCS<'a>>;

    fn new(
        import: &'a Self::Import,
        sim_params: SimParams,
        input_state: Self::InputState,
        output_state: Self::OutputState,
    ) -> anyhow::Result<Self> {
        let inst = import.instantiate_cs(
            "inst1",
            true,
            true,
            sim_params.event_mode_used,
            sim_params.early_return_allowed,
            &[],
        )?;
        let time = sim_params.start_time;
        Ok(Self {
            sim_params,
            input_state,
            output_state,
            inst,
            time,
            next_event_time: None,
        })
    }

    /// Main loop of the co-simulation
    fn main_loop(&mut self) -> anyhow::Result<()> {
        if self.sim_params.event_mode_used {
            self.inst
                .enter_step_mode()
                .ok()
                .context("enter_step_mode")?;
        }

        let mut num_steps = 0;

        loop {
            self.output_state
                .record_outputs(self.time, &mut self.inst)?;

            if self.time >= self.sim_params.stop_time {
                break;
            }

            // calculate next time point
            let next_regular_point = self.sim_params.start_time
                + (num_steps + 1) as f64 * self.sim_params.output_interval;

            let next_input_event_time = self.input_state.next_input_event(self.time);

            let input_event = next_input_event_time <= next_regular_point;

            // use `next_input_event` if it is earlier than `next_regular_point`
            let next_communication_point = next_input_event_time.min(next_regular_point);

            let step_size = next_communication_point - self.time;

            let mut event_encountered = false;
            let mut terminate_simulation = false;
            let mut early_return = false;
            let mut last_successful_time = 0.0;

            if self.sim_params.event_mode_used {
                // self.input_state.unwrap()
                todo!();
            } else {
                // self.input_state.apply_inputs
            }

            self.inst
                .do_step(
                    self.time,
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
                anyhow::bail!("Early return is not allowed.");
            }

            if terminate_simulation {
                break;
            }

            if early_return && last_successful_time < next_communication_point {
                self.time = last_successful_time;
            } else {
                self.time = next_communication_point;
            }

            if self.time == next_regular_point {
                num_steps += 1;
            }

            if self.sim_params.event_mode_used && (input_event || event_encountered) {
                log::trace!("Event encountered at t = {}", self.time);
                self.handle_events(input_event, &mut terminate_simulation)?;

                self.inst
                    .enter_step_mode()
                    .ok()
                    .context("enter_step_mode")?;
            }
        }

        self.inst.terminate().ok().context("terminate")?;

        Ok(())
    }
}

/// Run a co-simulation simulation
pub fn co_simulation(import: &Fmi3Import, options: SimOptions) -> anyhow::Result<RecordBatch> {
    let start_values = import.parse_start_values(&options.initial_values)?;

    let mut sim_state = SimState::<InstanceCS>::new_from_options(import, &options)?;
    sim_state.initialize(start_values, options.initial_fmu_state_file)?;
    sim_state.main_loop()?;

    Ok(sim_state.output_state.finish())
}
