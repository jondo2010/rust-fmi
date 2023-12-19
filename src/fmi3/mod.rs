pub mod instance;
#[cfg(feature = "disabled")]
pub mod model;
pub mod binding {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    include!(concat!(env!("OUT_DIR"), "/fmi3_bindings.rs"));
}
pub mod import;
pub(crate) mod logger;
// Re-export
pub use fmi_schema::fmi3 as schema;

use crate::Error;

#[derive(Debug)]
#[repr(usize)]
pub enum FmiStatus {
    Ok = binding::fmi3Status_fmi3OK as _,
    Warning = binding::fmi3Status_fmi3Warning as _,
    Discard = binding::fmi3Status_fmi3Discard as _,
    Error = binding::fmi3Status_fmi3Error as _,
    Fatal = binding::fmi3Status_fmi3Fatal as _,
}

impl From<binding::fmi3Status> for FmiStatus {
    fn from(status: binding::fmi3Status) -> Self {
        match status {
            binding::fmi3Status_fmi3OK => FmiStatus::Ok,
            binding::fmi3Status_fmi3Warning => FmiStatus::Warning,
            binding::fmi3Status_fmi3Discard => FmiStatus::Discard,
            binding::fmi3Status_fmi3Error => FmiStatus::Error,
            binding::fmi3Status_fmi3Fatal => FmiStatus::Fatal,
            _ => unreachable!("Invalid status"),
        }
    }
}

impl From<FmiStatus> for Result<(), Error> {
    fn from(status: FmiStatus) -> Self {
        match status {
            FmiStatus::Ok => Ok(()),
            //FmiStatus::Warning => Err(crate::FmiError::FmiStatusWarning),
            FmiStatus::Discard => Err(crate::Error::FmiStatusDiscard),
            FmiStatus::Error => Err(crate::Error::FmiStatusError),
            FmiStatus::Fatal => Err(crate::Error::FmiStatusFatal),
            _ => unreachable!("Invalid status"),
        }
    }
}
