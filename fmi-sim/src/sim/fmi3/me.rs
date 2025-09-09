use fmi::{
    EventFlags,
    fmi3::{Fmi3Model, import::Fmi3Import, instance::InstanceME},
};

use crate::{
    Error,
    sim::{InputState, RecorderState, SimState, SimStateTrait, params::SimParams},
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
            event_flags: EventFlags::default(),
        })
    }
}
