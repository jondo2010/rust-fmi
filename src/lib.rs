//! The `fmi` crate implements a Rust interface to FMUs (Functional Mockup Units) that follow FMI
//! Standard. This version of the library supports FMI 2.0 and 3.0. See http://www.fmi-standard.org/
//!
//! # Examples
//!
//! ```
//! #[cfg(target_os = "linux")] {
//!     let import = fmi::Import::new(std::path::Path::new("data/Modelica_Blocks_Sources_Sine.fmu")).unwrap();
//!     let instance1 = fmi::fmi2::InstanceME::new(&import, "inst1", false, true).unwrap();
//! }
//! ```

#![deny(clippy::all)]

#[cfg(feature = "fmi2")]
pub mod fmi2;
#[cfg(feature = "fmi3")]
pub mod fmi3;
pub mod import;

// Re-exports
pub use self::import::Import;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error instantiating import")]
    Instantiation,

    #[error("Unknown variable: {}", name)]
    UnknownVariable { name: String },

    #[error("Model type {0} not supported by this FMU")]
    UnsupportedFmuType(String),

    #[error("Unsupported platform {os}/{arch}")]
    UnsupportedPlatform { os: String, arch: String },

    #[error("Unsupported FMI version: {0}")]
    UnsupportedFmiVersion(String),

    //TODO: Fix
    //#[error(
    //    "TypesPlatform of loaded API ({:?}) doesn't match expected ({:?})",
    //    found,
    //    fmi2::fmi2TypesPlatform
    //)]
    //TypesPlatformMismatch { found: Box<[u8]> },
    #[error("FMI version of loaded API ({found}) doesn't match expected ({expected})")]
    FmiVersionMismatch { found: String, expected: String },

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Zip(#[from] zip::result::ZipError),

    #[error(transparent)]
    Schema(#[from] fmi_schema::Error),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error(transparent)]
    LibLoading {
        #[from]
        source: libloading::Error,
    },

    #[cfg(feature = "fmi2")]
    #[error(transparent)]
    Fmi2Error(#[from] fmi2::Fmi2Error),

    #[cfg(feature = "fmi3")]
    #[error(transparent)]
    Fmi3Error(#[from] fmi3::Fmi3Error),
}

/// Ok Status returned by wrapped FMI functions.
//#[derive(Debug, PartialEq)]
// pub enum FmiStatus {
//    Ok,
//    Warning,
//    Pending,
//}

// impl From<fmi2::fmi2Status> for std::result::Result<FmiStatus, FmiError> {
// fn from(fmi_status: fmi2::fmi2Status) -> Self {
// match fmi_status {
// fmi2::fmi2Status::OK => Ok(FmiStatus::Ok),
// fmi2::fmi2Status::Warning => Ok(FmiStatus::Warning),
// fmi2::fmi2Status::Discard => Err(FmiError::FmiStatusDiscard),
// fmi2::fmi2Status::Error => Err(FmiError::FmiStatusError),
// fmi2::fmi2Status::Fatal => Err(FmiError::FmiStatusFatal),
// fmi2::fmi2Status::Pending => Ok(FmiStatus::Pending),
// }
// }
// }

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}
