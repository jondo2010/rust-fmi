use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

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
