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

use crate::{
    variable_counts::{Counts, VariableCounts},
    Error,
};

impl FromStr for FmiModelDescription {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        yaserde::de::from_str(s).map_err(Error::XmlParse)
    }
}

impl VariableCounts for ModelVariables {
    fn model_counts(&self) -> Counts {
        let cts = Counts {
            num_real_vars: self.float32.len() + self.float64.len(),
            num_bool_vars: 0,
            num_integer_vars: self.int8.len()
                + self.uint8.len()
                + self.int16.len()
                + self.uint16.len()
                + self.int32.len()
                + self.uint32.len(),
            num_string_vars: 0,
            num_enum_vars: 0,
            ..Default::default()
        };

        let fl32 = self
            .float32
            .iter()
            .map(|sv| (sv.variability(), sv.causality()));
        let fl64 = self
            .float64
            .iter()
            .map(|sv| (sv.variability(), sv.causality()));
        let i8 = self
            .int8
            .iter()
            .map(|sv| (sv.variability(), sv.causality()));
        let u8 = self
            .uint8
            .iter()
            .map(|sv| (sv.variability(), sv.causality()));
        let i16 = self
            .int16
            .iter()
            .map(|sv| (sv.variability(), sv.causality()));
        let u16 = self
            .uint16
            .iter()
            .map(|sv| (sv.variability(), sv.causality()));
        let i32 = self
            .int32
            .iter()
            .map(|sv| (sv.variability(), sv.causality()));
        let u32 = self
            .uint32
            .iter()
            .map(|sv| (sv.variability(), sv.causality()));

        itertools::chain!(fl32, fl64, i8, u8, i16, u16, i32, u32).fold(
            cts,
            |mut cts, (variability, causality)| {
                match variability {
                    Variability::Constant => {
                        cts.num_constants += 1;
                    }
                    Variability::Continuous => {
                        cts.num_continuous += 1;
                    }
                    Variability::Discrete => {
                        cts.num_discrete += 1;
                    }
                    _ => {}
                }
                match causality {
                    Causality::CalculatedParameter => {
                        cts.num_calculated_parameters += 1;
                    }
                    Causality::Parameter => {
                        cts.num_parameters += 1;
                    }
                    Causality::Input => {
                        cts.num_inputs += 1;
                    }
                    Causality::Output => {
                        cts.num_outputs += 1;
                    }
                    Causality::Local => {
                        cts.num_local += 1;
                    }
                    Causality::Independent => {
                        cts.num_independent += 1;
                    }
                    _ => {}
                }
                cts
            },
        )
    }
}
