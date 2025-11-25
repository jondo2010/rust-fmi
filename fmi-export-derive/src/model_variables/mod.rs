//! Build a [`schema::ModelVariables`] from a [`Model`]
//!
//! Process the fields of a model struct into FMI model variables
//!
//! Key points:
//! - Each variable annotation `#[variable(...)]` becomes a model variable
//! - Each alias annotation `#[alias(...)]` also becomes a model variable
//! - Fields without variable or alias annotations are ignored (private/internal)
//! - Supports all FMI datatypes: f32, f64, i8, i16, i32, i64, u8, u16, u32, u64, bool, String

use proc_macro_error2::emit_error;

use fmi::fmi3::schema;

use crate::{
    model::{Field, FieldAttributeOuter},
    util::parse_start_value,
};

mod field_type;
pub use field_type::FieldType;

pub fn build_model_variables(fields: &[Field]) -> schema::ModelVariables {
    let mut model_variables = schema::ModelVariables::default();

    // Always add the required independent time variable with VR 0
    let time_variable = schema::FmiFloat64::new(
        "time".to_string(),
        0, // Value reference 0 is reserved for time
        Some("Simulation time".to_string()),
        schema::Causality::Independent,
        schema::Variability::Continuous,
        None, // No start value needed for independent variable
        None, // No initial value needed for independent variable
    );
    use fmi::schema::fmi3::AppendToModelVariables;
    time_variable.append_to_variables(&mut model_variables);

    let mut value_reference_counter = 1u32; // User variables start at VR 1

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
                    );
                    value_reference_counter += 1;
                }
                FieldAttributeOuter::Alias(alias_attr) => {
                    create_and_add_variable(
                        field,
                        alias_attr,
                        value_reference_counter,
                        alias_attr.name.clone(), // Use alias name if provided
                        &mut model_variables,
                    );
                    value_reference_counter += 1;
                }
                FieldAttributeOuter::Docstring(_) => {
                    // Skip docstrings - they're handled when creating variables
                }
            }
        }
    }

    model_variables
}

/// Create and add a variable to ModelVariables based on the field type
fn create_and_add_variable(
    field: &Field,
    attr: &crate::model::FieldAttribute,
    value_reference: u32,
    override_name: Option<String>,
    model_variables: &mut schema::ModelVariables,
) {
    let name = override_name.unwrap_or_else(|| field.ident.to_string());
    let description = get_variable_description(field, attr);
    let causality: schema::Causality = attr.causality.expect("Causality should be set").0;

    // Convert field type to VariableType
    let variability = get_variable_variability(attr, field.field_type.r#type);
    let initial = attr.initial.map(Into::into);

    use schema::{AppendToModelVariables, ArrayableVariableTrait};

    macro_rules! create_variable {
        ($type:ty, $var_type:ty) => {{
            let start = attr.start.as_ref().map(|start| {
                let parsed = parse_start_value::<$type>(start);
                if parsed.is_empty() {
                    emit_error!(start, format!("Failed to parse start value for {}", name));
                }
                parsed
            });
            let mut var = <$var_type>::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
                initial,
            );
            var.add_dimensions(&field.field_type.dimensions);
            var.append_to_variables(model_variables);
        }};
    }

    // Match on variable type and create appropriate FMI variable
    match field.field_type.r#type {
        schema::VariableType::FmiFloat32 => create_variable!(f32, schema::FmiFloat32),
        schema::VariableType::FmiFloat64 => create_variable!(f64, schema::FmiFloat64),
        schema::VariableType::FmiInt8 => create_variable!(i8, schema::FmiInt8),
        schema::VariableType::FmiInt16 => create_variable!(i16, schema::FmiInt16),
        schema::VariableType::FmiInt32 => create_variable!(i32, schema::FmiInt32),
        schema::VariableType::FmiInt64 => create_variable!(i64, schema::FmiInt64),
        schema::VariableType::FmiUInt8 => create_variable!(u8, schema::FmiUInt8),
        schema::VariableType::FmiUInt16 => create_variable!(u16, schema::FmiUInt16),
        schema::VariableType::FmiUInt32 => create_variable!(u32, schema::FmiUInt32),
        schema::VariableType::FmiUInt64 => create_variable!(u64, schema::FmiUInt64),
        schema::VariableType::FmiBoolean => {
            let start = attr.start.as_ref().map(parse_start_value);
            let variable = schema::FmiBoolean::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
                initial,
            );
            use fmi::schema::fmi3::AppendToModelVariables;
            variable.append_to_variables(model_variables);
        }
        schema::VariableType::FmiString => {
            let start = attr.start.as_ref().map(parse_start_value);
            let variable = schema::FmiString::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
                initial,
            );
            use fmi::schema::fmi3::AppendToModelVariables;
            variable.append_to_variables(model_variables);
        }
        schema::VariableType::FmiBinary => {
            let start = attr.start.as_ref().map(parse_start_value); // Binary uses string start values
            let variable = schema::FmiBinary::new(
                name,
                value_reference,
                description,
                causality,
                variability,
                start,
                initial,
            );
            use fmi::schema::fmi3::AppendToModelVariables;
            variable.append_to_variables(model_variables);
        }
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

/// Helper function to get variable variability from attribute with smart defaults
fn get_variable_variability(
    attr: &crate::model::FieldAttribute,
    variable_type: schema::VariableType,
) -> schema::Variability {
    if let Some(variability_ident) = &attr.variability {
        variability_ident.0
    } else {
        // Use sensible defaults based on variable type
        match variable_type {
            schema::VariableType::FmiFloat32 | schema::VariableType::FmiFloat64 => {
                schema::Variability::Continuous
            }
            _ => schema::Variability::Discrete,
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{Model, model::build_fields};

    use super::*;
    use fmi::{
        fmi3::schema,
        schema::fmi3::{
            AbstractVariableTrait, ArrayableVariableTrait, Dimension, InitializableVariableTrait,
        },
    };
    use syn::parse_quote;

    #[test]
    fn test_build_model_variables() {
        let input: syn::ItemStruct = syn::parse_quote! {
            struct TestModel {
                /// Height above ground (state output)
                #[variable(causality = Output, start = 1.0)]
                h: f64,
                /// Velocity of the ball
                #[variable(causality = Output, start = 0.0)]
                v: f64,
            }
        };
        let fields = build_fields(input.fields);
        let model_variables = build_model_variables(&fields);

        // Test that we have the correct number of float64 variables (including automatic time variable)
        assert_eq!(model_variables.float64().len(), 3);
        assert_eq!(model_variables.len(), 3);

        // Test the automatic time variable (first variable)
        let time_var = &model_variables.float64()[0];
        assert_eq!(time_var.name(), "time");
        assert_eq!(time_var.description(), Some("Simulation time"));
        assert_eq!(time_var.causality(), schema::Causality::Independent);
        assert_eq!(time_var.variability(), schema::Variability::Continuous);
        assert_eq!(time_var.value_reference(), 0);

        // Test the first user variable (h)
        let h_var = &model_variables.float64()[1]; // Now at index 1 due to time variable
        assert_eq!(h_var.name(), "h");
        assert_eq!(
            h_var.description(),
            Some("Height above ground (state output)")
        );
        assert_eq!(h_var.causality(), schema::Causality::Output);
        assert_eq!(h_var.start(), Some([1.0].as_slice()));

        // Test the second user variable (v)
        let v_var = &model_variables.float64()[2]; // Now at index 2 due to time variable
        assert_eq!(v_var.name(), "v");
        assert_eq!(v_var.description(), Some("Velocity of the ball"));
        assert_eq!(v_var.causality(), schema::Causality::Output);
        assert_eq!(v_var.start(), Some([0.0].as_slice()));
    }

    #[test]
    fn test_multiple_data_types() {
        let input: syn::ItemStruct = syn::parse_quote! {
            struct TestModel {
                #[variable(causality = Output, start = 1.5)]
                position: f32,
                #[variable(causality = Parameter, start = 42)]
                count: i32,
                #[variable(causality = Input, start = true)]
                enabled: bool,
                #[variable(causality = Parameter, start = "test")]
                name: String,
            }
        };
        let fields = build_fields(input.fields);
        let model_variables = build_model_variables(&fields);

        // Test that we have the correct distribution of variable types (including automatic time variable)
        assert_eq!(model_variables.float32().len(), 1);
        assert_eq!(model_variables.int32().len(), 1);
        assert_eq!(model_variables.boolean().len(), 1);
        assert_eq!(model_variables.string().len(), 1);
        assert_eq!(model_variables.len(), 5); // 4 user variables + 1 time variable

        // Test specific variables
        assert_eq!(model_variables.float32()[0].name(), "position");
        assert_eq!(model_variables.float32()[0].start.as_ref().map(|s| s.0.as_slice()), Some([1.5].as_slice()));

        assert_eq!(model_variables.int32()[0].name(), "count");
        assert_eq!(model_variables.int32()[0].start.as_ref().map(|s| s.0.as_slice()), Some([42].as_slice()));

        assert_eq!(model_variables.boolean()[0].name(), "enabled");
        assert_eq!(model_variables.boolean()[0].start.as_ref().map(|s| s.0.as_slice()), Some([true].as_slice()));

        assert_eq!(model_variables.string()[0].name(), "name");
        assert_eq!(
            &model_variables.string()[0].start[0].value,
            "test"
        );
    }

    #[test]
    fn test_variability_handling() {
        let input: syn::ItemStruct = syn::parse_quote! {
            struct TestModel {
                #[variable(causality = Output, variability = Continuous, start = 1.0)]
                continuous_var: f64,
                #[variable(causality = Output, variability = Discrete, start = 2.0)]
                discrete_var: f64,
                #[variable(causality = Parameter, variability = Fixed, start = 42)]
                fixed_param: i32,
                #[variable(causality = Parameter, variability = Constant, start = true)]
                constant_val: bool,
                #[variable(causality = Output, start = 3.14)]
                default_float: f32,
                #[variable(causality = Output, start = 100)]
                default_int: u32,
            }
        };
        let fields = build_fields(input.fields);

        let model_variables = build_model_variables(&fields);

        // Test that we have the correct number and distribution of variables (including automatic time variable)
        assert_eq!(model_variables.float64().len(), 3); // 2 user variables + 1 time variable
        assert_eq!(model_variables.float32().len(), 1);
        assert_eq!(model_variables.int32().len(), 1);
        assert_eq!(model_variables.boolean().len(), 1);
        assert_eq!(model_variables.uint32().len(), 1);
        assert_eq!(model_variables.len(), 7); // 6 user variables + 1 time variable

        // Test explicit variability settings (skip time variable at index 0)
        assert_eq!(
            model_variables.float64()[1].variability(), // First user variable
            schema::Variability::Continuous
        );
        assert_eq!(
            model_variables.float64()[2].variability(), // Second user variable
            schema::Variability::Discrete
        );
        assert_eq!(
            model_variables.int32()[0].variability(),
            schema::Variability::Fixed
        );
        assert_eq!(
            model_variables.boolean()[0].variability(),
            schema::Variability::Constant
        );

        // Test default variability settings
        assert_eq!(
            model_variables.float32()[0].variability(),
            schema::Variability::Continuous // Default for floats
        );
        assert_eq!(
            model_variables.uint32()[0].variability(),
            schema::Variability::Discrete // Default for integers
        );
    }

    #[test]
    fn test_arrays() {
        let input: syn::DeriveInput = parse_quote! {
            #[model()]
            struct ComprehensiveModel {
                /// Example of a fixed-size array
                #[variable(causality = Parameter, variability = Tunable, start = [0u16, 1, 2, 3])]
                f2: [u16; 4],

                /// Example of a fixed-2-dimensional array with starting values
                #[variable(causality = Parameter, variability = Tunable, start = [0.0, 0.1, 1.0, 1.1, 2.0, 2.1])]
                f3: [[f64; 2]; 2],
            }
        };

        let model = Model::from(input);
        let vars = build_model_variables(&model.fields);

        // Skip the automatic time variable at index 0
        assert_eq!(vars.uint16()[0].start(), Some([0u16, 1, 2, 3].as_slice()));
        assert_eq!(vars.uint16()[0].dimensions(), &[Dimension::Fixed(4)]);

        assert_eq!(
            vars.float64()[1].start(), // User variable is at index 1 (time variable is at index 0)
            Some([0.0, 0.1, 1.0, 1.1, 2.0, 2.1].as_slice())
        );
        assert_eq!(
            vars.float64()[1].dimensions(),
            &[Dimension::Fixed(2), Dimension::Fixed(2)]
        );
    }
}
