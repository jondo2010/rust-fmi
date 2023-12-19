pub mod instance;
pub mod logger;
pub mod schema;

pub mod binding {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/fmi2_bindings.rs"));
}

pub mod import;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct CallbackFunctions {
    pub logger: binding::fmi2CallbackLogger,
    pub allocate_memory: binding::fmi2CallbackAllocateMemory,
    pub free_memory: binding::fmi2CallbackFreeMemory,
    pub step_finished: binding::fmi2StepFinished,
    pub component_environment: binding::fmi2ComponentEnvironment,
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EventInfo {
    pub new_discrete_states_needed: binding::fmi2Boolean,
    pub terminate_simulation: binding::fmi2Boolean,
    pub nominals_of_continuous_states_changed: binding::fmi2Boolean,
    pub values_of_continuous_states_changed: binding::fmi2Boolean,
    pub next_event_time_defined: binding::fmi2Boolean,
    pub next_event_time: binding::fmi2Real,
}

#[repr(C)]
#[derive(Debug)]
pub enum StatusKind {
    /// Can be called when the fmi2DoStep function returned fmi2Pending. The function delivers
    /// fmi2Pending if the computation is not finished. Otherwise the function returns the result
    /// of the asynchronously executed fmi2DoStep call
    DoStepStatus = binding::fmi2StatusKind_fmi2DoStepStatus as _,
    /// Can be called when the fmi2DoStep function returned fmi2Pending. The function delivers a
    /// string which informs about the status of the currently running asynchronous fmi2DoStep
    /// computation.
    PendingStatus = binding::fmi2StatusKind_fmi2PendingStatus as _,
    /// Returns the end time of the last successfully completed communication step. Can be called
    /// after fmi2DoStep(...) returned fmi2Discard.
    LastSuccessfulTime = binding::fmi2StatusKind_fmi2LastSuccessfulTime as _,
    /// Returns true, if the slave wants to terminate the simulation. Can be called after
    /// fmi2DoStep(...) returned fmi2Discard. Use fmi2LastSuccessfulTime to determine the time
    /// instant at which the slave terminated
    Terminated = binding::fmi2StatusKind_fmi2Terminated as _,
}

//impl From<StatusKind> for binding::fmi2StatusKind

/*
#[derive(Debug)]
#[repr(usize)]
pub enum FmiStatus {
    /// Pending is returned only from the co-simulation interface, if the slave executes the
    /// function in an asynchronous way. That means the slave starts to compute but returns
    /// immediately. The master has to call fmi2GetStatus(..., fmi2DoStepStatus) to determine, if
    /// the slave has finished the computation. Can be returned only by `do_step` and by
    /// `get_status`
    Pending = binding::fmi2Status_fmi2Pending as _,
}
*/

#[derive(Debug)]
pub struct FmiStatus(binding::fmi2Status);

#[derive(Debug)]
pub enum FmiRes {
    /// All well
    OK,
    /// things are not quite right, but the computation can continue. Function “logger” was called
    /// in the model (see below) and it is expected that this function has shown the prepared
    /// information message to the user
    Warning,
}

#[derive(Debug)]
pub enum FmiErr {
    Discard,
    Error,
    Fatal,
}

impl From<binding::fmi2Status> for FmiStatus {
    fn from(status: binding::fmi2Status) -> Self {
        Self(status)
    }
}

impl From<FmiStatus> for std::result::Result<FmiRes, FmiErr> {
    fn from(FmiStatus(status): FmiStatus) -> Self {
        match status {
            binding::fmi2Status_fmi2OK => Ok(FmiRes::OK),
            binding::fmi2Status_fmi2Warning => Ok(FmiRes::Warning),
            binding::fmi2Status_fmi2Discard => Err(FmiErr::Discard),
            binding::fmi2Status_fmi2Error => Err(FmiErr::Error),
            binding::fmi2Status_fmi2Fatal => Err(FmiErr::Fatal),
            _ => unreachable!("Invalid status"),
        }
    }
}
