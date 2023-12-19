use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{Causality, CoSimulation, ModelExchange, ScalarVariableElement, Variability};

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct FmiModelDescription {
    /// Version of FMI that was used to generate the XML file.
    #[yaserde(attribute, rename = "fmiVersion")]
    pub fmi_version: String,

    /// The name of the model as used in the modeling environment that generated the XML file, such as Modelica.Mechanics.Rotational.Examples.CoupledClutches.
    #[yaserde(attribute, rename = "modelName")]
    pub model_name: String,

    #[yaserde(attribute)]
    pub guid: String,

    #[yaserde(attribute)]
    pub description: String,

    #[yaserde(attribute)]
    pub version: String,

    /// time/date of database creation according to ISO 8601 (preference: YYYY-MM-DDThh:mm:ss)
    /// Date and time when the XML file was generated. The format is a subset of dateTime and should be: YYYY-MM-DDThh:mm:ssZ (with one T between date and time; Z characterizes the Zulu time zone, in other words, Greenwich meantime) [for example 2009-12-08T14:33:22Z].
    #[yaserde(attribute, rename = "generationDateAndTime")]
    pub generation_date_and_time: Option<String>,

    /// Name of the tool that generated the XML file.
    #[yaserde(attribute, rename = "generationTool")]
    pub generation_tool: String,

    /// Defines whether the variable names in <ModelVariables> and in <TypeDefinitions> follow a particular convention.
    #[yaserde(attribute, rename = "variableNamingConvention")]
    pub variable_naming_convention: Option<String>,

    #[yaserde(attribute, rename = "numberOfEventIndicators")]
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
    model_variables: ModelVariables,

    #[yaserde(rename = "ModelStructure")]
    model_structure: ModelStructure,
}

#[derive(Debug, Default)]
pub struct Counts {
    pub num_constants: usize,
    pub num_parameters: usize,
    pub num_discrete: usize,
    pub num_continuous: usize,
    pub num_inputs: usize,
    pub num_outputs: usize,
    pub num_local: usize,
    pub num_independent: usize,
    pub num_calculated_parameters: usize,
    pub num_real_vars: usize,
    pub num_integer_vars: usize,
    pub num_enum_vars: usize,
    pub num_bool_vars: usize,
    pub num_string_vars: usize,
}

#[cfg(feature = "disabled")]
impl FmiModelDescription {
    /// The model name
    pub fn model_name(&self) -> &str {
        &self.model_name
    }

    // pub fn model_identifier(&self) -> &str {
    // &self.model_exchange
    // }

    /// Collect counts of variables in the model
    pub fn model_counts(&self) -> Counts {
        self.model_variables
            .variables
            .map
            .values()
            .fold(Counts::default(), |mut cts, ref sv| {
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

    /// Total number of variables
    pub fn num_variables(&self) -> usize {
        self.model_variables.map.len()
    }

    /// Get the number of continuous states (and derivatives)
    pub fn num_states(&self) -> usize {
        self.model_structure.derivatives.unknowns.len()
    }

    pub fn num_event_indicators(&self) -> usize {
        self.number_of_event_indicators as usize
    }

    /// Get a iterator of the SalarVariables
    pub fn get_model_variables(
        &self,
    ) -> impl Iterator<Item = (&binding::fmi2ValueReference, &ScalarVariable)> {
        self.model_variables.map.iter()
    }

    pub fn get_model_variable_by_vr(
        &self,
        vr: binding::fmi2ValueReference,
    ) -> Option<&ScalarVariable> {
        self.model_variables.map.get(&vr)
    }

    // pub fn model_variable_by

    /// Turns an UnknownList into a nested Vector of ScalarVariables and their Dependencies
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
    pub fn outputs(&self) -> Result<Vec<UnknownsTuple>, ModelDescriptionError> {
        self.map_unknowns(&self.model_structure.outputs)
    }

    /// Get a reference to the vector of Unknowns marked as derivatives
    pub fn derivatives(&self) -> Result<Vec<UnknownsTuple>, ModelDescriptionError> {
        self.map_unknowns(&self.model_structure.derivatives)
    }

    /// Get a reference to the vector of Unknowns marked as initial_unknowns
    pub fn initial_unknowns(&self) -> Result<Vec<UnknownsTuple>, ModelDescriptionError> {
        self.map_unknowns(&self.model_structure.initial_unknowns)
    }

    /// This private function is used to de-reference variable indices from the UnknownList and
    /// Real{derivative}
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
    #[yaserde(default, deserialize_with = "t_from_str")]
    pub start_time: f64,

    #[yaserde(default = "default_stop_time", deserialize_with = "t_from_str")]
    pub stop_time: f64,

    #[yaserde(default = "default_tolerance", deserialize_with = "t_from_str")]
    pub tolerance: f64,
}

fn default_stop_time() -> f64 {
    10.0
}
fn default_tolerance() -> f64 {
    1e-3
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct UnitDefinitions {
    #[yaserde(default, rename = "$value")]
    pub units: Vec<Unit>,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename_all = "camelCase")]
pub struct Unit {
    #[yaserde(attribute)]
    pub name: String,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename_all = "camelCase")]
pub struct TypeDefinitions {
    #[yaserde(default, rename = "$value")]
    pub types: Vec<SimpleType>,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename_all = "camelCase")]
pub struct SimpleType {}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
struct ModelVariables {
    #[yaserde(default, rename = "$value")]
    variables: Vec<ScalarVariable>,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "ModelStructure", rename_all = "PascalCase")]
struct ModelStructure {
    #[yaserde(default)]
    pub outputs: UnknownList,

    #[yaserde(default)]
    pub derivatives: UnknownList,

    #[yaserde(default)]
    pub initial_unknowns: UnknownList,
}
