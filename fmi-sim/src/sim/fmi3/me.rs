use anyhow::Context;
use arrow::record_batch::RecordBatch;
use fmi::{
    fmi3::{import::Fmi3Import, instance::InstanceME},
    traits::{FmiImport, FmiInstance, FmiModelExchange, FmiStatus},
};

use crate::{
    options::ModelExchangeOptions,
    sim::{
        fmi3::Fmi3Sim,
        interpolation::Linear,
        params::SimParams,
        solver::{self, Solver},
        traits::{FmiSchemaBuilder, InstanceRecordValues, InstanceSetValues},
        InputState, RecorderState, SimState, SimStats,
    },
    Error,
};

trait FmiMeSim<Inst> {
    fn main_loop<S>(&mut self, solver_params: S::Params) -> Result<SimStats, Error>
    where
        S: Solver<Inst>;
}

impl<Inst> FmiMeSim<Inst> for SimState<Inst>
where
    Inst: FmiInstance + FmiModelExchange + InstanceSetValues + InstanceRecordValues,
    Inst::Import: FmiSchemaBuilder,
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

        let mut solver = S::new(
            self.sim_params.start_time,
            self.sim_params.tolerance.unwrap_or_default(),
            nx,
            nz,
            solver_params,
        );

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
            let input_event = next_regular_point >= next_input_event_time;
            let time_event = next_regular_point >= self.next_event_time.unwrap_or(f64::INFINITY);

            let next_communication_point = if input_event || time_event {
                next_input_event_time.min(self.next_event_time.unwrap())
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
                num_steps += 1;
            }

            let step_event = false;
            let terminate = false;

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

impl<'a> SimState<InstanceME<'a>> {
    fn new(
        import: &'a Fmi3Import,
        sim_params: SimParams,
        input_state: InputState<InstanceME<'a>>,
        recorder_state: RecorderState<InstanceME<'a>>,
    ) -> anyhow::Result<Self> {
        let inst = import.instantiate_me("inst1", true, true)?;
        Ok(Self {
            sim_params,
            input_state,
            recorder_state,
            inst,
            next_event_time: None,
        })
    }

    /// Main loop of the model-exchange simulation
    fn main_loop<S>(&mut self, solver_params: S::Params) -> Result<SimStats, Error>
    where
        S: Solver<InstanceME<'a>>,
    {
        let mut stats = SimStats::default();

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
            let input_event = next_regular_point >= next_input_event_time;
            let time_event = next_regular_point >= self.next_event_time.unwrap_or(f64::INFINITY);

            let next_communication_point = if input_event || time_event {
                next_input_event_time.min(self.next_event_time.unwrap())
            } else {
                next_regular_point
            };

            let (time_reached, state_event) =
                solver.step(&mut self.inst, next_communication_point)?;
            time = time_reached;

            self.inst.set_time(time).ok().map_err(fmi::Error::from)?;

            self.input_state
                .apply_input::<Linear>(time, &mut self.inst, false, true, false)?;

            if time == next_regular_point {
                num_steps += 1;
            }

            let mut step_event = false;
            let mut terminate = false;
            self.inst
                .completed_integrator_step(true, &mut step_event, &mut terminate)
                .ok()
                .map_err(fmi::Error::from)?;

            if terminate {
                log::info!("Termination requested by FMU");
                break;
            }

            if input_event || time_event || state_event || step_event {
                log::trace!("Event encountered at t = {time}. [INPUT/TIME/STATE/STEP] = [{input_event}/{time_event}/{state_event}/{step_event}]");
                let mut terminate = false;
                let reset_solver = self.handle_events(time, input_event, &mut terminate)?;

                if terminate {
                    break;
                }

                self.inst
                    .enter_continuous_time_mode()
                    .ok()
                    .map_err(fmi::Error::from)?;

                if reset_solver {
                    solver.reset(&mut self.inst, time)?;
                }
            }
        }

        self.inst.terminate().ok().context("terminate")?;

        Ok(stats)
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
    let recorder_state = RecorderState::new(import, &sim_params);

    let mut sim_state =
        SimState::<InstanceME>::new(import, sim_params, input_state, recorder_state)?;

    sim_state.initialize(start_values, options.common.initial_fmu_state_file.as_ref())?;
    let stats = sim_state.main_loop::<solver::Euler>(())?;

    Ok(sim_state.recorder_state.finish())
}
