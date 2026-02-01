use crate::{Error, traits::FmiModelDescription};

use super::{
    CoSimulation, Fmi2Unit, Fmi2VariableDependency, ModelExchange, ScalarVariable, SimpleType,
};

#[derive(Default, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(
    tag = "fmiModelDescription",
    strict(unknown_attribute, unknown_element)
)]
pub struct Fmi2ModelDescription {
    /// Version of FMI (Clarification for FMI 2.0.2: for FMI 2.0.x revisions fmiVersion is defined
    /// as "2.0").
    #[xml(attr = "fmiVersion")]
    pub fmi_version: String,

    /// The name of the model as used in the modeling environment that generated the XML file, such
    /// as Modelica.Mechanics.Rotational.Examples.CoupledClutches.
    #[xml(attr = "modelName")]
    pub model_name: String,

    /// Fingerprint of xml-file content to verify that xml-file and C-functions are compatible to
    /// each other
    #[xml(attr = "guid")]
    pub guid: String,

    #[xml(attr = "description")]
    pub description: Option<String>,

    /// Version of FMU, e.g., "1.4.1"
    #[xml(attr = "version")]
    pub version: Option<String>,

    /// Information on intellectual property copyright for this FMU, such as "© MyCompany 2011"
    #[xml(attr = "copyright")]
    pub copyright: Option<String>,

    /// Information on intellectual property licensing for this FMU, such as "BSD license",
    /// "Proprietary", or "Public Domain"
    #[xml(attr = "license")]
    pub license: Option<String>,

    /// Name of the tool that generated the XML file.
    #[xml(attr = "generationTool")]
    pub generation_tool: Option<String>,

    /// time/date of database creation according to ISO 8601 (preference: YYYY-MM-DDThh:mm:ss)
    /// Date and time when the XML file was generated. The format is a subset of dateTime and
    /// should be: YYYY-MM-DDThh:mm:ssZ (with one T between date and time; Z characterizes the
    /// Zulu time zone, in other words, Greenwich meantime) [for example 2009-12-08T14:33:22Z].
    #[xml(attr = "generationDateAndTime")]
    pub generation_date_and_time: Option<String>,

    /// Defines whether the variable names in `<ModelVariables>` and in `<TypeDefinitions>` follow a
    /// particular convention.
    #[xml(attr = "variableNamingConvention")]
    pub variable_naming_convention: Option<String>,

    /// FMI 2.0: Required for ModelExchange, ignored for Co-Simulation (may be absent).
    #[xml(attr = "numberOfEventIndicators")]
    pub number_of_event_indicators: Option<u32>,

    /// If present, the FMU is based on FMI for Model Exchange
    #[xml(child = "ModelExchange")]
    pub model_exchange: Option<ModelExchange>,

    /// If present, the FMU is based on FMI for Co-Simulation
    #[xml(child = "CoSimulation")]
    pub co_simulation: Option<CoSimulation>,

    #[xml(child = "LogCategories")]
    pub log_categories: Option<LogCategories>,

    #[xml(child = "DefaultExperiment")]
    pub default_experiment: Option<DefaultExperiment>,

    #[xml(child = "UnitDefinitions")]
    pub unit_definitions: Option<UnitDefinitions>,

    #[xml(child = "TypeDefinitions")]
    pub type_definitions: Option<TypeDefinitions>,

    #[xml(child = "ModelVariables", default)]
    pub model_variables: ModelVariables,

    #[xml(child = "ModelStructure", default)]
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
        self.number_of_event_indicators.unwrap_or(0) as usize
    }

    /// Get a iterator of the SalarVariables
    pub fn get_model_variables(&self) -> impl Iterator<Item = &ScalarVariable> {
        self.model_variables.variables.iter()
    }

    #[cfg(false)]
    pub fn get_model_variable_by_vr(&self, vr: u32) -> Option<&ScalarVariable> {
        self.model_variables.map.get(&vr)
    }

    /// Turns an UnknownList into a nested Vector of ScalarVariables and their Dependencies
    #[cfg(false)]
    fn map_unknowns(
        &self,
        list: &UnknownList,
    ) -> Result<Vec<UnknownsTuple>, ModelDescriptionError> {
        list.unknowns
            .iter()
            .map(|unknown| {
                self.model_variables
                    .by_index
                    // Variable indices start at 1 in the modelDescription
                    .get(unknown.index as usize - 1)
                    .map(|vr| &self.model_variables.map[vr])
                    .ok_or_else(|| {
                        ModelDescriptionError::VariableAtIndexNotFound(
                            self.model_name.clone(),
                            unknown.index as usize,
                        )
                    })
                    .and_then(|var| {
                        let deps = unknown
                            .dependencies
                            .iter()
                            .map(|dep| {
                                self.model_variables
                                    .by_index
                                    .get(*dep as usize - 1)
                                    .map(|vr| &self.model_variables.map[vr])
                                    .ok_or_else(|| {
                                        ModelDescriptionError::VariableAtIndexNotFound(
                                            self.model_name.clone(),
                                            *dep as usize,
                                        )
                                    })
                            })
                            .collect::<Result<Vec<_>, ModelDescriptionError>>()?;

                        Ok((var, deps))
                    })
            })
            .collect()
    }

    /// Get a reference to the vector of Unknowns marked as outputs
    #[cfg(false)]
    pub fn outputs(&self) -> Result<Vec<UnknownsTuple>, ModelDescriptionError> {
        self.map_unknowns(&self.model_structure.outputs)
    }

    /// Get a reference to the vector of Unknowns marked as derivatives
    #[cfg(false)]
    pub fn derivatives(&self) -> Result<Vec<UnknownsTuple>, ModelDescriptionError> {
        self.map_unknowns(&self.model_structure.derivatives)
    }

    /// Get a reference to the vector of Unknowns marked as initial_unknowns
    #[cfg(false)]
    pub fn initial_unknowns(&self) -> Result<Vec<UnknownsTuple>, ModelDescriptionError> {
        self.map_unknowns(&self.model_structure.initial_unknowns)
    }

    /// Get a reference to the model variable with the given name
    pub fn model_variable_by_name(&self, name: &str) -> Result<&ScalarVariable, Error> {
        self.model_variables
            .variables
            .iter()
            .find(|var| var.name == name)
            .ok_or_else(|| Error::VariableNotFound(name.to_owned()))
    }

    /// This private function is used to de-reference variable indices from the UnknownList and
    /// Real{derivative}
    #[cfg(false)]
    fn model_variable_by_index(
        &self,
        idx: usize,
    ) -> Result<&ScalarVariable, ModelDescriptionError> {
        self.model_variables
            .by_index
            .get(idx - 1)
            .map(|vr| &self.model_variables.map[vr])
            .ok_or_else(|| {
                ModelDescriptionError::VariableAtIndexNotFound(
                    self.model_name.clone(),
                    idx as usize,
                )
            })
    }

    /// Return a vector of tuples `(&ScalarVariable, &ScalarVariabel)`, where the 1st is a
    /// continuous-time state, and the 2nd is its derivative.
    #[cfg(false)]
    pub fn continuous_states(
        &self,
    ) -> Result<Vec<(&ScalarVariable, &ScalarVariable)>, ModelDescriptionError> {
        self.model_structure
            .derivatives
            .unknowns
            .iter()
            .map(|unknown| {
                self.model_variable_by_index(unknown.index as usize)
                    .and_then(|der| {
                        if let ScalarVariableElement::Real { derivative, .. } = der.elem {
                            derivative
                                .ok_or_else(|| {
                                    ModelDescriptionError::VariableDerivativeMissing(
                                        der.name.clone(),
                                    )
                                })
                                .and_then(|der_idx| {
                                    self.model_variable_by_index(der_idx as usize)
                                        .map(|state| (state, der))
                                })
                        } else {
                            Err(ModelDescriptionError::VariableTypeMismatch(
                                ScalarVariableElementBase::Real,
                                ScalarVariableElementBase::from(&der.elem),
                            ))
                        }
                    })
            })
            .collect()
    }
}

impl FmiModelDescription for Fmi2ModelDescription {
    fn model_name(&self) -> &str {
        &self.model_name
    }

    fn version_string(&self) -> &str {
        &self.fmi_version
    }

    fn deserialize(xml: &str) -> Result<Self, crate::Error> {
        hard_xml::XmlRead::from_str(xml).map_err(crate::Error::XmlParse)
    }

    fn serialize(&self) -> Result<String, crate::Error> {
        hard_xml::XmlWrite::to_string(self).map_err(crate::Error::XmlParse)
    }
}

#[derive(Clone, Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "LogCategories", strict(unknown_attribute, unknown_element))]
pub struct LogCategories {
    #[xml(child = "Category")]
    pub categories: Vec<Category>,
}

#[derive(Clone, Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Category", strict(unknown_attribute, unknown_element))]
pub struct Category {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "description")]
    pub description: String,
}

#[derive(Clone, Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "DefaultExperiment")]
pub struct DefaultExperiment {
    /// Default start time of simulation
    #[xml(attr = "startTime")]
    pub start_time: Option<f64>,
    /// Default stop time of simulation
    #[xml(attr = "stopTime")]
    pub stop_time: Option<f64>,
    /// Default relative integration tolerance
    #[xml(attr = "tolerance")]
    pub tolerance: Option<f64>,
    /// ModelExchange: Default step size for fixed step integrators.
    /// CoSimulation: Preferred communicationStepSize.
    #[xml(attr = "stepSize")]
    pub step_size: Option<f64>,
}

impl DefaultExperiment {
    pub fn start_time(&self) -> f64 {
        self.start_time.unwrap_or(0.0)
    }

    pub fn stop_time(&self) -> f64 {
        self.stop_time.unwrap_or(10.0)
    }

    pub fn tolerance(&self) -> f64 {
        self.tolerance.unwrap_or(1e-3)
    }

    pub fn step_size(&self) -> Option<f64> {
        self.step_size
    }
}

#[derive(Default, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "UnitDefinitions", strict(unknown_attribute, unknown_element))]
pub struct UnitDefinitions {
    #[xml(child = "Unit")]
    pub units: Vec<Fmi2Unit>,
}

#[derive(Default, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "TypeDefinitions", strict(unknown_attribute, unknown_element))]
pub struct TypeDefinitions {
    #[xml(child = "SimpleType")]
    pub types: Vec<SimpleType>,
}

#[derive(Default, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "ModelVariables", strict(unknown_attribute, unknown_element))]
pub struct ModelVariables {
    #[xml(child = "ScalarVariable")]
    pub variables: Vec<ScalarVariable>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "ModelStructure", strict(unknown_attribute, unknown_element))]
pub struct ModelStructure {
    #[xml(child = "Outputs", default)]
    pub outputs: Outputs,

    #[xml(child = "Derivatives", default)]
    pub derivatives: Derivatives,

    #[xml(child = "InitialUnknowns", default)]
    pub initial_unknowns: InitialUnknowns,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Outputs")]
pub struct Outputs {
    #[xml(child = "Unknown")]
    pub unknowns: Vec<Fmi2VariableDependency>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Derivatives")]
pub struct Derivatives {
    #[xml(child = "Unknown")]
    pub unknowns: Vec<Fmi2VariableDependency>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "InitialUnknowns")]
pub struct InitialUnknowns {
    #[xml(child = "Unknown")]
    pub unknowns: Vec<Fmi2VariableDependency>,
}

#[cfg(test)]
mod tests {
    use hard_xml::XmlRead;

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
        let md = Fmi2ModelDescription::from_str(&s).unwrap();
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
        assert_eq!(md.number_of_event_indicators, Some(2));
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
