use super::fmi;
/// This module implements the ModelDescription datamodel and provides
/// attributes to serde_xml_rs to generate an XML deserializer.
use derive_more::Display;
use serde::{de, Deserialize, Deserializer};
use std::{
    collections::{BTreeMap, BTreeSet},
    str::FromStr,
};
use thiserror::Error;

// Re-exports
pub use serde_xml_rs::from_reader;

/// Generic parsing function
fn t_from_str<'de, T, D>(deser: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let s = <String>::deserialize(deser)?;
    T::from_str(&s).map_err(de::Error::custom)
}

/// Parse a string list into a Vec<T>
fn vec_from_str<'de, T, D>(deser: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    let s = <String>::deserialize(deser)?;
    if s.is_empty() {
        return Ok(Vec::<T>::new());
    }
    s.split(' ')
        .map(|i| T::from_str(&i).map_err(de::Error::custom))
        .collect()
}

fn dtparse_from_str<'de, D>(deser: D) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    use chrono::{DateTime, Utc};

    let s = <String>::deserialize(deser)?;
    dtparse::parse(&s)
        .map_err(|e| de::Error::custom(format!("{:?}", e)))
        .map(|dt| DateTime::<Utc>::from_utc(dt.0, Utc))
    //( chrono::naive::NaiveDateTime, Option<chrono::offset::FixedOffset>,)
}

#[derive(Debug, Error, PartialEq)]
pub enum ModelDescriptionError {
    #[error("ScalarVariable at index {} not found in Model '{}'.", .1, .0)]
    VariableAtIndexNotFound(String, usize),

    #[error("ScalarVariable '{}' not found in Model '{}'.", name, model)]
    VariableNotFound { model: String, name: String },

    #[error("Mismatched variable type: expected {} but found {}", .0, .1)]
    VariableTypeMismatch(ScalarVariableElementBase, ScalarVariableElementBase),

    #[error("ScalarVariable '{}' does not define a derivative.", .0)]
    VariableDerivativeMissing(String),
}

// fmiModelDescription
#[derive(Debug, Deserialize)]
#[serde(rename = "fmiModelDescription", rename_all = "camelCase")]
pub struct ModelDescription {
    pub fmi_version: String,
    pub model_name: String,
    pub guid: String,

    #[serde(default)]
    pub description: String,

    #[serde(default)]
    pub version: String,

    // #[serde(with = "parse_util::odr_dateformat", default = "Header::default_date")]
    /// time/date of database creation according to ISO 8601 (preference: YYYY-MM-DDThh:mm:ss)
    #[serde(deserialize_with = "dtparse_from_str")]
    pub generation_date_and_time: chrono::DateTime<chrono::Utc>,

    #[serde(default)]
    pub generation_tool: String,

    pub variable_naming_convention: String,

    #[serde(deserialize_with = "t_from_str")]
    pub number_of_event_indicators: u32,

    #[serde(rename = "ModelExchange")]
    pub model_exchange: Option<ModelExchange>,

    #[serde(rename = "CoSimulation")]
    pub co_simulation: Option<CoSimulation>,

    #[serde(rename = "LogCategories")]
    pub log_categories: Option<LogCategories>,

    #[serde(rename = "DefaultExperiment")]
    pub default_experiment: Option<DefaultExperiment>,

    #[serde(rename = "UnitDefinitions")]
    pub unit_definitions: Option<UnitDefinitions>,

    #[serde(rename = "TypeDefinitions")]
    pub type_definitions: Option<TypeDefinitions>,

    #[serde(rename = "ModelVariables")]
    model_variables: ModelVariables,

    #[serde(rename = "ModelStructure")]
    model_structure: ModelStructure,
}

pub type ScalarVariableMap<'a> = std::collections::HashMap<String, &'a ScalarVariable>;
pub type UnknownsTuple<'a> = (&'a ScalarVariable, Vec<&'a ScalarVariable>);

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

impl ModelDescription {
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
    pub fn get_model_variables(&self) -> impl Iterator<Item = (&ValueReference, &ScalarVariable)> {
        self.model_variables.map.iter()
    }

    pub fn get_model_variable_by_vr(&self, vr: ValueReference) -> Option<&ScalarVariable> {
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelExchange {
    // Short class name according to C-syntax
    pub model_identifier: String,

    #[serde(default)]
    // If true, a tool is needed to execute the model and the FMU just contains the communication
    // to this tool.
    pub needs_execution_tool: bool,

    #[serde(default)]
    pub completed_integrator_step_not_needed: bool,

    #[serde(default)]
    pub can_be_instantiated_only_once_per_process: bool,

    #[serde(default)]
    pub can_not_use_memory_management_functions: bool,

    #[serde(default, rename = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: bool,

    #[serde(default, rename = "canSerializeFMUState")]
    pub can_serialize_fmu_state: bool,

    #[serde(default)]
    // If true, the directional derivative of the equations can be computed with
    // fmi2GetDirectionalDerivative
    pub provides_directional_derivative: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CoSimulation {
    // Short class name according to C-syntax
    pub model_identifier: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogCategories {
    #[serde(default, rename = "$value")]
    pub categories: Vec<Category>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub name: String,

    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitDefinitions {
    #[serde(default, rename = "$value")]
    pub units: Vec<Unit>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimpleType {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeDefinitions {
    #[serde(default, rename = "$value")]
    pub types: Vec<SimpleType>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DefaultExperiment {
    #[serde(default, deserialize_with = "t_from_str")]
    pub start_time: f64,

    #[serde(default = "default_stop_time", deserialize_with = "t_from_str")]
    pub stop_time: f64,

    #[serde(default = "default_tolerance", deserialize_with = "t_from_str")]
    pub tolerance: f64,
}

fn default_stop_time() -> f64 {
    10.0
}
fn default_tolerance() -> f64 {
    1e-3
}

#[derive(Debug, Display, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Causality {
    Parameter,
    CalculatedParameter,
    Input,
    Output,
    Local,
    Independent,
    Unknown,
}

impl Default for Causality {
    fn default() -> Causality {
        Causality::Unknown
    }
}

/// Enumeration that defines the time dependency of the variable
#[derive(Debug, Display, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Variability {
    Constant,
    Fixed,
    Tunable,
    Discrete,
    Continuous,
    Unknown,
}

impl Default for Variability {
    fn default() -> Variability {
        Variability::Unknown
    }
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "camelCase")]
pub enum Initial {
    Exact,
    Approx,
    Calculated,
}

impl Default for Initial {
    fn default() -> Initial {
        Initial::Exact
    }
}

#[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct ValueReference(
    #[serde(deserialize_with = "t_from_str")] pub(crate) fmi::fmi2ValueReference,
);

#[derive(Debug, Deserialize, Display, Clone)]
#[serde(rename_all = "camelCase")]
#[display(
    fmt = "ScalarVariable {} {{{}, {}, {}}}",
    elem,
    name,
    causality,
    variability
)]
pub struct ScalarVariable {
    /// The full, unique name of the variable.
    pub name: String,

    /// A handle of the variable to efficiently identify the variable value in the model interface.
    //#[serde(deserialize_with = "t_from_str")]
    pub value_reference: ValueReference,

    /// An optional description string describing the meaning of the variable.
    #[serde(default)]
    pub description: String,

    /// Enumeration that defines the causality of the variable.
    #[serde(default)]
    pub causality: Causality,

    /// Enumeration that defines the time dependency of the variable, in other words it defines the
    /// time instants when a variable can change its value.
    #[serde(default)]
    pub variability: Variability,

    /// Enumeration that defines how the variable is initialized. It is not allowed to provide a
    /// value for initial if `causality`=`Input` or `Independent`.
    #[serde(default)]
    pub initial: Initial,

    #[serde(rename = "$value")]
    pub elem: ScalarVariableElement,
}

impl PartialEq for ScalarVariable {
    fn eq(&self, other: &ScalarVariable) -> bool {
        self.value_reference == other.value_reference
    }
}

impl Eq for ScalarVariable {}
impl std::hash::Hash for ScalarVariable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value_reference.hash(state);
    }
}
impl PartialOrd for ScalarVariable {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value_reference.partial_cmp(&other.value_reference)
    }
}
impl Ord for ScalarVariable {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value_reference.cmp(&other.value_reference)
    }
}

impl ScalarVariable {
    pub fn is_continuous_input(&self) -> bool {
        matches!(
            (&self.elem, &self.causality),
            (ScalarVariableElement::Real { .. }, Causality::Input)
        )
    }
}

#[derive(Debug, Deserialize, Display, PartialEq, Clone)]
pub enum ScalarVariableElement {
    #[serde(rename_all = "camelCase")]
    #[display(fmt = "Real({:?},{})", declared_type, start)]
    Real {
        declared_type: Option<String>,

        #[serde(default, deserialize_with = "t_from_str")]
        start: f64,

        #[serde(default, deserialize_with = "t_from_str")]
        relative_quantity: bool,

        //#[serde(default, deserialize_with = "deser_opt")]
        #[serde(default)]
        derivative: Option<u32>,
    },
    #[serde(rename_all = "camelCase")]
    #[display(fmt = "Int({},{})", declared_type, start)]
    Integer {
        #[serde(default)]
        declared_type: String,
        #[serde(default, deserialize_with = "t_from_str")]
        start: i64,
    },
    #[serde(rename_all = "camelCase")]
    #[display(fmt = "Bool({},{})", declared_type, start)]
    Boolean {
        #[serde(default)]
        declared_type: String,
        #[serde(default, deserialize_with = "t_from_str")]
        start: bool,
    },
    #[serde(rename_all = "camelCase")]
    #[display(fmt = "String({},{})", declared_type, start)]
    String {
        #[serde(default)]
        declared_type: String,
        start: String,
    },
    #[serde(rename_all = "camelCase")]
    #[display(fmt = "Enum({},{})", declared_type, start)]
    Enumeration {
        #[serde(default)]
        declared_type: String,
        #[serde(default, deserialize_with = "t_from_str")]
        start: i64,
    },
}

#[derive(Debug, Display, PartialEq)]
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

#[derive(Debug, Deserialize)]
struct ModelVariablesRaw {
    #[serde(default, rename = "$value")]
    variables: Vec<ScalarVariable>,
}

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

#[derive(Debug, Deserialize)]
#[serde(rename = "Unknown")]
pub struct Unknown {
    #[serde(deserialize_with = "t_from_str")]
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

#[derive(Debug, Deserialize)]
#[serde(rename = "ModelStructure", rename_all = "PascalCase")]
struct ModelStructure {
    #[serde(default)]
    pub outputs: UnknownList,

    #[serde(default)]
    pub derivatives: UnknownList,

    #[serde(default)]
    pub initial_unknowns: UnknownList,
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
        assert_eq!(x.variable_naming_convention, "structured");
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
