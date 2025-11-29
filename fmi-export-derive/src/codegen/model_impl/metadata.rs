use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use std::collections::HashMap;

use crate::Model;
use crate::model::{Field, FieldAttributeOuter};

pub struct BuildMetadataGen<'a> {
    model: &'a Model,
}

impl<'a> BuildMetadataGen<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl ToTokens for BuildMetadataGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Build a map of field names to their assigned value references
        let mut field_name_to_vr = HashMap::new();
        let mut current_vr = 0u32; // Start at 0, will be added to vr_offset (1) to get actual VRs starting at 1

        // First pass: assign VRs to all fields with variable attributes (excluding skipped ones)
        for field in &self.model.fields {
            if self.has_skip_attribute(field) {
                // Skip fields with skip=true
                continue;
            }

            for attr in &field.attrs {
                match attr {
                    FieldAttributeOuter::Variable(_) => {
                        field_name_to_vr.insert(field.ident.to_string(), current_vr);
                        current_vr += 1;
                    }
                    FieldAttributeOuter::Alias(_) => {
                        // Ignore Alias attributes for now
                    }
                    FieldAttributeOuter::Docstring(_) => {
                        // Skip docstrings
                    }
                }
            }
        }

        let mut variable_tokens = Vec::new();
        let mut model_structure_tokens = Vec::new();
        let vr_offset_tracking = quote! { let mut current_vr_offset = vr_offset; };

        // Second pass: generate the actual variable definitions
        for field in &self.model.fields {
            if self.has_skip_attribute(field) {
                // For fields with skip=true, don't generate anything
                continue;
            } else if self.has_no_variable_attributes(field) {
                // For fields with no attributes, recursively call build_metadata
                let field_type = &field.rust_type;
                variable_tokens.push(quote! {
                    let field_count = <#field_type as ::fmi_export::fmi3::Model>::build_metadata(variables, model_structure, current_vr_offset);
                    current_vr_offset += field_count;
                });
            } else {
                // Generate variables for this field
                for attr in &field.attrs {
                    match attr {
                        FieldAttributeOuter::Variable(var_attr) => {
                            let var_token = self.generate_variable_definition(
                                field,
                                var_attr,
                                &field.ident.to_string(),
                                &field_name_to_vr,
                            );
                            variable_tokens.push(var_token);

                            // Generate model structure entries for derivatives
                            if let Some(derivative_ref) = &var_attr.derivative {
                                let derivative_name = derivative_ref.to_string();
                                if field_name_to_vr.get(&derivative_name).is_some() {
                                    let current_vr = field_name_to_vr[&field.ident.to_string()];

                                    // Add this field (derivative) to continuous_state_derivative
                                    model_structure_tokens.push(quote! {
                                        model_structure.unknowns.push(::fmi::schema::fmi3::VariableDependency::ContinuousStateDerivative(::fmi::schema::fmi3::Fmi3Unknown {
                                            annotations: None,
                                            value_reference: current_vr_offset + #current_vr,
                                            dependencies: None,
                                            dependencies_kind: None,
                                        }));
                                    });

                                    // Add this field to initial_unknown if it has initial = Calculated
                                    if let Some(initial) = &var_attr.initial {
                                        let initial_schema: ::fmi::fmi3::schema::Initial =
                                            (*initial).into();
                                        if matches!(
                                            initial_schema,
                                            ::fmi::fmi3::schema::Initial::Calculated
                                        ) {
                                            model_structure_tokens.push(quote! {
                                                model_structure.unknowns.push(::fmi::schema::fmi3::VariableDependency::InitialUnknown(::fmi::schema::fmi3::Fmi3Unknown {
                                                    annotations: None,
                                                    value_reference: current_vr_offset + #current_vr,
                                                    dependencies: None,
                                                    dependencies_kind: None,
                                                }));
                                            });
                                        }
                                    }
                                }
                            }

                            // Generate outputs for fields with Output causality
                            if let Some(causality) = &var_attr.causality {
                                let causality_schema: ::fmi::fmi3::schema::Causality =
                                    (*causality).into();
                                if matches!(
                                    causality_schema,
                                    ::fmi::fmi3::schema::Causality::Output
                                ) {
                                    let current_vr = field_name_to_vr[&field.ident.to_string()];

                                    // Find if any other field has this field as derivative
                                    let mut derivative_vr = None;
                                    for other_field in &self.model.fields {
                                        for other_attr in &other_field.attrs {
                                            if let FieldAttributeOuter::Variable(other_var_attr) =
                                                other_attr
                                            {
                                                if let Some(other_derivative_ref) =
                                                    &other_var_attr.derivative
                                                {
                                                    if other_derivative_ref.to_string()
                                                        == field.ident.to_string()
                                                    {
                                                        if let Some(&other_vr) = field_name_to_vr
                                                            .get(&other_field.ident.to_string())
                                                        {
                                                            derivative_vr = Some(other_vr);
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        if derivative_vr.is_some() {
                                            break;
                                        }
                                    }

                                    if let Some(dep_vr) = derivative_vr {
                                        model_structure_tokens.push(quote! {
                                            model_structure.unknowns.push(::fmi::schema::fmi3::VariableDependency::Output(::fmi::schema::fmi3::Fmi3Unknown {
                                                annotations: None,
                                                value_reference: current_vr_offset + #current_vr,
                                                dependencies: Some(::fmi::schema::utils::AttrList(vec![current_vr_offset + #dep_vr])),
                                                dependencies_kind: Some(::fmi::schema::utils::AttrList(vec![::fmi::schema::fmi3::DependenciesKind::Dependent])),
                                            }));
                                        });
                                    } else {
                                        // Output without dependencies
                                        model_structure_tokens.push(quote! {
                                            model_structure.unknowns.push(::fmi::schema::fmi3::VariableDependency::Output(::fmi::schema::fmi3::Fmi3Unknown {
                                                annotations: None,
                                                value_reference: current_vr_offset + #current_vr,
                                                dependencies: None,
                                                dependencies_kind: None,
                                            }));
                                        });
                                    }
                                }
                            }
                        }
                        FieldAttributeOuter::Alias(_) => {
                            // Ignore Alias attributes for now
                        }
                        FieldAttributeOuter::Docstring(_) => {
                            // Skip docstrings
                        }
                    }
                }
            }
        }

        // Count how many variables this model defines
        let own_variable_count = field_name_to_vr.len() as u32;

        tokens.extend(quote! {
            use ::fmi_export::fmi3::FmiVariableBuilder;
            use ::fmi::schema::fmi3::AppendToModelVariables;
            #vr_offset_tracking
            #(#variable_tokens)*
            #(#model_structure_tokens)*
            current_vr_offset - vr_offset + #own_variable_count
        });
    }
}

impl BuildMetadataGen<'_> {
    /// Check if a field has the skip attribute set to true
    fn has_skip_attribute(&self, field: &Field) -> bool {
        field
            .attrs
            .iter()
            .any(|attr| matches!(attr, FieldAttributeOuter::Variable(var_attr) if var_attr.skip))
    }

    /// Check if a field has no variable attributes (ignoring docstrings and aliases)
    fn has_no_variable_attributes(&self, field: &Field) -> bool {
        !field
            .attrs
            .iter()
            .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)))
    }

    /// Generate a variable definition for a field
    fn generate_variable_definition(
        &self,
        field: &Field,
        var_attr: &crate::model::FieldAttribute,
        var_name: &str,
        field_name_to_vr: &HashMap<String, u32>,
    ) -> TokenStream2 {
        let field_type = &field.rust_type;
        let current_vr = field_name_to_vr[var_name];

        // Use the name attribute if specified, otherwise use the field name
        let variable_name = var_attr.name.as_ref()
            .map(|s| s.clone())
            .unwrap_or_else(|| field.ident.to_string());

        // Build the variable definition
        let mut builder_calls = Vec::new();

        // Set causality if specified
        if let Some(causality) = &var_attr.causality {
            let causality_schema: fmi::fmi3::schema::Causality = (*causality).into();
            let causality_str = format!("{:?}", causality_schema);
            let causality_variant = format_ident!("{}", causality_str);
            builder_calls.push(quote! {
                .with_causality(::fmi::fmi3::schema::Causality::#causality_variant)
            });
        }

        // Set variability if specified
        if let Some(variability) = &var_attr.variability {
            let variability_schema: fmi::fmi3::schema::Variability = (*variability).into();
            let variability_str = format!("{:?}", variability_schema);
            let variability_variant = format_ident!("{}", variability_str);
            builder_calls.push(quote! {
                .with_variability(::fmi::fmi3::schema::Variability::#variability_variant)
            });
        }

        // Set start value if specified
        if let Some(start) = &var_attr.start {
            builder_calls.push(quote! {
                .with_start(#start)
            });
        }

        // Set initial if specified
        if let Some(initial) = &var_attr.initial {
            let initial_schema: fmi::fmi3::schema::Initial = (*initial).into();
            let initial_str = format!("{:?}", initial_schema);
            let initial_variant = format_ident!("{}", initial_str);
            builder_calls.push(quote! {
                .with_initial(::fmi::fmi3::schema::Initial::#initial_variant)
            });
        }

        // Set derivative if specified
        if let Some(derivative_ref) = &var_attr.derivative {
            let derivative_name = derivative_ref.to_string();
            if let Some(&derivative_vr) = field_name_to_vr.get(&derivative_name) {
                builder_calls.push(quote! {
                    .with_derivative(#derivative_vr)
                });
            }
        }

        // Set max_size if specified (for Binary variables)
        if let Some(max_size) = var_attr.max_size {
            builder_calls.push(quote! {
                .with_max_size(#max_size)
            });
        }

        // Set mime_type if specified (for Binary variables)
        if let Some(mime_type) = &var_attr.mime_type {
            builder_calls.push(quote! {
                .with_mime_type(#mime_type)
            });
        }

        // Set clocks if specified
        if let Some(clocks) = &var_attr.clocks {
            // Convert Vec<syn::Ident> to Vec<u32> by looking up VRs and adding vr_offset
            let clock_vrs: Vec<u32> = clocks
                .iter()
                .filter_map(|clock_ident| {
                    let clock_name = clock_ident.to_string();
                    field_name_to_vr.get(&clock_name).map(|&vr| vr + 1) // Add 1 for vr_offset
                })
                .collect();
            
            if !clock_vrs.is_empty() {
                builder_calls.push(quote! {
                    .with_clocks(vec![#(#clock_vrs),*])
                });
            }
        }

        // Set description from field docstring or attribute
        let description = if let Some(attr_desc) = &var_attr.description {
            quote! { .with_description(#attr_desc) }
        } else {
            let field_desc = field.fold_description();
            if !field_desc.is_empty() {
                quote! { .with_description(#field_desc) }
            } else {
                quote! {}
            }
        };

        quote! {
            <#field_type as ::fmi_export::fmi3::FmiVariableBuilder>::variable(#variable_name, current_vr_offset + #current_vr)
                #description
                #(#builder_calls)*
                .finish()
                .append_to_variables(variables);
        }
    }
}
