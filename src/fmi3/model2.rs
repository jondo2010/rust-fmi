use std::collections::BTreeMap;

use libc::size_t;
use slotmap::{new_key_type, SlotMap};
use yaserde_derive::YaDeserialize;

new_key_type! {
    pub struct UnitKey;
    pub struct TypeKey;
    pub struct VariableKey;
}

#[derive(Debug)]
pub enum Type {
    Float64Type {
        name: String,
        description: Option<String>,
        quantity: Option<String>,
        unit: Option<UnitKey>,
        min: Option<f64>,
        max: Option<f64>,
        nominal: Option<f64>,
    },
}

impl Type {
    fn new(raw_type: raw::Type, unit_map: &BTreeMap<String, UnitKey>) -> Self {
        match raw_type {
            raw::Type::Float32Type { name, description } => todo!(),
            raw::Type::Float64Type {
                name,
                description,
                quantity,
                unit,
                min,
                max,
                nominal,
            } => Type::Float64Type {
                name,
                description,
                quantity,
                unit: unit.map(|ref unit_name| unit_map[unit_name]),
                min,
                max,
                nominal,
            },
        }
    }
}

#[derive(Debug)]
pub struct ModelDescription {
    /// Version of FMI that was used to generate the XML file.
    pub fmi_version: String,
    /// The name of the model as used in the modeling environment that generated the XML file, such as Modelica.Mechanics.Rotational.Examples.CoupledClutches.
    pub model_name: String,
    /// Optional string with a brief description of the model.
    pub description: String,
    /// The instantiationToken is a string that can be used by the FMU to check that the XML file is compatible with the implementation of the FMU.
    pub instantiation_token: String,
    /// String with the name and organization of the model author.
    pub author: Option<String>,
    /// Version of the model [for example 1.0].
    pub version: Option<String>,
    /// Information on the intellectual property copyright for this FMU [for example © My Company 2011].
    pub copyright: Option<String>,
    /// Information on the intellectual property licensing for this FMU [for example BSD license <license text or link to license>].
    pub license: Option<String>,
    /// Name of the tool that generated the XML file.
    pub generation_tool: Option<String>,
    ///  Date and time when the XML file was generated. The format is a subset of dateTime and should be: YYYY-MM-DDThh:mm:ssZ (with one T between date and time; Z characterizes the Zulu time zone, in other words, Greenwich meantime) [for example 2009-12-08T14:33:22Z].
    pub generation_date_and_time: Option<String>,
    /// Defines whether the variable names in <ModelVariables> and in <TypeDefinitions> follow a particular convention.
    pub variable_naming_convention: raw::VariableNamingConvention,

    units: SlotMap<UnitKey, raw::Unit>,
    types: SlotMap<TypeKey, Type>,
}

impl From<raw::ModelDescription> for ModelDescription {
    fn from(model_description: raw::ModelDescription) -> Self {
        let raw::ModelDescription {
            fmi_version,
            model_name,
            description,
            instantiation_token,
            author,
            version,
            copyright,
            license,
            generation_tool,
            generation_date_and_time,
            variable_naming_convention,

            unit_definitions,
            type_definitions,

            model_exchange,
            co_simulation,
            scheduled_execution,
            default_experiment
        } = model_description;

        let mut units: SlotMap<UnitKey, _> = SlotMap::with_key();
        let mut types: SlotMap<TypeKey, _> = SlotMap::with_key();

        let unit_map: BTreeMap<_, _> = unit_definitions
            .map(|unit_definitions| {
                unit_definitions
                    .units
                    .into_iter()
                    .map(|unit| (unit.name.clone(), units.insert(unit)))
                    .collect()
            })
            .unwrap_or_default();

        let type_map: BTreeMap<_, _> = type_definitions
            .map(|type_definition| {
                type_definition
                    .types
                    .into_iter()
                    .map(|ty| (ty.name().to_owned(), types.insert(Type::new(ty, &unit_map))))
                    .collect()
            })
            .unwrap_or_default();

        ModelDescription {
            fmi_version,
            model_name,
            description,
            instantiation_token,
            author,
            version,
            copyright,
            license,
            generation_tool,
            generation_date_and_time,
            variable_naming_convention,
            units,
            types,
        }
    }
}

mod raw {
    use super::*;

    #[derive(PartialEq, Debug, YaDeserialize)]
    pub enum VariableNamingConvention {
        /// A list of strings (the default).
        #[yaserde(rename = "flat")]
        Flat,
        /// Hierarchical names with . as hierarchy separator, and with array elements and derivative characterization.
        #[yaserde(rename = "structured")]
        Structured,
    }

    impl Default for VariableNamingConvention {
        fn default() -> Self {
            Self::Flat
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
        pub variable_naming_convention: VariableNamingConvention,
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
        Float32Type {
            #[yaserde(attribute)]
            name: String,
            #[yaserde(attribute)]
            description: Option<String>,
        },
        #[yaserde(tag = "Float64Type")]
        Float64Type {
            #[yaserde(attribute)]
            name: String,
            #[yaserde(attribute)]
            description: Option<String>,
            #[yaserde(attribute)]
            quantity: Option<String>,
            #[yaserde(attribute)]
            unit: Option<String>,
            #[yaserde(attribute)]
            min: Option<f64>,
            #[yaserde(attribute)]
            max: Option<f64>,
            #[yaserde(attribute)]
            nominal: Option<f64>,
        },
    }

    impl Type {
        pub fn name(&self) -> &str {
            match self {
                Type::Float32Type { name, description } => todo!(),
                Type::Float64Type { name, .. } => &name,
            }
        }
    }

    impl Default for Type {
        fn default() -> Self {
            todo!()
        }
    }
    
    #[derive(PartialEq, Debug, YaDeserialize)]
    struct ModelVariables {
        pub variables: Vec<Variable>,
    }

    enum Causality {
        /// A data value that is constant during the simulation
        Parameter,
        /// A data value that is constant during the simulation and is computed during initialization or when tunable parameters change.
        CalculatedParameter,
        /// The variable value can be provided by the importer.
        Input,
        /// The variable value can be used by the importer.
        Output,
        Local,
        /// The independent variable (usually time [but could also be, for example, angle]).
        Independent,
        /// The variable value can only be changed in Configuration Mode or Reconfiguration Mode.
        StructuralParameter,
    }

impl Default for Causality {
    /// The default of causality is local.
    fn default() -> Self {
        Self::Local
    }
}

    struct Poo {
        /// The full, unique name of the variable.
        name: String,
        /// A handle of the variable to efficiently identify the variable value in the model interface and for references within the modelDescription.xml
        valueReference: u32,
        /// An optional description string describing the meaning of the variable.
        description: Option<String>,
        /// Enumeration that defines the causality of the variable.
        causality: Causality,
    }


    #[derive(PartialEq, Debug, YaDeserialize)]
    enum Variable {
        Float32 { },
        Float64 { },
        Int8,
        UInt8,
        Int16,
        UInt16,
        Int32,
        UInt32,
        Int64,
        Uint64,
        Boolean,
        String,
        Binary,
        Enumeration,
        Clock,
    }

    impl Default for Variable {
        fn default() -> Self {
            todo!()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{raw, ModelDescription};

    #[test]
    fn test_model_descr() {
        let meta_content = std::env::current_dir()
            .map(|path| path.join("tests/data/FMI3.xml"))
            .and_then(std::fs::read_to_string)
            .unwrap();

        let meta: raw::ModelDescription = yaserde::de::from_str(&meta_content).unwrap();

        let meta2 = ModelDescription::from(meta);

        dbg!(meta2);
    }
}
