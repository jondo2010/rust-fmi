use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

use crate::model::{Field, FieldAttributeOuter, Model};

/// Generator for the ModelGetSetStates trait implementation
pub struct ModelGetSetStatesImpl<'a> {
    pub struct_name: &'a Ident,
    pub model: &'a Model,
}

impl<'a> ModelGetSetStatesImpl<'a> {
    pub fn new(struct_name: &'a Ident, model: &'a Model) -> Self {
        Self { struct_name, model }
    }

    fn child_fields(&self) -> impl Iterator<Item = &Field> {
        self.model.fields.iter().filter(|field| {
            field
                .attrs
                .iter()
                .any(|attr| matches!(attr, FieldAttributeOuter::Child(_)))
        })
    }

    /// Generate the NUM_STATES constant calculation
    fn generate_num_states(&self) -> TokenStream2 {
        let mut state_counts = Vec::new();

        for field in self.model.iter_continuous_states() {
            let field_type = &field.rust_type;
            state_counts.push(quote! {
                <#field_type as ::fmi_export::fmi3::ModelGetSetStates>::NUM_STATES
            });
        }

        for field in self.child_fields() {
            let field_type = &field.rust_type;
            state_counts.push(quote! {
                <#field_type as ::fmi_export::fmi3::ModelGetSetStates>::NUM_STATES
            });
        }

        if state_counts.is_empty() {
            quote! { 0 }
        } else {
            quote! { #(#state_counts)+* }
        }
    }

    /// Generate the get_continuous_states method implementation
    fn generate_get_continuous_states(&self) -> TokenStream2 {
        let mut state_assignments = Vec::new();
        let mut offset_var = quote! { 0usize };

        for field in self.model.iter_continuous_states() {
            let field_name = &field.ident;
            let field_type = &field.rust_type;
            let count_var = format_ident!("{}_count", field_name);

            state_assignments.push(quote! {
                let #count_var = <#field_type as ::fmi_export::fmi3::ModelGetSetStates>::NUM_STATES;
                <#field_type as ::fmi_export::fmi3::ModelGetSetStates>::get_continuous_states(
                    &self.#field_name,
                    &mut states[#offset_var..#offset_var + #count_var]
                )?;
            });

            // Update offset for next iteration
            offset_var = quote! { #offset_var + #count_var };
        }

        for field in self.child_fields() {
            let field_name = &field.ident;
            let field_type = &field.rust_type;
            let count_var = format_ident!("{}_count", field_name);

            state_assignments.push(quote! {
                let #count_var = <#field_type as ::fmi_export::fmi3::ModelGetSetStates>::NUM_STATES;
                <#field_type as ::fmi_export::fmi3::ModelGetSetStates>::get_continuous_states(
                    &self.#field_name,
                    &mut states[#offset_var..#offset_var + #count_var]
                )?;
            });

            offset_var = quote! { #offset_var + #count_var };
        }

        if state_assignments.is_empty() {
            quote! {
                // No continuous states in this model
                Ok(())
            }
        } else {
            quote! {
                #(#state_assignments)*
                Ok(())
            }
        }
    }

    /// Generate the set_continuous_states method implementation
    fn generate_set_continuous_states(&self) -> TokenStream2 {
        let mut state_assignments = Vec::new();
        let mut offset_var = quote! { 0usize };

        for field in self.model.iter_continuous_states() {
            let field_name = &field.ident;
            let field_type = &field.rust_type;
            let count_var = format_ident!("{}_count", field_name);

            state_assignments.push(quote! {
                let #count_var = <#field_type as ::fmi_export::fmi3::ModelGetSetStates>::NUM_STATES;
                <#field_type as ::fmi_export::fmi3::ModelGetSetStates>::set_continuous_states(
                    &mut self.#field_name,
                    &states[#offset_var..#offset_var + #count_var]
                )?;
            });

            // Update offset for next iteration
            offset_var = quote! { #offset_var + #count_var };
        }

        for field in self.child_fields() {
            let field_name = &field.ident;
            let field_type = &field.rust_type;
            let count_var = format_ident!("{}_count", field_name);

            state_assignments.push(quote! {
                let #count_var = <#field_type as ::fmi_export::fmi3::ModelGetSetStates>::NUM_STATES;
                <#field_type as ::fmi_export::fmi3::ModelGetSetStates>::set_continuous_states(
                    &mut self.#field_name,
                    &states[#offset_var..#offset_var + #count_var]
                )?;
            });

            offset_var = quote! { #offset_var + #count_var };
        }

        if state_assignments.is_empty() {
            quote! {
                // No continuous states in this model
                Ok(())
            }
        } else {
            quote! {
                #(#state_assignments)*
                Ok(())
            }
        }
    }

    /// Generate the get_continuous_state_derivatives method implementation
    fn generate_get_continuous_state_derivatives(&self) -> TokenStream2 {
        let mut derivative_assignments = Vec::new();

        // Create a mapping from state names to their positions for proper ordering
        let mut state_positions: std::collections::HashMap<String, usize> =
            std::collections::HashMap::new();

        // Build state position map
        for (index, field) in self.model.iter_continuous_states().enumerate() {
            state_positions.insert(field.ident.to_string(), index);
        }

                // Collect derivatives with their corresponding state positions
        let mut derivatives_with_positions: Vec<_> = self.model
            .iter_derivatives()
            .filter_map(|der_field| {
                // Find which state this derivative corresponds to
                for attr in &der_field.attrs {
                    // Check both Variable and Alias attributes for derivative information
                    let derivative_of = match attr {
                        FieldAttributeOuter::Variable(var_attr) => &var_attr.derivative,
                        FieldAttributeOuter::Alias(alias_attr) => &alias_attr.derivative,
                        _ => continue,
                    };
                    
                    if let Some(derivative_of) = derivative_of {
                        let state_name = derivative_of.to_string();
                        if let Some(&position) = state_positions.get(&state_name) {
                            return Some((position, der_field));
                        }
                    }
                }
                None
            })
            .collect();

        // Sort derivatives by their corresponding state positions to maintain correct order
        derivatives_with_positions.sort_by_key(|(position, _)| *position);

        let mut offset_var = quote! { 0usize };

        // Generate assignments for each derivative in the correct order
        for (_, der_field) in derivatives_with_positions {
            let der_field_name = &der_field.ident;
            let der_field_type = &der_field.rust_type;
            let count_var = format_ident!("{}_count", der_field_name);

            derivative_assignments.push(quote! {
                let #count_var = <#der_field_type as ::fmi_export::fmi3::ModelGetSetStates>::NUM_STATES;
                <#der_field_type as ::fmi_export::fmi3::ModelGetSetStates>::get_continuous_state_derivatives(
                    &mut self.#der_field_name,
                    &mut derivatives[#offset_var..#offset_var + #count_var]
                )?;
            });

            // Update offset for next iteration
            offset_var = quote! { #offset_var + #count_var };
        }

        for field in self.child_fields() {
            let field_name = &field.ident;
            let field_type = &field.rust_type;
            let count_var = format_ident!("{}_count", field_name);

            derivative_assignments.push(quote! {
                let #count_var = <#field_type as ::fmi_export::fmi3::ModelGetSetStates>::NUM_STATES;
                <#field_type as ::fmi_export::fmi3::ModelGetSetStates>::get_continuous_state_derivatives(
                    &mut self.#field_name,
                    &mut derivatives[#offset_var..#offset_var + #count_var]
                )?;
            });

            offset_var = quote! { #offset_var + #count_var };
        }

        if derivative_assignments.is_empty() {
            quote! {
                // No derivatives in this model
                Ok(())
            }
        } else {
            quote! {
                #(#derivative_assignments)*
                Ok(())
            }
        }
    }
}

impl ToTokens for ModelGetSetStatesImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = self.struct_name;
        let num_states = self.generate_num_states();
        let get_continuous_states_body = self.generate_get_continuous_states();
        let set_continuous_states_body = self.generate_set_continuous_states();
        let get_continuous_state_derivatives_body =
            self.generate_get_continuous_state_derivatives();

        // Always generate the trait implementation, even for models with no states
        tokens.extend(quote! {
            impl ::fmi_export::fmi3::ModelGetSetStates for #struct_name {
                const NUM_STATES: usize = #num_states;

                fn get_continuous_states(&self, states: &mut [f64]) -> Result<(), ::fmi::fmi3::Fmi3Error> {
                    #get_continuous_states_body
                }

                fn set_continuous_states(&mut self, states: &[f64]) -> Result<(), ::fmi::fmi3::Fmi3Error> {
                    #set_continuous_states_body
                }

                fn get_continuous_state_derivatives(
                    &mut self,
                    derivatives: &mut [f64],
                ) -> Result<(), ::fmi::fmi3::Fmi3Error> {
                    #get_continuous_state_derivatives_body
                }
            }
        });
    }
}
