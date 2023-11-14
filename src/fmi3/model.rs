use itertools::Itertools;
use slotmap::{new_key_type, SlotMap, SecondaryMap};
use thiserror::Error;

use super::schema::{self, AbstractVariableTrait, TypedArrayableariableTrait};

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("Error in model description: {0}")]
    ReferenceError(String),
}

new_key_type! { pub struct TypeKey; }
new_key_type! { pub struct UnitKey; }
new_key_type! { pub struct VariableKey; }

#[derive(Debug)]
struct UnitDefinition<'a> {
    name: &'a str,
}

#[derive(Debug)]
pub enum TypeDefinition<'a> {
    Float32 {
        name: &'a str,
    },
    Float64 {
        name: &'a str,
        quantity: Option<&'a str>,
        unit: Option<UnitKey>,
    },
}

#[derive(Debug)]
pub enum ModelVariable<'a> {
    Float32 {
        name: &'a str,
        value_reference: u32,
        declared_type: Option<TypeKey>,
        derivative: Option<VariableKey>,
    },
    Float64 {
        name: &'a str,
        value_reference: u32,
        declared_type: Option<TypeKey>,
        derivative: Option<VariableKey>,
    },
}

#[derive(Debug)]
pub struct ModelDescription<'a> {
    /// A global list of unit and display unit definitions
    unit_definitions: SlotMap<UnitKey, UnitDefinition<'a>>,
    /// A global list of type definitions that are utilized in `ModelVariables`
    type_definitions: SlotMap<TypeKey, TypeDefinition<'a>>,
    pub model_variables: SlotMap<VariableKey, ModelVariable<'a>>,

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
    mut unit_definitions: impl Iterator<Item = (UnitKey, &'a UnitDefinition<'a>)>,
    name: &str,
) -> Option<UnitKey> {
    unit_definitions.find_map(
        |(key, unit)| {
            if unit.name == name {
                Some(key)
            } else {
                None
            }
        },
    )
}

fn find_type_definition_by_name<'a>(
    mut type_definitions: impl Iterator<Item = (TypeKey, &'a TypeDefinition<'a>)>,
    name: &str,
) -> Option<TypeKey> {
    type_definitions.find_map(|(key, ty)| match ty {
        TypeDefinition::Float32 { name: ty_name, .. } if ty_name == &name => Some(key),
        TypeDefinition::Float64 { name: ty_name, .. } if ty_name == &name => Some(key),
        _ => None,
    })
}

fn find_variable_by_name<'a>(
    mut model_variables: impl Iterator<Item = (VariableKey, &'a ModelVariable<'a>)>,
    name: &str,
) -> Option<VariableKey> {
    model_variables.find_map(|(key, var)| match var {
        ModelVariable::Float32 { name: var_name, .. } if var_name == &name => Some(key),
        ModelVariable::Float64 { name: var_name, .. } if var_name == &name => Some(key),
        _ => None,
    })
}

fn find_variable_by_vr<'a>(
    mut model_variables: impl Iterator<Item = (VariableKey, &'a ModelVariable<'a>)>,
    vr: u32,
) -> Option<VariableKey> {
    model_variables.find_map(|(key, var)| match var {
        ModelVariable::Float32 { value_reference, .. } if value_reference == &vr => Some(key),
        ModelVariable::Float64 { value_reference, .. } if value_reference == &vr => Some(key),
        _ => None,
    })
}

fn build_unit_definitions<'a>(
    md: &'a schema::FmiModelDescription,
) -> Result<SlotMap<UnitKey, UnitDefinition<'a>>, ModelError> {
    Ok(md.unit_definitions.as_ref().map_or_else(
        || SlotMap::default(),
        |defs| {
            let mut map = SlotMap::with_key();
            for unit in &defs.units {
                map.insert(UnitDefinition { name: &unit.name });
            }
            map
        },
    ))
}

fn build_type_definitions<'a>(
    md: &'a schema::FmiModelDescription,
    unit_definitions: &SlotMap<UnitKey, UnitDefinition<'a>>,
) -> Result<SlotMap<TypeKey, TypeDefinition<'a>>, ModelError> {
    md.type_definitions.as_ref().map_or_else(
        || Ok(SlotMap::default()),
        |defs| {
            let mut map = SlotMap::with_key();
            for ty in &defs.float32_type {
                map.insert(TypeDefinition::Float32 {
                    name: &ty.base.name,
                });
            }
            for ty in &defs.float64_type {
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

                map.insert(TypeDefinition::Float64 {
                    name: &ty.base.name,
                    quantity: ty.base_attr.quantity.as_deref(),
                    unit,
                });
            }
            Ok(map)
        },
    )
}

fn build_model_variables<'a>(
    md: &'a schema::FmiModelDescription,
    type_definitions: &SlotMap<TypeKey, TypeDefinition<'a>>,
) -> Result<SlotMap<VariableKey, ModelVariable<'a>>, ModelError> {
    let mut model_variables = SlotMap::with_capacity_and_key(md.model_variables.len());

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
    let variable_by_vr = |model_variables: &SlotMap<VariableKey,_>, variable_vr: u32, var_name: &str| -> Result<VariableKey, ModelError> {
        find_variable_by_vr(model_variables.iter(), variable_vr).ok_or(
            ModelError::ReferenceError(format!(
                "Variable '{}' not found in ScalarVariable '{}'",
                variable_vr, var_name
            )),
        )
    };

    for var in &md.model_variables.float32 {
        let declared_type = if let Some(type_name) = var.declared_type() {
            Some(deref_declared_type(type_name, var.name())?)
        } else {
            None
        };

        let derivative = if let Some(derivative) = var.derivative() {
            Some(variable_by_vr(&model_variables, derivative, var.name())?)
        } else {
            None
        };

        model_variables.insert(ModelVariable::Float32 {
            name: var.name(),
            value_reference: var.value_reference(),
            declared_type,
            derivative,
        });
    }

    for var in md.model_variables.float64.iter().sorted_by_key(|var| var.derivative()) {
        let declared_type = if let Some(type_name) = var.declared_type() {
            Some(deref_declared_type(type_name, var.name())?)
        } else {
            None
        };

        let derivative = if let Some(derivative) = var.derivative() {
            Some(variable_by_vr(&model_variables, derivative, var.name())?)
        } else {
            None
        };

        model_variables.insert(ModelVariable::Float64 {
            name: var.name(),
            value_reference: var.value_reference(),
            declared_type,
            derivative,
        });
    }

    Ok(model_variables)
}

fn build_model_structure(
    md: &schema::FmiModelDescription,
    model_variables: &SlotMap<VariableKey, ModelVariable>,
) -> Result<ModelStructure, ModelError> {
    let mut outputs = Vec::new();
    let mut continuous_state_derivatives = Vec::new();
    let mut derivatives = Vec::new();
    let mut initial_unknowns = Vec::new();

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

impl<'a> TryFrom<&'a schema::FmiModelDescription> for ModelDescription<'a> {
    type Error = ModelError;

    fn try_from(md: &'a schema::FmiModelDescription) -> Result<Self, ModelError> {
        let unit_definitions = build_unit_definitions(md)?;
        let type_definitions = build_type_definitions(md, &unit_definitions)?;
        let model_variables = build_model_variables(md, &type_definitions)?;
        let model_structure = build_model_structure(md, &model_variables)?;

        Ok(Self {
            unit_definitions,
            type_definitions,
            model_variables,
            model_structure,
        })
    }
}
