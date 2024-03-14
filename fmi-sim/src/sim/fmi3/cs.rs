use anyhow::Context;
use arrow::record_batch::RecordBatch;
use fmi::{
    fmi3::{
        import::Fmi3Import,
        instance::{CoSimulation, Common, InstanceCS},
    },
    traits::FmiImport,
};

use crate::{
    options::CoSimulationOptions,
    sim::{
        interpolation::Linear,
        params::SimParams,
        solver::DummySolver,
        traits::{FmiSchemaBuilder, InstanceRecordValues},
        InputState, RecorderState, SimState,
    },
    Error,
};

use super::Fmi3Sim;

impl<'a> SimState<InstanceCS<'a>, DummySolver> {
    pub fn new(
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
            _phantom: std::marker::PhantomData,
        })
    }

    /// Main loop of the co-simulation
    fn main_loop(&mut self) -> Result<(), Error> {
        if self.sim_params.event_mode_used {
            self.inst.enter_step_mode().ok().map_err(fmi::Error::from)?;
        }

        let mut num_steps = 0;
        let mut time = self.sim_params.start_time;

        loop {
            self.inst.record_outputs(time, &mut self.recorder_state)?;

            if time >= self.sim_params.stop_time {
                break;
            }

            // calculate next time point
            let next_regular_point = self.sim_params.start_time
                + (num_steps + 1) as f64 * self.sim_params.output_interval;
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
                num_steps += 1;
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

        log::info!("Simulation finished at t = {time:.1} after {num_steps} steps.");

        self.inst.terminate().ok().context("terminate")?;

        Ok(())
    }
}

/// Run a co-simulation simulation
pub fn co_simulation(
    import: &Fmi3Import,
    options: &CoSimulationOptions,
    input_data: Option<RecordBatch>,
) -> Result<RecordBatch, Error> {
    let sim_params = SimParams::new_from_options(
        &options.common,
        import.model_description(),
        options.event_mode_used,
        options.early_return_allowed,
    );

    let start_values = import.parse_start_values(&options.common.initial_values)?;
    let input_state = InputState::new(import, input_data)?;
    let output_state = RecorderState::new(import, &sim_params);

    let mut sim_state =
        SimState::<InstanceCS, DummySolver>::new(import, sim_params, input_state, output_state)?;
    sim_state.initialize(start_values, options.common.initial_fmu_state_file.as_ref())?;
    sim_state.main_loop()?;

    Ok(sim_state.recorder_state.finish())
}
