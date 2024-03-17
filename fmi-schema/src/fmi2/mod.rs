//! FMI2.0 schema definitions
//!
//! This module contains the definitions of the FMI2.0 XML schema.

mod attribute_groups;
mod interface_type;
mod model_description;
mod scalar_variable;
mod r#type;
mod unit;
mod variable_dependency;

use std::str::FromStr;

pub use attribute_groups::*;
pub use interface_type::*;
pub use model_description::*;
pub use r#type::*;
pub use scalar_variable::*;
pub use unit::*;
pub use variable_dependency::*;

use crate::{
    variable_counts::{Counts, VariableCounts},
    Error,
};

pub type ScalarVariableMap<'a> = std::collections::HashMap<String, &'a ScalarVariable>;
pub type UnknownsTuple<'a> = (&'a ScalarVariable, Vec<&'a ScalarVariable>);

impl FromStr for Fmi2ModelDescription {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        yaserde::de::from_str(s).map_err(Error::XmlParse)
    }
}

impl crate::traits::DefaultExperiment for Fmi2ModelDescription {
    fn start_time(&self) -> Option<f64> {
        self.default_experiment.as_ref().map(|de| de.start_time)
    }

    fn stop_time(&self) -> Option<f64> {
        self.default_experiment.as_ref().map(|de| de.stop_time)
    }

    fn tolerance(&self) -> Option<f64> {
        self.default_experiment.as_ref().map(|de| de.tolerance)
    }

    fn step_size(&self) -> Option<f64> {
        None
    }
}

impl VariableCounts for ModelVariables {
    fn model_counts(&self) -> Counts {
        self.variables
            .iter()
            .fold(Counts::default(), |mut cts, sv| {
                match sv.variability {
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
                match sv.causality {
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
                match sv.elem {
                    ScalarVariableElement::Real { .. } => {
                        cts.num_real_vars += 1;
                    }
                    ScalarVariableElement::Integer { .. } => {
                        cts.num_integer_vars += 1;
                    }
                    ScalarVariableElement::Enumeration { .. } => {
                        cts.num_enum_vars += 1;
                    }
                    ScalarVariableElement::Boolean { .. } => {
                        cts.num_bool_vars += 1;
                    }
                    ScalarVariableElement::String { .. } => {
                        cts.num_string_vars += 1;
                    }
                }
                cts
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_experiment() {
        let s = r##"<DefaultExperiment stopTime="3.0" tolerance="0.0001"/>"##;
        let x: DefaultExperiment = yaserde::de::from_str(s).unwrap();
        assert_eq!(x.start_time, 0.0);
        assert_eq!(x.stop_time, 3.0);
        assert_eq!(x.tolerance, 0.0001);

        let s = r#"<DefaultExperiment startTime = "0.20000000000000000e+00" stopTime = "1.50000000000000000e+00" tolerance = "0.0001"/>"#;
        let x: DefaultExperiment = yaserde::de::from_str(s).unwrap();
        assert_eq!(x.start_time, 0.2);
        assert_eq!(x.stop_time, 1.5);
        assert_eq!(x.tolerance, 0.0001);
    }

    #[test]
    fn test_model_variables() {
        let s = r##"
            <ModelVariables>
                <ScalarVariable name="x[1]" valueReference="0" initial="exact"> <Real/> </ScalarVariable> <!-- idex="5" -->
                <ScalarVariable name="x[2]" valueReference="1" initial="exact"> <Real/> </ScalarVariable> <!-- index="6" -->
                <ScalarVariable name="der(x[1])" valueReference="2"> <Real derivative="5"/> </ScalarVariable> <!-- index="7" -->
                <ScalarVariable name="der(x[2])" valueReference="3"> <Real derivative="6"/> </ScalarVariable> <!-- index="8" -->
            </ModelVariables>
        "##;
        let x: ModelVariables = yaserde::de::from_str(s).unwrap();
        assert_eq!(x.variables.len(), 4);
        assert!(x
            .variables
            .iter()
            .map(|v| &v.name)
            .zip(["x[1]", "x[2]", "der(x[1])", "der(x[2])"].iter())
            .all(|(a, b)| a == b));
    }

    #[test]
    fn test_model_structure() {
        let s = r##"
            <ModelStructure>
                <Outputs>
                    <Unknown index="3" />
                    <Unknown index="4" />
                </Outputs>
                <Derivatives>
                    <Unknown index="7" />
                    <Unknown index="8" />
                </Derivatives>
                <InitialUnknowns>
                    <Unknown index="3" />
                    <Unknown index="4" />
                    <Unknown index="7" dependencies="5 2" />
                    <Unknown index="8" dependencies="5 6" />
                </InitialUnknowns>
            </ModelStructure>
        "##;
        let ms: ModelStructure = yaserde::de::from_str(s).unwrap();
        assert_eq!(ms.outputs.unknowns.len(), 2);
        assert_eq!(ms.outputs.unknowns[0].index, 3);
        assert_eq!(ms.outputs.unknowns[1].index, 4);
        assert_eq!(ms.derivatives.unknowns.len(), 2);
        assert_eq!(ms.derivatives.unknowns[0].index, 7);
        assert_eq!(ms.derivatives.unknowns[1].index, 8);
        assert_eq!(ms.initial_unknowns.unknowns.len(), 4);
        assert_eq!(ms.initial_unknowns.unknowns[0].index, 3);
        assert_eq!(ms.initial_unknowns.unknowns[1].index, 4);
        assert_eq!(ms.initial_unknowns.unknowns[2].index, 7);
        assert_eq!(ms.initial_unknowns.unknowns[2].dependencies, vec! {5, 2});
        assert_eq!(ms.initial_unknowns.unknowns[3].dependencies, vec! {5, 6});
    }
}
