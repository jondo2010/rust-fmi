//! FMI3.0 schema definitions
//!
//! This module contains the definitions of the FMI3.0 XML schema.

mod annotation;
mod attribute_groups;
mod interface_type;
mod model_description;
mod r#type;
mod unit;
mod variable;
mod variable_dependency;

use std::str::FromStr;

pub use annotation::Fmi3Annotations as Annotations;
pub use attribute_groups::*;
pub use interface_type::*;
pub use model_description::*;
pub use r#type::*;
pub use unit::*;
pub use variable::*;
pub use variable_dependency::*;

use crate::Error;

impl FromStr for FmiModelDescription {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        yaserde::de::from_str(s).map_err(|e| Error::XmlParse(e))
    }
}
