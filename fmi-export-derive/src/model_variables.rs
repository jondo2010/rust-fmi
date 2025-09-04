//! Build a [`schema::ModelVariables`] from a [`Model`]
//!
//! Process the fields of a model struct into FMI model variables
//!
//! Key points:
//! - Each variable annotation `#[variable(...)]` becomes a model variable
//! - Each alias annotation `#[alias(...)]` also becomes a model variable
//! - Fields without variable or alias annotations are ignored (private/internal)
//! - Supports all FMI datatypes: f32, f64, i8, i16, i32, i64, u8, u16, u32, u64, bool, String

use fmi::fmi3::schema;

use crate::{
    model::{Field, FieldAttributeOuter},
    util::{
        parse_bool_start_value, parse_numeric_start_value, parse_string_start_value,
        rust_type_to_variable_type,
    },
};

pub fn build_model_variables(fields: &[Field]) -> Result<schema::ModelVariables, String> {
    let mut model_variables = schema::ModelVariables::default();
    let mut value_reference_counter = 1u32; // FMI value references start at 1

    for field in fields {
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
    let causality: schema::Causality = attr.causality.expect("Causality should be set").0;

    // Convert field type to VariableType
    let variable_type = rust_type_to_variable_type(&field.ty)?;
    let variability = get_variable_variability(attr, &variable_type)?;
    let initial = get_variable_initial(attr);

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
                initial,
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
                initial,
            );
            model_variables.float64.push(variable);
        }
        schema::VariableType::FmiInt8 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<i8>)
                .unwrap_or_default();
            let variable = schema::FmiInt8::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start_vec,
                initial,
            );
            model_variables.int8.push(variable);
        }
        schema::VariableType::FmiInt16 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<i16>)
                .unwrap_or_default();
            let variable = schema::FmiInt16::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start_vec,
                initial,
            );
            model_variables.int16.push(variable);
        }
        schema::VariableType::FmiInt32 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<i32>)
                .unwrap_or_default();
            let variable = schema::FmiInt32::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start_vec,
                initial,
            );
            model_variables.int32.push(variable);
        }
        schema::VariableType::FmiInt64 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<i64>)
                .unwrap_or_default();
            let variable = schema::FmiInt64::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start_vec,
                initial,
            );
            model_variables.int64.push(variable);
        }
        schema::VariableType::FmiUInt8 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<u8>)
                .unwrap_or_default();
            let variable = schema::FmiUInt8::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start_vec,
                initial,
            );
            model_variables.uint8.push(variable);
        }
        schema::VariableType::FmiUInt16 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<u16>)
                .unwrap_or_default();
            let variable = schema::FmiUInt16::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start_vec,
                initial,
            );
            model_variables.uint16.push(variable);
        }
        schema::VariableType::FmiUInt32 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<u32>)
                .unwrap_or_default();
            let variable = schema::FmiUInt32::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start_vec,
                initial,
            );
            model_variables.uint32.push(variable);
        }
        schema::VariableType::FmiUInt64 => {
            let start_vec = attr
                .start
                .as_ref()
                .map(parse_numeric_start_value::<u64>)
                .unwrap_or_default();
            let variable = schema::FmiUInt64::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start_vec,
                initial,
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
                initial,
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
                initial,
            );
            model_variables.string.push(variable);
        }
        schema::VariableType::FmiBinary => {
            let start = attr
                .start
                .as_ref()
                .map(parse_string_start_value) // Binary uses string start values
                .unwrap_or_default();
            let variable = schema::FmiBinary::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
                initial,
            );
            model_variables.binary.push(variable);
        }
    }

    Ok(())
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

/// Get the initial value from the field attribute
fn get_variable_initial(attr: &crate::model::FieldAttribute) -> Option<schema::Initial> {
    attr.initial.as_ref().map(|ident| {
        match ident.to_string().as_str() {
            "Exact" => schema::Initial::Exact,
            "Calculated" => schema::Initial::Calculated,
            "Approx" => schema::Initial::Approx,
            _ => schema::Initial::Exact, // Default to Exact if unknown
        }
    })
}

/// Helper function to get variable variability from attribute with smart defaults
fn get_variable_variability(
    attr: &crate::model::FieldAttribute,
    variable_type: &schema::VariableType,
) -> Result<schema::Variability, String> {
    if let Some(variability_ident) = &attr.variability {
        Ok(variability_ident.0)
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

#[cfg(test)]
mod tests {

    use crate::Model;

    use super::*;
    use fmi::{fmi3::schema, schema::fmi3::AbstractVariableTrait};
    use syn::parse_quote;

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
                        causality: Some(schema::Causality::Output.into()),
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
                        causality: Some(schema::Causality::Output.into()),
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
                    causality: Some(schema::Causality::Output.into()),
                    start: Some(parse_quote!(1.5)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(count),
                ty: parse_quote!(i32),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(schema::Causality::Parameter.into()),
                    start: Some(parse_quote!(42)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(enabled),
                ty: parse_quote!(bool),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(schema::Causality::Input.into()),
                    start: Some(parse_quote!(true)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(name),
                ty: parse_quote!(String),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(schema::Causality::Parameter.into()),
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
        assert_eq!(model_variables.int32[0].start, vec![42]);

        assert_eq!(model_variables.boolean[0].name(), "enabled");
        assert_eq!(model_variables.boolean[0].start, vec![true]);

        assert_eq!(model_variables.string[0].name(), "name");
        let string_starts: Vec<&str> = model_variables.string[0].start().collect();
        assert_eq!(string_starts, vec!["test"]);
    }

    #[test]
    fn test_variability_handling() {
        use crate::model::{FieldAttribute, FieldAttributeOuter};

        let fields = vec![
            Field {
                ident: parse_quote!(continuous_var),
                ty: parse_quote!(f64),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(schema::Causality::Output.into()),
                    variability: Some(crate::model::Variability(schema::Variability::Continuous)),
                    start: Some(parse_quote!(1.0)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(discrete_var),
                ty: parse_quote!(f64),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(schema::Causality::Output.into()),
                    variability: Some(crate::model::Variability(schema::Variability::Discrete)),
                    start: Some(parse_quote!(2.0)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(fixed_param),
                ty: parse_quote!(i32),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(schema::Causality::Parameter.into()),
                    variability: Some(crate::model::Variability(schema::Variability::Fixed)),
                    start: Some(parse_quote!(42)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(constant_val),
                ty: parse_quote!(bool),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(schema::Causality::Parameter.into()),
                    variability: Some(crate::model::Variability(schema::Variability::Constant)),
                    start: Some(parse_quote!(true)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(default_float),
                ty: parse_quote!(f32),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(schema::Causality::Output.into()),
                    // No variability specified - should default to Continuous for floats
                    start: Some(parse_quote!(3.14)),
                    ..Default::default()
                })],
            },
            Field {
                ident: parse_quote!(default_int),
                ty: parse_quote!(u32),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(schema::Causality::Output.into()),
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

    #[test]
    fn test_foo() {
        let input: syn::DeriveInput = parse_quote! {
            #[model()]
            struct ComprehensiveModel {
                //f1: u32,

                /// Example of a fixed-size array
                #[variable(causality = Parameter, variability = Tunable, start = [0u16, 1, 2, 3])]
                f2: [u16; 2],

                /// Example of a fixed-2-dimensional array with starting values
                #[variable(causality = Parameter, variability = Tunable, start = [0.0, 0.1, 1.0, 1.1, 2.0, 2.1])]
                f3: [[f64; 2]; 2],
            }
        };

        let model = Model::from(input);
        dbg!(model);

        trait FmiVariableBuilder {
            type Var: schema::AbstractVariableTrait;
            type Start;
            fn build(
                name: &str,
                value_reference: u32,
                causality: schema::Causality,
                variability: schema::Variability,
                start: impl Into<Self::Start>,
            ) -> Self::Var;
        }

        impl FmiVariableBuilder for u16 {
            type Var = schema::FmiUInt16;
            type Start = Vec<u16>;
            fn build(
                name: &str,
                value_reference: u32,
                causality: schema::Causality,
                variability: schema::Variability,
                start: impl Into<Self::Start>,
            ) -> Self::Var {
                schema::FmiUInt16::new(
                    name.to_owned(),
                    value_reference,
                    None,
                    causality,
                    variability,
                    start.into(),
                    None,
                )
            }
        }

        impl FmiVariableBuilder for f64 {
            type Var = schema::FmiFloat64;
            type Start = Vec<f64>;
            fn build(
                name: &str,
                value_reference: u32,
                causality: schema::Causality,
                variability: schema::Variability,
                start: impl Into<Self::Start>,
            ) -> Self::Var {
                schema::FmiFloat64::new(
                    name.to_owned(),
                    value_reference,
                    None,
                    causality,
                    variability,
                    start.into(),
                    None,
                )
            }
        }

        impl<const N: usize, T> FmiVariableBuilder for [T; N]
        where
            T: FmiVariableBuilder,
            T::Var: schema::ArrayableVariableTrait,
            T::Start: Into<Vec<T>>,
        {
            type Var = T::Var;
            type Start = T::Start;
            fn build(
                name: &str,
                value_reference: u32,
                causality: schema::Causality,
                variability: schema::Variability,
                start: impl Into<Self::Start>,
            ) -> Self::Var {
                let mut var = <T as FmiVariableBuilder>::build(
                    name,
                    value_reference,
                    causality,
                    variability,
                    start,
                );
                schema::ArrayableVariableTrait::add_dimensions(
                    &mut var,
                    &[schema::Dimension::fixed(N)],
                );
                var
            }
        }

        let var_f2 = <[u16; 2] as FmiVariableBuilder>::build(
            "f2",
            0,
            schema::Causality::Parameter,
            schema::Variability::Tunable,
            vec![0u16, 1, 2, 3],
        );
        dbg!(var_f2);

        /*
        let var = <[[f64; 2]; 2] as FmiVariableBuilder>::build(
            "f3",
            0,
            schema::Causality::Parameter,
            schema::Variability::Tunable,
            vec![0.0f32, 0.1, 1.0, 1.1, 2.0, 2.1],
        );
        dbg!(var);
        */
    }
}
