//! FMI 2.0 API

pub mod import;
pub mod instance;
// Re-export
pub use fmi_schema::fmi2 as schema;
pub use fmi_sys::fmi2 as binding;

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
#[derive(Default, Debug, Copy, Clone)]
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

#[derive(Debug)]
pub enum Fmi2Res {
    /// All well
    OK,
    /// Things are not quite right, but the computation can continue. Function “logger” was called
    /// in the model (see below), and it is expected that this function has shown the prepared
    /// information message to the user.
    Warning,
    /// This status is returned only from the co-simulation interface, if the slave executes the
    /// function in an asynchronous way. That means the slave starts to compute but returns
    /// immediately.
    ///
    /// The master has to call [`instance::traits::CoSimulation::get_status`](...,
    /// fmi2DoStepStatus) to determine if the slave has finished the computation. Can be
    /// returned only by [`instance::traits::CoSimulation::do_step`]
    /// and by [`instance::traits::CoSimulation::get_status`].
    Pending,
}

#[derive(Debug, thiserror::Error)]
pub enum Fmi2Error {
    #[error("TypesPlatform of loaded API ({0}) doesn't match expected (default)")]
    TypesPlatformMismatch(String),

    /// For “model exchange”: It is recommended to perform a smaller step size and evaluate the
    /// model equations again, for example because an iterative solver in the model did not
    /// converge or because a function is outside of its domain (for example sqrt(<negative
    /// number>)). If this is not possible, the simulation has to be terminated.
    ///
    /// For “co-simulation”: [`Fmi2Err::Discard`] is returned also if the slave is not able to
    /// return the required status information. The master has to decide if the simulation run
    /// can be continued.
    ///
    /// In both cases, function “logger” was called in the FMU (see below) and it is expected that
    /// this function has shown the prepared information message to the user if the FMU was
    /// called in debug mode (loggingOn = true). Otherwise, “logger” should not show a message.
    #[error("Discard")]
    Discard,
    /// The FMU encountered an error. The simulation cannot be continued with this FMU instance. If
    /// one of the functions returns [`Fmi2Err::Error`], it can be tried to restart the
    /// simulation from a formerly stored FMU state by
    /// calling [`instance::traits::Common::set_fmu_state`]. This can be done if the capability
    /// flag `can_get_and_set_fmu_state` is true and
    /// [`instance::traits::Common::get_fmu_state`] was called before in non-erroneous state. If
    /// not, the simulation cannot be continued and [`instance::traits::Common::reset`] must be
    /// called afterwards.
    #[error("Error")]
    Error,
    /// The model computations are irreparably corrupted for all FMU instances.
    #[error("Fatal")]
    Fatal,
}

#[derive(Debug)]
pub struct Fmi2Status(binding::fmi2Status);

impl Fmi2Status {
    /// Convert to [`Result<Fmi2Res, Fmi2Error>`]
    #[inline]
    pub fn ok(self) -> Result<Fmi2Res, Fmi2Error> {
        self.into()
    }

    #[inline]
    pub fn is_error(&self) -> bool {
        self.0 == binding::fmi2Status_fmi2Error || self.0 == binding::fmi2Status_fmi2Fatal
    }
}

impl From<binding::fmi2Status> for Fmi2Status {
    fn from(status: binding::fmi2Status) -> Self {
        Self(status)
    }
}

impl From<Fmi2Status> for Result<Fmi2Res, Fmi2Error> {
    fn from(Fmi2Status(status): Fmi2Status) -> Self {
        match status {
            binding::fmi2Status_fmi2OK => Ok(Fmi2Res::OK),
            binding::fmi2Status_fmi2Warning => Ok(Fmi2Res::Warning),
            binding::fmi2Status_fmi2Discard => Err(Fmi2Error::Discard),
            binding::fmi2Status_fmi2Error => Err(Fmi2Error::Error),
            binding::fmi2Status_fmi2Fatal => Err(Fmi2Error::Fatal),
            _ => unreachable!("Invalid status"),
        }
    }
}
