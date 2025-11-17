//! Code generation for the derive macro

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

use crate::model::{FieldAttributeOuter, Model, Field};

pub struct SetStartValuesGen<'a>(&'a Model);

impl<'a> SetStartValuesGen<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self(model)
    }
}

/// Check if a field has the skip attribute set to true
fn has_skip_attribute(field: &Field) -> bool {
    field
        .attrs
        .iter()
        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(var_attr) if var_attr.skip))
}

impl ToTokens for SetStartValuesGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut assignments = Vec::new();

        for field in &self.0.fields {
            // Skip fields with skip=true
            if has_skip_attribute(field) {
                continue;
            }

            for attr in &field.attrs {
                if let FieldAttributeOuter::Variable(var_attr) = attr {
                    if let Some(start_expr) = &var_attr.start {
                        let field_name = &field.ident;

                        // Use the trait-based approach - no type introspection needed
                        let assignment_expr = quote! {
                            ::fmi_export::fmi3::InitializeFromStart::set_from_start(&mut self.#field_name, #start_expr);
                        };

                        assignments.push(assignment_expr);
                    }
                }
            }
        }

        tokens.extend(quote! {
            #(#assignments)*
        });
    }
}
