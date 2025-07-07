use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::traits::FmiModelDescription;

use super::{
    AbstractVariableTrait, Annotations, Float32Type, Float64Type, Fmi3CoSimulation,
    Fmi3ModelExchange, Fmi3ScheduledExecution, Fmi3Unit, Fmi3Unknown, FmiBinary, FmiBoolean,
    FmiFloat32, FmiFloat64, FmiInt16, FmiInt32, FmiInt64, FmiInt8, FmiString, FmiUInt16, FmiUInt32,
    FmiUInt64, FmiUInt8, InitializableVariableTrait,
};

#[derive(Default, Debug, YaDeserialize)]
#[yaserde(rename = "fmiModelDescription")]
pub struct Fmi3ModelDescription {
    /// Version of FMI that was used to generate the XML file.
    #[yaserde(attribute = true, rename = "fmiVersion")]
    pub fmi_version: String,

    /// The name of the model as used in the modeling environment that generated the XML file, such
    /// as Modelica.Mechanics.Rotational.Examples.CoupledClutches.
    #[yaserde(attribute = true, rename = "modelName")]
    pub model_name: String,

    /// The instantiationToken is a string that can be used by the FMU to check that the XML file
    /// is compatible with the implementation of the FMU.
    #[yaserde(attribute = true, rename = "instantiationToken")]
    pub instantiation_token: String,

    /// Optional string with a brief description of the model.
    #[yaserde(attribute = true, rename = "description")]
    pub description: Option<String>,

    /// String with the name and organization of the model author.
    #[yaserde(attribute = true, rename = "author")]
    pub author: Option<String>,

    /// Version of the model [for example 1.0].
    #[yaserde(attribute = true, rename = "version")]
    pub version: Option<String>,

    /// Information on the intellectual property copyright for this FMU [for example © My Company
    /// 2011].
    #[yaserde(attribute = true, rename = "copyright")]
    pub copyright: Option<String>,

    /// Information on the intellectual property licensing for this FMU [for example BSD license
    /// <license text or link to license>].
    #[yaserde(attribute = true, rename = "license")]
    pub license: Option<String>,

    /// Name of the tool that generated the XML file.
    #[yaserde(attribute = true, rename = "generationTool")]
    pub generation_tool: Option<String>,

    ///  Date and time when the XML file was generated. The format is a subset of dateTime and
    /// should be: YYYY-MM-DDThh:mm:ssZ (with one T between date and time; Z characterizes the Zulu
    /// time zone, in other words, Greenwich meantime) [for example 2009-12-08T14:33:22Z].
    #[yaserde(attribute = true, rename = "generationDateAndTime")]
    // pub generation_date_and_time: Option<DateTime>,
    pub generation_date_and_time: Option<String>,

    /// Defines whether the variable names in <ModelVariables> and in <TypeDefinitions> follow a
    /// particular convention.
    #[yaserde(attribute = true, rename = "variableNamingConvention")]
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

    #[yaserde(rename = "ModelVariables", default = "default_model_variables")]
    pub model_variables: ModelVariables,

    /// The model structure defines the dependency structure of the model variables.
    #[yaserde(rename = "ModelStructure")]
    pub model_structure: ModelStructure,

    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
}

fn default_model_variables() -> ModelVariables {
    ModelVariables::default()
}

impl FmiModelDescription for Fmi3ModelDescription {
    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn version_string(&self) -> &str {
        &self.fmi_version
    }
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(tag = "UnitDefinitions")]
pub struct UnitDefinitions {
    #[yaserde(rename = "Unit")]
    pub units: Vec<Fmi3Unit>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct TypeDefinitions {
    #[yaserde(rename = "Float32Type")]
    pub float32types: Vec<Float32Type>,
    #[yaserde(rename = "Float64Type")]
    pub float64types: Vec<Float64Type>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct LogCategories {
    #[yaserde(rename = "Category")]
    pub categories: Vec<Category>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Category {
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
    #[yaserde(attribute = true)]
    pub name: String,
    #[yaserde(attribute = true)]
    pub description: Option<String>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct DefaultExperiment {
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
    #[yaserde(attribute = true, rename = "startTime")]
    pub start_time: Option<f64>,
    #[yaserde(attribute = true, rename = "stopTime")]
    pub stop_time: Option<f64>,
    #[yaserde(attribute = true, rename = "tolerance")]
    pub tolerance: Option<f64>,
    #[yaserde(attribute = true, rename = "stepSize")]
    pub step_size: Option<f64>,
}

#[derive(Default, Debug, YaDeserialize)]
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
    pub int16: Vec<FmiInt16>,
    #[yaserde(rename = "UInt16")]
    pub uint16: Vec<FmiUInt16>,
    #[yaserde(rename = "Int32")]
    pub int32: Vec<FmiInt32>,
    #[yaserde(rename = "UInt32")]
    pub uint32: Vec<FmiUInt32>,
    #[yaserde(rename = "Int64")]
    pub int64: Vec<FmiInt64>,
    #[yaserde(rename = "UInt64")]
    pub uint64: Vec<FmiUInt64>,
    #[yaserde(rename = "Boolean")]
    pub boolean: Vec<FmiBoolean>,
    #[yaserde(rename = "String")]
    pub string: Vec<FmiString>,
    #[yaserde(rename = "Binary")]
    pub binary: Vec<FmiBinary>,
}

impl ModelVariables {
    /// Returns the total number of variables in the model description
    pub fn len(&self) -> usize {
        self.iter_abstract().count()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
            self.int64.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.uint64.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.boolean.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.string.iter().map(|v| v as &dyn AbstractVariableTrait),
            self.binary.iter().map(|v| v as &dyn AbstractVariableTrait),
        )
    }

    /// Returns an iterator over all the float32 and float64 variables in the model description
    pub fn iter_floating(&self) -> impl Iterator<Item = &dyn InitializableVariableTrait> {
        itertools::chain!(
            self.float32
                .iter()
                .map(|v| v as &dyn InitializableVariableTrait),
            self.float64
                .iter()
                .map(|v| v as &dyn InitializableVariableTrait),
        )
    }
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
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

        let md: Fmi3ModelDescription = yaserde::de::from_str(xml).unwrap();

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

    #[test]
    fn test_model_variables() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<ModelVariables>
    <Float64 name="time" valueReference="0" causality="independent" variability="continuous"/>

    <Float32 name="Float32_continuous_input"  valueReference="1" causality="input" start="0"/>
    <Float32 name="Float32_continuous_output" valueReference="2" causality="output"/>
    <Float32 name="Float32_discrete_input"    valueReference="3" causality="input" variability="discrete" start="0"/>
    <Float32 name="Float32_discrete_output"   valueReference="4" causality="output" variability="discrete"/>

    <Float64 name="Float64_fixed_parameter" valueReference="5" causality="parameter" variability="fixed" start="0"/>
    <Float64 name="Float64_tunable_parameter" valueReference="6" causality="parameter" variability="tunable" start="0"/>
    <Float64 name="Float64_continuous_input" valueReference="7" causality="input" start="0" initial="exact"/>
    <Float64 name="Float64_continuous_output" valueReference="8" causality="output" initial="calculated"/>
    <Float64 name="Float64_discrete_input" valueReference="9" causality="input" variability="discrete" start="0"/>
    <Float64 name="Float64_discrete_output" valueReference="10" causality="output" variability="discrete" initial="calculated"/>

    <Int8 name="Int8_input" valueReference="11" causality="input" start="0"/>
    <Int8 name="Int8_output" valueReference="12" causality="output"/>

    <UInt8 name="UInt8_input" valueReference="13" causality="input" start="0"/>
    <UInt8 name="UInt8_output" valueReference="14" causality="output"/>

    <Int16 name="Int16_input" valueReference="15" causality="input" start="0"/>
    <Int16 name="Int16_output" valueReference="16" causality="output"/>

    <UInt16 name="UInt16_input" valueReference="17" causality="input" start="0"/>
    <UInt16 name="UInt16_output" valueReference="18" causality="output"/>

    <Int32 name="Int32_input" valueReference="19" causality="input" start="0"/>
    <Int32 name="Int32_output" valueReference="20" causality="output"/>

    <UInt32 name="UInt32_input" valueReference="21" causality="input" start="0"/>
    <UInt32 name="UInt32_output" valueReference="22" causality="output"/>

    <Int64 name="Int64_input" valueReference="23" causality="input" start="0"/>
    <Int64 name="Int64_output" valueReference="24" causality="output"/>

    <UInt64 name="UInt64_input" valueReference="25" causality="input" start="0"/>
    <UInt64 name="UInt64_output" valueReference="26" causality="output"/>

    <Boolean name="Boolean_input" valueReference="27" causality="input" start="false"/>
    <Boolean name="Boolean_output" valueReference="28" causality="output" initial="calculated"/>

    <String name="String_parameter" valueReference="29" causality="parameter" variability="fixed">
        <Start value="Set me!"/>
    </String>

    <Binary name="Binary_input" valueReference="30" causality="input">
        <Start value="666f6f"/>
    </Binary>
    <Binary name="Binary_output" valueReference="31" causality="output"/>

    <Enumeration name="Enumeration_input" declaredType="Option" valueReference="32" causality="input" start="1"/>
    <Enumeration name="Enumeration_output" declaredType="Option" valueReference="33" causality="output"/>
</ModelVariables>"#;

        let mv: ModelVariables = yaserde::de::from_str(xml).unwrap();

        assert_eq!(mv.float32.len(), 4);
        assert_eq!(mv.float64.len(), 7);
        assert_eq!(mv.int8.len(), 2);
        assert_eq!(mv.uint8.len(), 2);
        assert_eq!(mv.int16.len(), 2);
        assert_eq!(mv.uint16.len(), 2);
        assert_eq!(mv.int32.len(), 2);
        assert_eq!(mv.uint32.len(), 2);
        assert_eq!(mv.int64.len(), 2);
        assert_eq!(mv.uint64.len(), 2);
        assert_eq!(mv.boolean.len(), 2);
        assert_eq!(mv.boolean[0].name(), "Boolean_input");
        assert_eq!(mv.boolean[0].causality(), crate::fmi3::Causality::Input);
        assert_eq!(mv.boolean[0].start, vec![false]);
        assert_eq!(mv.string.len(), 1);
        // assert_eq!(mv.binary.len(), 2);
        // assert_eq!(mv.enumeration.len(), 2);
    }
}
