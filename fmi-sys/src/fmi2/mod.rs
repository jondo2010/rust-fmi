#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(clippy::all)]
include!(concat!(env!("OUT_DIR"), "/fmi2_bindings.rs"));

pub mod logger;

impl Default for fmi2EventInfo {
    fn default() -> Self {
        fmi2EventInfo {
            newDiscreteStatesNeeded: 0,
            terminateSimulation: 0,
            nominalsOfContinuousStatesChanged: 0,
            valuesOfContinuousStatesChanged: 0,
            nextEventTimeDefined: 0,
            nextEventTime: 0.0,
        }
    }
}
