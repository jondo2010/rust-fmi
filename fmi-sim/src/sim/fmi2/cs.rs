use fmi::{
    fmi2::{
        import::Fmi2Import,
        instance::{CoSimulation, InstanceCS},
        Fmi2Error,
    },
    traits::{FmiInstance, FmiStatus},
};

use crate::{
    sim::{
        interpolation::Linear,
        io::StartValues,
        params::SimParams,
        traits::{InstRecordValues, InstSetValues, SimApplyStartValues},
        InputState, RecorderState, SimState, SimStateTrait, SimStats,
    },
    Error,
};

impl<'a> SimStateTrait<'a, InstanceCS<'a>, Fmi2Import> for SimState<InstanceCS<'a>> {
    fn new(
        import: &'a Fmi2Import,
        sim_params: SimParams,
        input_state: InputState<InstanceCS<'a>>,
        recorder_state: RecorderState<InstanceCS<'a>>,
    ) -> Result<Self, Error> {
        log::trace!("Instantiating CS Simulation: {sim_params:#?}");
        let inst = import.instantiate_cs("inst1", true, true)?;
        Ok(Self {
            sim_params,
            input_state,
            recorder_state,
            inst,
            next_event_time: None,
        })
    }
}

impl SimApplyStartValues<InstanceCS<'_>> for SimState<InstanceCS<'_>> {
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

impl<'a> SimState<InstanceCS<'a>> {
    /// Main loop of the co-simulation
    pub fn main_loop(&mut self) -> Result<SimStats, Fmi2Error> {
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
