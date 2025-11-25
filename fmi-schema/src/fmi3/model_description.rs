use crate::{Error, fmi3::Fmi3Unknown, traits::FmiModelDescription};

use super::{
    Annotations, Fmi3CoSimulation, Fmi3ModelExchange, Fmi3ScheduledExecution, Fmi3Unit,
    ModelVariables, TypeDefinitions, VariableDependency,
};

#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(
    tag = "fmiModelDescription",
    strict(unknown_attribute, unknown_element)
)]
pub struct Fmi3ModelDescription {
    /// Version of FMI that was used to generate the XML file.
    #[xml(attr = "fmiVersion")]
    pub fmi_version: String,

    /// The name of the model as used in the modeling environment that generated the XML file, such
    /// as "Modelica.Mechanics.Rotational.Examples.CoupledClutches".
    #[xml(attr = "modelName")]
    pub model_name: String,

    /// The instantiationToken is a string that can be used by the FMU to check that the XML file
    /// is compatible with the implementation of the FMU.
    #[xml(attr = "instantiationToken")]
    pub instantiation_token: String,

    /// Optional string with a brief description of the model.
    #[xml(attr = "description")]
    pub description: Option<String>,

    /// String with the name and organization of the model author.
    #[xml(attr = "author")]
    pub author: Option<String>,

    /// Version of the model [for example 1.0].
    #[xml(attr = "version")]
    pub version: Option<String>,

    /// Information on the intellectual property copyright for this FMU [for example © My Company
    /// 2011].
    #[xml(attr = "copyright")]
    pub copyright: Option<String>,

    /// Information on the intellectual property licensing for this FMU [for example BSD license
    /// <license text or link to license>].
    #[xml(attr = "license")]
    pub license: Option<String>,

    /// Name of the tool that generated the XML file.
    #[xml(attr = "generationTool")]
    pub generation_tool: Option<String>,

    ///  Date and time when the XML file was generated. The format is a subset of dateTime and
    /// should be: YYYY-MM-DDThh:mm:ssZ (with one T between date and time; Z characterizes the Zulu
    /// time zone, in other words, Greenwich meantime) [for example 2009-12-08T14:33:22Z].
    #[xml(attr = "generationDateAndTime")]
    pub generation_date_and_time: Option<String>,

    /// Defines whether the variable names in <ModelVariables> and in <TypeDefinitions> follow a
    /// particular convention.
    #[xml(attr = "variableNamingConvention")]
    pub variable_naming_convention: Option<String>,

    /// If present, the FMU is based on FMI for Model Exchange
    #[xml(child = "ModelExchange")]
    pub model_exchange: Option<Fmi3ModelExchange>,

    /// If present, the FMU is based on FMI for Co-Simulation
    #[xml(child = "CoSimulation")]
    pub co_simulation: Option<Fmi3CoSimulation>,

    /// If present, the FMU is based on FMI for Scheduled Execution
    #[xml(child = "ScheduledExecution")]
    pub scheduled_execution: Option<Fmi3ScheduledExecution>,

    /// A global list of unit and display unit definitions
    #[xml(child = "UnitDefinitions")]
    pub unit_definitions: Option<UnitDefinitions>,

    /// A global list of type definitions that are utilized in `ModelVariables`
    #[xml(child = "TypeDefinitions")]
    pub type_definitions: Option<TypeDefinitions>,

    /// Categories for logging purposes
    #[xml(child = "LogCategories")]
    pub log_categories: Option<LogCategories>,

    /// Providing default settings for the integrator, such as stop time and relative tolerance.
    #[xml(child = "DefaultExperiment")]
    pub default_experiment: Option<DefaultExperiment>,

    /// The model variables defined in the model.
    #[xml(child = "ModelVariables", default)]
    pub model_variables: ModelVariables,

    /// The model structure defines the dependency structure of the model variables.
    #[xml(child = "ModelStructure", default)]
    pub model_structure: ModelStructure,

    /// Optional annotations for the top-level element.
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
}

impl FmiModelDescription for Fmi3ModelDescription {
    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn version_string(&self) -> &str {
        &self.fmi_version
    }

    fn serialize(&self) -> Result<String, Error> {
        hard_xml::XmlWrite::to_string(self).map_err(Error::XmlParse)
    }

    fn deserialize(xml: &str) -> Result<Self, crate::Error> {
        hard_xml::XmlRead::from_str(xml).map_err(crate::Error::XmlParse)
    }
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "UnitDefinitions", strict(unknown_attribute, unknown_element))]
pub struct UnitDefinitions {
    #[xml(child = "Unit")]
    pub units: Vec<Fmi3Unit>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "LogCategories", strict(unknown_attribute, unknown_element))]
pub struct LogCategories {
    #[xml(child = "Category")]
    pub categories: Vec<Category>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Category", strict(unknown_attribute, unknown_element))]
pub struct Category {
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "description")]
    pub description: Option<String>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
#[xml(tag = "DefaultExperiment", strict(unknown_attribute, unknown_element))]
pub struct DefaultExperiment {
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "crate::utils::deserialize_optional_f64_from_string")
    )]
    #[xml(attr = "startTime")]
    pub start_time: Option<f64>,
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "crate::utils::deserialize_optional_f64_from_string")
    )]
    #[xml(attr = "stopTime")]
    pub stop_time: Option<f64>,
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "crate::utils::deserialize_optional_f64_from_string")
    )]
    #[xml(attr = "tolerance")]
    pub tolerance: Option<f64>,
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "crate::utils::deserialize_optional_f64_from_string")
    )]
    #[xml(attr = "stepSize")]
    pub step_size: Option<f64>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "ModelStructure", strict(unknown_attribute, unknown_element))]
pub struct ModelStructure {
    #[xml(
        child = "Output",
        child = "ContinuousStateDerivative",
        child = "ClockedState",
        child = "InitialUnknown",
        child = "EventIndicator"
    )]
    pub unknowns: Vec<VariableDependency>,
}

impl ModelStructure {
    pub fn outputs(&self) -> impl Iterator<Item = &Fmi3Unknown> {
        self.unknowns.iter().filter_map(|dep| match dep {
            VariableDependency::Output(unknown) => Some(unknown),
            _ => None,
        })
    }
    pub fn continuous_state_derivatives(&self) -> impl Iterator<Item = &Fmi3Unknown> {
        self.unknowns.iter().filter_map(|dep| match dep {
            VariableDependency::ContinuousStateDerivative(unknown) => Some(unknown),
            _ => None,
        })
    }
    pub fn clocked_states(&self) -> impl Iterator<Item = &Fmi3Unknown> {
        self.unknowns.iter().filter_map(|dep| match dep {
            VariableDependency::ClockedState(unknown) => Some(unknown),
            _ => None,
        })
    }
    pub fn initial_unknowns(&self) -> impl Iterator<Item = &Fmi3Unknown> {
        self.unknowns.iter().filter_map(|dep| match dep {
            VariableDependency::InitialUnknown(unknown) => Some(unknown),
            _ => None,
        })
    }
    pub fn event_indicators(&self) -> impl Iterator<Item = &Fmi3Unknown> {
        self.unknowns.iter().filter_map(|dep| match dep {
            VariableDependency::EventIndicator(unknown) => Some(unknown),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use hard_xml::XmlRead;

    use crate::fmi3::Fmi3Unknown;

    use super::*;
    #[test]
    fn test_model_descr() {
        let _ = env_logger::builder()
            .is_test(true)
            .format_timestamp(None)
            .try_init();

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

        let md = Fmi3ModelDescription::from_str(xml).unwrap();

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
                unknowns: vec![VariableDependency::Output(Fmi3Unknown {
                    value_reference: 1,
                    ..Default::default()
                })],
            }
        );
    }
}
