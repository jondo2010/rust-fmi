use fmi::fmi3::{import::Fmi3Import, instance::InstanceME, Fmi3Model};

use crate::{
    sim::{params::SimParams, InputState, RecorderState, SimState, SimStateTrait},
    Error,
};

impl<'a> SimStateTrait<'a, InstanceME<'a>, Fmi3Import> for SimState<InstanceME<'a>> {
    fn new(
        import: &'a Fmi3Import,
        sim_params: SimParams,
        input_state: InputState<InstanceME<'a>>,
        recorder_state: RecorderState<InstanceME<'a>>,
    ) -> Result<Self, Error> {
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
