//! FMI 3.0 API

pub mod import;
pub mod instance;
pub(crate) mod logger;
#[cfg(feature = "disabled")]
pub mod model;
// Re-export
pub use fmi_schema::fmi3 as schema;
pub use fmi_sys::fmi3 as binding;

#[derive(Debug)]
pub enum Fmi3Res {
    /// The call was successful. The output argument values are defined.
    OK,
    /// A non-critical problem was detected, but the computation may continue. The output argument
    /// values are defined. Function logMessage should be called by the FMU with further
    /// information before returning this status, respecting the current logging settings.
    Warning,
}

#[derive(Debug, thiserror::Error)]
pub enum Fmi3Error {
    /// The call was not successful and the FMU is in the same state as before the call. The output
    /// argument values are undefined, but the computation may continue. Function logMessage should
    /// be called by the FMU with further information before returning this status, respecting the
    /// current logging settings. Advanced importers may try alternative approaches to continue the
    /// simulation by calling the function with different arguments or calling another function -
    /// except in FMI for Scheduled Execution where repeating failed function calls is not allowed.
    /// Otherwise the simulation algorithm must treat this return code like [`Fmi3Error::Error`]
    /// and must terminate the simulation.
    ///
    /// [Examples for usage of `Discard` are handling of min/max violation, or signal numerical
    /// problems during model evaluation forcing smaller step sizes.]
    #[error("Discard")]
    Discard,
    /// The call failed. The output argument values are undefined and the simulation must not be
    /// continued. Function logMessage should be called by the FMU with further information before
    /// returning this status, respecting the current logging settings. If a function returns
    /// [`Fmi3Error::Error`], it is possible to restore a previously retrieved FMU state by calling
    /// [`set_fmu_state`]`. Otherwise [`FreeInstance`] or `Reset` must be called.  When detecting
    /// illegal arguments or a function call not allowed in the current state according to the
    /// respective state machine, the FMU must return fmi3Error. Other instances of this FMU are
    /// not affected by the error.
    #[error("Error")]
    Error,
    #[error("Fatal")]
    Fatal,
}

#[derive(Debug)]
pub struct Fmi3Status(binding::fmi3Status);

impl Fmi3Status {
    /// Convert to [`Result<Fmi3Res, Fmi3Err>`]
    #[inline]
    pub fn ok(self) -> Result<Fmi3Res, Fmi3Error> {
        self.into()
    }
}

impl From<binding::fmi3Status> for Fmi3Status {
    fn from(status: binding::fmi3Status) -> Self {
        Self(status)
    }
}

impl From<Fmi3Status> for Result<Fmi3Res, Fmi3Error> {
    fn from(Fmi3Status(status): Fmi3Status) -> Self {
        match status {
            binding::fmi3Status_fmi3OK => Ok(Fmi3Res::OK),
            binding::fmi3Status_fmi3Warning => Ok(Fmi3Res::Warning),
            binding::fmi3Status_fmi3Discard => Err(Fmi3Error::Discard),
            binding::fmi3Status_fmi3Error => Err(Fmi3Error::Error),
            binding::fmi3Status_fmi3Fatal => Err(Fmi3Error::Fatal),
            _ => unreachable!("Invalid status"),
        }
    }
}
