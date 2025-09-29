//! Code generation for the derive macro

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

use crate::model::{FieldAttributeOuter, Model};

pub struct SetStartValuesGen<'a>(&'a Model);

impl<'a> SetStartValuesGen<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self(model)
    }
}

impl ToTokens for SetStartValuesGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut assignments = Vec::new();

        for field in &self.0.fields {
            for attr in &field.attrs {
                if let FieldAttributeOuter::Variable(var_attr) = attr {
                    if let Some(start_expr) = &var_attr.start {
                        let field_name = &field.ident;

                        assignments.push(quote! {
                            self.#field_name = #start_expr;
                        });
                    }
                }
            }
        }

        tokens.extend(quote! {
            #(#assignments)*
        });
    }
}
