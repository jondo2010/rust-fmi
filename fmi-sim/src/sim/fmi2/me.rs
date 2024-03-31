use arrow::record_batch::RecordBatch;
use fmi::{
    fmi2::{
        import::Fmi2Import,
        instance::{Common, InstanceME},
        Fmi2Error,
    },
    traits::{FmiImport, FmiInstance, FmiModelExchange, FmiStatus},
};

use crate::{
    options::ModelExchangeOptions,
    sim::{
        fmi2::Fmi2Sim,
        params::SimParams,
        solver::{self, Solver},
        traits::FmiSchemaBuilder,
        InputState, RecorderState, SimState, SimStats,
    },
    Error,
};

impl<'a> Fmi2Sim<'a, InstanceME<'a>> for SimState<InstanceME<'a>> {
    fn new(
        import: &'a Fmi2Import,
        sim_params: SimParams,
        input_state: InputState<InstanceME<'a>>,
        recorder_state: RecorderState<InstanceME<'a>>,
    ) -> Result<Self, fmi::Error> {
        log::trace!("Instantiating ME Simulation: {sim_params:#?}");
        let inst = import.instantiate_me("inst1", true, true)?;
        Ok(Self {
            sim_params,
            input_state,
            recorder_state,
            inst,
            next_event_time: None,
        })
    }

    fn default_initialize(&mut self) -> Result<(), Fmi2Error> {
        self.inst
            .setup_experiment(
                self.sim_params.tolerance,
                self.sim_params.start_time,
                Some(self.sim_params.stop_time),
            )
            .ok()?;
        self.inst.enter_initialization_mode().ok()?;
        self.inst.exit_initialization_mode().ok()?;

        let mut event_info = EventInfo::default();
        event_info.new_discrete_states_needed = 1;

        while event_info.new_discrete_states_needed > 0 {
            //self.inst.new_discrete_states(&mut event_info).ok()?;
        }

        self.next_event_time =
            (event_info.next_event_time_defined > 0).then(|| event_info.next_event_time);

        self.inst
            .enter_continuous_time_mode()
            .ok()
            .map_err(Into::into)?;

        Ok(())
    }
}

impl<'a> SimState<InstanceME<'a>> {
    /// Main loop of the model-exchange simulation
    fn main_loop<S>(&mut self, solver_params: S::Params) -> Result<SimStats, Fmi2Error>
    where
        S: Solver<InstanceME<'a>>,
    {
        let mut stats = SimStats::default();

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

        Ok(stats)
    }
}

pub fn model_exchange(
    import: &Fmi2Import,
    options: &ModelExchangeOptions,
    input_data: Option<RecordBatch>,
) -> Result<RecordBatch, Error> {
    let sim_params =
        SimParams::new_from_options(&options.common, import.model_description(), true, false);

    let start_values = import.parse_start_values(&options.common.initial_values)?;
    let input_state = InputState::new(import, input_data)?;
    let output_state = RecorderState::new(import, &sim_params);

    let mut sim_state = SimState::<InstanceME>::new(import, sim_params, input_state, output_state)?;
    sim_state
        .initialize(start_values, options.common.initial_fmu_state_file.as_ref())
        .map_err(fmi::Error::from)?;
    let stats = sim_state
        .main_loop::<solver::Euler>(())
        .map_err(fmi::Error::from)?;

    log::info!(
        "Simulation finished at t = {:.1} after {} steps.",
        stats.end_time,
        stats.num_steps
    );

    Ok(sim_state.recorder_state.finish())
}
