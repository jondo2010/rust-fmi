use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{
    AbstractVariableTrait, Annotations, Float32Type, Float64Type, Fmi3CoSimulation,
    Fmi3ModelExchange, Fmi3ScheduledExecution, Fmi3Unit, Fmi3Unknown, FmiFloat32, FmiFloat64,
    FmiInt8, FmiUInt8,
};

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "fmiModelDescription")]
pub struct FmiModelDescription {
    /// Version of FMI that was used to generate the XML file.
    #[yaserde(attribute, rename = "fmiVersion")]
    pub fmi_version: String,

    /// The name of the model as used in the modeling environment that generated the XML file, such as Modelica.Mechanics.Rotational.Examples.CoupledClutches.
    #[yaserde(attribute, rename = "modelName")]
    pub model_name: String,

    /// The instantiationToken is a string that can be used by the FMU to check that the XML file is compatible with the implementation of the FMU.
    #[yaserde(attribute, rename = "instantiationToken")]
    pub instantiation_token: String,

    /// Optional string with a brief description of the model.
    #[yaserde(attribute, rename = "description")]
    pub description: Option<String>,

    /// String with the name and organization of the model author.
    #[yaserde(attribute, rename = "author")]
    pub author: Option<String>,

    /// Version of the model [for example 1.0].
    #[yaserde(attribute, rename = "version")]
    pub version: Option<String>,

    /// Information on the intellectual property copyright for this FMU [for example © My Company 2011].
    #[yaserde(attribute, rename = "copyright")]
    pub copyright: Option<String>,

    /// Information on the intellectual property licensing for this FMU [for example BSD license <license text or link to license>].
    #[yaserde(attribute, rename = "license")]
    pub license: Option<String>,

    /// Name of the tool that generated the XML file.
    #[yaserde(attribute, rename = "generationTool")]
    pub generation_tool: Option<String>,

    ///  Date and time when the XML file was generated. The format is a subset of dateTime and should be: YYYY-MM-DDThh:mm:ssZ (with one T between date and time; Z characterizes the Zulu time zone, in other words, Greenwich meantime) [for example 2009-12-08T14:33:22Z].
    #[yaserde(attribute, rename = "generationDateAndTime")]
    //pub generation_date_and_time: Option<DateTime>,
    pub generation_date_and_time: Option<String>,

    /// Defines whether the variable names in <ModelVariables> and in <TypeDefinitions> follow a particular convention.
    #[yaserde(attribute, rename = "variableNamingConvention")]
    pub variable_naming_convention: Option<String>,

    /// If present, the FMU is based on FMI for Model Exchange
    #[yaserde(rename = "ModelExchange")]
    pub model_exchange: Option<Fmi3ModelExchange>,

    /// If present, the FMU is based on FMI for Co-Simulation
    #[yaserde(rename = "CoSimulation")]
    pub co_simulation: Option<Fmi3CoSimulation>,

    /// If present, the FMU is based on FMI for Scheduled Execution
    #[yaserde(rename = "ScheduledExecution")]
    pub scheduled_execution: Option<Fmi3ScheduledExecution>,

    /// A global list of unit and display unit definitions
    #[yaserde(rename = "UnitDefinitions")]
    pub unit_definitions: Option<UnitDefinitions>,

    /// A global list of type definitions that are utilized in `ModelVariables`
    #[yaserde(rename = "TypeDefinitions")]
    pub type_definitions: Option<TypeDefinitions>,

    #[yaserde(rename = "LogCategories")]
    pub log_categories: Option<LogCategories>,

    /// Providing default settings for the integrator, such as stop time and relative tolerance.
    #[yaserde(rename = "DefaultExperiment")]
    pub default_experiment: Option<DefaultExperiment>,

    #[yaserde(child, flatten, rename = "ModelVariables")]
    pub model_variables: ModelVariables,

    /// The model structure defines the dependency structure of the model variables.
    #[yaserde(rename = "ModelStructure")]
    pub model_structure: ModelStructure,

    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(tag = "UnitDefinitions")]
pub struct UnitDefinitions {
    #[yaserde(rename = "Unit")]
    pub units: Vec<Fmi3Unit>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(root = "TypeDefinitions")]
pub struct TypeDefinitions {
    #[yaserde(rename = "Float32Type")]
    pub float32types: Vec<Float32Type>,
    #[yaserde(rename = "Float64Type")]
    pub float64types: Vec<Float64Type>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(root = "LogCategories")]
pub struct LogCategories {
    #[yaserde(rename = "Category")]
    pub categories: Vec<Category>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(root = "Category")]
pub struct Category {
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
    #[yaserde(attribute)]
    pub name: String,
    #[yaserde(attribute)]
    pub description: Option<String>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct DefaultExperiment {
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
    #[yaserde(attribute, rename = "startTime")]
    pub start_time: Option<f64>,
    #[yaserde(attribute, rename = "stopTime")]
    pub stop_time: Option<f64>,
    #[yaserde(attribute, rename = "tolerance")]
    pub tolerance: Option<f64>,
    #[yaserde(attribute, rename = "stepSize")]
    pub step_size: Option<f64>,
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
#[yaserde(root = "ModelVariables")]
pub struct ModelVariables {
    #[yaserde(rename = "Float32")]
    pub float32: Vec<FmiFloat32>,
    #[yaserde(rename = "Float64")]
    pub float64: Vec<FmiFloat64>,
    #[yaserde(rename = "Int8")]
    pub int8: Vec<FmiInt8>,
    #[yaserde(rename = "UInt8")]
    pub uint8: Vec<FmiUInt8>,
    #[yaserde(rename = "Int16")]
    pub int16: Vec<FmiInt8>,
    #[yaserde(rename = "UInt16")]
    pub uint16: Vec<FmiUInt8>,
    #[yaserde(rename = "Int32")]
    pub int32: Vec<FmiInt8>,
    #[yaserde(rename = "UInt32")]
    pub uint32: Vec<FmiUInt8>,
}

impl ModelVariables {
    /// Returns the total number of variables in the model description
    pub fn len(&self) -> usize {
        self.iter_abstract().count()
    }

    /// Returns an iterator over all the AbstractVariables in the model description
    pub fn iter_abstract(&self) -> impl Iterator<Item = &dyn AbstractVariableTrait> {
        itertools::chain!(
            self.float32.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.float64.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.int8.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.uint8.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.int16.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.uint16.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.int32.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.uint32.iter().map(|v| v as &dyn AbstractVariableTrait),
        )
    }
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(root = "ModelStructure")]
pub struct ModelStructure {
    #[yaserde(rename = "Output")]
    pub outputs: Vec<Fmi3Unknown>,

    #[yaserde(rename = "ContinuousStateDerivative")]
    pub continuous_state_derivative: Vec<Fmi3Unknown>,

    #[yaserde(rename = "ClockedState")]
    pub clocked_state: Vec<Fmi3Unknown>,

    #[yaserde(rename = "InitialUnknown")]
    pub initial_unknown: Vec<Fmi3Unknown>,

    #[yaserde(rename = "EventIndicator")]
    pub event_indicator: Vec<Fmi3Unknown>,
}

#[test]
fn test_model_descr() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
    <fmiModelDescription
        fmiVersion="3.0-beta.2"
        modelName="FMI3"
        instantiationToken="{fmi3}"
        description="FMI3 Test FMU"
        generationTool="FMI3"
        generationDateAndTime="2021-03-01T00:00:00Z"
        variableNamingConvention="flat">
        <DefaultExperiment startTime="0" stopTime="3" stepSize="1e-3"/>
        <ModelStructure>
            <Output valueReference="1" />
        </ModelStructure>
    </fmiModelDescription>"#;

    let md: FmiModelDescription = yaserde::de::from_str(xml).unwrap();

    assert_eq!(md.fmi_version, "3.0-beta.2");
    assert_eq!(md.model_name, "FMI3");
    assert_eq!(md.instantiation_token, "{fmi3}");
    assert_eq!(md.description.as_deref(), Some("FMI3 Test FMU"));
    assert_eq!(md.variable_naming_convention.as_deref(), Some("flat"));
    assert_eq!(md.generation_tool.as_deref(), Some("FMI3"));
    assert_eq!(
        md.default_experiment,
        Some(DefaultExperiment {
            start_time: Some(0.0),
            stop_time: Some(3.0),
            step_size: Some(1e-3),
            ..Default::default()
        })
    );
    assert_eq!(
        md.model_structure,
        ModelStructure {
            outputs: vec![Fmi3Unknown {
                value_reference: 1,
                ..Default::default()
            }],
            ..Default::default()
        }
    );
}
