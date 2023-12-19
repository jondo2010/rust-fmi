use std::str::FromStr;

use itertools::Itertools;
use slotmap::{new_key_type, SecondaryMap, SlotMap};
use thiserror::Error;

use super::{
    schema::{self, AbstractVariableTrait, Category, Fmi3Unit, TypedArrayableariableTrait},
    DateTime,
};

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Error in model description: {0}")]
    ReferenceError(String),
}

new_key_type! { pub struct TypeKey; }
new_key_type! { pub struct UnitKey; }
new_key_type! { pub struct VariableKey; }
new_key_type! { pub struct LogCategoryKey; }

#[derive(Debug)]
struct UnitDefinition<'a> {
    //TODO
    unit: Fmi3Unit<'a>,
}

#[derive(Debug)]
pub enum TypeDefinition {
    Float32(schema::Float32Type),
    Float64 {
        r#type: schema::Float64Type,
        unit: Option<UnitKey>,
    },
}

#[derive(Debug, PartialEq)]
pub enum ModelVariable {
    Float32 {
        var: schema::FmiFloat32,
        declared_type: Option<TypeKey>,
        derivative: Option<VariableKey>,
    },
    Float64 {
        var: schema::FmiFloat64,
        declared_type: Option<TypeKey>,
        derivative: Option<VariableKey>,
    },
}

#[derive(Debug)]
pub struct ModelDescription {
    /// Version of FMI the XML file complies with.
    pub fmi_version: String,
    /// The name of the model as used in the modeling environment that generated the XML file, such as Modelica.Mechanics.Rotational.Examples.CoupledClutches.
    pub model_name: String,
    /// The instantiationToken is a string that may be used by the FMU to check that the XML file is compatible with the implementation of the FMU. For this purpose the importer must pass the instantiationToken from the modelDescription.xml to the fmi3InstantiateXXX function call.
    pub instantiation_token: String,
    /// Optional string with a brief description of the model.
    pub description: Option<String>,
    /// Optional date and time when the XML file was generated.
    pub generation_date_and_time: Option<super::DateTime>,
    /// A list of log categories that can be set to define the log information that is supported from the FMU.
    pub log_categories: SlotMap<LogCategoryKey, Category>,
    /// A global list of unit and display unit definitions
    unit_definitions: SlotMap<UnitKey, UnitDefinition>,
    /// A global list of type definitions that are utilized in `ModelVariables`
    type_definitions: SlotMap<TypeKey, TypeDefinition>,
    pub model_variables: SlotMap<VariableKey, ModelVariable>,

    pub model_structure: ModelStructure,
}

#[derive(Debug)]
pub struct ModelStructure {
    pub outputs: Vec<VariableKey>,
    pub continuous_state_derivatives: Vec<VariableKey>,
    pub derivatives: Vec<VariableKey>,
    pub initial_unknowns: Vec<VariableKey>,
}

fn find_unit_definition_by_name<'a>(
    mut unit_definitions: impl Iterator<Item = (UnitKey, &'a UnitDefinition)>,
    name: &str,
) -> Option<UnitKey> {
    unit_definitions.find_map(|(key, unit)| {
        if unit.unit.name == name {
            Some(key)
        } else {
            None
        }
    })
}

fn find_type_definition_by_name<'a>(
    mut type_definitions: impl Iterator<Item = (TypeKey, &'a TypeDefinition)>,
    name: &str,
) -> Option<TypeKey> {
    type_definitions.find_map(|(key, ty)| match ty {
        TypeDefinition::Float32(inner_ty) if inner_ty.base.name == name => Some(key),
        TypeDefinition::Float64 {
            r#type: inner_ty, ..
        } if inner_ty.base.name == name => Some(key),
        _ => None,
    })
}

fn find_variable_by_name<'a>(
    mut model_variables: impl Iterator<Item = (VariableKey, &'a ModelVariable)>,
    name: &str,
) -> Option<VariableKey> {
    model_variables.find_map(|(key, var)| match var {
        ModelVariable::Float32 { var, .. } if var.name() == name => Some(key),
        ModelVariable::Float64 { var, .. } if var.name() == name => Some(key),
        _ => None,
    })
}

fn find_variable_by_vr<'a>(
    mut model_variables: impl Iterator<Item = (VariableKey, &'a ModelVariable)>,
    vr: u32,
) -> Option<VariableKey> {
    model_variables.find_map(|(key, var)| match var {
        ModelVariable::Float32 { var, .. } if var.value_reference() == vr => Some(key),
        ModelVariable::Float64 { var, .. } if var.value_reference() == vr => Some(key),
        _ => None,
    })
}

fn build_unit_definitions<'a>(
    unit_definitions: schema::UnitDefinitions,
) -> Result<SlotMap<UnitKey, UnitDefinition>, ModelError> {
    let mut map = SlotMap::with_key();
    for unit in unit_definitions.units {
        map.insert(UnitDefinition { unit });
    }
    Ok(map)

    //Ok(md.unit_definitions.as_ref().map_or_else( || SlotMap::default(), |defs| { },))
}

fn build_type_definitions<'a>(
    type_definitions: schema::TypeDefinitions,
    unit_definitions: &SlotMap<UnitKey, UnitDefinition>,
) -> Result<SlotMap<TypeKey, TypeDefinition>, ModelError> {
    let mut map = SlotMap::with_key();
    for ty in type_definitions.float32_type {
        map.insert(TypeDefinition::Float32(ty));
    }
    for ty in type_definitions.float64_type {
        let unit = match &ty.base_attr.unit {
            Some(unit_name) => Some(
                find_unit_definition_by_name(unit_definitions.iter(), unit_name).ok_or(
                    ModelError::ReferenceError(format!(
                        "Unit '{}' not found in TypeDefinition '{}'",
                        unit_name, ty.base.name,
                    )),
                )?,
            ),
            None => None,
        };

        map.insert(TypeDefinition::Float64 { r#type: ty, unit });
    }
    Ok(map)
}

fn build_model_variables<'a>(
    model_variables: schema::ModelVariables,
    type_definitions: &SlotMap<TypeKey, TypeDefinition>,
) -> Result<SlotMap<VariableKey, ModelVariable>, ModelError> {
    let mut ret_map = SlotMap::with_capacity_and_key(model_variables.len());

    // Helper function to dereference a declared type
    let deref_declared_type =
        |declared_type: &str, var_name: &str| -> Result<TypeKey, ModelError> {
            find_type_definition_by_name(type_definitions.iter(), declared_type).ok_or(
                ModelError::ReferenceError(format!(
                    "TypeDefinition '{}' not found in ScalarVariable '{}'",
                    declared_type, var_name
                )),
            )
        };

    // Helper function to find a declared variable
    let variable_by_vr = |model_variables: &SlotMap<VariableKey, _>,
                          variable_vr: u32,
                          var_name: &str|
     -> Result<VariableKey, ModelError> {
        find_variable_by_vr(model_variables.iter(), variable_vr).ok_or(ModelError::ReferenceError(
            format!(
                "Variable '{}' not found in ScalarVariable '{}'",
                variable_vr, var_name
            ),
        ))
    };

    for var in model_variables
        .float32
        .into_iter()
        .sorted_by_key(|var| var.derivative())
    {
        let declared_type = if let Some(type_name) = var.declared_type() {
            Some(deref_declared_type(type_name, var.name())?)
        } else {
            None
        };

        let derivative = if let Some(derivative) = var.derivative() {
            Some(variable_by_vr(&ret_map, derivative, var.name())?)
        } else {
            None
        };

        ret_map.insert(ModelVariable::Float32 {
            var,
            declared_type,
            derivative,
        });
    }

    for var in model_variables
        .float64
        .into_iter()
        .sorted_by_key(|var| var.derivative())
    {
        let x = var
            .declared_type()
            .map(|x| deref_declared_type(x, var.name()));

        let declared_type = if let Some(type_name) = var.declared_type() {
            Some(deref_declared_type(type_name, var.name())?)
        } else {
            None
        };

        let derivative = if let Some(derivative) = var.derivative() {
            Some(variable_by_vr(&ret_map, derivative, var.name())?)
        } else {
            None
        };

        ret_map.insert(ModelVariable::Float64 {
            var,
            declared_type,
            derivative,
        });
    }

    Ok(ret_map)
}

fn build_model_structure(
    model_structure: schema::ModelStructure,
    model_variables: &SlotMap<VariableKey, ModelVariable>,
) -> Result<ModelStructure, ModelError> {
    let mut outputs = Vec::new();
    let mut continuous_state_derivatives = Vec::new();
    let mut derivatives = Vec::new();
    let mut initial_unknowns = Vec::new();

    for var in model_structure.continuous_state_derivative {
        continuous_state_derivatives.push(
            find_variable_by_vr(model_variables.iter(), var.value_reference).ok_or(
                ModelError::ReferenceError(format!(
                    "Variable '{}' not found in ModelStructure",
                    var.value_reference
                )),
            )?,
        );
    }

    /*
    for var in &md.model_structure.output {
        outputs.push(find_variable_by_name(model_variables.iter(), var).ok_or(
            ModelError::ReferenceError(format!(
                "Variable '{}' not found in ModelStructure",
                var
            )),
        )?);
    }

    for var in &md.model_structure.derivative {
        derivatives.push(find_variable_by_name(model_variables.iter(), var).ok_or(
            ModelError::ReferenceError(format!(
                "Variable '{}' not found in ModelStructure",
                var
            )),
        )?);
    }

    for var in &md.model_structure.initial_unknown {
        initial_unknowns.push(find_variable_by_name(model_variables.iter(), var).ok_or(
            ModelError::ReferenceError(format!(
                "Variable '{}' not found in ModelStructure",
                var
            )),
        )?);
    }
    */

    Ok(ModelStructure {
        outputs,
        continuous_state_derivatives,
        derivatives,
        initial_unknowns,
    })
}

impl TryFrom<schema::FmiModelDescription> for ModelDescription {
    type Error = ModelError;

    fn try_from(md: schema::FmiModelDescription) -> Result<Self, ModelError> {
        let generation_date_and_time = if let Some(s) = md.generation_date_and_time {
            Some(DateTime::from_str(&s).expect("todo"))
        } else {
            None
        };

        let log_categories = md.log_categories.map_or(Ok(Default::default()), |cats| {
            let mut map = SlotMap::with_key();
            for c in cats.categories {
                map.insert(c);
            }
            Ok(map)
        })?;

        let unit_definitions = md
            .unit_definitions
            .map_or(Ok(Default::default()), |defs| build_unit_definitions(defs))?;

        let type_definitions = md.type_definitions.map_or(Ok(Default::default()), |defs| {
            build_type_definitions(defs, &unit_definitions)
        })?;

        let model_variables = build_model_variables(md.model_variables, &type_definitions)?;
        let model_structure = build_model_structure(md.model_structure, &model_variables)?;

        Ok(Self {
            fmi_version: md.fmi_version,
            model_name: md.model_name,
            instantiation_token: md.instantiation_token,
            description: md.description,
            generation_date_and_time,
            log_categories,
            unit_definitions,
            type_definitions,
            model_variables,
            model_structure,
        })
    }
}
