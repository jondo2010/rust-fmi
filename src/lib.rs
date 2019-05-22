//! The `fmi` crate implements a Rust interface to FMUs (Functional Mockup Units) that follow FMI
//! Standard. This version of the library supports FMI2.0. See http://www.fmi-standard.org/
//!
//! # Examples
//!
//! ```
//! #[cfg(target_os = "linux")] {
//!     let import = fmi::Import::new(std::path::Path::new("data/Modelica_Blocks_Sources_Sine.fmu")).unwrap();
//!     let instance1 = fmi::InstanceME::new(&import, "inst1", false, true).unwrap();
//! }
//! ```

pub mod fmi;
pub mod import;
pub mod instance;
pub mod logger;
pub mod model_descr;
pub mod variable;

// Re-exports
pub use self::fmi::FmiApi;
pub use self::import::Import;
pub use self::instance::{CoSimulation, Common, InstanceCS, InstanceME, ModelExchange};
pub use self::variable::Var;

use failure::{Error, Fail};

/// Crate-wide Result type
pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Fail)]
pub enum FmiError {
    #[fail(display = "error instantiating import")]
    Instantiation,

    #[fail(display = "Invalid fmi2Status {}", status)]
    InvalidStatus { status: u32 },

    /// For ME: It is recommended to perform a smaller step size and evaluate the model equations
    /// again, for example because an iterative solver in the model did not converge or because a
    /// function is outside of its domain (for example sqrt(<negative number>)). If this is not
    /// possible, the simulation has to be terminated.
    ///
    /// For CS: fmi2Discard is returned also if the slave is not able to return the required status
    /// information. The master has to decide if the simulation run can be continued.
    #[fail(display = "fmi2Discard")]
    FmiStatusDiscard,

    /// The FMU encountered an error.
    /// The simulation cannot be continued with this FMU instance.
    /// If one of the functions returns fmi2Error, it can be tried to restart the simulation from
    /// a formerly stored FMU state by calling fmi2SetFMUstate. This can be done if the capability
    /// flag canGetAndSetFMUstate is true and fmu2GetFMUstate was called before in non-erroneous
    /// state. If not, the simulation cannot be continued and fmi2FreeInstance or fmi2Reset must
    /// be called afterwards.
    #[fail(display = "fmi2Error")]
    FmiStatusError,

    /// The model computations are irreparably corrupted for all FMU instances.
    /// [For example, due to a run-time exception such as access violation or integer division by
    /// zero during the execution of an fmi function].
    /// It is not possible to call any other function for any of the FMU instances.
    #[fail(display = "fmi2Fatal")]
    FmiStatusFatal,

    #[fail(display = "Unknown variable: {}", name)]
    UnknownVariable { name: String },

    #[fail(display = "unknown toolchain version: {}", version)]
    UnknownToolchainVersion { version: String },
}

/// Helper function to handle returned fmi2Status values
fn handle_status_u32(status: u32) -> Result<()> {
    use num_traits::cast::FromPrimitive;
    if let Some(status) = fmi::Status::from_u32(status) {
        match status {
            fmi::Status::OK => Ok(()),
            fmi::Status::Warning => Ok(()),
            fmi::Status::Discard => Err(FmiError::FmiStatusDiscard),
            fmi::Status::Error => Err(FmiError::FmiStatusError),
            fmi::Status::Fatal => Err(FmiError::FmiStatusFatal),
            fmi::Status::Pending => Ok(()),
        }
        .map_err(Error::from)
    } else {
        Err(FmiError::InvalidStatus { status })?
    }
}

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
