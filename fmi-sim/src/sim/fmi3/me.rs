use fmi::{
    EventFlags,
    fmi3::{Fmi3Model, import::Fmi3Import, instance::InstanceME},
};

use crate::{
    Error,
    sim::{InputState, RecorderState, SimState, SimStateTrait, params::SimParams},
};

impl SimStateTrait<InstanceME, Fmi3Import> for SimState<InstanceME> {
    fn new(
        import: &Fmi3Import,
        sim_params: SimParams,
        input_state: InputState<InstanceME>,
        recorder_state: RecorderState<InstanceME>,
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
