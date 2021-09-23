use std::collections::BTreeMap;

use libc::size_t;
use slotmap::SlotMap;
use yaserde_derive::YaDeserialize;

pub struct Model {}

impl From<ModelDescription> for Model {
    fn from(model_description: ModelDescription) -> Self {
        let mut units = SlotMap::new();

        let unit_map: BTreeMap<_, _> = model_description
            .unit_definitions
            .map(|unit_definitions| {
                unit_definitions
                    .units
                    .into_iter()
                    .map(|unit| {
                        (unit.name.clone(), units.insert(unit))
                    })
                    .collect()
            })
            .unwrap_or_default();

        let type_map: BTreeMap<_, _> = model_description
            .type_definitions
            .map(|type_definition| {
                type_definition
                    .types
                    .into_iter()
                    .map(|ty| (ty.name().to_owned(), ty))
                    .collect()
            })
            .unwrap_or_default();

        Model {}
    }
}

#[derive(Default, PartialEq, Debug, YaDeserialize)]
#[yaserde(rename = "fmiModelDescription")]
pub struct ModelDescription {
    /// Version of FMI that was used to generate the XML file.
    #[yaserde(attribute, rename = "fmiVersion")]
    pub fmi_version: String,
    /// The name of the model as used in the modeling environment that generated the XML file, such as Modelica.Mechanics.Rotational.Examples.CoupledClutches.
    #[yaserde(attribute, rename = "modelName")]
    pub model_name: String,
    /// Optional string with a brief description of the model.
    #[yaserde(attribute)]
    pub description: String,
    /// The instantiationToken is a string that can be used by the FMU to check that the XML file is compatible with the implementation of the FMU.
    #[yaserde(attribute, rename = "instantiationToken")]
    pub instantiation_token: String,
    /// If present, the FMU is based on FMI for Model Exchange
    #[yaserde(child, rename = "ModelExchange")]
    pub model_exchange: Option<ModelExchange>,
    /// If present, the FMU is based on FMI for Co-Simulation
    #[yaserde(child, rename = "CoSimulation")]
    pub co_simulation: Option<CoSimulation>,
    /// If present, the FMU is based on FMI for Scheduled Execution
    #[yaserde(child, rename = "ScheduledExecution")]
    pub scheduled_execution: Option<ScheduledExecution>,
    /// Providing default settings for the integrator, such as stop time and relative tolerance.
    #[yaserde(child, rename = "DefaultExperiment")]
    pub default_experiment: Option<DefaultExperiment>,
    /// A global list of unit and display unit definitions
    #[yaserde(child, rename = "UnitDefinitions")]
    pub unit_definitions: Option<UnitDefinitions>,
    /// A global list of type definitions that are utilized in `ModelVariables`
    #[yaserde(child, rename = "TypeDefinitions")]
    pub type_definitions: Option<TypeDefinitions>,
}

#[derive(Default, PartialEq, Debug, YaDeserialize)]
pub struct ModelExchange {
    #[yaserde(attribute, rename = "modelIdentifier")]
    pub model_identifier: String,
    #[yaserde(attribute, rename = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: bool,
    #[yaserde(attribute, rename = "canSerializeFMUState")]
    pub can_serialize_fmu_state: bool,
}

#[derive(Default, PartialEq, Debug, YaDeserialize)]
pub struct CoSimulation {
    #[yaserde(attribute, rename = "modelIdentifier")]
    pub model_identifier: String,
    #[yaserde(attribute, rename = "canGetAndSetFMUState")]
    pub can_get_and_set_fmu_state: bool,
    #[yaserde(attribute, rename = "canSerializeFMUState")]
    pub can_serialize_fmu_state: bool,
    #[yaserde(attribute, rename = "canHandleVariableCommunicationStepSize")]
    pub can_handle_variable_communication_step_size: bool,
    #[yaserde(attribute, rename = "providesIntermediateUpdate")]
    pub provides_intermediate_update: bool,
    #[yaserde(attribute, rename = "canReturnEarlyAfterIntermediateUpdate")]
    pub can_return_early_after_intermediate_update: bool,
    #[yaserde(attribute, rename = "fixedInternalStepSize")]
    pub fixed_internal_step_size: f64,
}

#[derive(Default, PartialEq, Debug, YaDeserialize)]
pub struct ScheduledExecution {
    #[yaserde(attr = "modelIdentifier")]
    pub model_identifier: String,
}

#[derive(Default, PartialEq, Debug, YaDeserialize)]
pub struct DefaultExperiment {
    #[yaserde(attribute, rename = "startTime")]
    pub start_time: Option<f64>,
    #[yaserde(attribute, rename = "stopTime")]
    pub stop_time: Option<f64>,
    #[yaserde(attribute, rename = "tolerance")]
    pub tolerange: Option<f64>,
    #[yaserde(attribute, rename = "stepSize")]
    pub step_size: Option<f64>,
}

/// https://fmi-standard.org/docs/3.0-dev/#_definition_of_units
#[derive(Default, PartialEq, Debug, YaDeserialize)]
pub struct UnitDefinitions {
    #[yaserde(child, rename = "Unit")]
    pub units: Vec<Unit>,
}

#[derive(Default, PartialEq, Debug, YaDeserialize)]
pub struct Unit {
    #[yaserde(attribute)]
    pub name: String,
    #[yaserde(child, rename = "BaseUnit")]
    pub base_unit: BaseUnit,
}

fn default_factor() -> f64 {
    1.0
}

/// The Unit definition consists of the exponents of the 7 SI base units kg, m, s, A, K, mol, cd, the exponent of the SI derived unit rad, and optionally a factor and an offset.
#[derive(Default, PartialEq, Debug, YaDeserialize)]
pub struct BaseUnit {
    #[yaserde(attribute)]
    pub kg: i32,
    #[yaserde(attribute)]
    pub m: i32,
    #[yaserde(attribute)]
    pub s: i32,
    #[yaserde(attribute)]
    pub A: i32,
    #[yaserde(attribute)]
    pub K: i32,
    #[yaserde(attribute)]
    pub mol: i32,
    #[yaserde(attribute)]
    pub cd: i32,
    #[yaserde(attribute)]
    pub rad: i32,
    #[yaserde(attribute, default = "default_factor")]
    pub factor: f64,
    #[yaserde(attribute)]
    pub offset: f64,
}

#[derive(Default, PartialEq, Debug, YaDeserialize)]
pub struct TypeDefinitions {
    #[yaserde(child = "Float64Type")]
    pub types: Vec<Type>,
}

#[derive(PartialEq, Debug, YaDeserialize)]
pub enum Type {
    #[yaserde(tag = "Float64Type")]
    Float64Type {
        #[yaserde(attr = "name")]
        name: String,
        #[yaserde(attr = "description")]
        description: Option<String>,
        #[yaserde(attr = "quantity")]
        quantity: Option<String>,
        #[yaserde(attr = "unit")]
        unit: Option<String>,
    },
}

impl Type {
    pub fn name(&self) -> &str {
        match self {
            Type::Float64Type { name, .. } => &name,
        }
    }
}

impl Default for Type {
    fn default() -> Self {
        todo!()
    }
}
