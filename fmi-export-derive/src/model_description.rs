//! Convert a [`Model`] struct into an [`schema::Fmi3ModelDescription`]
use std::{default, ffi::CStr};

use chrono::Utc;
use fmi::fmi3::{binding, schema, schema::AbstractVariableTrait};
use uuid::Uuid;

use crate::model_new::{Field, FieldAttributeOuter, Model, StructAttributeOuter};

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
            ..Default::default()
        })
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
fn get_variable_description(
    field: &Field,
    attr: &crate::model_new::FieldAttribute,
) -> Option<String> {
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
    attr: &crate::model_new::FieldAttribute,
) -> Result<schema::Causality, String> {
    attr.causality
        .as_ref()
        .map(|ident| build_causality(ident))
        .transpose()
        .map(|causality| causality.unwrap_or_default())
}

/// Helper function to get variable variability from attribute with smart defaults
fn get_variable_variability(
    attr: &crate::model_new::FieldAttribute,
    field_type: &str,
) -> Result<schema::Variability, String> {
    if let Some(variability_ident) = &attr.variability {
        build_variability(variability_ident)
    } else {
        // Use sensible defaults based on type
        match field_type {
            "f32" | "f64" => Ok(schema::Variability::Continuous),
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
    attr: &crate::model_new::FieldAttribute,
    value_reference: u32,
    override_name: Option<String>,
    model_variables: &mut schema::ModelVariables,
) -> Result<(), String> {
    let name = override_name.unwrap_or_else(|| field.ident.to_string());
    let description = get_variable_description(field, attr);
    let causality = get_variable_causality(attr)?;

    // Match on field type and create appropriate FMI variable
    match &field.ty {
        syn::Type::Path(type_path) => {
            let type_name = &type_path.path.segments.last().unwrap().ident;
            let type_str = type_name.to_string();
            let variability = get_variable_variability(attr, &type_str)?;

            match type_str.as_str() {
                "f32" => {
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
                "f64" => {
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
                "i8" => {
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
                "i16" => {
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
                "i32" => {
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
                "i64" => {
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
                "u8" => {
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
                "u16" => {
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
                "u32" => {
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
                "u64" => {
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
                "bool" => {
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
                "String" => {
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
                _ => {
                    return Err(format!(
                        "Unsupported field type '{}' for field '{}'. Supported types are: f32, f64, i8, i16, i32, i64, u8, u16, u32, u64, bool, String",
                        type_name, field.ident
                    ));
                }
            }
        }
        _ => {
            return Err(format!(
                "Unsupported field type for field '{}'. Only path types are supported.",
                field.ident
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        use crate::model_new::{FieldAttribute, FieldAttributeOuter};
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
        use crate::model_new::{FieldAttribute, FieldAttributeOuter};
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
        use crate::model_new::{FieldAttribute, FieldAttributeOuter};
        use syn::parse_quote;

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
