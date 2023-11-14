use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{
    Annotations, Float32Type, Float64Type, Fmi3CoSimulation, Fmi3ModelExchange,
    Fmi3ScheduledExecution, Fmi3Unit, Fmi3Unknown, FmiFloat32, FmiFloat64,
};

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct FmiModelDescription {
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
    pub unit_definitions: Option<fmi_model_description::UnitDefinitionsType>,

    /// A global list of type definitions that are utilized in `ModelVariables`
    #[yaserde(rename = "TypeDefinitions")]
    pub type_definitions: Option<fmi_model_description::TypeDefinitionsType>,

    #[yaserde(rename = "LogCategories")]
    pub log_categories: Option<fmi_model_description::LogCategoriesType>,

    /// Providing default settings for the integrator, such as stop time and relative tolerance.
    #[yaserde(rename = "DefaultExperiment")]
    pub default_experiment: Option<fmi_model_description::DefaultExperimentType>,

    #[yaserde(rename = "ModelVariables")]
    pub model_variables: fmi_model_description::ModelVariablesType,

    #[yaserde(rename = "ModelStructure")]
    pub model_structure: fmi_model_description::ModelStructureType,

    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,

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
    #[yaserde(attribute)]
    pub description: Option<String>,

    /// String with the name and organization of the model author.
    #[yaserde(attribute)]
    pub author: Option<String>,

    /// Version of the model [for example 1.0].
    #[yaserde(attribute)]
    pub version: Option<String>,

    /// Information on the intellectual property copyright for this FMU [for example © My Company 2011].
    #[yaserde(attribute)]
    pub copyright: Option<String>,

    /// Information on the intellectual property licensing for this FMU [for example BSD license <license text or link to license>].
    #[yaserde(attribute)]
    pub license: Option<String>,

    /// Name of the tool that generated the XML file.
    #[yaserde(attribute, rename = "generationTool")]
    pub generation_tool: Option<String>,

    ///  Date and time when the XML file was generated. The format is a subset of dateTime and should be: YYYY-MM-DDThh:mm:ssZ (with one T between date and time; Z characterizes the Zulu time zone, in other words, Greenwich meantime) [for example 2009-12-08T14:33:22Z].
    #[yaserde(attribute, rename = "generationDateAndTime")]
    pub generation_date_and_time: Option<String>,

    /// Defines whether the variable names in <ModelVariables> and in <TypeDefinitions> follow a particular convention.
    #[yaserde(attribute, rename = "variableNamingConvention")]
    pub variable_naming_convention: Option<String>,
}

pub mod fmi_model_description {

    use super::*;

    #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
    pub struct UnitDefinitionsType {
        #[yaserde(rename = "Unit")]
        pub units: Vec<Fmi3Unit>,
    }

    #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
    #[yaserde(flatten)]
    pub struct TypeDefinitionsType {
        #[yaserde(rename = "Float32Type")]
        pub float32_type: Vec<Float32Type>,
        #[yaserde(rename = "Float64Type")]
        pub float64_type: Vec<Float64Type>,
    }

    #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
    pub struct LogCategoriesType {
        #[yaserde(rename = "Category")]
        pub categories: Vec<log_categories_type::CategoryType>,
    }

    pub mod log_categories_type {
        use super::*;

        #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
        pub struct CategoryType {
            #[yaserde(rename = "Annotations")]
            pub annotations: Option<Annotations>,

            #[yaserde(attribute, rename = "name")]
            pub name: String,

            #[yaserde(attribute, rename = "description")]
            pub description: Option<String>,
        }
    }

    #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
    pub struct DefaultExperimentType {
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

    #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
    #[yaserde(flatten)]
    pub struct ModelVariablesType {
        #[yaserde(rename = "Float32")]
        pub float32: Vec<FmiFloat32>,
        #[yaserde(rename = "Float64")]
        pub float64: Vec<FmiFloat64>,
    }

    impl ModelVariablesType {
        /// Returns the total number of variables in the model description
        pub fn len(&self) -> usize {
            self.float32.len() + self.float64.len()
        }
    }

    #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
    pub struct ModelStructureType {
        #[yaserde(rename = "Output")]
        pub output: Vec<Fmi3Unknown>,

        #[yaserde(rename = "ContinuousStateDerivative")]
        pub continuous_state_derivative: Vec<Fmi3Unknown>,

        #[yaserde(rename = "ClockedState")]
        pub clocked_state: Vec<Fmi3Unknown>,

        #[yaserde(rename = "InitialUnknown")]
        pub initial_unknown: Vec<Fmi3Unknown>,

        #[yaserde(rename = "EventIndicator")]
        pub event_indicator: Vec<Fmi3Unknown>,
    }
}
