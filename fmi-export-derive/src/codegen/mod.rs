//! Code generation for the derive macro

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

use crate::model::Model;
use crate::model_structure;
use crate::model_variables;
use fmi::fmi3::schema;

mod model_impl;
mod util;
mod value_ref;

/// Main code generation structure
pub struct CodeGenerator {
    pub model: Model,
    pub model_variables: schema::ModelVariables,
    pub model_structure: schema::ModelStructure,
}

impl CodeGenerator {
    pub fn new(model: Model) -> Self {
        // Build model variables and structure directly
        let model_variables = model_variables::build_model_variables(&model.fields);

        let model_structure =
            model_structure::build_model_structure(&model.fields, &model_variables)
                .expect("Failed to build model structure");

        Self {
            model,
            model_variables,
            model_structure,
        }
    }
}

impl ToTokens for CodeGenerator {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = &self.model.ident;

        // Generate value reference enum
        let value_ref_enum = value_ref::ValueRefEnum::new(&self.model, &self.model_variables);

        // Generate logging category enum
        //let logging_category_enum = logging_category::LoggingCategoryEnum::new(&self.model);

        // Generate Model implementation
        let model_impl = model_impl::ModelImpl::new(
            struct_name,
            &self.model,
            &self.model_variables,
            &self.model_structure,
        );

        // Combine all implementations
        tokens.extend(quote! {
            #value_ref_enum
            //#logging_category_enum
            #model_impl
        });
    }
}
