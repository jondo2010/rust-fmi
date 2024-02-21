use std::path::PathBuf;

use anyhow::Context;
use arrow::record_batch::RecordBatch;
use fmi::{
    fmi3::{
        import::Fmi3Import,
        instance::{CoSimulation, Common, InstanceCS, InstanceME},
    },
    traits::FmiInstance,
};

use super::{
    input_state::StartValues, interpolation, options, params::SimParams, traits::SimTrait,
    InputState, OutputState,
};

struct SimState<Inst: FmiInstance> {
    sim_params: SimParams,
    input_state: InputState<Inst>,
    output_state: OutputState,
    inst: Inst,
    time: f64,
    next_event_time: Option<f64>,
}

impl<I: Common> SimState<I> {
    fn initialize(
        &mut self,
        start_values: StartValues,
        initial_fmu_state_file: Option<PathBuf>,
    ) -> anyhow::Result<()> {
        if let Some(_initial_state_file) = &initial_fmu_state_file {
            unimplemented!("initial_fmu_state_file");
            // self.inst.restore_fmu_state_from_file(initial_state_file)?;
        }

        // set start values
        self.input_state
            .apply_start_values(&mut self.inst, &start_values)?;

        self.input_state.apply_input::<interpolation::Linear>(
            self.sim_params.start_time,
            &mut self.inst,
            true,
            true,
            false,
        )?;

        // Default initialization
        if initial_fmu_state_file.is_none() {
            self.default_initialize()?;
        }

        Ok(())
    }

    fn default_initialize(&mut self) -> anyhow::Result<()> {
        self.inst
            .enter_initialization_mode(
                self.sim_params.tolerance,
                self.time,
                Some(self.sim_params.stop_time),
            )
            .ok()
            .context("enter_initialization_mode")?;

        self.inst
            .exit_initialization_mode()
            .ok()
            .context("exit_initialization_mode")?;

        if self.sim_params.event_mode_used {
            // update discrete states
            let mut discrete_states_need_update = true;
            let mut nominals_of_continuous_states_changed = false;
            let mut values_of_continuous_states_changed = false;
            while discrete_states_need_update {
                let mut terminate_simulation = false;

                self.inst
                    .update_discrete_states(
                        &mut discrete_states_need_update,
                        &mut terminate_simulation,
                        &mut nominals_of_continuous_states_changed,
                        &mut values_of_continuous_states_changed,
                        &mut self.next_event_time,
                    )
                    .ok()
                    .context("update_discrete_states")?;

                if terminate_simulation {
                    self.inst.terminate().ok().context("terminate")?;
                    anyhow::bail!("update_discrete_states() requested termination.");
                }
            }
        }

        Ok(())
    }

    fn handle_events(
        &mut self,
        input_event: bool,
        terminate_simulation: &mut bool,
    ) -> Result<bool, anyhow::Error> {
        self.output_state
            .record_variables(&mut self.inst, self.time)?;
        self.inst
            .enter_event_mode()
            .ok()
            .context("enter_event_mode")?;
        if input_event {
            self.input_state.apply_input::<interpolation::Linear>(
                self.time,
                &mut self.inst,
                true,
                true,
                true,
            )?;
        }
        let mut reset_solver = false;
        let mut discrete_states_need_update = true;
        let mut nominals_of_continuous_states_changed = false;
        let mut values_of_continuous_states_changed = false;
        while discrete_states_need_update {
            self.inst
                .update_discrete_states(
                    &mut discrete_states_need_update,
                    terminate_simulation,
                    &mut nominals_of_continuous_states_changed,
                    &mut values_of_continuous_states_changed,
                    &mut self.next_event_time,
                )
                .ok()
                .context("update_discrete_states")?;

            if *terminate_simulation {
                break;
            }
            reset_solver |=
                nominals_of_continuous_states_changed || values_of_continuous_states_changed;
        }
        Ok(reset_solver)
    }
}

impl<'a> SimTrait<'a> for SimState<InstanceCS<'a>> {
    type Import = Fmi3Import;
    type InputState = InputState<InstanceCS<'a>>;
    type OutputState = OutputState;

    fn new(
        import: &'a Fmi3Import,
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
                .record_variables(&mut self.inst, self.time)?;

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

impl<'a> SimTrait<'a> for SimState<InstanceME<'a>> {
    type Import = Fmi3Import;
    type InputState = InputState<InstanceME<'a>>;
    type OutputState = OutputState;

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

    fn main_loop(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}

pub fn co_simulation(
    import: &Fmi3Import,
    options: options::SimOptions,
) -> anyhow::Result<RecordBatch> {
    let start_values = StartValues::parse_start_values(import, &options.initial_values)?;
    let mut sim_state = SimState::<InstanceCS>::new_from_options(import, &options)?;

    sim_state.initialize(start_values, options.initial_fmu_state_file)?;
    sim_state.main_loop()?;

    Ok(sim_state.output_state.finish())
}
