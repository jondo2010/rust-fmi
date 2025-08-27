//! Code generation for the derive macro

use convert_case::{Case, Casing};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

use crate::model_description::rust_type_to_variable_type;
use crate::model_new::{FieldAttributeOuter, Model};
use fmi::fmi3::schema::{Fmi3ModelDescription, VariableType};

/// Main code generation structure
pub struct CodeGenerator {
    pub model: Model,
    pub model_description: Fmi3ModelDescription,
}

impl CodeGenerator {
    pub fn new(model: Model) -> Self {
        // Generate the model description using the new front-end
        let model_description = Fmi3ModelDescription::try_from(model.clone())
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
        let value_ref_enum = ValueRefEnum::new(&self.model);

        // Generate Model implementation
        let model_impl = ModelImpl::new(struct_name, &self.model, &self.model_description);

        // Generate GetSet implementation
        let getset_impl = GetSetImpl::new(struct_name, &self.model);

        // Combine all implementations
        tokens.extend(quote! {
            #value_ref_enum
            #model_impl
            #getset_impl
        });
    }
}

/// Generate the ValueRef enum
struct ValueRefEnum<'a> {
    model: &'a Model,
}

impl<'a> ValueRefEnum<'a> {
    fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl ToTokens for ValueRefEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut value_ref_variants = Vec::new();
        let mut from_u32_arms = Vec::new();
        let mut into_u32_arms = Vec::new();

        let mut vr_counter = 0u32; // FMI value references start at 0

        for field in &self.model.fields {
            // First, add the main field variable
            let has_variable = field
                .attrs
                .iter()
                .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
            if has_variable {
                let variant_name = format_ident!("{}", to_pascal_case(&field.ident.to_string()));

                value_ref_variants.push(quote! {
                    #variant_name = #vr_counter
                });

                from_u32_arms.push(quote! {
                    #vr_counter => ValueRef::#variant_name
                });

                into_u32_arms.push(quote! {
                    ValueRef::#variant_name => #vr_counter
                });

                vr_counter += 1;
            }

            // Then add any alias variables with their custom names
            for attr in &field.attrs {
                if let FieldAttributeOuter::Alias(alias_attr) = attr {
                    let field_name_str = field.ident.to_string();
                    let alias_name = alias_attr.name.as_deref().unwrap_or(&field_name_str);

                    let variant_name = generate_variant_name(alias_name);

                    value_ref_variants.push(quote! {
                        #variant_name = #vr_counter
                    });

                    from_u32_arms.push(quote! {
                        #vr_counter => ValueRef::#variant_name
                    });

                    into_u32_arms.push(quote! {
                        ValueRef::#variant_name => #vr_counter
                    });

                    vr_counter += 1;
                }
            }
        }

        tokens.extend(quote! {
            #[repr(u32)]
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            enum ValueRef {
                #(#value_ref_variants,)*
            }

            impl From<fmi::fmi3::binding::fmi3ValueReference> for ValueRef {
                fn from(value: fmi::fmi3::binding::fmi3ValueReference) -> Self {
                    match value {
                        #(#from_u32_arms,)*
                        _ => panic!("Invalid value reference: {}", value),
                    }
                }
            }

            impl From<ValueRef> for fmi::fmi3::binding::fmi3ValueReference {
                fn from(value: ValueRef) -> Self {
                    match value {
                        #(#into_u32_arms,)*
                    }
                }
            }
        });
    }
}

/// Helper function to convert a field name to PascalCase for enum variants
fn to_pascal_case(name: &str) -> String {
    // Handle special characters for alias names like "der(h)" -> "DerH"
    let cleaned = name
        .replace("(", "_")
        .replace(")", "")
        .replace("-", "_")
        .replace(" ", "_")
        .replace(".", "_");

    cleaned.to_case(Case::Pascal)
}

/// Generate the Model trait implementation
struct ModelImpl<'a> {
    struct_name: &'a Ident,
    model: &'a Model,
    model_description: &'a Fmi3ModelDescription,
}

impl<'a> ModelImpl<'a> {
    fn new(
        struct_name: &'a Ident,
        model: &'a Model,
        model_description: &'a Fmi3ModelDescription,
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
        let set_start_values_body = SetStartValuesGen::new(&self.model);
        let get_continuous_states_body = GetContinuousStatesGen::new(&self.model);
        let set_continuous_states_body = SetContinuousStatesGen::new(&self.model);
        let get_derivatives_body = GetDerivativesGen::new(&self.model);
        let variable_validation_body = VariableValidationGen::new(&self.model);

        let number_of_continuous_states = count_continuous_states(&self.model);
        let number_of_event_indicators = count_event_indicators(&self.model);

        tokens.extend(quote! {
            impl ::fmi_export::fmi3::Model for #struct_name {
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

                fn get_continuous_state_derivatives(&mut self, derivatives: &mut [f64]) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
                    #get_derivatives_body
                }

                fn get_number_of_continuous_states() -> usize {
                    #number_of_continuous_states
                }

                fn get_number_of_event_indicators() -> usize {
                    #number_of_event_indicators
                }

                fn model_variables() -> fmi::fmi3::schema::ModelVariables {
                    unimplemented!("model_variables will be replaced in next step")
                }

                fn model_structure() -> fmi::fmi3::schema::ModelStructure {
                    unimplemented!("model_structure will be replaced in next step")
                }

                fn model_description() -> fmi::fmi3::schema::Fmi3ModelDescription {
                    unimplemented!("model_description will be replaced in next step")
                }

                fn validate_variable_setting(
                    vr: fmi::fmi3::binding::fmi3ValueReference,
                    state: &fmi_export::fmi3::ModelState,
                ) -> Result<(), &'static str> {
                    #variable_validation_body
                }
            }
        });
    }
}

/// Generate the GetSet trait implementation
struct GetSetImpl<'a> {
    struct_name: &'a Ident,
    model: &'a Model,
}

impl<'a> GetSetImpl<'a> {
    fn new(struct_name: &'a Ident, model: &'a Model) -> Self {
        Self { struct_name, model }
    }
}

impl ToTokens for GetSetImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = self.struct_name;

        let float64_getter_cases = Float64GetterGen::new(self.model);
        let float64_setter_cases = Float64SetterGen::new(self.model);
        let float32_getter_cases = Float32GetterGen::new(self.model);
        let float32_setter_cases = Float32SetterGen::new(self.model);

        tokens.extend(quote! {
            impl ::fmi::fmi3::GetSet for #struct_name {
                type ValueRef = ::fmi::fmi3::binding::fmi3ValueReference;

                fn get_float64(
                    &mut self,
                    vrs: &[Self::ValueRef],
                    values: &mut [f64],
                ) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
                    for (vr, value) in vrs.iter().zip(values.iter_mut()) {
                        match ValueRef::from(*vr) {
                            #float64_getter_cases
                            _ => {} // Ignore unknown VRs for robustness
                        }
                    }
                    Ok(fmi::fmi3::Fmi3Res::OK)
                }

                fn set_float64(
                    &mut self,
                    vrs: &[Self::ValueRef],
                    values: &[f64],
                ) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
                    for (vr, value) in vrs.iter().zip(values.iter()) {
                        match ValueRef::from(*vr) {
                            #float64_setter_cases
                            _ => {} // Ignore unknown VRs for robustness
                        }
                    }
                    Ok(fmi::fmi3::Fmi3Res::OK)
                }

                fn get_float32(
                    &mut self,
                    vrs: &[Self::ValueRef],
                    values: &mut [f32],
                ) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
                    for (vr, value) in vrs.iter().zip(values.iter_mut()) {
                        match ValueRef::from(*vr) {
                            #float32_getter_cases
                            _ => {} // Ignore unknown VRs for robustness
                        }
                    }
                    Ok(fmi::fmi3::Fmi3Res::OK)
                }

                fn set_float32(
                    &mut self,
                    vrs: &[Self::ValueRef],
                    values: &[f32],
                ) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
                    for (vr, value) in vrs.iter().zip(values.iter()) {
                        match ValueRef::from(*vr) {
                            #float32_setter_cases
                            _ => {} // Ignore unknown VRs for robustness
                        }
                    }
                    Ok(fmi::fmi3::Fmi3Res::OK)
                }
            }
        });
    }
}

// Helper generators for specific function bodies

struct SetStartValuesGen<'a>(&'a Model);

impl<'a> SetStartValuesGen<'a> {
    fn new(model: &'a Model) -> Self {
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
                                VariableType::FmiFloat64 => {
                                    assignments.push(quote! {
                                        self.#field_name = #start_expr;
                                    });
                                }
                                VariableType::FmiFloat32 => {
                                    assignments.push(quote! {
                                        self.#field_name = #start_expr;
                                    });
                                }
                                VariableType::FmiInt8
                                | VariableType::FmiInt16
                                | VariableType::FmiInt32
                                | VariableType::FmiInt64 => {
                                    assignments.push(quote! {
                                        self.#field_name = #start_expr;
                                    });
                                }
                                VariableType::FmiUInt8
                                | VariableType::FmiUInt16
                                | VariableType::FmiUInt32
                                | VariableType::FmiUInt64 => {
                                    assignments.push(quote! {
                                        self.#field_name = #start_expr;
                                    });
                                }
                                VariableType::FmiBoolean => {
                                    assignments.push(quote! {
                                        self.#field_name = #start_expr;
                                    });
                                }
                                VariableType::FmiString => {
                                    assignments.push(quote! {
                                        self.#field_name = #start_expr.to_string();
                                    });
                                }
                                VariableType::FmiBinary => {
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
                        let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self);
                        derivatives[#i] = self.#field_name;
                    }
                });
            } else {
                // Fallback to old behavior if no alias found
                let derivative_field_name = format_ident!("der_{}", state_name);
                derivative_assignments.push(quote! {
                    if #i < derivatives.len() {
                        let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self);
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
                        if matches!(vtype, VariableType::FmiFloat32 | VariableType::FmiFloat64) {
                            let variant_name =
                                format_ident!("{}", to_pascal_case(&field.ident.to_string()));
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

struct Float64GetterGen<'a>(&'a Model);

impl<'a> Float64GetterGen<'a> {
    fn new(model: &'a Model) -> Self {
        Self(model)
    }
}

/// Generate the variant name for a ValueRef enum from a variable name
fn generate_variant_name(name: &str) -> syn::Ident {
    let variant_name = if name.starts_with("der(") && name.ends_with(")") {
        let inner = &name[4..name.len() - 1];
        format!("Der{}", to_pascal_case(inner))
    } else {
        to_pascal_case(name)
    };
    format_ident!("{}", variant_name)
}

impl ToTokens for Float64GetterGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for field in &self.0.fields {
            if let Ok(vtype) = rust_type_to_variable_type(&field.ty) {
                if vtype == VariableType::FmiFloat64 {
                    // Add case for main variable
                    let has_variable = field
                        .attrs
                        .iter()
                        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
                    if has_variable {
                        let variant_name =
                            format_ident!("{}", to_pascal_case(&field.ident.to_string()));
                        let field_name = &field.ident;

                        tokens.extend(quote! {
                            ValueRef::#variant_name => *value = self.#field_name,
                        });
                    }

                    // Add cases for aliases of this variable
                    for attr in &field.attrs {
                        if let FieldAttributeOuter::Alias(alias_attr) = attr {
                            if let Some(alias_name) = &alias_attr.name {
                                let alias_variant_name = generate_variant_name(alias_name);
                                let field_name = &field.ident;

                                tokens.extend(quote! {
                                    ValueRef::#alias_variant_name => {
                                        let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self);
                                        *value = self.#field_name;
                                    },
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

struct Float64SetterGen<'a>(&'a Model);

impl<'a> Float64SetterGen<'a> {
    fn new(model: &'a Model) -> Self {
        Self(model)
    }
}

impl ToTokens for Float64SetterGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for field in &self.0.fields {
            if let Ok(vtype) = rust_type_to_variable_type(&field.ty) {
                if vtype == VariableType::FmiFloat64 {
                    // Only generate setter for main variable (not aliases)
                    let has_variable = field
                        .attrs
                        .iter()
                        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
                    if has_variable {
                        let variant_name =
                            format_ident!("{}", to_pascal_case(&field.ident.to_string()));
                        let field_name = &field.ident;

                        tokens.extend(quote! {
                            ValueRef::#variant_name => self.#field_name = *value,
                        });
                    }
                    // Note: Aliases (especially derivatives) typically shouldn't be settable
                }
            }
        }
    }
}

struct Float32GetterGen<'a>(&'a Model);

impl<'a> Float32GetterGen<'a> {
    fn new(model: &'a Model) -> Self {
        Self(model)
    }
}

impl ToTokens for Float32GetterGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for field in &self.0.fields {
            if let Ok(vtype) = rust_type_to_variable_type(&field.ty) {
                if vtype == VariableType::FmiFloat32 {
                    // Add case for main variable
                    let has_variable = field
                        .attrs
                        .iter()
                        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
                    if has_variable {
                        let variant_name =
                            format_ident!("{}", to_pascal_case(&field.ident.to_string()));
                        let field_name = &field.ident;

                        tokens.extend(quote! {
                            ValueRef::#variant_name => *value = self.#field_name,
                        });
                    }

                    // Add cases for aliases of this variable
                    for attr in &field.attrs {
                        if let FieldAttributeOuter::Alias(alias_attr) = attr {
                            if let Some(alias_name) = &alias_attr.name {
                                let alias_variant_name = generate_variant_name(alias_name);
                                let field_name = &field.ident;

                                tokens.extend(quote! {
                                    ValueRef::#alias_variant_name => {
                                        let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self);
                                        *value = self.#field_name;
                                    },
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

struct Float32SetterGen<'a>(&'a Model);

impl<'a> Float32SetterGen<'a> {
    fn new(model: &'a Model) -> Self {
        Self(model)
    }
}

impl ToTokens for Float32SetterGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for field in &self.0.fields {
            if let Ok(vtype) = rust_type_to_variable_type(&field.ty) {
                if vtype == VariableType::FmiFloat32 {
                    // Only generate setter for main variable (not aliases)
                    let has_variable = field
                        .attrs
                        .iter()
                        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
                    if has_variable {
                        let variant_name =
                            format_ident!("{}", to_pascal_case(&field.ident.to_string()));
                        let field_name = &field.ident;

                        tokens.extend(quote! {
                            ValueRef::#variant_name => self.#field_name = *value,
                        });
                    }
                    // Note: Aliases (especially derivatives) typically shouldn't be settable
                }
            }
        }
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
