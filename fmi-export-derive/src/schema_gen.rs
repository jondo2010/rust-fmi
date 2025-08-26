//! Generate FMI 3.0 model description from model information

use fmi::fmi3::{schema, binding};
use uuid::Uuid;
use chrono::Utc;

use crate::model::{ExtendedModelInfo, VariableInfo};
use crate::parsing::{is_float64_type, is_float32_type};

/// Generate FMI 3.0 model description
pub fn generate_model_description(model: &ExtendedModelInfo) -> schema::Fmi3ModelDescription {
    let model_variables = generate_model_variables(&model.all_variables);
    let model_structure = generate_model_structure(&model.all_variables);

    // Generate instantiation token using UUID v5
    let instantiation_token = generate_instantiation_token(&model.model.name);

    // Use model docstring if available, otherwise default
    let description = model.model.description
        .as_deref()
        .unwrap_or("Auto-generated FMU model");

    schema::Fmi3ModelDescription {
        fmi_version: unsafe { std::str::from_utf8_unchecked(binding::fmi3Version).to_owned() },
        model_name: model.model.name.clone(),
        instantiation_token,
        description: Some(description.to_owned()),
        generation_tool: Some("rust-fmi".to_owned()),
        generation_date_and_time: Some(Utc::now().to_rfc3339()),
        model_variables,
        model_structure,
        ..Default::default()
    }
}

/// Generate model variables from variable info
fn generate_model_variables(variables: &[VariableInfo]) -> schema::ModelVariables {
    let mut float64_vars = Vec::new();
    let mut float32_vars = Vec::new();
    let mut vr_counter = 0u32;

    for var in variables {
        let name = &var.name;
        let description = var
            .description
            .as_deref()
            .unwrap_or("Auto-generated variable");

        // Parse causality
        let causality = parse_causality(var.causality.as_deref());
        let variability = parse_variability(var.variability.as_deref(), &var.field_type);
        let initial = parse_initial(var.initial.as_deref());

        if is_float64_type(&var.field_type) {
            let start_value = parse_f64_start(var.start.as_deref());
            float64_vars.push(create_float64_variable(
                name,
                description,
                vr_counter,
                causality,
                variability,
                initial,
                start_value,
            ));
        } else if is_float32_type(&var.field_type) {
            let start_value = parse_f32_start(var.start.as_deref());
            float32_vars.push(create_float32_variable(
                name,
                description,
                vr_counter,
                causality,
                variability,
                initial,
                start_value,
            ));
        }

        vr_counter += 1;

        // Process aliases for this variable
        for alias in &var.aliases {
            let alias_causality = parse_causality(alias.causality.as_deref());
            let alias_variability = parse_variability(Some("continuous"), &var.field_type);
            let alias_initial = parse_initial(Some("calculated"));

            // Find derivative value reference if this is a derivative alias
            let derivative_vr = if let Some(derivative_of) = &alias.derivative {
                find_variable_vr(variables, derivative_of)
            } else {
                None
            };

            let alias_description = alias
                .description
                .as_deref()
                .unwrap_or("Auto-generated alias variable");

            if is_float64_type(&var.field_type) {
                float64_vars.push(create_float64_variable_with_derivative(
                    &alias.name,
                    alias_description,
                    vr_counter,
                    alias_causality,
                    alias_variability,
                    alias_initial,
                    vec![], // Aliases typically don't have start values
                    derivative_vr,
                ));
            } else if is_float32_type(&var.field_type) {
                float32_vars.push(create_float32_variable_with_derivative(
                    &alias.name,
                    alias_description,
                    vr_counter,
                    alias_causality,
                    alias_variability,
                    alias_initial,
                    vec![], // Aliases typically don't have start values
                    derivative_vr,
                ));
            }

            vr_counter += 1;
        }
    }

    schema::ModelVariables {
        float64: float64_vars,
        float32: float32_vars,
        ..Default::default()
    }
}

/// Generate model structure from variable info
fn generate_model_structure(variables: &[VariableInfo]) -> schema::ModelStructure {
    let mut outputs = Vec::new();
    let mut derivatives = Vec::new();
    let mut initial_unknowns = Vec::new();
    let mut event_indicators = Vec::new();

    let mut vr_counter = 0u32;

    for var in variables {
        // Check if this is an output variable
        if var.causality.as_deref() == Some("output") {
            outputs.push(schema::Fmi3Unknown {
                value_reference: vr_counter,
                dependencies: vec![],
                dependencies_kind: vec![],
                ..Default::default()
            });
        }

        // Check if this variable should be an event indicator (e.g., height for bouncing ball)
        if var.name == "h" && var.is_state {
            event_indicators.push(schema::Fmi3Unknown {
                value_reference: vr_counter,
                dependencies: vec![],
                dependencies_kind: vec![],
                ..Default::default()
            });
        }

        vr_counter += 1;

        // Process aliases for this variable
        for alias in &var.aliases {
            // Check if this is a derivative alias
            if alias.name.starts_with("der(") && alias.causality.as_deref() == Some("local") {
                // Add as ContinuousStateDerivative
                derivatives.push(schema::Fmi3Unknown {
                    value_reference: vr_counter,
                    dependencies: vec![],
                    dependencies_kind: vec![],
                    ..Default::default()
                });

                // Add as InitialUnknown - derivative depends on the field that contains it
                let base_vr = vr_counter - 1; // The variable this alias belongs to
                initial_unknowns.push(schema::Fmi3Unknown {
                    value_reference: vr_counter,
                    dependencies: vec![base_vr],
                    dependencies_kind: vec![schema::DependenciesKind::Constant],
                    ..Default::default()
                });
            }

            vr_counter += 1;
        }
    }

    schema::ModelStructure {
        outputs,
        continuous_state_derivative: derivatives,
        initial_unknown: initial_unknowns,
        event_indicator: event_indicators,
        ..Default::default()
    }
}

// Helper functions for parsing attributes

fn parse_causality(causality: Option<&str>) -> schema::Causality {
    match causality {
        Some("parameter") => schema::Causality::Parameter,
        Some("input") => schema::Causality::Input,
        Some("output") => schema::Causality::Output,
        Some("local") => schema::Causality::Local,
        Some("independent") => schema::Causality::Independent,
        Some("calculatedParameter") => schema::Causality::CalculatedParameter,
        Some("structuralParameter") => schema::Causality::StructuralParameter,
        _ => schema::Causality::Local,
    }
}

fn parse_variability(variability: Option<&str>, field_type: &syn::Type) -> Option<schema::Variability> {
    match variability {
        Some("constant") => Some(schema::Variability::Constant),
        Some("fixed") => Some(schema::Variability::Fixed),
        Some("tunable") => Some(schema::Variability::Tunable),
        Some("discrete") => Some(schema::Variability::Discrete),
        Some("continuous") => Some(schema::Variability::Continuous),
        _ => {
            if is_float64_type(field_type) || is_float32_type(field_type) {
                Some(schema::Variability::Continuous)
            } else {
                Some(schema::Variability::Discrete)
            }
        }
    }
}

fn parse_initial(initial: Option<&str>) -> Option<schema::Initial> {
    match initial {
        Some("exact") => Some(schema::Initial::Exact),
        Some("approx") => Some(schema::Initial::Approx),
        Some("calculated") => Some(schema::Initial::Calculated),
        _ => None,
    }
}

fn parse_f64_start(start: Option<&str>) -> Vec<f64> {
    if let Some(start_str) = start {
        if let Ok(value) = start_str.parse::<f64>() {
            vec![value]
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

fn parse_f32_start(start: Option<&str>) -> Vec<f32> {
    if let Some(start_str) = start {
        if let Ok(value) = start_str.parse::<f32>() {
            vec![value]
        } else {
            vec![]
        }
    } else {
        vec![]
    }
}

fn find_variable_vr(variables: &[VariableInfo], target_name: &str) -> Option<u32> {
    let mut vr_counter = 0u32;

    for var in variables {
        if var.name == target_name {
            return Some(vr_counter);
        }
        vr_counter += 1;

        // Skip alias entries when counting
        for _ in &var.aliases {
            vr_counter += 1;
        }
    }

    None
}

// Helper functions for creating schema objects

fn create_float64_variable(
    name: &str,
    description: &str,
    value_reference: u32,
    causality: schema::Causality,
    variability: Option<schema::Variability>,
    initial: Option<schema::Initial>,
    start: Vec<f64>,
) -> schema::FmiFloat64 {
    schema::FmiFloat64 {
        init_var: schema::InitializableVariable {
            typed_arrayable_var: schema::TypedArrayableVariable {
                arrayable_var: schema::ArrayableVariable {
                    abstract_var: schema::AbstractVariable {
                        name: name.to_string(),
                        value_reference,
                        description: Some(description.to_string()),
                        causality,
                        variability,
                        can_handle_multiple_set_per_time_instant: None,
                    },
                    dimensions: vec![],
                    intermediate_update: None,
                    previous: None,
                },
                declared_type: None,
            },
            initial,
        },
        start,
        ..Default::default()
    }
}

fn create_float64_variable_with_derivative(
    name: &str,
    description: &str,
    value_reference: u32,
    causality: schema::Causality,
    variability: Option<schema::Variability>,
    initial: Option<schema::Initial>,
    start: Vec<f64>,
    derivative: Option<u32>,
) -> schema::FmiFloat64 {
    schema::FmiFloat64 {
        init_var: schema::InitializableVariable {
            typed_arrayable_var: schema::TypedArrayableVariable {
                arrayable_var: schema::ArrayableVariable {
                    abstract_var: schema::AbstractVariable {
                        name: name.to_string(),
                        value_reference,
                        description: Some(description.to_string()),
                        causality,
                        variability,
                        can_handle_multiple_set_per_time_instant: None,
                    },
                    dimensions: vec![],
                    intermediate_update: None,
                    previous: None,
                },
                declared_type: None,
            },
            initial,
        },
        start,
        real_var_attr: schema::RealVariableAttributes {
            derivative,
            reinit: None,
        },
        ..Default::default()
    }
}

fn create_float32_variable(
    name: &str,
    description: &str,
    value_reference: u32,
    causality: schema::Causality,
    variability: Option<schema::Variability>,
    initial: Option<schema::Initial>,
    start: Vec<f32>,
) -> schema::FmiFloat32 {
    schema::FmiFloat32 {
        init_var: schema::InitializableVariable {
            typed_arrayable_var: schema::TypedArrayableVariable {
                arrayable_var: schema::ArrayableVariable {
                    abstract_var: schema::AbstractVariable {
                        name: name.to_string(),
                        value_reference,
                        description: Some(description.to_string()),
                        causality,
                        variability,
                        can_handle_multiple_set_per_time_instant: None,
                    },
                    dimensions: vec![],
                    intermediate_update: None,
                    previous: None,
                },
                declared_type: None,
            },
            initial,
        },
        start,
        ..Default::default()
    }
}

fn create_float32_variable_with_derivative(
    name: &str,
    description: &str,
    value_reference: u32,
    causality: schema::Causality,
    variability: Option<schema::Variability>,
    initial: Option<schema::Initial>,
    start: Vec<f32>,
    derivative: Option<u32>,
) -> schema::FmiFloat32 {
    schema::FmiFloat32 {
        init_var: schema::InitializableVariable {
            typed_arrayable_var: schema::TypedArrayableVariable {
                arrayable_var: schema::ArrayableVariable {
                    abstract_var: schema::AbstractVariable {
                        name: name.to_string(),
                        value_reference,
                        description: Some(description.to_string()),
                        causality,
                        variability,
                        can_handle_multiple_set_per_time_instant: None,
                    },
                    dimensions: vec![],
                    intermediate_update: None,
                    previous: None,
                },
                declared_type: None,
            },
            initial,
        },
        start,
        real_var_attr: schema::RealVariableAttributes {
            derivative,
            reinit: None,
        },
        ..Default::default()
    }
}

/// Generate an instantiation token at compile time using proper UUID v5
pub fn generate_instantiation_token(model_name: &str) -> String {
    // Use the same namespace UUID as used elsewhere in rust-fmi
    const RUST_FMI_NAMESPACE: Uuid = uuid::uuid!("6ba7b810-9dad-11d1-80b4-00c04fd430c8");

    // Generate UUID v5 based on the model name
    let uuid = Uuid::new_v5(&RUST_FMI_NAMESPACE, model_name.as_bytes());

    // Format with curly braces as shown in FMI examples
    format!("{{{}}}", uuid)
}
