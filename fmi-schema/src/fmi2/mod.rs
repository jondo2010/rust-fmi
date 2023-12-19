//! This module implements the ModelDescription datamodel and provides
//! attributes to serde_xml_rs to generate an XML deserializer.

use yaserde::{de, YaDeserialize};

mod interface_type;
mod model_description;
mod variable;

pub use interface_type::*;
pub use model_description::*;
pub use variable::*;
use yaserde_derive::{YaDeserialize, YaSerialize};

pub type ScalarVariableMap<'a> = std::collections::HashMap<String, &'a ScalarVariable>;
pub type UnknownsTuple<'a> = (&'a ScalarVariable, Vec<&'a ScalarVariable>);

#[derive(Debug, PartialEq)]
pub enum ScalarVariableElementBase {
    Real,
    Integer,
    Boolean,
    String,
    Enumeration,
}

impl From<&ScalarVariableElement> for ScalarVariableElementBase {
    fn from(sve: &ScalarVariableElement) -> Self {
        match sve {
            ScalarVariableElement::Real { .. } => Self::Real,
            ScalarVariableElement::Integer { .. } => Self::Integer,
            ScalarVariableElement::Boolean { .. } => Self::Boolean,
            ScalarVariableElement::String { .. } => Self::String,
            ScalarVariableElement::Enumeration { .. } => Self::Enumeration,
        }
    }
}

/*
#[derive(Debug, Deserialize)]
#[serde(from = "ModelVariablesRaw")]
pub struct ModelVariables {
    /// A Vec of the ValueReferences in the order they appeared, for index lookup.
    by_index: Vec<ValueReference>,
    /// A Map of ValueReference to ScalarVariable
    map: BTreeMap<ValueReference, ScalarVariable>,
}

impl From<ModelVariablesRaw> for ModelVariables {
    fn from(raw: ModelVariablesRaw) -> Self {
        Self {
            by_index: raw
                .variables
                .iter()
                .map(|variable| variable.value_reference.clone())
                .collect(),
            map: raw
                .variables
                .into_iter()
                .map(|variable| (variable.value_reference.clone(), variable))
                .collect(),
        }
    }
}
*/

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "Unknown")]
pub struct Unknown {
    pub index: u32,
    #[serde(default, deserialize_with = "vec_from_str")]
    pub dependencies: Vec<u32>,
}

#[derive(Debug, Deserialize)]
pub struct UnknownList {
    #[serde(default, rename = "$value")]
    pub unknowns: Vec<Unknown>,
}
impl Default for UnknownList {
    fn default() -> UnknownList {
        UnknownList {
            unknowns: Vec::<Unknown>::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_model_exchange() {
        let s = r##"<ModelExchange modelIdentifier="MyLibrary_SpringMassDamper"/>"##;
        let x: ModelExchange = serde_xml_rs::from_reader(s.as_bytes()).unwrap();
        assert!(x.model_identifier == "MyLibrary_SpringMassDamper");
    }

    #[test]
    fn test_default_experiment() {
        let s = r##"<DefaultExperiment stopTime="3.0" tolerance="0.0001"/>"##;
        let x: DefaultExperiment = serde_xml_rs::from_reader(s.as_bytes()).unwrap();
        assert_approx_eq!(x.start_time, 0.0, f64::EPSILON);
        assert_approx_eq!(x.stop_time, 3.0, f64::EPSILON);
        assert_approx_eq!(x.tolerance, 0.0001, f64::EPSILON);

        let s = r#"<DefaultExperiment startTime = "0.10000000000000000e+00" stopTime  = "1.50000000000000000e+00" tolerance = "0.0001"/>"#;
        let x: DefaultExperiment = serde_xml_rs::from_reader(s.as_bytes()).unwrap();
        assert_approx_eq!(x.start_time, 0.1, f64::EPSILON);
        assert_approx_eq!(x.stop_time, 1.5, f64::EPSILON);
        assert_approx_eq!(x.tolerance, 0.0001, f64::EPSILON);
    }

    #[test]
    fn test_scalar_variable() {
        let s = r##"
<ScalarVariable name="inertia1.J" valueReference="1073741824" description="Moment of load inertia" causality="parameter" variability="fixed">
<Real declaredType="Modelica.SIunits.Inertia" start="1"/>
</ScalarVariable>
        "##;
        let x: ScalarVariable = serde_xml_rs::from_reader(s.as_bytes()).unwrap();
        assert_eq!(x.name, "inertia1.J");
        assert_eq!(x.value_reference, ValueReference(1073741824));
        assert_eq!(x.description, "Moment of load inertia");
        assert_eq!(x.causality, Causality::Parameter);
        assert_eq!(x.variability, Variability::Fixed);
        assert_eq!(
            x.elem,
            ScalarVariableElement::Real {
                declared_type: Some("Modelica.SIunits.Inertia".to_string()),
                start: 1.0,
                relative_quantity: false,
                derivative: None
            }
        );
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
        let x: ModelVariables = serde_xml_rs::from_reader(s.as_bytes()).unwrap();
        assert_eq!(x.map.len(), 4);
        assert!(x
            .map
            .values()
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
        let x: ModelStructure = serde_xml_rs::from_reader(s.as_bytes()).unwrap();
        assert_eq!(x.outputs.unknowns[0].index, 3);
        assert_eq!(x.outputs.unknowns[1].index, 4);
        assert_eq!(x.derivatives.unknowns[0].index, 7);
        assert_eq!(x.derivatives.unknowns[1].index, 8);
        assert_eq!(x.initial_unknowns.unknowns[0].index, 3);
        assert_eq!(x.initial_unknowns.unknowns[1].index, 4);
        assert_eq!(x.initial_unknowns.unknowns[2].index, 7);
        assert_eq!(x.initial_unknowns.unknowns[2].dependencies, vec! {5, 2});
        assert_eq!(x.initial_unknowns.unknowns[3].dependencies, vec! {5, 6});
    }

    #[test]
    fn test_model_description() {
        let s = r##"
<?xml version="1.0" encoding="UTF8"?>
<fmiModelDescription
 fmiVersion="2.0"
 modelName="MyLibrary.SpringMassDamper"
 guid="{8c4e810f-3df3-4a00-8276-176fa3c9f9e0}"
 description="Rotational Spring Mass Damper System"
 version="1.0"
 generationDateAndTime="2011-09-23T16:57:33Z"
 variableNamingConvention="structured"
 numberOfEventIndicators="2">
 <ModelVariables>
    <ScalarVariable name="x[1]" valueReference="0" initial="exact"> <Real/> </ScalarVariable> <!-- idex="5" -->
    <ScalarVariable name="x[2]" valueReference="1" initial="exact"> <Real/> </ScalarVariable> <!-- index="6" -->
    <ScalarVariable name="PI.x" valueReference="46" description="State of block" causality="local" variability="continuous" initial="calculated">
        <Real relativeQuantity="false" />
    </ScalarVariable>
    <ScalarVariable name="der(PI.x)" valueReference="45" causality="local" variability="continuous" initial="calculated">
        <Real relativeQuantity="false" derivative="3" />
    </ScalarVariable>
 </ModelVariables>
 <ModelStructure>
    <Outputs><Unknown index="1" dependencies="1 2" /><Unknown index="2" /></Outputs>
    <Derivatives><Unknown index="4" dependencies="1 2" /></Derivatives>
    <InitialUnknowns />
</ModelStructure>
</fmiModelDescription>
        "##;
        let x: ModelDescription = serde_xml_rs::from_str(s).expect("hello");
        assert_eq!(x.fmi_version, "2.0");
        assert_eq!(x.model_name, "MyLibrary.SpringMassDamper");
        assert_eq!(x.guid, "{8c4e810f-3df3-4a00-8276-176fa3c9f9e0}");
        assert_eq!(x.description, "Rotational Spring Mass Damper System");
        assert_eq!(x.version, "1.0");
        // assert_eq!(x.generation_date_and_time, chrono::DateTime<chrono::Utc>::from)
        assert_eq!(x.variable_naming_convention, Some("structured".to_owned()));
        assert_eq!(x.number_of_event_indicators, 2);
        assert_eq!(x.model_variables.map.len(), 4);

        let outputs = x.outputs().unwrap();
        assert_eq!(outputs[0].0.name, "x[1]");
        assert_eq!(outputs[0].1.len(), 2);
        assert_eq!(outputs[0].1[0].name, "x[1]");
        assert_eq!(outputs[1].0.name, "x[2]");
        assert_eq!(outputs[1].1.len(), 0);

        let derivatives = x.derivatives().unwrap();
        assert_eq!(derivatives[0].0.name, "der(PI.x)");
        assert_eq!(
            derivatives[0].0.elem,
            ScalarVariableElement::Real {
                declared_type: None,
                start: 0.0,
                relative_quantity: false,
                derivative: Some(3)
            }
        );

        let states = x.continuous_states().unwrap();
        assert_eq!(
            states
                .iter()
                .map(|(der, state)| (der.name.as_str(), state.name.as_str()))
                .collect::<Vec<(_, _)>>(),
            vec![("PI.x", "der(PI.x)")]
        );
    }

    // #[test]
    // fn test_file() {
    // let file = std::fs::File::open("modelDescription.xml").unwrap();
    // let file = std::io::BufReader::new(file);
    // let x: ModelDescription = serde_xml_rs::deserialize(file).unwrap();
    // println!("{:#?}", x);
    // }
}
