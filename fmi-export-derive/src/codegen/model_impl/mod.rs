use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;
use uuid;

use crate::codegen::util;
use crate::model::{FieldAttributeOuter, Model};
use fmi::fmi3::schema;

mod get_set_states;
mod getter_setter;
mod start_values;

pub use getter_setter::GetterSetterGen;

/// Generate the Model trait implementation
pub struct ModelImpl<'a> {
    struct_name: &'a Ident,
    model: &'a Model,
    model_variables: &'a schema::ModelVariables,
    model_structure: &'a schema::ModelStructure,
}

impl<'a> ModelImpl<'a> {
    pub fn new(
        struct_name: &'a Ident,
        model: &'a Model,
        model_variables: &'a schema::ModelVariables,
        model_structure: &'a schema::ModelStructure,
    ) -> Self {
        Self {
            struct_name,
            model,
            model_variables,
            model_structure,
        }
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

        // Serialize the individual components as XML fragments
        let model_variables_xml = fmi::schema::serialize(self.model_variables, true)
            .expect("Failed to serialize model variables");
        let model_structure_xml = fmi::schema::serialize(self.model_structure, true)
            .expect("Failed to serialize model structure");

        // Generate function bodies
        let set_start_values_body = start_values::SetStartValuesGen::new(&self.model);
        let get_continuous_states_body = get_set_states::GetContinuousStatesGen::new(&self.model);
        let set_continuous_states_body = get_set_states::SetContinuousStatesGen::new(&self.model);
        let get_derivatives_body = GetDerivativesGen::new(&self.model);
        let variable_validation_body = VariableValidationGen::new(&self.model);
        let getter_setter_methods = GetterSetterGen::new(&self.model);

        let number_of_continuous_states = count_continuous_states(&self.model);
        let number_of_event_indicators = count_event_indicators(&self.model);

        tokens.extend(quote! {
            impl ::fmi_export::fmi3::Model for #struct_name {
                type ValueRef = ::fmi::fmi3::binding::fmi3ValueReference;

                const MODEL_NAME: &'static str = #model_name;
                const MODEL_VARIABLES_XML: &'static str = #model_variables_xml;
                const MODEL_STRUCTURE_XML: &'static str = #model_structure_xml;
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
    model
        .fields
        .iter()
        .filter_map(|field| {
            let is_state = field.attrs.iter().any(|attr| {
                matches!(attr, FieldAttributeOuter::Variable(var_attr) if var_attr.state == Some(true))
            });
            if is_state {
                // For arrays, count the number of elements; for scalars, count as 1
                Some(field.field_type.total_elements())
            } else {
                None
            }
        })
        .sum()
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
        let mut derivative_index = 0usize;
        for state_field in state_fields.iter() {
            let state_name = &state_field.ident.to_string();
            let derivative_name = format!("der({})", state_name);

            // Look for a field that has an alias matching the derivative name OR
            // Look for a field with derivative attribute pointing to this state
            let mut derivative_field = None;
            for field in &self.0.fields {
                // Check for alias attribute first
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

                // If not found, check for derivative attribute
                if derivative_field.is_none() {
                    for attr in &field.attrs {
                        if let FieldAttributeOuter::Variable(var_attr) = attr {
                            if let Some(derivative_of) = &var_attr.derivative {
                                if &derivative_of.to_string() == state_name {
                                    derivative_field = Some(field);
                                    break;
                                }
                            }
                        }
                    }
                }

                if derivative_field.is_some() {
                    break;
                }
            }

            if let Some(der_field) = derivative_field {
                let der_field_name = &der_field.ident;

                if state_field.field_type.dimensions.is_empty() {
                    // Scalar state and derivative
                    derivative_assignments.push(quote! {
                        if #derivative_index < derivatives.len() {
                            derivatives[#derivative_index] = self.#der_field_name;
                        }
                    });
                    derivative_index += 1;
                } else {
                    // Array state and derivative - copy each element
                    let total_elements = state_field.field_type.total_elements();
                    for i in 0..total_elements {
                        derivative_assignments.push(quote! {
                            if #derivative_index < derivatives.len() {
                                derivatives[#derivative_index] = self.#der_field_name[#i];
                            }
                        });
                        derivative_index += 1;
                    }
                }
            } else {
                // Fallback to old behavior if no derivative field found
                let derivative_field_name = format_ident!("der_{}", state_name);
                if state_field.field_type.dimensions.is_empty() {
                    derivative_assignments.push(quote! {
                        if #derivative_index < derivatives.len() {
                            derivatives[#derivative_index] = self.#derivative_field_name;
                        }
                    });
                    derivative_index += 1;
                } else {
                    let total_elements = state_field.field_type.total_elements();
                    for i in 0..total_elements {
                        derivative_assignments.push(quote! {
                            if #derivative_index < derivatives.len() {
                                derivatives[#derivative_index] = self.#derivative_field_name[#i];
                            }
                        });
                        derivative_index += 1;
                    }
                }
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
        let struct_name = &self.0.ident;
        let value_ref_enum_name = format_ident!("{}ValueRef", struct_name);

        let mut cases = Vec::new();

        for field in &self.0.fields {
            let vtype = field.field_type.r#type;
            for attr in &field.attrs {
                if let FieldAttributeOuter::Variable(var_attr) = attr {
                    if matches!(
                        vtype,
                        schema::VariableType::FmiFloat32 | schema::VariableType::FmiFloat64
                    ) {
                        let variant_name =
                            format_ident!("{}", util::to_pascal_case(&field.ident.to_string()));
                        let var_name = &field.ident.to_string();

                        // Generate validation based on causality and variability
                        let causality = var_attr.causality.as_ref().map(|c| &c.0);
                        let variability = var_attr.variability.as_ref().map(|v| &v.0);

                        let validation = match (causality, variability) {
                            (
                                Some(schema::Causality::Parameter),
                                Some(schema::Variability::Fixed),
                            )
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
        }

        tokens.extend(quote! {
            match #value_ref_enum_name::from(vr) {
                #(#cases)*
                _ => Ok(()), // Unknown variables are allowed by default
            }
        });
    }
}
