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
pub use self::import::Import;
pub use self::instance::{CoSimulation, Common, InstanceCS, InstanceME, ModelExchange};
pub use self::variable::{Value, Var};

use derive_more::Display;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FmiError {
    #[error("Error instantiating import")]
    Instantiation,

    #[error("Invalid fmi2Status {}", status)]
    InvalidStatus { status: u32 },

    /// For ME: It is recommended to perform a smaller step size and evaluate the model equations
    /// again, for example because an iterative solver in the model did not converge or because a
    /// function is outside of its domain (for example sqrt(<negative number>)). If this is not
    /// possible, the simulation has to be terminated.
    ///
    /// For CS: fmi2Discard is returned also if the slave is not able to return the required status
    /// information. The master has to decide if the simulation run can be continued.
    #[error("fmi2Discard")]
    FmiStatusDiscard,

    /// The FMU encountered an error.
    /// The simulation cannot be continued with this FMU instance.
    /// If one of the functions returns fmi2Error, it can be tried to restart the simulation from
    /// a formerly stored FMU state by calling fmi2SetFMUstate. This can be done if the capability
    /// flag canGetAndSetFMUstate is true and fmu2GetFMUstate was called before in non-erroneous
    /// state. If not, the simulation cannot be continued and fmi2FreeInstance or fmi2Reset must
    /// be called afterwards.
    #[error("fmi2Error")]
    FmiStatusError,

    /// The model computations are irreparably corrupted for all FMU instances.
    /// [For example, due to a run-time exception such as access violation or integer division by
    /// zero during the execution of an fmi function].
    /// It is not possible to call any other function for any of the FMU instances.
    #[error("fmi2Fatal")]
    FmiStatusFatal,

    #[error("Unknown variable: {}", name)]
    UnknownVariable { name: String },

    #[error("unknown toolchain version: {}", version)]
    UnknownToolchainVersion { version: String },

    #[error("Model type {} not supported by this FMU", .0)]
    UnsupportedFmuType(fmi::fmi2Type),

    #[error(
        "TypesPlatform of loaded API ({:?}) doesn't match expected ({:?})",
        found,
        fmi::fmi2TypesPlatform
    )]
    TypesPlatformMismatch { found: Box<[u8]> },

    #[error(
        "FMI version of loaded API ({:?}) doesn't match expected ({:?})",
        found,
        expected
    )]
    FmiVersionMismatch {
        found: Box<[u8]>,
        expected: Box<[u8]>,
    },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),

    #[error(transparent)]
    Xml(#[from] serde_xml_rs::Error),

    #[error(transparent)]
    Dlopen(#[from] dlopen::Error),

    #[error(transparent)]
    ModelDescr(#[from] model_descr::ModelDescriptionError),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
}

/// Ok Status returned by wrapped FMI functions.
#[derive(Debug, PartialEq, Display)]
pub enum FmiStatus {
    Ok,
    Warning,
    Pending,
}

/// Crate-wide Result type
pub type Result<T> = std::result::Result<T, FmiError>;

impl From<fmi::fmi2Status> for std::result::Result<FmiStatus, FmiError> {
    fn from(fmi_status: fmi::fmi2Status) -> Self {
        match fmi_status {
            fmi::fmi2Status::OK => Ok(FmiStatus::Ok),
            fmi::fmi2Status::Warning => Ok(FmiStatus::Warning),
            fmi::fmi2Status::Discard => Err(FmiError::FmiStatusDiscard),
            fmi::fmi2Status::Error => Err(FmiError::FmiStatusError),
            fmi::fmi2Status::Fatal => Err(FmiError::FmiStatusFatal),
            fmi::fmi2Status::Pending => Ok(FmiStatus::Pending),
        }
    }
}

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
