//! This module implements the ModelDescription datamodel and provides
//! attributes to serde_xml_rs to generate an XML deserializer.

use yaserde_derive::{YaDeserialize, YaSerialize};

mod attribute_groups;
mod counts;
mod interface_type;
mod model_description;
mod scalar_variable;
mod r#type;
mod unit;
mod variable_dependency;

pub use attribute_groups::*;
pub use interface_type::*;
pub use model_description::*;
pub use r#type::*;
pub use scalar_variable::*;
pub use unit::*;
pub use variable_dependency::*;

pub type ScalarVariableMap<'a> = std::collections::HashMap<String, &'a ScalarVariable>;
pub type UnknownsTuple<'a> = (&'a ScalarVariable, Vec<&'a ScalarVariable>);

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct UnknownList {
    #[yaserde(child)]
    pub unknowns: Vec<Unknown>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_exchange() {
        let s = r##"<ModelExchange modelIdentifier="MyLibrary_SpringMassDamper"/>"##;
        let x: ModelExchange = yaserde::de::from_str(s).unwrap();
        assert!(x.model_identifier == "MyLibrary_SpringMassDamper");
    }

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
                <Outputs> <Unknown index="3" /> <Unknown index="4" /> </Outputs>
                <Derivatives> <Unknown index="7" /> <Unknown index="8" /> </Derivatives>
                <InitialUnknowns> <Unknown index="3" /> <Unknown index="4" /> <Unknown index="7" dependencies="5 2" /> <Unknown index="8" dependencies="5 6" /> </InitialUnknowns>
            </ModelStructure>
        "##;
        let x: ModelStructure = yaserde::de::from_str(s).unwrap();
        assert_eq!(x.outputs.unknowns[0].index, 3);
        assert_eq!(x.outputs.unknowns[1].index, 4);
        assert_eq!(x.derivatives.unknowns[0].index, 7);
        assert_eq!(x.derivatives.unknowns[1].index, 8);
        assert_eq!(x.initial_unknowns.unknowns[0].index, 3);
        assert_eq!(x.initial_unknowns.unknowns[1].index, 4);
        assert_eq!(x.initial_unknowns.unknowns[2].index, 7);
        //assert_eq!(x.initial_unknowns.unknowns[2].dependencies, vec! {5, 2});
        //assert_eq!(x.initial_unknowns.unknowns[3].dependencies, vec! {5, 6});
    }

    // #[test]
    // fn test_file() {
    // let file = std::fs::File::open("modelDescription.xml").unwrap();
    // let file = std::io::BufReader::new(file);
    // let x: ModelDescription = serde_xml_rs::deserialize(file).unwrap();
    // println!("{:#?}", x);
    // }
}
