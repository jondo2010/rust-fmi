use fmi::{
    fmi2::{import::Fmi2Import, instance::InstanceME},
    traits::FmiInstance,
};

use crate::{
    Error,
    sim::{
        InputState, RecorderState, SimState, SimStateTrait,
        io::StartValues,
        params::SimParams,
        traits::{InstSetValues, SimApplyStartValues},
    },
};

impl<'a> SimStateTrait<'a, InstanceME<'a>, Fmi2Import> for SimState<InstanceME<'a>> {
    fn new(
        import: &'a Fmi2Import,
        sim_params: SimParams,
        input_state: InputState<InstanceME<'a>>,
        recorder_state: RecorderState<InstanceME<'a>>,
    ) -> Result<Self, Error> {
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
}

impl SimApplyStartValues<InstanceME<'_>> for SimState<InstanceME<'_>> {
    fn apply_start_values(
        &mut self,
        start_values: &StartValues<<InstanceME as FmiInstance>::ValueRef>,
    ) -> Result<(), Error> {
        start_values.variables.iter().for_each(|(vr, ary)| {
            self.inst.set_array(&[*vr], ary);
        });
        Ok(())
    }
}
