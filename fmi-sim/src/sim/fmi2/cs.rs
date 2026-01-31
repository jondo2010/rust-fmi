use fmi::{
    EventFlags,
    fmi2::{
        Fmi2Error,
        import::Fmi2Import,
        instance::{CoSimulation, InstanceCS},
    },
    traits::FmiInstance,
};

use crate::{
    Error,
    sim::{
        InputState, RecorderState, SimState, SimStateTrait, SimStats,
        interpolation::Linear,
        io::StartValues,
        params::SimParams,
        traits::{InstRecordValues, InstSetValues, SimApplyStartValues},
    },
};

impl SimStateTrait<InstanceCS, Fmi2Import> for SimState<InstanceCS> {
    fn new(
        import: &Fmi2Import,
        sim_params: SimParams,
        input_state: InputState<InstanceCS>,
        recorder_state: RecorderState<InstanceCS>,
    ) -> Result<Self, Error> {
        log::trace!("Instantiating CS Simulation: {sim_params:#?}");
        let inst = import.instantiate_cs("inst1", true, true)?;
        Ok(Self {
            sim_params,
            input_state,
            recorder_state,
            inst,
            event_flags: EventFlags::default(),
        })
    }
}

impl SimApplyStartValues<InstanceCS> for SimState<InstanceCS> {
    fn apply_start_values(
        &mut self,
        start_values: &StartValues<<InstanceCS as FmiInstance>::ValueRef>,
    ) -> Result<(), Error> {
        start_values.variables.iter().for_each(|(vr, ary)| {
            self.inst.set_array(&[*vr], ary);
        });
        Ok(())
    }
}

impl SimState<InstanceCS> {
    /// Main loop of the co-simulation
    pub fn main_loop(&mut self) -> Result<SimStats, Fmi2Error> {
        let mut stats = SimStats::default();

        loop {
            let time = self.sim_params.start_time
                + stats.num_steps as f64 * self.sim_params.output_interval;

            self.inst
                .record_outputs(time, &mut self.recorder_state)
                .expect("Failed to record outputs");

            self.input_state
                .apply_input::<Linear>(time, &mut self.inst, true, true, false)
                .expect("Failed to apply inputs");

            if time >= self.sim_params.stop_time {
                stats.end_time = time;
                break;
            }

            match self
                .inst
                .do_step(time, self.sim_params.output_interval, true)
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

        self.inst.terminate().map_err(Into::into)?;

        Ok(stats)
    }
}
