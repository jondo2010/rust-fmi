//! Build a [`schema::Fmi3ModelDescription`] from a [`Model`]
use std::{default, ffi::CStr};

use chrono::Utc;
use fmi::fmi3::{binding, schema};
use uuid::Uuid;

use crate::model::{Field, FieldAttributeOuter, Model, ModelExchangeAttribute};

//TODO: move this into `fmi` crate?
const RUST_FMI_NAMESPACE: Uuid = uuid::uuid!("6ba7b810-9dad-11d1-80b4-00c04fd430c8");

impl TryFrom<Model> for schema::Fmi3ModelDescription {
    type Error = String;

    fn try_from(model: Model) -> Result<Self, Self::Error> {
        let model_name = model.ident.to_string();

        // Generate UUID v5 based on the model name
        let uuid = Uuid::new_v5(&RUST_FMI_NAMESPACE, model_name.as_bytes());

        // Format with curly braces as shown in FMI examples
        let instantiation_token = format!("{{{uuid}}}");

        let description = model.fold_description();

        let model_variables = build_model_variables(&model.fields)?;
        let model_structure = default::Default::default(); // TODO: populate from model

        // Create interface types based on model attributes
        let model_exchange = model
            .model_exchange()
            .map(|me_attr| model_exchange_from_attribute(me_attr, &model_name));

        let co_simulation = model
            .co_simulation()
            .map(|cs_attr| co_simulation_from_attribute(cs_attr, &model_name));

        Ok(schema::Fmi3ModelDescription {
            fmi_version: unsafe {
                CStr::from_ptr(binding::fmi3Version.as_ptr() as *const i8)
                    .to_str()
                    .unwrap()
                    .to_owned()
            },
            model_name,
            instantiation_token,
            description: Some(description.to_owned()),
            generation_tool: Some("rust-fmi".to_owned()),
            generation_date_and_time: Some(Utc::now().to_rfc3339()),
            model_variables,
            model_structure,
            model_exchange,
            co_simulation,
            ..Default::default()
        })
    }
}

/// Helper function to create `Fmi3ModelExchange` from `ModelExchangeAttribute`
fn model_exchange_from_attribute(
    me_attr: &ModelExchangeAttribute,
    default_model_identifier: &str,
) -> schema::Fmi3ModelExchange {
    let model_identifier = me_attr
        .model_identifier
        .as_deref()
        .unwrap_or(default_model_identifier);
        
    schema::Fmi3ModelExchange {
        model_identifier: model_identifier.to_string(),
        needs_completed_integrator_step: me_attr.needs_completed_integrator_step,
        provides_evaluate_discrete_states: me_attr.provides_evaluate_discrete_states,
        needs_execution_tool: me_attr.needs_execution_tool,
        can_be_instantiated_only_once_per_process: me_attr
            .can_be_instantiated_only_once_per_process,
        can_get_and_set_fmu_state: me_attr.can_get_and_set_fmu_state,
        can_serialize_fmu_state: me_attr.can_serialize_fmu_state,
        provides_directional_derivatives: me_attr.provides_directional_derivatives,
        provides_adjoint_derivatives: me_attr.provides_adjoint_derivatives,
        provides_per_element_dependencies: me_attr.provides_per_element_dependencies,
        ..Default::default()
    }
}

/// Helper function to create `schema::Fmi3CoSimulation` from `CoSimulationAttribute`
fn co_simulation_from_attribute(
    cs_attr: &crate::model::CoSimulationAttribute,
    default_model_identifier: &str,
) -> schema::Fmi3CoSimulation {
    let model_identifier = cs_attr
        .model_identifier
        .as_deref()
        .unwrap_or(default_model_identifier);
        
    schema::Fmi3CoSimulation {
        model_identifier: model_identifier.to_string(),
        can_handle_variable_communication_step_size: cs_attr
            .can_handle_variable_communication_step_size,
        fixed_internal_step_size: cs_attr.fixed_internal_step_size,
        max_output_derivative_order: cs_attr.max_output_derivative_order,
        recommended_intermediate_input_smoothness: cs_attr
            .recommended_intermediate_input_smoothness,
        provides_intermediate_update: cs_attr.provides_intermediate_update,
        might_return_early_from_do_step: cs_attr.might_return_early_from_do_step,
        can_return_early_after_intermediate_update: cs_attr
            .can_return_early_after_intermediate_update,
        has_event_mode: cs_attr.has_event_mode,
        provides_evaluate_discrete_states: cs_attr.provides_evaluate_discrete_states,
        needs_execution_tool: cs_attr.needs_execution_tool,
        can_be_instantiated_only_once_per_process: cs_attr
            .can_be_instantiated_only_once_per_process,
        can_get_and_set_fmu_state: cs_attr.can_get_and_set_fmu_state,
        can_serialize_fmu_state: cs_attr.can_serialize_fmu_state,
        provides_directional_derivatives: cs_attr.provides_directional_derivatives,
        provides_adjoint_derivatives: cs_attr.provides_adjoint_derivatives,
        provides_per_element_dependencies: cs_attr.provides_per_element_dependencies,
        ..Default::default()
    }
}

/// Convert a syn::Type to a schema::VariableType
pub fn rust_type_to_variable_type(ty: &syn::Type) -> Result<schema::VariableType, String> {
    match ty {
        syn::Type::Path(type_path) => {
            let type_name = &type_path.path.segments.last().unwrap().ident;
            let type_str = type_name.to_string();

            match type_str.as_str() {
                "f32" => Ok(schema::VariableType::FmiFloat32),
                "f64" => Ok(schema::VariableType::FmiFloat64),
                "i8" => Ok(schema::VariableType::FmiInt8),
                "i16" => Ok(schema::VariableType::FmiInt16),
                "i32" => Ok(schema::VariableType::FmiInt32),
                "i64" => Ok(schema::VariableType::FmiInt64),
                "u8" => Ok(schema::VariableType::FmiUInt8),
                "u16" => Ok(schema::VariableType::FmiUInt16),
                "u32" => Ok(schema::VariableType::FmiUInt32),
                "u64" => Ok(schema::VariableType::FmiUInt64),
                "bool" => Ok(schema::VariableType::FmiBoolean),
                "String" => Ok(schema::VariableType::FmiString),
                _ => Err(format!(
                    "Unsupported field type '{}'. Supported types are: f32, f64, i8, i16, i32, i64, u8, u16, u32, u64, bool, String",
                    type_name
                )),
            }
        }
        _ => Err("Unsupported field type. Only path types are supported.".to_string()),
    }
}

/// Convert a syn::Ident representing causality into the corresponding enum
fn build_causality(ident: &syn::Ident) -> Result<schema::Causality, String> {
    match ident.to_string().as_str() {
        "Parameter" => Ok(schema::Causality::Parameter),
        "Input" => Ok(schema::Causality::Input),
        "Output" => Ok(schema::Causality::Output),
        "Local" => Ok(schema::Causality::Local),
        "Independent" => Ok(schema::Causality::Independent),
        "CalculatedParameter" => Ok(schema::Causality::CalculatedParameter),
        _ => Err(format!("Unknown causality: {}", ident)),
    }
}

/// Convert a syn::Ident representing variability into the corresponding enum
fn build_variability(ident: &syn::Ident) -> Result<schema::Variability, String> {
    match ident.to_string().as_str() {
        "Constant" => Ok(schema::Variability::Constant),
        "Fixed" => Ok(schema::Variability::Fixed),
        "Tunable" => Ok(schema::Variability::Tunable),
        "Discrete" => Ok(schema::Variability::Discrete),
        "Continuous" => Ok(schema::Variability::Continuous),
        _ => Err(format!("Unknown variability: {}", ident)),
    }
}

/// Helper function to get variable description from field and attribute
fn get_variable_description(field: &Field, attr: &crate::model::FieldAttribute) -> Option<String> {
    attr.description.clone().or_else(|| {
        let field_desc = field.fold_description();
        if field_desc.is_empty() {
            None
        } else {
            Some(field_desc)
        }
    })
}

/// Helper function to get variable causality from attribute
fn get_variable_causality(
    attr: &crate::model::FieldAttribute,
) -> Result<schema::Causality, String> {
    attr.causality
        .as_ref()
        .map(|ident| build_causality(ident))
        .transpose()
        .map(|causality| causality.unwrap_or_default())
}

/// Helper function to get variable variability from attribute with smart defaults
fn get_variable_variability(
    attr: &crate::model::FieldAttribute,
    variable_type: &schema::VariableType,
) -> Result<schema::Variability, String> {
    if let Some(variability_ident) = &attr.variability {
        build_variability(variability_ident)
    } else {
        // Use sensible defaults based on variable type
        match variable_type {
            schema::VariableType::FmiFloat32 | schema::VariableType::FmiFloat64 => {
                Ok(schema::Variability::Continuous)
            }
            _ => Ok(schema::Variability::Discrete),
        }
    }
}

/// Helper function to create InitializableVariable structure
/// Parse numeric start value from syn::Expr
fn parse_numeric_start_value<T>(expr: &syn::Expr) -> Vec<T>
where
    T: std::str::FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Display,
{
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Float(lit_float),
            ..
        }) => {
            if let Ok(value) = lit_float.base10_parse::<T>() {
                vec![value]
            } else {
                vec![]
            }
        }
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Int(lit_int),
            ..
        }) => {
            if let Ok(value) = lit_int.base10_parse::<T>() {
                vec![value]
            } else {
                vec![]
            }
        }
        _ => vec![], // For now, only support numeric literals
    }
}

/// Parse boolean start value from syn::Expr
fn parse_bool_start_value(expr: &syn::Expr) -> Vec<bool> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Bool(lit_bool),
            ..
        }) => vec![lit_bool.value],
        _ => vec![], // Only support boolean literals
    }
}

/// Parse string start value from syn::Expr
fn parse_string_start_value(expr: &syn::Expr) -> Vec<String> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str),
            ..
        }) => vec![lit_str.value()],
        _ => vec![], // Only support string literals
    }
}

/// Process the fields of a model struct into FMI model variables
///
/// Key points:
/// - Each variable annotation `#[variable(...)]` becomes a model variable
/// - Each alias annotation `#[alias(...)]` also becomes a model variable
/// - Fields without variable or alias annotations are ignored (private/internal)
/// - Supports all FMI datatypes: f32, f64, i8, i16, i32, i64, u8, u16, u32, u64, bool, String
fn build_model_variables(fields: &[Field]) -> Result<schema::ModelVariables, String> {
    let mut model_variables = schema::ModelVariables::default();
    let mut value_reference_counter = 1u32; // FMI value references start at 1

    for field in fields.iter() {
        // Process each variable and alias attribute for this field
        for attr in &field.attrs {
            match attr {
                FieldAttributeOuter::Variable(var_attr) => {
                    create_and_add_variable(
                        field,
                        var_attr,
                        value_reference_counter,
                        None, // Use field name
                        &mut model_variables,
                    )?;
                    value_reference_counter += 1;
                }
                FieldAttributeOuter::Alias(alias_attr) => {
                    create_and_add_variable(
                        field,
                        alias_attr,
                        value_reference_counter,
                        alias_attr.name.clone(), // Use alias name if provided
                        &mut model_variables,
                    )?;
                    value_reference_counter += 1;
                }
                FieldAttributeOuter::Docstring(_) => {
                    // Skip docstrings - they're handled when creating variables
                }
            }
        }
    }

    Ok(model_variables)
}

/// Create and add a variable to ModelVariables based on the field type
fn create_and_add_variable(
    field: &Field,
    attr: &crate::model::FieldAttribute,
    value_reference: u32,
    override_name: Option<String>,
    model_variables: &mut schema::ModelVariables,
) -> Result<(), String> {
    let name = override_name.unwrap_or_else(|| field.ident.to_string());
    let description = get_variable_description(field, attr);
    let causality = get_variable_causality(attr)?;

    // Convert field type to VariableType
    let variable_type = rust_type_to_variable_type(&field.ty)?;
    let variability = get_variable_variability(attr, &variable_type)?;

    // Match on variable type and create appropriate FMI variable
    match variable_type {
        schema::VariableType::FmiFloat32 => {
            let start = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<f32>)
                .unwrap_or_default();
            let variable = schema::FmiFloat32::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
            );
            model_variables.float32.push(variable);
        }
        schema::VariableType::FmiFloat64 => {
            let start = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<f64>)
                .unwrap_or_default();
            let variable = schema::FmiFloat64::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
            );
            model_variables.float64.push(variable);
        }
        schema::VariableType::FmiInt8 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<i8>)
                .unwrap_or_default();
            let start = start_vec.into_iter().next();
            let variable = schema::FmiInt8::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
            );
            model_variables.int8.push(variable);
        }
        schema::VariableType::FmiInt16 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<i16>)
                .unwrap_or_default();
            let start = start_vec.into_iter().next();
            let variable = schema::FmiInt16::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
            );
            model_variables.int16.push(variable);
        }
        schema::VariableType::FmiInt32 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<i32>)
                .unwrap_or_default();
            let start = start_vec.into_iter().next();
            let variable = schema::FmiInt32::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
            );
            model_variables.int32.push(variable);
        }
        schema::VariableType::FmiInt64 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<i64>)
                .unwrap_or_default();
            let start = start_vec.into_iter().next();
            let variable = schema::FmiInt64::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
            );
            model_variables.int64.push(variable);
        }
        schema::VariableType::FmiUInt8 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<u8>)
                .unwrap_or_default();
            let start = start_vec.into_iter().next();
            let variable = schema::FmiUInt8::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
            );
            model_variables.uint8.push(variable);
        }
        schema::VariableType::FmiUInt16 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<u16>)
                .unwrap_or_default();
            let start = start_vec.into_iter().next();
            let variable = schema::FmiUInt16::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
            );
            model_variables.uint16.push(variable);
        }
        schema::VariableType::FmiUInt32 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<u32>)
                .unwrap_or_default();
            let start = start_vec.into_iter().next();
            let variable = schema::FmiUInt32::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
            );
            model_variables.uint32.push(variable);
        }
        schema::VariableType::FmiUInt64 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<u64>)
                .unwrap_or_default();
            let start = start_vec.into_iter().next();
            let variable = schema::FmiUInt64::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
            );
            model_variables.uint64.push(variable);
        }
        schema::VariableType::FmiBoolean => {
            let start = attr
                .start
                .as_ref()
                .map(parse_bool_start_value)
                .unwrap_or_default();
            let variable = schema::FmiBoolean::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
            );
            model_variables.boolean.push(variable);
        }
        schema::VariableType::FmiString => {
            let start = attr
                .start
                .as_ref()
                .map(parse_string_start_value)
                .unwrap_or_default();
            let variable = schema::FmiString::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
            );
            model_variables.string.push(variable);
        }
        schema::VariableType::FmiBinary => {
            return Err(
                "Binary variables are not yet supported in this implementation.".to_string(),
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::StructAttributeOuter;
    use schema::AbstractVariableTrait;
    use syn::parse_quote;

    #[test]
    fn test_model_description_generation() {
        let model = Model {
            ident: parse_quote!(BouncingBall),
            attrs: vec![StructAttributeOuter::Docstring(
                "A simple bouncing ball model".to_string(),
            )],
            fields: vec![],
        };

        let description = schema::Fmi3ModelDescription::try_from(model).unwrap();

        assert_eq!(description.model_name, "BouncingBall");
        assert_eq!(
            description.description,
            Some("A simple bouncing ball model".to_string())
        );
        assert_eq!(description.generation_tool, Some("rust-fmi".to_string()));
        assert!(description.instantiation_token.starts_with('{'));
        assert!(description.instantiation_token.ends_with('}'));
        assert_eq!(description.fmi_version, "3.0");
    }

    #[test]
    fn test_build_model_variables() {
        use crate::model::{FieldAttribute, FieldAttributeOuter};
        use syn::parse_quote;

        let fields = vec![
            Field {
                ident: parse_quote!(h),
                ty: parse_quote!(f64),
                attrs: vec![
                    FieldAttributeOuter::Docstring(
                        "Height above ground (state output)".to_string(),
                    ),
                    FieldAttributeOuter::Variable(FieldAttribute {
                        causality: Some(parse_quote!(Output)),
                        start: Some(parse_quote!(1.0)),
                        ..Default::default()
                    }),
                ],
            },
            Field {
                ident: parse_quote!(v),
                ty: parse_quote!(f64),
                attrs: vec![
                    FieldAttributeOuter::Docstring("Velocity of the ball".to_string()),
                    FieldAttributeOuter::Variable(FieldAttribute {
                        causality: Some(parse_quote!(Output)),
                        start: Some(parse_quote!(0.0)),
                        ..Default::default()
                    }),
                ],
            },
        ];

        let model_variables = build_model_variables(&fields).unwrap();

        // Test that we have the correct number of float64 variables
        assert_eq!(model_variables.float64.len(), 2);
        assert_eq!(model_variables.len(), 2);

        // Test the first variable
        let h_var = &model_variables.float64[0];
        assert_eq!(h_var.name(), "h");
        assert_eq!(
            h_var.description(),
            Some("Height above ground (state output)")
        );
        assert_eq!(h_var.causality(), schema::Causality::Output);
        assert_eq!(h_var.start(), &[1.0]);

        // Test the second variable
        let v_var = &model_variables.float64[1];
        assert_eq!(v_var.name(), "v");
        assert_eq!(v_var.description(), Some("Velocity of the ball"));
        assert_eq!(v_var.causality(), schema::Causality::Output);
        assert_eq!(v_var.start(), &[0.0]);
    }

    #[test]
    fn test_multiple_data_types() {
        use crate::model::{FieldAttribute, FieldAttributeOuter};
        use syn::parse_quote;

        let fields = vec![
            Field {
                ident: parse_quote!(position),
                ty: parse_quote!(f32),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(parse_quote!(Output)),
                    start: Some(parse_quote!(1.5)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(count),
                ty: parse_quote!(i32),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(parse_quote!(Parameter)),
                    start: Some(parse_quote!(42)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(enabled),
                ty: parse_quote!(bool),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(parse_quote!(Input)),
                    start: Some(parse_quote!(true)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(name),
                ty: parse_quote!(String),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(parse_quote!(Parameter)),
                    start: Some(parse_quote!("test")),
                    ..Default::default()
                })],
            },
        ];

        let model_variables = build_model_variables(&fields).unwrap();

        // Test that we have the correct distribution of variable types
        assert_eq!(model_variables.float32.len(), 1);
        assert_eq!(model_variables.int32.len(), 1);
        assert_eq!(model_variables.boolean.len(), 1);
        assert_eq!(model_variables.string.len(), 1);
        assert_eq!(model_variables.len(), 4);

        // Test specific variables
        assert_eq!(model_variables.float32[0].name(), "position");
        assert_eq!(model_variables.float32[0].start(), &[1.5]);

        assert_eq!(model_variables.int32[0].name(), "count");
        assert_eq!(model_variables.int32[0].start, Some(42));

        assert_eq!(model_variables.boolean[0].name(), "enabled");
        assert_eq!(model_variables.boolean[0].start, vec![true]);

        assert_eq!(model_variables.string[0].name(), "name");
        let string_starts: Vec<&str> = model_variables.string[0].start().collect();
        assert_eq!(string_starts, vec!["test"]);
    }

    #[test]
    fn test_full_model_to_fmi_description() {
        let input: syn::ItemStruct = syn::parse_quote! {
            /// A comprehensive test model with multiple data types
            #[model()]
            struct TestModel {
                /// Position (float32)
                #[variable(causality = Output, start = 10.5)]
                position: f32,

                /// Count (signed integer)
                #[variable(causality = Parameter, variability = Tunable, start = 100)]
                count: i32,

                /// Enabled flag
                #[variable(causality = Input, start = false)]
                enabled: bool,

                /// Model name
                #[variable(causality = Parameter, start = "TestModel")]
                model_name: String,
            }
        };
        let model = Model::from(input);

        let fmi_description = schema::Fmi3ModelDescription::try_from(model).unwrap();

        // Test model-level attributes
        assert_eq!(fmi_description.model_name, "TestModel");
        assert_eq!(
            fmi_description.description,
            Some("A comprehensive test model with multiple data types".to_string())
        );
        assert_eq!(fmi_description.fmi_version, "3.0");

        // Test that model variables are correctly populated
        assert_eq!(fmi_description.model_variables.len(), 4);
        assert_eq!(fmi_description.model_variables.float32.len(), 1);
        assert_eq!(fmi_description.model_variables.int32.len(), 1);
        assert_eq!(fmi_description.model_variables.boolean.len(), 1);
        assert_eq!(fmi_description.model_variables.string.len(), 1);
    }

    #[test]
    fn test_variability_handling() {
        use crate::model::{FieldAttribute, FieldAttributeOuter};

        let fields = vec![
            Field {
                ident: parse_quote!(continuous_var),
                ty: parse_quote!(f64),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(parse_quote!(Output)),
                    variability: Some(parse_quote!(Continuous)),
                    start: Some(parse_quote!(1.0)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(discrete_var),
                ty: parse_quote!(f64),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(parse_quote!(Output)),
                    variability: Some(parse_quote!(Discrete)),
                    start: Some(parse_quote!(2.0)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(fixed_param),
                ty: parse_quote!(i32),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(parse_quote!(Parameter)),
                    variability: Some(parse_quote!(Fixed)),
                    start: Some(parse_quote!(42)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(constant_val),
                ty: parse_quote!(bool),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(parse_quote!(Parameter)),
                    variability: Some(parse_quote!(Constant)),
                    start: Some(parse_quote!(true)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(default_float),
                ty: parse_quote!(f32),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(parse_quote!(Output)),
                    // No variability specified - should default to Continuous for floats
                    start: Some(parse_quote!(3.14)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(default_int),
                ty: parse_quote!(u32),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(parse_quote!(Output)),
                    // No variability specified - should default to Discrete for integers
                    start: Some(parse_quote!(100)),
                    ..Default::default()
                })],
            },
        ];

        let model_variables = build_model_variables(&fields).unwrap();

        // Test that we have the correct number and distribution of variables
        assert_eq!(model_variables.float64.len(), 2);
        assert_eq!(model_variables.float32.len(), 1);
        assert_eq!(model_variables.int32.len(), 1);
        assert_eq!(model_variables.boolean.len(), 1);
        assert_eq!(model_variables.uint32.len(), 1);
        assert_eq!(model_variables.len(), 6);

        // Test explicit variability settings
        assert_eq!(
            model_variables.float64[0]
                .init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .variability,
            Some(schema::Variability::Continuous)
        );
        assert_eq!(
            model_variables.float64[1]
                .init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .variability,
            Some(schema::Variability::Discrete)
        );
        assert_eq!(
            model_variables.int32[0]
                .init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .variability,
            Some(schema::Variability::Fixed)
        );
        assert_eq!(
            model_variables.boolean[0]
                .init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .variability,
            Some(schema::Variability::Constant)
        );

        // Test default variability settings
        assert_eq!(
            model_variables.float32[0]
                .init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .variability,
            Some(schema::Variability::Continuous) // Default for floats
        );
        assert_eq!(
            model_variables.uint32[0]
                .init_var
                .typed_arrayable_var
                .arrayable_var
                .abstract_var
                .variability,
            Some(schema::Variability::Discrete) // Default for integers
        );
    }
}
