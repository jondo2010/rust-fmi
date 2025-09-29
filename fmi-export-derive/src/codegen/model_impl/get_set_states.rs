use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};

use crate::{Model, model::FieldAttributeOuter};

pub struct GetContinuousStatesGen<'a>(&'a Model);

impl<'a> GetContinuousStatesGen<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self(model)
    }
}

impl ToTokens for GetContinuousStatesGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut state_assignments = Vec::new();
        let mut index = 0usize;

        for field in &self.0.fields {
            let mut is_state = false;
            for attr in &field.attrs {
                if let FieldAttributeOuter::Variable(var_attr) = attr {
                    // Check if this is explicitly marked as a state variable
                    if var_attr.state == Some(true) {
                        is_state = true;
                        break;
                    }
                }
            }

            if is_state {
                let field_name = &field.ident;

                if field.field_type.dimensions.is_empty() {
                    // Scalar field
                    state_assignments.push(quote! {
                        if #index < states.len() {
                            states[#index] = self.#field_name;
                        } else {
                            return Err(fmi::fmi3::Fmi3Error::Error);
                        }
                    });
                    index += 1;
                } else {
                    // Array field - copy each element
                    let total_elements = field.field_type.total_elements();
                    for i in 0..total_elements {
                        state_assignments.push(quote! {
                            if #index < states.len() {
                                states[#index] = self.#field_name[#i];
                            } else {
                                return Err(fmi::fmi3::Fmi3Error::Error);
                            }
                        });
                        index += 1;
                    }
                }
            }
        }

        if state_assignments.is_empty() {
            tokens.extend(quote! {
                // No continuous states in this model
                Ok(fmi::fmi3::Fmi3Res::OK)
            });
        } else {
            tokens.extend(quote! {
                #(#state_assignments)*
                Ok(fmi::fmi3::Fmi3Res::OK)
            });
        }
    }
}

pub struct SetContinuousStatesGen<'a>(&'a Model);

impl<'a> SetContinuousStatesGen<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self(model)
    }
}

impl ToTokens for SetContinuousStatesGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut state_assignments = Vec::new();
        let mut index = 0usize;

        for field in &self.0.fields {
            let mut is_state = false;
            for attr in &field.attrs {
                if let FieldAttributeOuter::Variable(var_attr) = attr {
                    // Check if this is explicitly marked as a state variable
                    if var_attr.state == Some(true) {
                        is_state = true;
                        break;
                    }
                }
            }

            if is_state {
                let field_name = &field.ident;

                if field.field_type.dimensions.is_empty() {
                    // Scalar field
                    state_assignments.push(quote! {
                        if #index < states.len() {
                            self.#field_name = states[#index];
                        } else {
                            return Err(fmi::fmi3::Fmi3Error::Error);
                        }
                    });
                    index += 1;
                } else {
                    // Array field - copy each element
                    let total_elements = field.field_type.total_elements();
                    for i in 0..total_elements {
                        state_assignments.push(quote! {
                            if #index < states.len() {
                                self.#field_name[#i] = states[#index];
                            } else {
                                return Err(fmi::fmi3::Fmi3Error::Error);
                            }
                        });
                        index += 1;
                    }
                }
            }
        }

        if state_assignments.is_empty() {
            tokens.extend(quote! {
                // No continuous states in this model
                Ok(fmi::fmi3::Fmi3Res::OK)
            });
        } else {
            tokens.extend(quote! {
                #(#state_assignments)*
                Ok(fmi::fmi3::Fmi3Res::OK)
            });
        }
    }
}

pub struct GetDerivativesGen<'a>(&'a Model);

impl<'a> GetDerivativesGen<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self(model)
    }
}

impl ToTokens for GetDerivativesGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut derivative_assignments = Vec::new();
        let mut state_fields = Vec::new();

        // Collect state fields
        for field in &self.0.fields {
            let mut is_state = false;
            for attr in &field.attrs {
                if let FieldAttributeOuter::Variable(var_attr) = attr {
                    if var_attr.state == Some(true) {
                        is_state = true;
                        break;
                    }
                }
            }
            if is_state {
                state_fields.push(field);
            }
        }

        if state_fields.is_empty() {
            tokens.extend(quote! {
                // No derivatives in this model
                Ok(fmi::fmi3::Fmi3Res::OK)
            });
            return;
        }

        // Generate assignments that find the derivative field for each state
        let mut derivative_index = 0usize;
        for state_field in state_fields.iter() {
            let state_name = &state_field.ident.to_string();
            let derivative_name = format!("der({})", state_name);

            // Look for a field that has an alias matching the derivative name OR
            // Look for a field with derivative attribute pointing to this state
            let mut derivative_field = None;
            for field in &self.0.fields {
                // Check for alias attribute first
                for attr in &field.attrs {
                    if let FieldAttributeOuter::Alias(alias_attr) = attr {
                        if let Some(alias_name) = &alias_attr.name {
                            if alias_name == &derivative_name {
                                derivative_field = Some(field);
                                break;
                            }
                        }
                    }
                }

                // If not found, check for derivative attribute
                if derivative_field.is_none() {
                    for attr in &field.attrs {
                        if let FieldAttributeOuter::Variable(var_attr) = attr {
                            if let Some(derivative_of) = &var_attr.derivative {
                                if &derivative_of.to_string() == state_name {
                                    derivative_field = Some(field);
                                    break;
                                }
                            }
                        }
                    }
                }

                if derivative_field.is_some() {
                    break;
                }
            }

            if let Some(der_field) = derivative_field {
                let der_field_name = &der_field.ident;

                if state_field.field_type.dimensions.is_empty() {
                    // Scalar state and derivative
                    derivative_assignments.push(quote! {
                        if #derivative_index < derivatives.len() {
                            derivatives[#derivative_index] = self.#der_field_name;
                        }
                    });
                    derivative_index += 1;
                } else {
                    // Array state and derivative - copy each element
                    let total_elements = state_field.field_type.total_elements();
                    for i in 0..total_elements {
                        derivative_assignments.push(quote! {
                            if #derivative_index < derivatives.len() {
                                derivatives[#derivative_index] = self.#der_field_name[#i];
                            }
                        });
                        derivative_index += 1;
                    }
                }
            } else {
                // Fallback to old behavior if no derivative field found
                let derivative_field_name = format_ident!("der_{}", state_name);
                if state_field.field_type.dimensions.is_empty() {
                    derivative_assignments.push(quote! {
                        if #derivative_index < derivatives.len() {
                            derivatives[#derivative_index] = self.#derivative_field_name;
                        }
                    });
                    derivative_index += 1;
                } else {
                    let total_elements = state_field.field_type.total_elements();
                    for i in 0..total_elements {
                        derivative_assignments.push(quote! {
                            if #derivative_index < derivatives.len() {
                                derivatives[#derivative_index] = self.#derivative_field_name[#i];
                            }
                        });
                        derivative_index += 1;
                    }
                }
            }
        }

        tokens.extend(quote! {
            #(#derivative_assignments)*
            Ok(fmi::fmi3::Fmi3Res::OK)
        });
    }
}
