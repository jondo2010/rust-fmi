//! Code generation for the derive macro

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

use crate::model::Model;

mod model_get_set;
mod model_get_set_states;
mod model_impl;
mod user_model_impl;

/// Main code generation structure
pub struct CodeGenerator {
    pub model: Model,
}

impl CodeGenerator {
    pub fn new(model: Model) -> Self {
        Self { model }
    }
}

impl ToTokens for CodeGenerator {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = &self.model.ident;

        // Generate Model implementation
        let model_impl = model_impl::ModelImpl::new(struct_name, &self.model);

        let model_get_set_impl = model_get_set::ModelGetSetImpl {
            struct_name,
            model: &self.model,
        };

        let model_get_set_states_impl =
            model_get_set_states::ModelGetSetStatesImpl::new(struct_name, &self.model);
        let user_model_impl = user_model_impl::UserModelImpl::new(struct_name, &self.model);

        // Combine all implementations
        tokens.extend(quote! {
            #model_impl
            #user_model_impl
            #model_get_set_impl
            #model_get_set_states_impl
        });
    }
}
