use yaserde_derive::{YaDeserialize, YaSerialize};

use crate::{default_wrapper, traits::FmiModelDescription, Error};

use super::{
    CoSimulation, Fmi2Unit, Fmi2VariableDependency, ModelExchange, ScalarVariable, SimpleType,
};

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct Fmi2ModelDescription {
    /// Version of FMI (Clarification for FMI 2.0.2: for FMI 2.0.x revisions fmiVersion is defined
    /// as "2.0").
    #[yaserde(attribute = true, rename = "fmiVersion")]
    pub fmi_version: String,

    /// The name of the model as used in the modeling environment that generated the XML file, such
    /// as Modelica.Mechanics.Rotational.Examples.CoupledClutches.
    #[yaserde(attribute = true, rename = "modelName")]
    pub model_name: String,

    /// Fingerprint of xml-file content to verify that xml-file and C-functions are compatible to
    /// each other
    #[yaserde(attribute = true)]
    pub guid: String,

    #[yaserde(attribute = true)]
    pub description: Option<String>,

    /// Version of FMU, e.g., "1.4.1"
    #[yaserde(attribute = true)]
    pub version: Option<String>,

    /// Information on intellectual property copyright for this FMU, such as “© MyCompany 2011“
    #[yaserde(attribute = true)]
    pub copyright: Option<String>,

    /// Information on intellectual property licensing for this FMU, such as “BSD license”,
    /// "Proprietary", or "Public Domain"
    #[yaserde(attribute = true)]
    pub license: Option<String>,

    /// Name of the tool that generated the XML file.
    #[yaserde(attribute = true, rename = "generationTool")]
    pub generation_tool: Option<String>,

    /// time/date of database creation according to ISO 8601 (preference: YYYY-MM-DDThh:mm:ss)
    /// Date and time when the XML file was generated. The format is a subset of dateTime and
    /// should be: YYYY-MM-DDThh:mm:ssZ (with one T between date and time; Z characterizes the
    /// Zulu time zone, in other words, Greenwich meantime) [for example 2009-12-08T14:33:22Z].
    #[yaserde(attribute = true, rename = "generationDateAndTime")]
    pub generation_date_and_time: Option<String>,

    /// Defines whether the variable names in <ModelVariables> and in <TypeDefinitions> follow a
    /// particular convention.
    #[yaserde(attribute = true, rename = "variableNamingConvention")]
    pub variable_naming_convention: Option<String>,

    #[yaserde(attribute = true, rename = "numberOfEventIndicators")]
    pub number_of_event_indicators: u32,

    /// If present, the FMU is based on FMI for Model Exchange
    #[yaserde(rename = "ModelExchange")]
    pub model_exchange: Option<ModelExchange>,

    /// If present, the FMU is based on FMI for Co-Simulation
    #[yaserde(rename = "CoSimulation")]
    pub co_simulation: Option<CoSimulation>,

    #[yaserde(rename = "LogCategories")]
    pub log_categories: Option<LogCategories>,

    #[yaserde(rename = "DefaultExperiment")]
    pub default_experiment: Option<DefaultExperiment>,

    #[yaserde(rename = "UnitDefinitions")]
    pub unit_definitions: Option<UnitDefinitions>,

    #[yaserde(rename = "TypeDefinitions")]
    pub type_definitions: Option<TypeDefinitions>,

    #[yaserde(rename = "ModelVariables")]
    pub model_variables: ModelVariables,

    #[yaserde(rename = "ModelStructure")]
    pub model_structure: ModelStructure,
}

impl Fmi2ModelDescription {
    /// Total number of variables
    pub fn num_variables(&self) -> usize {
        self.model_variables.variables.len()
    }

    /// Get the number of continuous states (and derivatives)
    pub fn num_states(&self) -> usize {
        self.model_structure.derivatives.unknowns.len()
    }

    pub fn num_event_indicators(&self) -> usize {
        self.number_of_event_indicators as usize
    }

    /// Get a iterator of the SalarVariables
    pub fn get_model_variables(&self) -> impl Iterator<Item = &ScalarVariable> {
        self.model_variables.variables.iter()
    }

    /// Get a reference to the model variable with the given name
    pub fn model_variable_by_name(&self, name: &str) -> Result<&ScalarVariable, Error> {
        self.model_variables
            .variables
            .iter()
            .find(|var| var.name == name)
            .ok_or_else(|| Error::VariableNotFound(name.to_owned()))
    }
}

impl FmiModelDescription for Fmi2ModelDescription {
    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn version_string(&self) -> &str {
        &self.fmi_version
    }
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct LogCategories {
    #[yaserde(rename = "Category")]
    pub categories: Vec<Category>,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Category {
    #[yaserde(attribute = true)]
    pub name: String,
    #[yaserde(attribute = true)]
    pub description: String,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct DefaultExperiment {
    #[yaserde(attribute = true, default = "default_start_time", rename = "startTime")]
    pub start_time: f64,
    #[yaserde(attribute = true, default = "default_stop_time", rename = "stopTime")]
    pub stop_time: f64,
    #[yaserde(attribute = true, default = "default_tolerance", rename = "tolerance")]
    pub tolerance: f64,
}

const fn default_start_time() -> f64 {
    0.0
}

const fn default_stop_time() -> f64 {
    10.0
}
const fn default_tolerance() -> f64 {
    1e-3
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct UnitDefinitions {
    #[yaserde(rename = "Unit")]
    pub units: Vec<Fmi2Unit>,
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct TypeDefinitions {
    #[yaserde(rename = "SimpleType")]
    pub types: Vec<SimpleType>,
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct ModelVariables {
    #[yaserde(rename = "ScalarVariable")]
    pub variables: Vec<ScalarVariable>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "ModelStructure")]
pub struct ModelStructure {
    #[yaserde(rename = "Outputs")]
    pub outputs: UnknownList,

    #[yaserde(rename = "Derivatives", default = "default_wrapper")]
    pub derivatives: UnknownList,

    #[yaserde(rename = "InitialUnknowns", default = "default_wrapper")]
    pub initial_unknowns: UnknownList,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct UnknownList {
    #[yaserde(rename = "Unknown")]
    pub unknowns: Vec<Fmi2VariableDependency>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_description() {
        let s = r##"<?xml version="1.0" encoding="UTF8"?>
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
</fmiModelDescription>"##;
        let md: Fmi2ModelDescription = yaserde::de::from_str(s).unwrap();
        assert_eq!(md.fmi_version, "2.0");
        assert_eq!(md.model_name, "MyLibrary.SpringMassDamper");
        assert_eq!(md.guid, "{8c4e810f-3df3-4a00-8276-176fa3c9f9e0}");
        assert_eq!(
            md.description.as_deref(),
            Some("Rotational Spring Mass Damper System")
        );
        assert_eq!(md.version.as_deref(), Some("1.0"));
        // assert_eq!(x.generation_date_and_time, chrono::DateTime<chrono::Utc>::from)
        assert_eq!(md.variable_naming_convention, Some("structured".to_owned()));
        assert_eq!(md.number_of_event_indicators, 2);
        assert_eq!(md.model_variables.variables.len(), 4);

        let outputs = &md.model_structure.outputs.unknowns;
        assert_eq!(outputs.len(), 2);
        assert_eq!(outputs[0].index, 1);
        assert_eq!(outputs[0].dependencies, vec![1, 2]);
        assert_eq!(outputs[1].index, 2);
        assert!(outputs[1].dependencies.is_empty());

        let derivatives = &md.model_structure.derivatives.unknowns;
        assert_eq!(derivatives.len(), 1);
        assert_eq!(derivatives[0].index, 4);
        assert_eq!(derivatives[0].dependencies, vec![1, 2]);
    }
}
