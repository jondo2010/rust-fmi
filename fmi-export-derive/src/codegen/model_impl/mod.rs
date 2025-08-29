use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

use crate::codegen::util;
use crate::model::{FieldAttributeOuter, Model};
use crate::model_description::rust_type_to_variable_type;
use fmi::fmi3::schema;

mod getter_setter;
mod start_values;

pub use getter_setter::GetterSetterGen;

/// Generate the Model trait implementation
pub struct ModelImpl<'a> {
    struct_name: &'a Ident,
    model: &'a Model,
    model_description: &'a schema::Fmi3ModelDescription,
}

impl<'a> ModelImpl<'a> {
    pub fn new(
        struct_name: &'a Ident,
        model: &'a Model,
        model_description: &'a schema::Fmi3ModelDescription,
    ) -> Self {
        Self {
            struct_name,
            model,
            model_description,
        }
    }
}

impl ToTokens for ModelImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = self.struct_name;
        let model_name = &self.model_description.model_name;
        let model_description_xml = yaserde::ser::to_string(self.model_description)
            .expect("Failed to serialize model description");
        let instantiation_token = &self.model_description.instantiation_token;

        // Generate function bodies
        let set_start_values_body = start_values::SetStartValuesGen::new(&self.model);
        let get_continuous_states_body = GetContinuousStatesGen::new(&self.model);
        let set_continuous_states_body = SetContinuousStatesGen::new(&self.model);
        let get_derivatives_body = GetDerivativesGen::new(&self.model);
        let variable_validation_body = VariableValidationGen::new(&self.model);
        let getter_setter_methods = GetterSetterGen::new(&self.model);

        let number_of_continuous_states = count_continuous_states(&self.model);
        let number_of_event_indicators = count_event_indicators(&self.model);

        tokens.extend(quote! {
            impl ::fmi_export::fmi3::Model for #struct_name {
                type ValueRef = ::fmi::fmi3::binding::fmi3ValueReference;

                const MODEL_NAME: &'static str = #model_name;
                const MODEL_DESCRIPTION: &'static str = #model_description_xml;
                const INSTANTIATION_TOKEN: &'static str = #instantiation_token;

                fn set_start_values(&mut self) {
                    #set_start_values_body
                }

                fn get_continuous_states(&self, states: &mut [f64]) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
                    #get_continuous_states_body
                }

                fn set_continuous_states(&mut self, states: &[f64]) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
                    #set_continuous_states_body
                }

                fn get_continuous_state_derivatives(
                    &mut self,
                    derivatives: &mut [f64],
                    context: &::fmi_export::fmi3::ModelContext<Self>
                ) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
                    #get_derivatives_body
                }

                fn get_number_of_continuous_states() -> usize {
                    #number_of_continuous_states
                }

                fn get_number_of_event_indicators() -> usize {
                    #number_of_event_indicators
                }

                fn validate_variable_setting(
                    vr: fmi::fmi3::binding::fmi3ValueReference,
                    state: &fmi_export::fmi3::ModelState,
                ) -> Result<(), &'static str> {
                    #variable_validation_body
                }

                #getter_setter_methods
            }
        });
    }
}

/// Count the number of continuous states in the model
fn count_continuous_states(model: &Model) -> usize {
    let mut count = 0;
    for field in &model.fields {
        for attr in &field.attrs {
            if let FieldAttributeOuter::Variable(var_attr) = attr {
                if var_attr.state == Some(true) {
                    count += 1;
                    break;
                }
            }
        }
    }
    count
}

/// Count the number of event indicators in the model
fn count_event_indicators(model: &Model) -> usize {
    // For now, simple heuristic: if there's a field named 'h', it's an event indicator
    for field in &model.fields {
        if field.ident.to_string() == "h" {
            return 1;
        }
    }
    0
}

// Helper generators for specific function bodies

struct GetContinuousStatesGen<'a>(&'a Model);

impl<'a> GetContinuousStatesGen<'a> {
    fn new(model: &'a Model) -> Self {
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
                state_assignments.push(quote! {
                    if #index < states.len() {
                        states[#index] = self.#field_name;
                    }
                });
                index += 1;
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

struct SetContinuousStatesGen<'a>(&'a Model);

impl<'a> SetContinuousStatesGen<'a> {
    fn new(model: &'a Model) -> Self {
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
                state_assignments.push(quote! {
                    if #index < states.len() {
                        self.#field_name = states[#index];
                    }
                });
                index += 1;
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

struct GetDerivativesGen<'a>(&'a Model);

impl<'a> GetDerivativesGen<'a> {
    fn new(model: &'a Model) -> Self {
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
        for (i, state_field) in state_fields.iter().enumerate() {
            let state_name = &state_field.ident.to_string();
            let derivative_name = format!("der({})", state_name);

            // Look for a field that has an alias matching the derivative name
            let mut derivative_field = None;
            for field in &self.0.fields {
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
                if derivative_field.is_some() {
                    break;
                }
            }

            if let Some(der_field) = derivative_field {
                let field_name = &der_field.ident;
                derivative_assignments.push(quote! {
                    if #i < derivatives.len() {
                        let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self, context);
                        derivatives[#i] = self.#field_name;
                    }
                });
            } else {
                // Fallback to old behavior if no alias found
                let derivative_field_name = format_ident!("der_{}", state_name);
                derivative_assignments.push(quote! {
                    if #i < derivatives.len() {
                        let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self, context);
                        derivatives[#i] = self.#derivative_field_name;
                    }
                });
            }
        }

        tokens.extend(quote! {
            #(#derivative_assignments)*
            Ok(fmi::fmi3::Fmi3Res::OK)
        });
    }
}

struct VariableValidationGen<'a>(&'a Model);

impl<'a> VariableValidationGen<'a> {
    fn new(model: &'a Model) -> Self {
        Self(model)
    }
}

impl ToTokens for VariableValidationGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut cases = Vec::new();

        for field in &self.0.fields {
            for attr in &field.attrs {
                if let FieldAttributeOuter::Variable(var_attr) = attr {
                    if let Ok(vtype) = rust_type_to_variable_type(&field.ty) {
                        if matches!(
                            vtype,
                            schema::VariableType::FmiFloat32 | schema::VariableType::FmiFloat64
                        ) {
                            let variant_name =
                                format_ident!("{}", util::to_pascal_case(&field.ident.to_string()));
                            let var_name = &field.ident.to_string();

                            // Generate validation based on causality and variability
                            let causality_str = var_attr
                                .causality
                                .as_ref()
                                .map(|c| c.to_string())
                                .unwrap_or_default();
                            let variability_str = var_attr
                                .variability
                                .as_ref()
                                .map(|v| v.to_string())
                                .unwrap_or_default();

                            let validation = match (
                                causality_str.as_str(),
                                variability_str.as_str(),
                            ) {
                                ("Parameter", "Fixed") | ("Parameter", "") => {
                                    quote! {
                                        ValueRef::#variant_name => {
                                            match state {
                                                fmi_export::fmi3::ModelState::Instantiated
                                                | fmi_export::fmi3::ModelState::InitializationMode => Ok(()),
                                                _ => Err(concat!("Variable ", #var_name, " (fixed parameter) can only be set after instantiation or in initialization mode.")),
                                            }
                                        }
                                    }
                                }
                                ("Parameter", "Tunable") => {
                                    quote! {
                                        ValueRef::#variant_name => {
                                            match state {
                                                fmi_export::fmi3::ModelState::Instantiated
                                                | fmi_export::fmi3::ModelState::InitializationMode
                                                | fmi_export::fmi3::ModelState::EventMode => Ok(()),
                                                _ => Err(concat!("Variable ", #var_name, " (tunable parameter) can only be set after instantiation, in initialization mode or event mode.")),
                                            }
                                        }
                                    }
                                }
                                ("Local", "Fixed") | ("Local", "") => {
                                    quote! {
                                        ValueRef::#variant_name => {
                                            match state {
                                                fmi_export::fmi3::ModelState::Instantiated
                                                | fmi_export::fmi3::ModelState::InitializationMode => Ok(()),
                                                _ => Err(concat!("Variable ", #var_name, " (fixed local) can only be set after instantiation or in initialization mode.")),
                                            }
                                        }
                                    }
                                }
                                ("Input", _) => {
                                    quote! {
                                        ValueRef::#variant_name => {
                                            match state {
                                                fmi_export::fmi3::ModelState::Terminated => Err(concat!("Variable ", #var_name, " (input) cannot be set in terminated state.")),
                                                _ => Ok(()),
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    quote! {
                                        ValueRef::#variant_name => {
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
            }
        }

        tokens.extend(quote! {
            match ValueRef::from(vr) {
                #(#cases)*
                _ => Ok(()), // Unknown variables are allowed by default
            }
        });
    }
}
