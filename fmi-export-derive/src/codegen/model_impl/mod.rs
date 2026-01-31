use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::Ident;
use uuid;

use crate::model::{FieldAttributeOuter, Model};

mod metadata;
mod start_values;

/// Generate the Model trait implementation
pub struct ModelImpl<'a> {
    struct_name: &'a Ident,
    model: &'a Model,
}

impl<'a> ModelImpl<'a> {
    pub fn new(struct_name: &'a Ident, model: &'a Model) -> Self {
        Self { struct_name, model }
    }
}

impl ToTokens for ModelImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = self.struct_name;

        // Extract model name from the struct name
        let model_name = &self.model.ident.to_string();

        // Generate a UUID-based instantiation token from the model name
        let instantiation_token =
            uuid::Uuid::new_v5(&crate::RUST_FMI_NAMESPACE, model_name.as_bytes()).to_string();

        // Generate function bodies
        let build_metadata_body = metadata::BuildMetadataGen::new(&self.model);
        let set_start_values_body = start_values::SetStartValuesGen::new(&self.model);

        let number_of_event_indicators = count_event_indicators(&self.model);

        let supports_me = self.model.supports_model_exchange();
        let supports_cs = self.model.supports_co_simulation();
        let supports_se = self.model.supports_scheduled_execution();

        tokens.extend(quote! {
            #[automatically_derived]
            impl ::fmi_export::fmi3::Model for #struct_name {
                const MODEL_NAME: &'static str = #model_name;
                const INSTANTIATION_TOKEN: &'static str = #instantiation_token;

                const MAX_EVENT_INDICATORS: usize = #number_of_event_indicators;

                const SUPPORTS_MODEL_EXCHANGE: bool = #supports_me;
                const SUPPORTS_CO_SIMULATION: bool = #supports_cs;
                const SUPPORTS_SCHEDULED_EXECUTION: bool = #supports_se;

                fn build_metadata(
                    variables: &mut ::fmi::schema::fmi3::ModelVariables,
                    model_structure: &mut ::fmi::schema::fmi3::ModelStructure,
                    vr_offset: u32,
                    prefix: Option<&str>,
                ) -> u32 {
                    #build_metadata_body
                }

                fn set_start_values(&mut self) {
                    #set_start_values_body
                }

                fn validate_variable_setting(
                    vr: ::fmi::fmi3::binding::fmi3ValueReference,
                    state: &::fmi_export::fmi3::ModelState,
                ) -> Result<(), &'static str> {
                    //#variable_validation_body
                    Ok(())
                }
            }
        });
    }
}

/// Count the number of event indicators in the model
fn count_event_indicators(model: &Model) -> usize {
    model.fields
        .iter()
        .filter(|field| {
            field.attrs.iter().any(|attr| {
                matches!(attr, FieldAttributeOuter::Variable(var_attr) if var_attr.event_indicator == Some(true))
            })
        })
        .count()
}
