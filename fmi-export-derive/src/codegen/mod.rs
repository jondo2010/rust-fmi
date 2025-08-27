//! Code generation for the derive macro

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

use crate::model::Model;
use fmi::fmi3::schema;

mod get_set;
mod model_impl;
mod util;
mod value_ref;

/// Main code generation structure
pub struct CodeGenerator {
    pub model: Model,
    pub model_description: schema::Fmi3ModelDescription,
}

impl CodeGenerator {
    pub fn new(model: Model) -> Self {
        // Generate the model description using the new front-end
        let model_description = schema::Fmi3ModelDescription::try_from(model.clone())
            .expect("Failed to generate model description");

        Self {
            model,
            model_description,
        }
    }
}

impl ToTokens for CodeGenerator {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = &self.model.ident;

        // Generate value reference enum
        let value_ref_enum = value_ref::ValueRefEnum::new(&self.model, &self.model_description);

        // Generate Model implementation
        let model_impl =
            model_impl::ModelImpl::new(struct_name, &self.model, &self.model_description);

        // Generate GetSet implementation
        let getset_impl = get_set::GetSetImpl::new(struct_name, &self.model);

        // Combine all implementations
        tokens.extend(quote! {
            #value_ref_enum
            #model_impl
            #getset_impl
        });
    }
}
