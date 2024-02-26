use anyhow::Context;
use arrow::record_batch::RecordBatch;
use fmi::fmi3::{
    import::Fmi3Import,
    instance::{Common, InstanceME, ModelExchange},
};

use crate::{
    options::CommonOptions,
    sim::{
        interpolation::Linear,
        params::SimParams,
        traits::{FmiSchemaBuilder, SimInput, SimOutput, SimTrait},
        InputState, OutputState, SimState,
    },
};

impl<'a> SimTrait<'a> for SimState<InstanceME<'a>> {
    type Import = Fmi3Import;
    type InputState = InputState<InstanceME<'a>>;
    type OutputState = OutputState<InstanceME<'a>>;

    fn new(
        import: &'a Self::Import,
        sim_params: SimParams,
        input_state: Self::InputState,
        output_state: Self::OutputState,
    ) -> anyhow::Result<Self> {
        let inst = import.instantiate_me("inst1", true, true)?;
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

    /// Main loop of the model-exchange simulation
    fn main_loop(&mut self) -> anyhow::Result<()> {
        self.inst
            .enter_continuous_time_mode()
            .ok()
            .context("enter_continuous_time_mode")?;

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

            // use `next_input_event` if it is earlier than `next_regular_point`
            let input_event = next_regular_point >= next_input_event_time;
            let time_event = next_regular_point >= self.next_event_time.unwrap_or(f64::INFINITY);

            let _next_communication_point = if input_event || time_event {
                next_input_event_time.min(self.next_event_time.unwrap())
            } else {
                next_regular_point
            };

            //CALL(settings->solverStep(solver, nextCommunicationPoint, &time, &stateEvent));
            //self.solver.setp(next_communication_point, &mut self.time)
            let state_event = false;

            self.inst.set_time(self.time).ok()?;

            self.input_state.apply_input::<Linear>(
                self.time,
                &mut self.inst,
                false,
                true,
                false,
            )?;

            if self.time == next_regular_point {
                num_steps += 1;
            }

            let (step_event, terminate) = self.inst.completed_integrator_step(true)?;

            if terminate {
                log::info!("Termination requested by FMU");
                break;
            }

            if input_event || time_event || state_event || step_event {
                let mut terminate = false;
                let reset_solver = self.handle_events(input_event, &mut terminate)?;

                if terminate {
                    break;
                }

                self.inst.enter_continuous_time_mode().ok()?;

                if reset_solver {
                    //self.solver.reset(self.time);
                }
            }
        }

        self.inst.terminate().ok().context("terminate")?;

        Ok(())
    }
}

/// Run a model-exchange simulation
pub fn model_exchange(import: &Fmi3Import, options: CommonOptions) -> anyhow::Result<RecordBatch> {
    let start_values = import.parse_start_values(&options.initial_values)?;

    let mut sim_state = SimState::<InstanceME>::new_from_options(import, &options, true, false)?;
    sim_state.initialize(start_values, options.initial_fmu_state_file)?;
    sim_state.main_loop()?;

    Ok(sim_state.output_state.finish())
}
