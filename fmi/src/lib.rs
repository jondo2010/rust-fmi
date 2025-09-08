//! The `fmi` crate implements a Rust interface to FMUs (Functional Mockup Units) that follow FMI
//! Standard. This version of the library supports FMI 2.0 and 3.0. See <http://www.fmi-standard.org/>
//!
//! ## Examples
//!
//! ### Loading an FMI 2.0 FMU
//!
//! ```rust,no_run
//! use fmi::{fmi2::import::Fmi2Import, import, traits::{FmiImport, FmiInstance}};
//!
//! // Load an FMU from a file path
//! let import: Fmi2Import = import::from_path("path/to/model.fmu").unwrap();
//! assert_eq!(import.model_description().fmi_version, "2.0");
//!
//! // Create a Model Exchange instance
//! let me = import.instantiate_me("inst1", false, true).unwrap();
//! assert_eq!(me.get_version(), "2.0");
//! ```
//!
//! ### Loading an FMI 3.0 FMU
//!
//! ```rust,no_run
//! use fmi::{fmi3::{import::Fmi3Import, Fmi3Model}, import, traits::{FmiImport, FmiInstance}};
//!
//! // Load an FMU from a file path
//! let import: Fmi3Import = import::from_path("path/to/model.fmu").unwrap();
//! assert_eq!(import.model_description().fmi_version, "3.0");
//!
//! // Create a Model Exchange instance
//! let me = import.instantiate_me("inst1", false, true).unwrap();
//! assert_eq!(me.get_version(), "3.0");
//! ```
//!
//! ### Checking FMU version before loading
//!
//! ```rust,no_run
//! use fmi::{import, schema::{MajorVersion, traits::FmiModelDescription}};
//!
//! // Peek at the FMU metadata without fully extracting it
//! let model_desc = import::peek_descr_path("path/to/model.fmu").unwrap();
//! let version = model_desc.major_version().unwrap();
//! match version {
//!     MajorVersion::FMI2 => {
//!         // Load as FMI 2.0
//!         let import: fmi::fmi2::import::Fmi2Import = import::from_path("path/to/model.fmu").unwrap();
//!         // ... use import
//!     }
//!     MajorVersion::FMI3 => {
//!         // Load as FMI 3.0
//!         let import: fmi::fmi3::import::Fmi3Import = import::from_path("path/to/model.fmu").unwrap();
//!         // ... use import
//!     }
//!     _ => panic!("Unsupported FMI version"),
//! }
//! ```
#![doc = document_features::document_features!()]
#![deny(clippy::all)]

// Re-export the fmi-schema crate
pub use fmi_schema as schema;

use schema::MajorVersion;

mod event_flags;
#[cfg(feature = "fmi2")]
pub mod fmi2;
#[cfg(feature = "fmi3")]
pub mod fmi3;
pub mod import;
pub mod traits;

pub use event_flags::EventFlags;

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

    #[error("Unsupported FMI version: {0:?}")]
    UnsupportedFmiVersion(MajorVersion),

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

#[derive(Debug)]
pub enum InterfaceType {
    ModelExchange,
    CoSimulation,
    ScheduledExecution,
}

/// Tag for Model Exchange
pub struct ME;

impl traits::InstanceTag for ME {
    const TYPE: InterfaceType = InterfaceType::ModelExchange;
}

/// Tag for Co-Simulation
pub struct CS;

impl traits::InstanceTag for CS {
    const TYPE: InterfaceType = InterfaceType::CoSimulation;
}

/// Tag for Scheduled Execution
pub struct SE;

impl traits::InstanceTag for SE {
    const TYPE: InterfaceType = InterfaceType::ScheduledExecution;
}
