//! Code generation for the derive macro

use proc_macro_error2::abort;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

use crate::model::{ExtendedModelInfo, VariableInfo};
use crate::parsing::{is_float32_type, is_float64_type, to_pascal_case};
use crate::schema_gen::generate_model_description;

/// Main code generation structure
pub struct CodeGenerator {
    pub model: ExtendedModelInfo,
    pub model_description_xml: String,
}

impl CodeGenerator {
    pub fn new(model: ExtendedModelInfo) -> Self {
        // Generate the model description
        let model_desc = generate_model_description(&model);

        // Serialize to XML at compile time
        let model_description_xml = match yaserde::ser::to_string(&model_desc) {
            Ok(xml) => xml,
            Err(e) => {
                abort!(
                    "Warning: Failed to serialize model description to XML: {}",
                    e
                );
            }
        };

        Self {
            model,
            model_description_xml,
        }
    }
}

impl ToTokens for CodeGenerator {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = format_ident!("{}", &self.model.model.name);

        // Generate value reference enum
        let value_ref_enum = ValueRefEnum::new(&self.model.all_variables);

        // Generate Model implementation
        let model_impl = ModelImpl::new(&struct_name, &self.model, &self.model_description_xml);

        // Generate GetSet implementation
        let getset_impl = GetSetImpl::new(&struct_name, &self.model.all_variables);

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
    variables: &'a [VariableInfo],
}

impl<'a> ValueRefEnum<'a> {
    fn new(variables: &'a [VariableInfo]) -> Self {
        Self { variables }
    }
}

impl ToTokens for ValueRefEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut value_ref_variants = Vec::new();
        let mut from_u32_arms = Vec::new();
        let mut into_u32_arms = Vec::new();

        let mut vr_counter = 0u32;

        for var in self.variables {
            let variant_name = format_ident!("{}", to_pascal_case(&var.name));

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

            // Add aliases for this variable
            for alias in &var.aliases {
                let alias_variant_name = format_ident!("{}", to_pascal_case(&alias.name));

                value_ref_variants.push(quote! {
                    #alias_variant_name = #vr_counter
                });

                from_u32_arms.push(quote! {
                    #vr_counter => ValueRef::#alias_variant_name
                });

                into_u32_arms.push(quote! {
                    ValueRef::#alias_variant_name => #vr_counter
                });

                vr_counter += 1;
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

/// Generate the Model trait implementation
struct ModelImpl<'a> {
    struct_name: &'a Ident,
    model: &'a ExtendedModelInfo,
    model_description_xml: &'a str,
}

impl<'a> ModelImpl<'a> {
    fn new(
        struct_name: &'a Ident,
        model: &'a ExtendedModelInfo,
        model_description_xml: &'a str,
    ) -> Self {
        Self {
            struct_name,
            model,
            model_description_xml,
        }
    }
}

impl ToTokens for ModelImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = self.struct_name;
        let model_name = &self.model.model.name;
        let _model_description = self
            .model
            .model
            .description
            .as_deref()
            .unwrap_or("Auto-generated FMU model");
        let model_description_xml = self.model_description_xml;

        // Generate instantiation token
        let instantiation_token = crate::schema_gen::generate_instantiation_token(model_name);

        // Generate function bodies
        let set_start_values_body = SetStartValuesGen::new(&self.model.model.variables);
        let get_continuous_states_body = GetContinuousStatesGen::new(&self.model.model.variables);
        let set_continuous_states_body = SetContinuousStatesGen::new(&self.model.model.variables);
        let get_derivatives_body = GetDerivativesGen::new(&self.model.model.variables);
        let model_variables_body = ModelVariablesGen::new(&self.model.all_variables);
        let model_structure_body = ModelStructureGen::new(&self.model.all_variables);
        let variable_validation_body = VariableValidationGen::new(&self.model.all_variables);

        let number_of_continuous_states = count_continuous_states(&self.model.model.variables);

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
                    1
                }

                fn model_variables() -> fmi::fmi3::schema::ModelVariables {
                    #model_variables_body
                }

                fn model_structure() -> fmi::fmi3::schema::ModelStructure {
                    #model_structure_body
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
    variables: &'a [VariableInfo],
}

impl<'a> GetSetImpl<'a> {
    fn new(struct_name: &'a Ident, variables: &'a [VariableInfo]) -> Self {
        Self {
            struct_name,
            variables,
        }
    }
}

impl ToTokens for GetSetImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = self.struct_name;

        let float64_getter_cases = Float64GetterGen::new(self.variables);
        let float64_setter_cases = Float64SetterGen::new(self.variables);
        let float32_getter_cases = Float32GetterGen::new(self.variables);

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
            }
        });
    }
}

// Helper generators for specific function bodies

struct SetStartValuesGen<'a>(&'a [VariableInfo]);

impl<'a> SetStartValuesGen<'a> {
    fn new(variables: &'a [VariableInfo]) -> Self {
        Self(variables)
    }
}

impl ToTokens for SetStartValuesGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut assignments = Vec::new();

        for var in self.0 {
            if let Some(start_value) = &var.start {
                let field_name = format_ident!("{}", &var.name);

                if is_float64_type(&var.field_type) {
                    if let Ok(value) = start_value.parse::<f64>() {
                        assignments.push(quote! {
                            self.#field_name = #value;
                        });
                    }
                } else if is_float32_type(&var.field_type) {
                    if let Ok(value) = start_value.parse::<f32>() {
                        assignments.push(quote! {
                            self.#field_name = #value;
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

struct GetContinuousStatesGen<'a>(&'a [VariableInfo]);

impl<'a> GetContinuousStatesGen<'a> {
    fn new(variables: &'a [VariableInfo]) -> Self {
        Self(variables)
    }
}

impl ToTokens for GetContinuousStatesGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut state_assignments = Vec::new();
        let mut index = 0usize;

        for var in self.0 {
            if var.is_state {
                let field_name = format_ident!("{}", var.name);
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

struct SetContinuousStatesGen<'a>(&'a [VariableInfo]);

impl<'a> SetContinuousStatesGen<'a> {
    fn new(variables: &'a [VariableInfo]) -> Self {
        Self(variables)
    }
}

impl ToTokens for SetContinuousStatesGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut state_assignments = Vec::new();
        let mut index = 0usize;

        for var in self.0 {
            if var.is_state {
                let field_name = format_ident!("{}", var.name);
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

struct GetDerivativesGen<'a>(&'a [VariableInfo]);

impl<'a> GetDerivativesGen<'a> {
    fn new(variables: &'a [VariableInfo]) -> Self {
        Self(variables)
    }
}

impl ToTokens for GetDerivativesGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut derivative_assignments = Vec::new();
        let mut state_variables = Vec::new();

        // Collect state variables in order
        for var in self.0 {
            if var.is_state {
                state_variables.push(var);
            }
        }

        if state_variables.is_empty() {
            tokens.extend(quote! {
                // No derivatives in this model
                Ok(fmi::fmi3::Fmi3Res::OK)
            });
            return;
        }

        // Generate assignments that find the derivative field for each state
        for (i, state_var) in state_variables.iter().enumerate() {
            let derivative_name = format!("der({})", state_var.name);

            // Look for a field that has an alias matching the derivative name
            let mut derivative_field = None;
            for var in self.0 {
                for alias in &var.aliases {
                    if alias.name == derivative_name {
                        derivative_field = Some(&var.name);
                        break;
                    }
                }
                if derivative_field.is_some() {
                    break;
                }
            }

            if let Some(der_field_name) = derivative_field {
                let field_name = format_ident!("{}", der_field_name);
                derivative_assignments.push(quote! {
                    if #i < derivatives.len() {
                        let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self);
                        derivatives[#i] = self.#field_name;
                    }
                });
            } else {
                // Fallback to old behavior if no alias found
                let derivative_field_name = format_ident!("der_{}", state_var.name);
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

struct ModelVariablesGen<'a>(&'a [VariableInfo]);

impl<'a> ModelVariablesGen<'a> {
    fn new(variables: &'a [VariableInfo]) -> Self {
        Self(variables)
    }
}

impl ToTokens for ModelVariablesGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut float64_vars = Vec::new();
        let mut float32_vars = Vec::new();
        let mut vr_counter = 0u32;

        for var in self.0 {
            let name = &var.name;
            let description = var
                .description
                .as_deref()
                .unwrap_or("Auto-generated variable");

            // Parse causality
            let causality = match var.causality.as_deref() {
                Some("parameter") => quote! { fmi::fmi3::schema::Causality::Parameter },
                Some("input") => quote! { fmi::fmi3::schema::Causality::Input },
                Some("output") => quote! { fmi::fmi3::schema::Causality::Output },
                Some("local") => quote! { fmi::fmi3::schema::Causality::Local },
                Some("independent") => quote! { fmi::fmi3::schema::Causality::Independent },
                Some("calculatedParameter") => {
                    quote! { fmi::fmi3::schema::Causality::CalculatedParameter }
                }
                Some("structuralParameter") => {
                    quote! { fmi::fmi3::schema::Causality::StructuralParameter }
                }
                _ => quote! { fmi::fmi3::schema::Causality::Local },
            };

            // Parse variability with appropriate defaults based on causality
            let variability = match var.variability.as_deref() {
                Some("constant") => quote! { Some(fmi::fmi3::schema::Variability::Constant) },
                Some("fixed") => quote! { Some(fmi::fmi3::schema::Variability::Fixed) },
                Some("tunable") => quote! { Some(fmi::fmi3::schema::Variability::Tunable) },
                Some("discrete") => quote! { Some(fmi::fmi3::schema::Variability::Discrete) },
                Some("continuous") => quote! { Some(fmi::fmi3::schema::Variability::Continuous) },
                _ => {
                    // Apply FMI 3.0 default variability rules based on causality
                    match var.causality.as_deref() {
                        Some("local") => quote! { Some(fmi::fmi3::schema::Variability::Fixed) },
                        Some("parameter") => quote! { Some(fmi::fmi3::schema::Variability::Fixed) },
                        _ => {
                            // For input/output variables, use type-based defaults
                            if is_float64_type(&var.field_type) || is_float32_type(&var.field_type)
                            {
                                quote! { Some(fmi::fmi3::schema::Variability::Continuous) }
                            } else {
                                quote! { Some(fmi::fmi3::schema::Variability::Discrete) }
                            }
                        }
                    }
                }
            };

            // Parse initial
            let initial = match var.initial.as_deref() {
                Some("exact") => quote! { Some(fmi::fmi3::schema::Initial::Exact) },
                Some("approx") => quote! { Some(fmi::fmi3::schema::Initial::Approx) },
                Some("calculated") => quote! { Some(fmi::fmi3::schema::Initial::Calculated) },
                _ => quote! { None },
            };

            if is_float64_type(&var.field_type) {
                // Parse start value for f64
                let start_value = if let Some(start) = &var.start {
                    if let Ok(value) = start.parse::<f64>() {
                        quote! { vec![#value] }
                    } else {
                        quote! { vec![] }
                    }
                } else {
                    quote! { vec![] }
                };

                float64_vars.push(quote! {
                    fmi::fmi3::schema::FmiFloat64 {
                        init_var: fmi::fmi3::schema::InitializableVariable {
                            typed_arrayable_var: fmi::fmi3::schema::TypedArrayableVariable {
                                arrayable_var: fmi::fmi3::schema::ArrayableVariable {
                                    abstract_var: fmi::fmi3::schema::AbstractVariable {
                                        name: #name.to_string(),
                                        value_reference: #vr_counter,
                                        description: Some(#description.to_string()),
                                        causality: #causality,
                                        variability: #variability,
                                        can_handle_multiple_set_per_time_instant: None,
                                    },
                                    dimensions: vec![],
                                    intermediate_update: None,
                                    previous: None,
                                },
                                declared_type: None,
                            },
                            initial: #initial,
                        },
                        start: #start_value,
                        ..Default::default()
                    }
                });
            } else if is_float32_type(&var.field_type) {
                // Parse start value for f32
                let start_value = if let Some(start) = &var.start {
                    if let Ok(value) = start.parse::<f32>() {
                        quote! { vec![#value] }
                    } else {
                        quote! { vec![] }
                    }
                } else {
                    quote! { vec![] }
                };

                float32_vars.push(quote! {
                    fmi::fmi3::schema::FmiFloat32 {
                        init_var: fmi::fmi3::schema::InitializableVariable {
                            typed_arrayable_var: fmi::fmi3::schema::TypedArrayableVariable {
                                arrayable_var: fmi::fmi3::schema::ArrayableVariable {
                                    abstract_var: fmi::fmi3::schema::AbstractVariable {
                                        name: #name.to_string(),
                                        value_reference: #vr_counter,
                                        description: Some(#description.to_string()),
                                        causality: #causality,
                                        variability: #variability,
                                        can_handle_multiple_set_per_time_instant: None,
                                    },
                                    dimensions: vec![],
                                    intermediate_update: None,
                                    previous: None,
                                },
                                declared_type: None,
                            },
                            initial: #initial,
                        },
                        start: #start_value,
                        ..Default::default()
                    }
                });
            }

            vr_counter += 1;

            // Process aliases for this variable
            for alias in &var.aliases {
                let alias_name = &alias.name;
                let alias_description = alias
                    .description
                    .as_deref()
                    .unwrap_or("Auto-generated alias variable");

                // Parse alias causality
                let alias_causality = match alias.causality.as_deref() {
                    Some("parameter") => quote! { fmi::fmi3::schema::Causality::Parameter },
                    Some("input") => quote! { fmi::fmi3::schema::Causality::Input },
                    Some("output") => quote! { fmi::fmi3::schema::Causality::Output },
                    Some("local") => quote! { fmi::fmi3::schema::Causality::Local },
                    Some("independent") => quote! { fmi::fmi3::schema::Causality::Independent },
                    Some("calculatedParameter") => {
                        quote! { fmi::fmi3::schema::Causality::CalculatedParameter }
                    }
                    Some("structuralParameter") => {
                        quote! { fmi::fmi3::schema::Causality::StructuralParameter }
                    }
                    _ => quote! { fmi::fmi3::schema::Causality::Local },
                };

                // Aliases are usually continuous and calculated
                let alias_variability = quote! { Some(fmi::fmi3::schema::Variability::Continuous) };
                let alias_initial = quote! { Some(fmi::fmi3::schema::Initial::Calculated) };

                // Find the derivative attribute - if this alias is a derivative, set the derivative field
                let derivative_vr = if let Some(derivative_of) = &alias.derivative {
                    // Find the value reference of the variable this is a derivative of
                    let mut derivative_target_vr = None;
                    let mut temp_vr = 0u32;

                    for check_var in self.0 {
                        if check_var.name == *derivative_of {
                            derivative_target_vr = Some(temp_vr);
                            break;
                        }
                        temp_vr += 1;
                        // Skip alias entries when counting
                        for _ in &check_var.aliases {
                            temp_vr += 1;
                        }
                    }

                    if let Some(target_vr) = derivative_target_vr {
                        quote! { Some(#target_vr) }
                    } else {
                        quote! { None }
                    }
                } else {
                    quote! { None }
                };

                if is_float64_type(&var.field_type) {
                    float64_vars.push(quote! {
                        fmi::fmi3::schema::FmiFloat64 {
                            init_var: fmi::fmi3::schema::InitializableVariable {
                                typed_arrayable_var: fmi::fmi3::schema::TypedArrayableVariable {
                                    arrayable_var: fmi::fmi3::schema::ArrayableVariable {
                                        abstract_var: fmi::fmi3::schema::AbstractVariable {
                                            name: #alias_name.to_string(),
                                            value_reference: #vr_counter,
                                            description: Some(#alias_description.to_string()),
                                            causality: #alias_causality,
                                            variability: #alias_variability,
                                            can_handle_multiple_set_per_time_instant: None,
                                        },
                                        dimensions: vec![],
                                        intermediate_update: None,
                                        previous: None,
                                    },
                                    declared_type: None,
                                },
                                initial: #alias_initial,
                            },
                            start: vec![], // Aliases typically don't have start values
                            real_var_attr: fmi::fmi3::schema::RealVariableAttributes {
                                derivative: #derivative_vr,
                                reinit: None,
                            },
                            ..Default::default()
                        }
                    });
                } else if is_float32_type(&var.field_type) {
                    float32_vars.push(quote! {
                        fmi::fmi3::schema::FmiFloat32 {
                            init_var: fmi::fmi3::schema::InitializableVariable {
                                typed_arrayable_var: fmi::fmi3::schema::TypedArrayableVariable {
                                    arrayable_var: fmi::fmi3::schema::ArrayableVariable {
                                        abstract_var: fmi::fmi3::schema::AbstractVariable {
                                            name: #alias_name.to_string(),
                                            value_reference: #vr_counter,
                                            description: Some(#alias_description.to_string()),
                                            causality: #alias_causality,
                                            variability: #alias_variability,
                                            can_handle_multiple_set_per_time_instant: None,
                                        },
                                        dimensions: vec![],
                                        intermediate_update: None,
                                        previous: None,
                                    },
                                    declared_type: None,
                                },
                                initial: #alias_initial,
                            },
                            start: vec![], // Aliases typically don't have start values
                            real_var_attr: fmi::fmi3::schema::RealVariableAttributes {
                                derivative: #derivative_vr,
                                reinit: None,
                            },
                            ..Default::default()
                        }
                    });
                }

                vr_counter += 1;
            }
        }

        tokens.extend(quote! {
            fmi::fmi3::schema::ModelVariables {
                float64: vec![#(#float64_vars),*],
                float32: vec![#(#float32_vars),*],
                ..Default::default()
            }
        });
    }
}

struct ModelStructureGen<'a>(&'a [VariableInfo]);

impl<'a> ModelStructureGen<'a> {
    fn new(variables: &'a [VariableInfo]) -> Self {
        Self(variables)
    }
}

impl ToTokens for ModelStructureGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut outputs = Vec::new();
        let mut derivatives = Vec::new();
        let mut initial_unknowns = Vec::new();
        let mut event_indicators = Vec::new();

        let mut vr_counter = 0u32;

        for var in self.0 {
            // Check if this is an output variable
            if var.causality.as_deref() == Some("output") {
                outputs.push(quote! {
                    fmi::fmi3::schema::Fmi3Unknown {
                        value_reference: #vr_counter,
                        dependencies: vec![],
                        dependencies_kind: vec![],
                        ..Default::default()
                    }
                });
            }

            // Check if this variable should be an event indicator (e.g., height for bouncing ball)
            if var.name == "h" && var.is_state {
                event_indicators.push(quote! {
                    fmi::fmi3::schema::Fmi3Unknown {
                        value_reference: #vr_counter,
                        dependencies: vec![],
                        dependencies_kind: vec![],
                        ..Default::default()
                    }
                });
            }

            vr_counter += 1;

            // Process aliases for this variable
            for alias in &var.aliases {
                // Check if this is a derivative alias
                if alias.name.starts_with("der(") && alias.causality.as_deref() == Some("local") {
                    // Add as ContinuousStateDerivative
                    derivatives.push(quote! {
                        fmi::fmi3::schema::Fmi3Unknown {
                            value_reference: #vr_counter,
                            dependencies: vec![], // Will be computed properly by the solver
                            dependencies_kind: vec![],
                            ..Default::default()
                        }
                    });

                    // Add as InitialUnknown - derivative depends on the field that contains it
                    let base_vr = vr_counter - 1; // The variable this alias belongs to
                    initial_unknowns.push(quote! {
                        fmi::fmi3::schema::Fmi3Unknown {
                            value_reference: #vr_counter,
                            dependencies: vec![#base_vr],
                            dependencies_kind: vec![fmi::fmi3::schema::DependenciesKind::Constant],
                            ..Default::default()
                        }
                    });
                }

                vr_counter += 1;
            }
        }

        tokens.extend(quote! {
            fmi::fmi3::schema::ModelStructure {
                outputs: vec![#(#outputs),*],
                continuous_state_derivative: vec![#(#derivatives),*],
                initial_unknown: vec![#(#initial_unknowns),*],
                event_indicator: vec![#(#event_indicators),*],
                ..Default::default()
            }
        });
    }
}

struct VariableValidationGen<'a>(&'a [VariableInfo]);

impl<'a> VariableValidationGen<'a> {
    fn new(variables: &'a [VariableInfo]) -> Self {
        Self(variables)
    }
}

impl ToTokens for VariableValidationGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut cases = Vec::new();

        // Helper function to determine effective variability including defaults
        fn effective_variability(var: &VariableInfo) -> &str {
            match var.variability.as_deref() {
                Some(v) => v,
                None => {
                    // Apply FMI 3.0 default variability rules based on causality
                    match var.causality.as_deref() {
                        Some("local") => "fixed",
                        Some("parameter") => "fixed",
                        _ => {
                            // For input/output variables, use type-based defaults
                            if is_float64_type(&var.field_type) || is_float32_type(&var.field_type)
                            {
                                "continuous"
                            } else {
                                "discrete"
                            }
                        }
                    }
                }
            }
        }

        for var in self.0 {
            if is_float64_type(&var.field_type) {
                let variant_name = format_ident!("{}", to_pascal_case(&var.name));
                let var_name = &var.name;

                // Skip derivative variables
                if var.name.starts_with("der_") {
                    continue;
                }

                // Generate validation based on causality and effective variability
                let eff_variability = effective_variability(var);
                let validation = match (var.causality.as_deref(), eff_variability) {
                    (Some("parameter"), "fixed") => {
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
                    (Some("parameter"), "tunable") => {
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
                    (Some("local"), "fixed") => {
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
                    (Some("input"), _) => {
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

        tokens.extend(quote! {
            match ValueRef::from(vr) {
                #(#cases)*
                _ => Ok(()), // Unknown variables are allowed by default
            }
        });
    }
}

struct Float64GetterGen<'a>(&'a [VariableInfo]);

impl<'a> Float64GetterGen<'a> {
    fn new(variables: &'a [VariableInfo]) -> Self {
        Self(variables)
    }
}

impl ToTokens for Float64GetterGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for var in self.0 {
            if is_float64_type(&var.field_type) {
                let variant_name = format_ident!("{}", to_pascal_case(&var.name));
                let field_name = format_ident!("{}", &var.name);

                tokens.extend(quote! {
                    ValueRef::#variant_name => *value = self.#field_name,
                });

                // Add cases for aliases of this variable
                for alias in &var.aliases {
                    let alias_variant_name = format_ident!("{}", to_pascal_case(&alias.name));

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

struct Float64SetterGen<'a>(&'a [VariableInfo]);

impl<'a> Float64SetterGen<'a> {
    fn new(variables: &'a [VariableInfo]) -> Self {
        Self(variables)
    }
}

impl ToTokens for Float64SetterGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for var in self.0 {
            if is_float64_type(&var.field_type)
                && !var.name.starts_with("der_")
                && !var.name.starts_with("der(")
            {
                let variant_name = format_ident!("{}", to_pascal_case(&var.name));
                let field_name = format_ident!("{}", &var.name);

                tokens.extend(quote! {
                    ValueRef::#variant_name => self.#field_name = *value,
                });
            }
        }
    }
}

struct Float32GetterGen<'a>(&'a [VariableInfo]);

impl<'a> Float32GetterGen<'a> {
    fn new(variables: &'a [VariableInfo]) -> Self {
        Self(variables)
    }
}

impl ToTokens for Float32GetterGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for var in self.0 {
            if is_float32_type(&var.field_type) && !var.name.starts_with("der_") {
                let variant_name = format_ident!("{}", to_pascal_case(&var.name));
                let field_name = format_ident!("{}", &var.name);

                tokens.extend(quote! {
                    ValueRef::#variant_name => *value = self.#field_name,
                });
            }
        }
    }
}

/// Count the number of continuous states
fn count_continuous_states(variables: &[VariableInfo]) -> usize {
    variables.iter().filter(|var| var.is_state).count()
}
