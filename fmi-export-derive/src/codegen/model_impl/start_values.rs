//! Code generation for the derive macro

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

use crate::model::{FieldAttributeOuter, Model};
use crate::model_description::rust_type_to_variable_type;
use fmi::fmi3::schema;

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
                        let variable_type = rust_type_to_variable_type(&field.ty).ok();

                        if let Some(vtype) = variable_type {
                            match vtype {
                                schema::VariableType::FmiFloat64 => {
                                    assignments.push(quote! {
                                        self.#field_name = #start_expr;
                                    });
                                }
                                schema::VariableType::FmiFloat32 => {
                                    assignments.push(quote! {
                                        self.#field_name = #start_expr;
                                    });
                                }
                                schema::VariableType::FmiInt8
                                | schema::VariableType::FmiInt16
                                | schema::VariableType::FmiInt32
                                | schema::VariableType::FmiInt64 => {
                                    assignments.push(quote! {
                                        self.#field_name = #start_expr;
                                    });
                                }
                                schema::VariableType::FmiUInt8
                                | schema::VariableType::FmiUInt16
                                | schema::VariableType::FmiUInt32
                                | schema::VariableType::FmiUInt64 => {
                                    assignments.push(quote! {
                                        self.#field_name = #start_expr;
                                    });
                                }
                                schema::VariableType::FmiBoolean => {
                                    assignments.push(quote! {
                                        self.#field_name = #start_expr;
                                    });
                                }
                                schema::VariableType::FmiString => {
                                    assignments.push(quote! {
                                        self.#field_name = #start_expr.to_string();
                                    });
                                }
                                schema::VariableType::FmiBinary => {
                                    // Skip binary for now
                                }
                            }
                        }
                    }
                }
            }
        }

        tokens.extend(quote! {
            #(#assignments)*
        });
    }
}
