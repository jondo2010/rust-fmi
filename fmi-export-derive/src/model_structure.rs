//! Build the ModelStructure from fields and model variables
//!
//! This function identifies:
//! - Outputs: variables with causality = Output
//! - Continuous state derivatives: variables that are derivatives of continuous states
//! - Initial unknowns: variables that are outputs or local variables
//! - Event indicators: variables marked with event_indicator = true

use crate::model::{Field, FieldAttributeOuter};

use fmi::fmi3::schema;

pub fn build_model_structure(
    fields: &[Field],
    model_variables: &schema::ModelVariables,
) -> Result<schema::ModelStructure, String> {
    let mut outputs = Vec::new();
    let mut continuous_state_derivatives = Vec::new();
    let mut initial_unknowns = Vec::new();
    let mut event_indicators = Vec::new();

    // Create a mapping from variable names to value references
    let mut name_to_value_ref = std::collections::HashMap::new();

    // Collect all variable names and their value references
    for var in model_variables.iter_abstract() {
        name_to_value_ref.insert(var.name().to_string(), var.value_reference());
    }

    // Track state variables and their derivative relationships
    let mut state_variables = std::collections::HashMap::new();
    let mut derivative_variables = Vec::new();

    // First pass: identify state variables and derivatives
    for field in fields {
        for attr in &field.attrs {
            match attr {
                FieldAttributeOuter::Variable(var_attr) => {
                    if var_attr.state == Some(true) {
                        state_variables.insert(field.ident.to_string(), field.ident.to_string());
                    }

                    // Check if this is an event indicator
                    if var_attr.event_indicator == Some(true) {
                        if let Some(&value_ref) = name_to_value_ref.get(&field.ident.to_string()) {
                            event_indicators.push(schema::Fmi3Unknown {
                                value_reference: value_ref,
                                ..Default::default()
                            });
                        }
                    }

                    // Check if this is an output variable
                    if let Some(causality_ident) = &var_attr.causality {
                        if causality_ident.to_string() == "Output" {
                            if let Some(&value_ref) =
                                name_to_value_ref.get(&field.ident.to_string())
                            {
                                outputs.push(schema::Fmi3Unknown {
                                    value_reference: value_ref,
                                    ..Default::default()
                                });

                                // Outputs are also initial unknowns
                                initial_unknowns.push(schema::Fmi3Unknown {
                                    value_reference: value_ref,
                                    ..Default::default()
                                });
                            }
                        } else if causality_ident.to_string() == "Local" {
                            // Local variables are initial unknowns
                            if let Some(&value_ref) =
                                name_to_value_ref.get(&field.ident.to_string())
                            {
                                initial_unknowns.push(schema::Fmi3Unknown {
                                    value_reference: value_ref,
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
                FieldAttributeOuter::Alias(alias_attr) => {
                    // Check if this alias represents a derivative
                    if let Some(alias_name) = &alias_attr.name {
                        if alias_name.starts_with("der(") && alias_name.ends_with(")") {
                            derivative_variables
                                .push((alias_name.clone(), field.ident.to_string()));
                        }
                    }

                    // Check if this alias is an event indicator
                    if alias_attr.event_indicator == Some(true) {
                        let field_name = field.ident.to_string();
                        let var_name = alias_attr.name.as_ref().unwrap_or(&field_name);
                        if let Some(&value_ref) = name_to_value_ref.get(var_name) {
                            event_indicators.push(schema::Fmi3Unknown {
                                value_reference: value_ref,
                                ..Default::default()
                            });
                        }
                    }

                    // Check if this is an output alias
                    if let Some(causality_ident) = &alias_attr.causality {
                        if causality_ident.to_string() == "Output" {
                            let field_name = field.ident.to_string();
                            let var_name = alias_attr.name.as_ref().unwrap_or(&field_name);
                            if let Some(&value_ref) = name_to_value_ref.get(var_name) {
                                outputs.push(schema::Fmi3Unknown {
                                    value_reference: value_ref,
                                    ..Default::default()
                                });

                                // Outputs are also initial unknowns
                                initial_unknowns.push(schema::Fmi3Unknown {
                                    value_reference: value_ref,
                                    ..Default::default()
                                });
                            }
                        } else if causality_ident.to_string() == "Local" {
                            let field_name = field.ident.to_string();
                            let var_name = alias_attr.name.as_ref().unwrap_or(&field_name);
                            if let Some(&value_ref) = name_to_value_ref.get(var_name) {
                                // Check if this is a derivative of a state variable
                                if let Some(alias_name) = &alias_attr.name {
                                    if alias_name.starts_with("der(") && alias_name.ends_with(")") {
                                        continuous_state_derivatives.push(schema::Fmi3Unknown {
                                            value_reference: value_ref,
                                            ..Default::default()
                                        });
                                    }
                                }

                                // Local variables are also initial unknowns
                                initial_unknowns.push(schema::Fmi3Unknown {
                                    value_reference: value_ref,
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
                FieldAttributeOuter::Docstring(_) => {
                    // Skip docstrings
                }
            }
        }
    }

    Ok(schema::ModelStructure {
        outputs,
        continuous_state_derivative: continuous_state_derivatives,
        initial_unknown: initial_unknowns,
        event_indicator: event_indicators,
        ..Default::default()
    })
}

#[cfg(test)]
mod tests {
    use fmi::schema::fmi3::AbstractVariableTrait;

    use crate::model_variables::build_model_variables;

    use super::*;

    #[test]
    fn test_model_structure() {
        use crate::model::{FieldAttribute, FieldAttributeOuter};
        use syn::parse_quote;

        let fields = vec![
            // State variable (height)
            Field {
                ident: parse_quote!(h),
                ty: parse_quote!(f64),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(parse_quote!(Output)),
                    state: Some(true),
                    start: Some(parse_quote!(1.0)),
                    ..Default::default()
                })],
            },
            // State variable (velocity) with derivative alias
            Field {
                ident: parse_quote!(v),
                ty: parse_quote!(f64),
                attrs: vec![
                    FieldAttributeOuter::Variable(FieldAttribute {
                        causality: Some(parse_quote!(Output)),
                        state: Some(true),
                        start: Some(parse_quote!(0.0)),
                        ..Default::default()
                    }),
                    FieldAttributeOuter::Alias(FieldAttribute {
                        name: Some("der(h)".to_string()),
                        causality: Some(parse_quote!(Local)),
                        derivative: Some(parse_quote!(h)),
                        ..Default::default()
                    }),
                ],
            },
            // Gravitational acceleration (derivative of velocity)
            Field {
                ident: parse_quote!(g),
                ty: parse_quote!(f64),
                attrs: vec![
                    FieldAttributeOuter::Variable(FieldAttribute {
                        causality: Some(parse_quote!(Parameter)),
                        start: Some(parse_quote!(-9.81)),
                        ..Default::default()
                    }),
                    FieldAttributeOuter::Alias(FieldAttribute {
                        name: Some("der(v)".to_string()),
                        causality: Some(parse_quote!(Local)),
                        derivative: Some(parse_quote!(v)),
                        ..Default::default()
                    }),
                ],
            },
        ];

        let model_variables = build_model_variables(&fields).unwrap();
        let model_structure = build_model_structure(&fields, &model_variables).unwrap();

        // Test outputs: h and v should be outputs
        assert_eq!(model_structure.outputs.len(), 2);

        // Find value references for h and v
        let h_value_ref = model_variables
            .float64
            .iter()
            .find(|var| var.name() == "h")
            .map(|var| var.value_reference())
            .unwrap();
        let v_value_ref = model_variables
            .float64
            .iter()
            .find(|var| var.name() == "v")
            .map(|var| var.value_reference())
            .unwrap();

        assert!(
            model_structure
                .outputs
                .iter()
                .any(|out| out.value_reference == h_value_ref)
        );
        assert!(
            model_structure
                .outputs
                .iter()
                .any(|out| out.value_reference == v_value_ref)
        );

        // Test continuous state derivatives: der(h) and der(v)
        assert_eq!(model_structure.continuous_state_derivative.len(), 2);

        // Find value references for der(h) and der(v)
        let der_h_value_ref = model_variables
            .float64
            .iter()
            .find(|var| var.name() == "der(h)")
            .map(|var| var.value_reference())
            .unwrap();
        let der_v_value_ref = model_variables
            .float64
            .iter()
            .find(|var| var.name() == "der(v)")
            .map(|var| var.value_reference())
            .unwrap();

        assert!(
            model_structure
                .continuous_state_derivative
                .iter()
                .any(|der| der.value_reference == der_h_value_ref)
        );
        assert!(
            model_structure
                .continuous_state_derivative
                .iter()
                .any(|der| der.value_reference == der_v_value_ref)
        );

        // Test initial unknowns: should include outputs and local variables
        assert!(model_structure.initial_unknown.len() >= 2); // At least outputs + locals
    }
}
