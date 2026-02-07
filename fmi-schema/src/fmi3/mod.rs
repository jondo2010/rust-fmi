//! FMI3.0 schema definitions
//!
//! This module contains the definitions of the FMI3.0 XML schema.

mod annotation;
mod build_description;
mod interface_type;
mod model_description;
mod r#type;
mod terminals_and_icons;
mod unit;
mod variable;
mod variable_dependency;

pub use annotation::{Annotation, Fmi3Annotations as Annotations};
pub use build_description::*;
pub use interface_type::*;
pub use model_description::*;
pub use r#type::*;
pub use terminals_and_icons::*;
pub use unit::*;
pub use variable::*;
pub use variable_dependency::*;

use crate::variable_counts::{Counts, VariableCounts};

impl crate::traits::DefaultExperiment for Fmi3ModelDescription {
    fn start_time(&self) -> Option<f64> {
        self.default_experiment
            .as_ref()
            .and_then(|de| de.start_time)
    }

    fn stop_time(&self) -> Option<f64> {
        self.default_experiment.as_ref().and_then(|de| de.stop_time)
    }

    fn tolerance(&self) -> Option<f64> {
        self.default_experiment.as_ref().and_then(|de| de.tolerance)
    }

    fn step_size(&self) -> Option<f64> {
        self.default_experiment.as_ref().and_then(|de| de.step_size)
    }
}

impl VariableCounts for ModelVariables {
    fn model_counts(&self) -> Counts {
        use variable::Variable;

        // Count variables by type
        let mut num_real_vars = 0;
        let mut num_bool_vars = 0;
        let mut num_integer_vars = 0;
        let mut num_string_vars = 0;

        for var in &self.variables {
            match var {
                Variable::Float32(_) | Variable::Float64(_) => num_real_vars += 1,
                Variable::Boolean(_) => num_bool_vars += 1,
                Variable::Int8(_)
                | Variable::UInt8(_)
                | Variable::Int16(_)
                | Variable::UInt16(_)
                | Variable::Int32(_)
                | Variable::UInt32(_)
                | Variable::Int64(_)
                | Variable::UInt64(_) => num_integer_vars += 1,
                Variable::String(_) => num_string_vars += 1,
                Variable::Binary(_) | Variable::Clock(_) => {}
            }
        }

        let cts = Counts {
            num_real_vars,
            num_bool_vars,
            num_integer_vars,
            num_string_vars,
            num_enum_vars: 0,
            ..Default::default()
        };

        // Count by variability and causality
        self.iter_abstract().fold(cts, |mut cts, var| {
            match var.variability() {
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
            match var.causality() {
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
        })
    }
}
