use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;
use uuid;

use crate::codegen::util;
use crate::model::{FieldAttributeOuter, Model};
use fmi::fmi3::schema;

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
        let variable_validation_body = VariableValidationGen::new(&self.model);

        let number_of_event_indicators = count_event_indicators(&self.model);

        tokens.extend(quote! {
            #[automatically_derived]
            impl ::fmi_export::fmi3::Model for #struct_name {
                const MODEL_NAME: &'static str = #model_name;
                const INSTANTIATION_TOKEN: &'static str = #instantiation_token;

                fn build_metadata(
                    variables: &mut ::fmi::schema::fmi3::ModelVariables,
                    model_structure: &mut ::fmi::schema::fmi3::ModelStructure,
                    vr_offset: u32,
                ) -> u32 {
                    #build_metadata_body
                }

                fn set_start_values(&mut self) {
                    #set_start_values_body
                }

                fn get_number_of_event_indicators() -> usize {
                    #number_of_event_indicators
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

// Helper generators for specific function bodies

struct VariableValidationGen<'a>(&'a Model);

impl<'a> VariableValidationGen<'a> {
    fn new(model: &'a Model) -> Self {
        Self(model)
    }
}

#[cfg(false)]
impl ToTokens for VariableValidationGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut cases = Vec::new();

        for field in &self.0.fields {
            for attr in &field.attrs {
                if let FieldAttributeOuter::Variable(var_attr) = attr {
                    let variant_name =
                        format_ident!("{}", util::to_pascal_case(&field.ident.to_string()));
                    let var_name = &field.ident.to_string();

                    // Generate validation based on causality and variability
                    let causality = var_attr.causality.as_ref().map(|c| &c.0);
                    let variability = var_attr.variability.as_ref().map(|v| &v.0);

                    let validation = match (causality, variability) {
                        (Some(schema::Causality::Parameter), Some(schema::Variability::Fixed))
                        | (Some(schema::Causality::Parameter), None) => {
                            quote! {
                                #value_ref_enum_name::#variant_name => {
                                    match state {
                                        fmi_export::fmi3::ModelState::Instantiated
                                        | fmi_export::fmi3::ModelState::InitializationMode => Ok(()),
                                        _ => Err(concat!("Variable ", #var_name, " (fixed parameter) can only be set after instantiation or in initialization mode.")),
                                    }
                                }
                            }
                        }
                        (
                            Some(schema::Causality::Parameter),
                            Some(schema::Variability::Tunable),
                        ) => {
                            quote! {
                                #value_ref_enum_name::#variant_name => {
                                    match state {
                                        fmi_export::fmi3::ModelState::Instantiated
                                        | fmi_export::fmi3::ModelState::InitializationMode
                                        | fmi_export::fmi3::ModelState::EventMode => Ok(()),
                                        _ => Err(concat!("Variable ", #var_name, " (tunable parameter) can only be set after instantiation, in initialization mode or event mode.")),
                                    }
                                }
                            }
                        }
                        (Some(schema::Causality::Local), Some(schema::Variability::Fixed))
                        | (Some(schema::Causality::Local), None) => {
                            quote! {
                                #value_ref_enum_name::#variant_name => {
                                    match state {
                                        fmi_export::fmi3::ModelState::Instantiated
                                        | fmi_export::fmi3::ModelState::InitializationMode => Ok(()),
                                        _ => Err(concat!("Variable ", #var_name, " (fixed local) can only be set after instantiation or in initialization mode.")),
                                    }
                                }
                            }
                        }
                        (Some(schema::Causality::Input), _) => {
                            quote! {
                                #value_ref_enum_name::#variant_name => {
                                    match state {
                                        fmi_export::fmi3::ModelState::Terminated => Err(concat!("Variable ", #var_name, " (input) cannot be set in terminated state.")),
                                        _ => Ok(()),
                                    }
                                }
                            }
                        }
                        _ => {
                            quote! {
                                #value_ref_enum_name::#variant_name => {
                                    match state {
                                        fmi_export::fmi3::ModelState::Terminated => Err(concat!("Variable ", #var_name, " cannot be set in terminated state.")),
                                        _ => Ok(()),
                                    }
                                }
                            }
                        }
                    };

                    cases.push(validation);
                }
            }
        }

        tokens.extend(quote! {
            match vr {
                #(#cases)*
                _ => Ok(()), // Unknown variables are allowed by default
            }
        });
    }
}
