use slotmap::{new_key_type, SlotMap};
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
    },
    Float64 {
        name: &'a str,
        value_reference: u32,
        declared_type: Option<TypeKey>,
    },
}

#[derive(Debug)]
pub struct ModelDescription<'a> {
    /// A global list of unit and display unit definitions
    unit_definitions: SlotMap<UnitKey, UnitDefinition<'a>>,
    /// A global list of type definitions that are utilized in `ModelVariables`
    type_definitions: SlotMap<TypeKey, TypeDefinition<'a>>,
    pub model_variables: SlotMap<VariableKey, ModelVariable<'a>>,
}

fn find_unit_definition<'a>(
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

fn find_type_definition<'a>(
    mut type_definitions: impl Iterator<Item = (TypeKey, &'a TypeDefinition<'a>)>,
    name: &str,
) -> Option<TypeKey> {
    type_definitions.find_map(|(key, ty)| match ty {
        TypeDefinition::Float32 { name: ty_name, .. } if ty_name == &name => Some(key),
        TypeDefinition::Float64 { name: ty_name, .. } if ty_name == &name => Some(key),
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
                        find_unit_definition(unit_definitions.iter(), unit_name).ok_or(
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
    let mut map = SlotMap::with_key();

    for var in &md.model_variables.float32 {
        map.insert(ModelVariable::Float32 {
            name: var.name(),
            value_reference: 0,
        });
    }

    for var in &md.model_variables.float64 {
        let declared_type = match var.declared_type() {
            Some(type_name) => Some(
                find_type_definition(type_definitions.iter(), type_name).ok_or(
                    ModelError::ReferenceError(format!(
                        "TypeDefinition '{}' not found in ScalarVariable '{}'",
                        type_name,
                        var.name()
                    )),
                )?,
            ),
            None => None,
        };

        map.insert(ModelVariable::Float64 {
            name: var.name(),
            value_reference: 0,
            declared_type,
        });
    }

    Ok(map)
}

impl<'a> TryFrom<&'a schema::FmiModelDescription> for ModelDescription<'a> {
    type Error = ModelError;

    fn try_from(md: &'a schema::FmiModelDescription) -> Result<Self, ModelError> {
        let unit_definitions = build_unit_definitions(md)?;
        let type_definitions = build_type_definitions(md, &unit_definitions)?;
        let model_variables = build_model_variables(md, &type_definitions)?;

        Ok(Self {
            unit_definitions,
            type_definitions,
            model_variables,
        })
    }
}
