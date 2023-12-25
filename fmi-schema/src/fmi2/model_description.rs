use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{
    CoSimulation, Fmi2Unit, Fmi2VariableDependency, ModelExchange, ScalarVariable, SimpleType,
};

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct FmiModelDescription {
    /// Version of FMI (Clarification for FMI 2.0.2: for FMI 2.0.x revisions fmiVersion is defined as "2.0").
    #[yaserde(attribute, rename = "fmiVersion")]
    pub fmi_version: String,

    /// The name of the model as used in the modeling environment that generated the XML file, such as
    /// Modelica.Mechanics.Rotational.Examples.CoupledClutches.
    #[yaserde(attribute, rename = "modelName")]
    pub model_name: String,

    /// Fingerprint of xml-file content to verify that xml-file and C-functions are compatible to each other
    #[yaserde(attribute)]
    pub guid: String,

    #[yaserde(attribute)]
    pub description: Option<String>,

    /// Version of FMU, e.g., "1.4.1"
    #[yaserde(attribute)]
    pub version: Option<String>,

    /// Information on intellectual property copyright for this FMU, such as “© MyCompany 2011“
    #[yaserde(attribute)]
    pub copyright: Option<String>,

    /// Information on intellectual property licensing for this FMU, such as “BSD license”, "Proprietary", or "Public
    /// Domain"
    #[yaserde(attribute)]
    pub license: Option<String>,

    /// Name of the tool that generated the XML file.
    #[yaserde(attribute, rename = "generationTool")]
    pub generation_tool: String,

    /// time/date of database creation according to ISO 8601 (preference: YYYY-MM-DDThh:mm:ss)
    /// Date and time when the XML file was generated. The format is a subset of dateTime and should be:
    /// YYYY-MM-DDThh:mm:ssZ (with one T between date and time; Z characterizes the Zulu time zone, in other words,
    /// Greenwich meantime) [for example 2009-12-08T14:33:22Z].
    #[yaserde(attribute, rename = "generationDateAndTime")]
    pub generation_date_and_time: Option<String>,

    /// Defines whether the variable names in <ModelVariables> and in <TypeDefinitions> follow a particular convention.
    #[yaserde(attribute, rename = "variableNamingConvention")]
    pub variable_naming_convention: Option<String>,

    #[yaserde(attribute, rename = "numberOfEventIndicators")]
    pub number_of_event_indicators: u32,

    /// If present, the FMU is based on FMI for Model Exchange
    #[yaserde(child, rename = "ModelExchange")]
    pub model_exchange: Option<ModelExchange>,

    /// If present, the FMU is based on FMI for Co-Simulation
    #[yaserde(child, rename = "CoSimulation")]
    pub co_simulation: Option<CoSimulation>,

    #[yaserde(child, rename = "LogCategories")]
    pub log_categories: Option<LogCategories>,

    #[yaserde(child, rename = "DefaultExperiment")]
    pub default_experiment: Option<DefaultExperiment>,

    #[yaserde(child, rename = "UnitDefinitions")]
    pub unit_definitions: Option<UnitDefinitions>,

    #[yaserde(child, rename = "TypeDefinitions")]
    pub type_definitions: Option<TypeDefinitions>,

    #[yaserde(child, rename = "ModelVariables")]
    pub model_variables: ModelVariables,

    #[yaserde(child, rename = "ModelStructure")]
    pub model_structure: ModelStructure,
}

impl FmiModelDescription {
    /// The model name
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

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

    #[cfg(feature = "disable")]
    pub fn get_model_variable_by_vr(&self, vr: u32) -> Option<&ScalarVariable> {
        self.model_variables.map.get(&vr)
    }

    /// Turns an UnknownList into a nested Vector of ScalarVariables and their Dependencies
    #[cfg(feature = "disable")]
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
    #[cfg(feature = "disable")]
    pub fn outputs(&self) -> Result<Vec<UnknownsTuple>, ModelDescriptionError> {
        self.map_unknowns(&self.model_structure.outputs)
    }

    /// Get a reference to the vector of Unknowns marked as derivatives
    #[cfg(feature = "disable")]
    pub fn derivatives(&self) -> Result<Vec<UnknownsTuple>, ModelDescriptionError> {
        self.map_unknowns(&self.model_structure.derivatives)
    }

    /// Get a reference to the vector of Unknowns marked as initial_unknowns
    #[cfg(feature = "disable")]
    pub fn initial_unknowns(&self) -> Result<Vec<UnknownsTuple>, ModelDescriptionError> {
        self.map_unknowns(&self.model_structure.initial_unknowns)
    }

    /// This private function is used to de-reference variable indices from the UnknownList and
    /// Real{derivative}
    #[cfg(feature = "disable")]
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
    #[cfg(feature = "disable")]
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

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct LogCategories {
    #[yaserde(rename = "Category")]
    pub categories: Vec<Category>,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Category {
    #[yaserde(attribute)]
    pub name: String,

    #[yaserde(attribute)]
    pub description: String,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct DefaultExperiment {
    #[yaserde(attribute, rename = "startTime")]
    pub start_time: f64,

    #[yaserde(attribute, default = "default_stop_time", rename = "stopTime")]
    pub stop_time: f64,

    #[yaserde(attribute, default = "default_tolerance", rename = "tolerance")]
    pub tolerance: f64,
}

fn default_stop_time() -> f64 {
    10.0
}
fn default_tolerance() -> f64 {
    1e-3
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct UnitDefinitions {
    #[yaserde(child, rename = "Unit")]
    pub units: Vec<Fmi2Unit>,
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct TypeDefinitions {
    #[yaserde(child, rename = "SimpleType")]
    pub types: Vec<SimpleType>,
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct ModelVariables {
    #[yaserde(child, rename = "ScalarVariable")]
    pub variables: Vec<ScalarVariable>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "ModelStructure")]
pub struct ModelStructure {
    #[yaserde(child, rename = "Outputs")]
    pub outputs: UnknownList,

    #[yaserde(child, rename = "Derivatives")]
    pub derivatives: UnknownList,

    #[yaserde(child, rename = "InitialUnknowns")]
    pub initial_unknowns: UnknownList,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct UnknownList {
    #[yaserde(child, rename = "Unknown")]
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
        let md: FmiModelDescription = yaserde::de::from_str(s).unwrap();
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

        /*
        let outputs = md.outputs().unwrap();
        assert_eq!(outputs[0].0.name, "x[1]");
        assert_eq!(outputs[0].1.len(), 2);
        assert_eq!(outputs[0].1[0].name, "x[1]");
        assert_eq!(outputs[1].0.name, "x[2]");
        assert_eq!(outputs[1].1.len(), 0);

        let derivatives = md.derivatives().unwrap();
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

        let states = md.continuous_states().unwrap();
        assert_eq!(
            states
                .iter()
                .map(|(der, state)| (der.name.as_str(), state.name.as_str()))
                .collect::<Vec<(_, _)>>(),
            vec![("PI.x", "der(PI.x)")]
        );
        */
    }
}
