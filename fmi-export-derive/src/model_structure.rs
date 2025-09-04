//! Build the ModelStructure from fields and model variables
//!
//! This function identifies:
//! - Outputs: variables with causality = Output
//! - Continuous state derivatives: variables that are derivatives of continuous states
//! - Initial unknowns: variables that are outputs or local variables
//! - Event indicators: variables marked with event_indicator = true

use crate::model::{Field, FieldAttributeOuter};
use std::collections::{HashMap, HashSet};

use fmi::fmi3::schema::{self, AbstractVariableTrait, InitializableVariableTrait};

/// Collection of mappings derived from ModelVariables for efficient lookup
struct VariableMappings {
    name_to_value_ref: HashMap<String, u32>,
    name_to_initial: HashMap<String, schema::Initial>,
}

impl VariableMappings {
    /// Create variable mappings from ModelVariables
    fn new(model_variables: &schema::ModelVariables) -> Self {
        let mut name_to_value_ref = HashMap::new();
        let mut name_to_initial = HashMap::new();

        // Helper macro to collect variables with initial attributes
        macro_rules! collect_vars {
            ($vars:expr) => {
                for var in $vars {
                    name_to_value_ref.insert(var.name().to_string(), var.value_reference());
                    let initial = var.initial().unwrap_or(schema::Initial::Exact);
                    name_to_initial.insert(var.name().to_string(), initial);
                }
            };
        }

        // Collect all variable types that have initial attributes
        collect_vars!(&model_variables.float32);
        collect_vars!(&model_variables.float64);
        collect_vars!(&model_variables.int8);
        collect_vars!(&model_variables.uint8);
        collect_vars!(&model_variables.int16);
        collect_vars!(&model_variables.uint16);
        collect_vars!(&model_variables.int32);
        collect_vars!(&model_variables.uint32);
        collect_vars!(&model_variables.int64);
        collect_vars!(&model_variables.uint64);
        collect_vars!(&model_variables.boolean);
        collect_vars!(&model_variables.string);
        collect_vars!(&model_variables.binary);

        Self {
            name_to_value_ref,
            name_to_initial,
        }
    }

    /// Check if a variable should be an InitialUnknown based on FMI3 specification:
    /// - Must not be an event indicator
    /// - Must have initial="calculated" or initial="approx" (not initial="exact")
    fn should_be_initial_unknown(
        &self,
        var_name: &str,
        event_indicators: &HashSet<u32>,
    ) -> Option<u32> {
        let value_ref = *self.name_to_value_ref.get(var_name)?;

        // Exclude event indicators
        if event_indicators.contains(&value_ref) {
            return None;
        }

        // Only include variables with initial="calculated" or initial="approx"
        match self.name_to_initial.get(var_name) {
            Some(schema::Initial::Calculated | schema::Initial::Approx) => Some(value_ref),
            _ => None, // initial="exact" or missing -> not an InitialUnknown
        }
    }

    fn get_value_ref(&self, var_name: &str) -> Option<u32> {
        self.name_to_value_ref.get(var_name).copied()
    }
}

pub fn build_model_structure(
    fields: &[Field],
    model_variables: &schema::ModelVariables,
) -> Result<schema::ModelStructure, String> {
    let mappings = VariableMappings::new(model_variables);

    // Track which variables are event indicators to exclude them from InitialUnknowns
    let mut event_indicator_value_refs = HashSet::new();

    // First pass: identify event indicators and collect their value references
    let event_indicators =
        collect_event_indicators(fields, &mappings, &mut event_indicator_value_refs)?;

    // Second pass: identify outputs, derivatives, and initial unknowns
    let (outputs, continuous_state_derivatives, initial_unknowns) =
        collect_outputs_and_unknowns(fields, &mappings, &event_indicator_value_refs)?;

    Ok(schema::ModelStructure {
        outputs,
        continuous_state_derivative: continuous_state_derivatives,
        initial_unknown: initial_unknowns,
        event_indicator: event_indicators,
        ..Default::default()
    })
}

/// Collect event indicators from fields and populate the value reference set
fn collect_event_indicators(
    fields: &[Field],
    mappings: &VariableMappings,
    event_indicator_value_refs: &mut HashSet<u32>,
) -> Result<Vec<schema::Fmi3Unknown>, String> {
    let mut event_indicators = Vec::new();

    for field in fields {
        for attr in &field.attrs {
            match attr {
                FieldAttributeOuter::Variable(var_attr) => {
                    if var_attr.event_indicator == Some(true) {
                        if let Some(value_ref) = mappings.get_value_ref(&field.ident.to_string()) {
                            event_indicators.push(schema::Fmi3Unknown {
                                value_reference: value_ref,
                                ..Default::default()
                            });
                            event_indicator_value_refs.insert(value_ref);
                        }
                    }
                }
                FieldAttributeOuter::Alias(alias_attr) => {
                    if alias_attr.event_indicator == Some(true) {
                        let field_name = field.ident.to_string();
                        let var_name = alias_attr.name.as_ref().unwrap_or(&field_name);
                        if let Some(value_ref) = mappings.get_value_ref(var_name) {
                            event_indicators.push(schema::Fmi3Unknown {
                                value_reference: value_ref,
                                ..Default::default()
                            });
                            event_indicator_value_refs.insert(value_ref);
                        }
                    }
                }
                FieldAttributeOuter::Docstring(_) => {
                    // Skip docstrings
                }
            }
        }
    }

    Ok(event_indicators)
}

/// Collect outputs, continuous state derivatives, and initial unknowns
fn collect_outputs_and_unknowns(
    fields: &[Field],
    mappings: &VariableMappings,
    event_indicator_value_refs: &HashSet<u32>,
) -> Result<
    (
        Vec<schema::Fmi3Unknown>,
        Vec<schema::Fmi3Unknown>,
        Vec<schema::Fmi3Unknown>,
    ),
    String,
> {
    let mut outputs = Vec::new();
    let mut continuous_state_derivatives = Vec::new();
    let mut initial_unknowns = Vec::new();

    for field in fields {
        for attr in &field.attrs {
            match attr {
                FieldAttributeOuter::Variable(var_attr) => {
                    let var_name = &field.ident.to_string();
                    process_variable_attribute(
                        var_attr,
                        var_name,
                        mappings,
                        event_indicator_value_refs,
                        &mut outputs,
                        &mut continuous_state_derivatives,
                        &mut initial_unknowns,
                    )?;
                }
                FieldAttributeOuter::Alias(alias_attr) => {
                    let field_name = field.ident.to_string();
                    let var_name = alias_attr.name.as_ref().unwrap_or(&field_name);
                    process_alias_attribute(
                        alias_attr,
                        var_name,
                        mappings,
                        event_indicator_value_refs,
                        &mut outputs,
                        &mut continuous_state_derivatives,
                        &mut initial_unknowns,
                    )?;
                }
                FieldAttributeOuter::Docstring(_) => {
                    // Skip docstrings
                }
            }
        }
    }

    Ok((outputs, continuous_state_derivatives, initial_unknowns))
}

/// Process a variable attribute and update the appropriate collections
fn process_variable_attribute(
    var_attr: &crate::model::FieldAttribute,
    var_name: &str,
    mappings: &VariableMappings,
    event_indicator_value_refs: &HashSet<u32>,
    outputs: &mut Vec<schema::Fmi3Unknown>,
    continuous_state_derivatives: &mut Vec<schema::Fmi3Unknown>,
    initial_unknowns: &mut Vec<schema::Fmi3Unknown>,
) -> Result<(), String> {
    if let Some(causality_ident) = &var_attr.causality {
        match &causality_ident.0 {
            schema::Causality::Output => {
                if let Some(value_ref) = mappings.get_value_ref(var_name) {
                    outputs.push(schema::Fmi3Unknown {
                        value_reference: value_ref,
                        ..Default::default()
                    });

                    // Check if this should be an InitialUnknown
                    if let Some(initial_unknown_ref) =
                        mappings.should_be_initial_unknown(var_name, event_indicator_value_refs)
                    {
                        initial_unknowns.push(schema::Fmi3Unknown {
                            value_reference: initial_unknown_ref,
                            ..Default::default()
                        });
                    }
                }
            }
            schema::Causality::Local => {
                if let Some(value_ref) = mappings.get_value_ref(var_name) {
                    // Check if this is a derivative of a state variable
                    if var_attr.derivative.is_some() {
                        continuous_state_derivatives.push(schema::Fmi3Unknown {
                            value_reference: value_ref,
                            ..Default::default()
                        });
                    }

                    // Local variables are potential InitialUnknowns
                    if let Some(initial_unknown_ref) =
                        mappings.should_be_initial_unknown(var_name, event_indicator_value_refs)
                    {
                        initial_unknowns.push(schema::Fmi3Unknown {
                            value_reference: initial_unknown_ref,
                            ..Default::default()
                        });
                    }
                }
            }
            _ => {
                // Other causality types don't need special handling here
            }
        }
    }

    Ok(())
}

/// Process an alias attribute and update the appropriate collections
fn process_alias_attribute(
    alias_attr: &crate::model::FieldAttribute,
    var_name: &str,
    mappings: &VariableMappings,
    event_indicator_value_refs: &HashSet<u32>,
    outputs: &mut Vec<schema::Fmi3Unknown>,
    continuous_state_derivatives: &mut Vec<schema::Fmi3Unknown>,
    initial_unknowns: &mut Vec<schema::Fmi3Unknown>,
) -> Result<(), String> {
    match alias_attr.causality.as_ref().map(|c| &c.0) {
        Some(schema::Causality::Output) => {
            if let Some(value_ref) = mappings.get_value_ref(var_name) {
                outputs.push(schema::Fmi3Unknown {
                    value_reference: value_ref,
                    ..Default::default()
                });

                // Check if this should be an InitialUnknown
                if let Some(initial_unknown_ref) =
                    mappings.should_be_initial_unknown(var_name, event_indicator_value_refs)
                {
                    initial_unknowns.push(schema::Fmi3Unknown {
                        value_reference: initial_unknown_ref,
                        ..Default::default()
                    });
                }
            }
        }
        Some(schema::Causality::Local) => {
            if let Some(value_ref) = mappings.get_value_ref(var_name) {
                // Check if this is a derivative of a state variable
                if let Some(alias_name) = &alias_attr.name {
                    if alias_name.starts_with("der(") && alias_name.ends_with(")") {
                        continuous_state_derivatives.push(schema::Fmi3Unknown {
                            value_reference: value_ref,
                            ..Default::default()
                        });
                    }
                }

                // Check if this should be an InitialUnknown
                if let Some(initial_unknown_ref) =
                    mappings.should_be_initial_unknown(var_name, event_indicator_value_refs)
                {
                    initial_unknowns.push(schema::Fmi3Unknown {
                        value_reference: initial_unknown_ref,
                        ..Default::default()
                    });
                }
            }
        }
        _ => {
            // Other causalities are not processed for aliases
        }
    }

    Ok(())
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

        let input: syn::DeriveInput = parse_quote! {
            /// A comprehensive FMI model demonstrating all supported datatypes
            #[model()]
            struct Dummy {
            }
        };

        let fields = vec![
            // State variable (height)
            Field {
                ident: parse_quote!(h),
                ty: parse_quote!(f64),
                attrs: vec![FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(crate::model::Causality(schema::Causality::Output)),
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
                        causality: Some(crate::model::Causality(schema::Causality::Output)),
                        state: Some(true),
                        start: Some(parse_quote!(0.0)),
                        ..Default::default()
                    }),
                    FieldAttributeOuter::Alias(FieldAttribute {
                        name: Some("der(h)".to_string()),
                        causality: Some(crate::model::Causality(schema::Causality::Local)),
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
                        causality: Some(crate::model::Causality(schema::Causality::Parameter)),
                        start: Some(parse_quote!(-9.81)),
                        ..Default::default()
                    }),
                    FieldAttributeOuter::Alias(FieldAttribute {
                        name: Some("der(v)".to_string()),
                        causality: Some(crate::model::Causality(schema::Causality::Local)),
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

        // Test initial unknowns: According to FMI3 specification, only variables with
        // initial="calculated" or initial="approx" should be InitialUnknowns.
        // Variables with initial="exact" (default) are NOT InitialUnknowns since they have known values.
        // In this test, all variables use the default initial="exact", so there should be no InitialUnknowns.
        assert_eq!(model_structure.initial_unknown.len(), 0);
    }
}
