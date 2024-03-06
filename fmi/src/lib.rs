//! The `fmi` crate implements a Rust interface to FMUs (Functional Mockup Units) that follow FMI
//! Standard. This version of the library supports FMI 2.0 and 3.0. See http://www.fmi-standard.org/
//!
//! ## Examples
//!
//! ```rust,no_run,ignore
//! use fmi::{FmiImport as _, FmiInstance as _};
//! let import = fmi::Import::new("Modelica_Blocks_Sources_Sine.fmu")
//!     .unwrap()
//!     .as_fmi2()
//!     .unwrap();
//! assert_eq!(import.model_description().fmi_version, "2.0");
//! let me = import.instantiate_me("inst1", false, true).unwrap();
//! assert_eq!(me.get_version(), "2.0");
//! ```
#![doc = document_features::document_features!()]
#![deny(clippy::all)]

#[cfg(feature = "fmi2")]
pub mod fmi2;
#[cfg(feature = "fmi3")]
pub mod fmi3;
pub mod import;
pub mod traits;

pub mod built_info {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

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

    #[error("Unsupported Interface type: {0}")]
    UnsupportedInterface(String),

    #[error("FMI version of loaded API ({found}) doesn't match expected ({expected})")]
    FmiVersionMismatch { found: String, expected: String },

    #[error("FMU archive structure is not as expected: {0}")]
    ArchiveStructure(String),

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
