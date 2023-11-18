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
pub use interface_type::{
    Fmi3CoSimulation, Fmi3InterfaceType, Fmi3ModelExchange, Fmi3ScheduledExecution,
};
pub use model_description::{
    fmi_model_description::{
        log_categories_type::CategoryType, ModelStructureType, ModelVariablesType,
        TypeDefinitionsType, UnitDefinitionsType,
    },
    FmiModelDescription,
};
pub use r#type::*;
pub use unit::Fmi3Unit;
pub use variable::*;
pub use variable_dependency::Fmi3Unknown;
