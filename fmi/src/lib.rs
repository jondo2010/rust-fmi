#![doc = include_str!("../README.md")]
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

//#[cfg(feature = "ls-bus")]
//pub mod ls_bus;

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

#[derive(Debug, Clone, Copy, PartialEq)]
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
