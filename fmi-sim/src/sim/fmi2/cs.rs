use arrow::record_batch::RecordBatch;
use fmi::fmi2::instance::{CoSimulation, Common, InstanceCS};
use fmi::fmi2::Fmi2Error;
use fmi::{fmi2::import::Fmi2Import, traits::FmiImport};

use crate::sim::interpolation::Linear;
use crate::sim::traits::InstanceRecordValues;
use crate::sim::SimStats;
use crate::{
    options::CoSimulationOptions,
    sim::{
        params::SimParams, solver::DummySolver, traits::FmiSchemaBuilder, InputState,
        RecorderState, SimState,
    },
    Error,
};

use super::Fmi2Sim;

impl<'a> SimState<InstanceCS<'a>, DummySolver> {
    pub fn new(
        import: &'a Fmi2Import,
        sim_params: SimParams,
        input_state: InputState<InstanceCS<'a>>,
        recorder_state: RecorderState<InstanceCS<'a>>,
    ) -> Result<Self, Error> {
        let inst = import.instantiate_cs("inst1", true, true)?;
        Ok(Self {
            sim_params,
            input_state,
            recorder_state,
            inst,
            next_event_time: None,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Main loop of the co-simulation
    fn main_loop(&mut self) -> Result<SimStats, Fmi2Error> {
        let mut stats = SimStats::default();

        loop {
            let time = self.sim_params.start_time
                + stats.num_steps as f64 * self.sim_params.output_interval;

            self.inst
                .record_outputs(time, &mut self.recorder_state)
                .unwrap();
            self.input_state
                .apply_input::<Linear>(time, &mut self.inst, true, true, false)
                .unwrap();

            if time >= self.sim_params.stop_time {
                stats.end_time = time;
                break;
            }

            match self
                .inst
                .do_step(time, self.sim_params.output_interval, true)
                .ok()
            {
                Err(Fmi2Error::Discard) => {
                    if self.inst.terminated()? {
                        let time = self.inst.last_successful_time()?;

                        self.inst
                            .record_outputs(time, &mut self.recorder_state)
                            .unwrap();

                        stats.end_time = time;
                        break;
                    }
                }
                Err(e) => return Err(e),
                _ => {}
            }

            stats.num_steps += 1;
        }

        //TODO save final FMU state

        self.inst.terminate().ok()?;

        Ok(stats)
    }
}

/// Run a co-simulation simulation
pub fn co_simulation(
    import: &Fmi2Import,
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
    let recorder_state = RecorderState::new(import, &sim_params);

    let mut sim_state =
        SimState::<InstanceCS, DummySolver>::new(import, sim_params, input_state, recorder_state)?;
    sim_state
        .initialize(start_values, options.common.initial_fmu_state_file.as_ref())
        .map_err(fmi::Error::from)?;
    let stats = sim_state.main_loop().map_err(fmi::Error::from)?;

    log::info!(
        "Simulation finished at t = {:.1} after {} steps.",
        stats.end_time,
        stats.num_steps
    );

    Ok(sim_state.recorder_state.finish())
}
