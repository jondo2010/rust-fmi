use anyhow::Context;
use arrow::record_batch::RecordBatch;
use fmi::{
    fmi3::{
        import::Fmi3Import,
        instance::{Common, InstanceME, ModelExchange},
    },
    traits::{FmiImport, FmiInstance},
};

use crate::{
    options::ModelExchangeOptions,
    sim::{
        interpolation::Linear,
        params::SimParams,
        solver::{self, Solver},
        traits::FmiSchemaBuilder,
        InputState, OutputState, SimState,
    },
    Error,
};

impl<'a, S> SimState<InstanceME<'a>, S>
where
    S: Solver<InstanceME<'a>>,
{
    fn new(
        import: &'a Fmi3Import,
        sim_params: SimParams,
        input_state: InputState<InstanceME<'a>>,
        output_state: OutputState<InstanceME<'a>>,
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
            _phantom: std::marker::PhantomData,
        })
    }

    /// Main loop of the model-exchange simulation
    fn main_loop(&mut self, solver_params: S::Params) -> Result<(), Error> {
        self.inst
            .enter_continuous_time_mode()
            .ok()
            .map_err(fmi::Error::from)?;

        let nx = self.inst.get_number_of_continuous_state_values();
        let nz = self.inst.get_number_of_event_indicator_values();

        let mut solver = S::new(
            self.sim_params.start_time,
            self.sim_params.tolerance.unwrap_or_default(),
            nx,
            nz,
            solver_params,
        );

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

            let next_communication_point = if input_event || time_event {
                next_input_event_time.min(self.next_event_time.unwrap())
            } else {
                next_regular_point
            };

            let (time, state_event) = solver.step(&mut self.inst, next_communication_point)?;
            self.time = time;

            self.inst
                .set_time(self.time)
                .ok()
                .map_err(fmi::Error::from)?;

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

            let (step_event, terminate) = self
                .inst
                .completed_integrator_step(true)
                .map_err(fmi::Error::from)?;

            if terminate {
                log::info!("Termination requested by FMU");
                break;
            }

            if input_event || time_event || state_event || step_event {
                log::trace!("Event encountered at t = {}. [INPUT/TIME/STATE/STEP] = [{input_event}/{time_event}/{state_event}/{step_event}]", self.time);
                let mut terminate = false;
                let reset_solver = self.handle_events(input_event, &mut terminate)?;

                if terminate {
                    break;
                }

                self.inst
                    .enter_continuous_time_mode()
                    .ok()
                    .map_err(fmi::Error::from)?;

                if reset_solver {
                    solver.reset(&mut self.inst, self.time)?;
                }
            }
        }

        self.inst.terminate().ok().context("terminate")?;

        Ok(())
    }
}

/// Run a model-exchange simulation
pub fn model_exchange(
    import: &Fmi3Import,
    options: &ModelExchangeOptions,
    input_data: Option<RecordBatch>,
) -> Result<RecordBatch, Error> {
    let sim_params =
        SimParams::new_from_options(&options.common, import.model_description(), true, false);

    let start_values = import.parse_start_values(&options.common.initial_values)?;
    let input_state = InputState::new(import, input_data)?;
    let output_state = OutputState::new(import, &sim_params);

    let mut sim_state =
        SimState::<InstanceME, solver::Euler>::new(import, sim_params, input_state, output_state)?;

    sim_state.initialize(start_values, options.common.initial_fmu_state_file.as_ref())?;
    sim_state.main_loop(())?;

    Ok(sim_state.output_state.finish())
}
