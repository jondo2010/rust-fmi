use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DataStruct, DeriveInput, FieldsNamed, Ident, Meta, Type, parse_macro_input,
};

/// Main derive macro for FMU models
#[proc_macro_derive(FmuModel, attributes(model, variable, alias))]
pub fn derive_fmu_model(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    match fmu_model_impl(&input) {
        Ok(expanded) => expanded.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn fmu_model_impl(input: &DeriveInput) -> syn::Result<TokenStream2> {
    let struct_name = &input.ident;

    // Parse the model attribute to determine FMI interface type
    let _interface_type = parse_model_attribute(&input.attrs)?;

    // Extract struct docstring for model description
    let struct_description = extract_docstring(&input.attrs);

    // Extract fields and their variable attributes
    let fields = extract_fields(input)?;
    let user_variables = parse_variable_attributes(&fields)?;

    // Create extended variable info that includes auto-generated derivatives
    let extended_vars = ExtendedVariableInfo::from_user_variables(user_variables);

    // Generate value reference enum (include all variables in same order as model_variables)
    let value_ref_enum = generate_value_ref_enum(&extended_vars.all_variables);

    // Generate Model implementation
    let model_data_impl = generate_model_data_impl(struct_name, &extended_vars, struct_description);

    // Generate derivative field storage and access methods
    let derivative_storage = generate_derivative_storage(struct_name, &extended_vars);

    Ok(quote! {
        #derivative_storage
        #value_ref_enum
        #model_data_impl
    })
}

/// Parse the #[model(...)] attribute
fn parse_model_attribute(attrs: &[Attribute]) -> syn::Result<String> {
    for attr in attrs {
        if attr.path().is_ident("model") {
            match &attr.meta {
                Meta::List(meta_list) => {
                    // Extract the interface type (e.g., ModelExchange, CoSimulation)
                    let tokens = &meta_list.tokens;
                    return Ok(tokens.to_string());
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        attr,
                        "Expected #[model(InterfaceType)]",
                    ));
                }
            }
        }
    }
    Err(syn::Error::new(
        proc_macro2::Span::call_site(),
        "Missing #[model(...)] attribute",
    ))
}

/// Extract fields from the struct
fn extract_fields(input: &DeriveInput) -> syn::Result<&FieldsNamed> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => Ok(fields),
        _ => Err(syn::Error::new_spanned(
            input,
            "FmuModel can only be derived for structs with named fields",
        )),
    }
}

/// Extract docstring from attributes
/// Docstrings are stored as `#[doc = "..."]` attributes in Rust
fn extract_docstring(attrs: &[Attribute]) -> Option<String> {
    let mut docstring_parts = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(name_value) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &name_value.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        let doc_line = lit_str.value();
                        // Remove leading space that rustdoc adds
                        let cleaned = doc_line.strip_prefix(' ').unwrap_or(&doc_line);
                        docstring_parts.push(cleaned.to_string());
                    }
                }
            }
        }
    }

    if docstring_parts.is_empty() {
        None
    } else {
        // Join multiple doc lines with spaces
        Some(docstring_parts.join(" ").trim().to_string())
    }
}

/// Information about a variable
#[derive(Debug, Clone)]
struct VariableInfo {
    name: String,
    field_type: Type,
    causality: Option<String>,
    variability: Option<String>,
    initial: Option<String>,
    start: Option<String>,
    is_state: bool,
    description: Option<String>,
    /// If this variable is an alias for another variable, store the target name
    alias_of: Option<String>,
    /// If this variable is a derivative of another variable, store the target name
    derivative_of: Option<String>,
    /// Aliases defined for this field (additional variable references)
    aliases: Vec<AliasInfo>,
}

/// Information about an alias for a field
#[derive(Debug, Clone)]
struct AliasInfo {
    name: String,
    causality: Option<String>,
    variability: Option<String>,
    initial: Option<String>,
    start: Option<String>,
    derivative: Option<String>,
    description: Option<String>,
}

/// Extended variable information including derived variables (derivatives)
#[derive(Debug, Clone)]
struct ExtendedVariableInfo {
    /// Original user-defined variables
    user_variables: Vec<VariableInfo>,
    /// Auto-generated derivative variables for each state variable
    derivative_variables: Vec<VariableInfo>,
    /// All variables combined (user + derivatives)
    all_variables: Vec<VariableInfo>,
}

impl ExtendedVariableInfo {
    fn from_user_variables(user_vars: Vec<VariableInfo>) -> Self {
        let mut derivative_variables = Vec::new();
        let mut all_variables = user_vars.clone();

        // Derivative fields are explicitly declared by the user in their struct definition
        // The derive macro processes all fields including manually declared derivative fields
        /*
        // NOTE: Previously we considered automatic derivative generation, but explicit declaration
        // provides better clarity and control for users
        // Generate derivative variables for each state variable
        for var in &user_vars {
            if var.is_state {
                let der_var = VariableInfo {
                    name: format!("der_{}", var.name),
                    field_type: var.field_type.clone(), // Same type as state variable
                    causality: Some("Local".to_string()),
                    variability: Some("Continuous".to_string()),
                    initial: None,
                    start: Some("0.0".to_string()), // Default derivative start value
                    is_state: false,                // Derivatives are not states themselves
                    description: Some(format!("Derivative of {} with respect to time", var.name)),
                };
                derivative_variables.push(der_var.clone());
                all_variables.push(der_var);
            }
        }
        */

        Self {
            user_variables: user_vars,
            derivative_variables,
            all_variables,
        }
    }
}

/// Parse variable attributes from fields
fn parse_variable_attributes(fields: &FieldsNamed) -> syn::Result<Vec<VariableInfo>> {
    let mut variables = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let field_type = field.ty.clone();

        // Extract docstring from field attributes
        let field_docstring = extract_docstring(&field.attrs);

        // Look for #[variable(...)] attribute
        let mut var_info = VariableInfo {
            name: field_name,
            field_type,
            causality: None,
            variability: None,
            initial: None,
            start: None,
            is_state: false,
            description: field_docstring, // Use docstring as default description
            alias_of: None,
            derivative_of: None,
            aliases: Vec::new(),
        };

        for attr in &field.attrs {
            if attr.path().is_ident("variable") {
                parse_variable_attribute_content(attr, &mut var_info)?;
            } else if attr.path().is_ident("alias") {
                let alias_info = parse_alias_attribute_content(attr)?;
                var_info.aliases.push(alias_info);
            }
        }

        variables.push(var_info);
    }

    Ok(variables)
}

/// Parse the content of a #[variable(...)] attribute
fn parse_variable_attribute_content(
    attr: &Attribute,
    var_info: &mut VariableInfo,
) -> syn::Result<()> {
    match &attr.meta {
        Meta::List(meta_list) => {
            let content = meta_list.tokens.to_string();

            // Simple parser for key = value pairs
            // In a real implementation, you'd want a more robust parser
            for pair in content.split(',') {
                let pair = pair.trim();
                if let Some((key, value)) = pair.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();

                    match key {
                        "causality" => var_info.causality = Some(value.to_string()),
                        "variability" => var_info.variability = Some(value.to_string()),
                        "initial" => var_info.initial = Some(value.to_string()),
                        "start" => var_info.start = Some(value.to_string()),
                        "state" => var_info.is_state = value == "true",
                        "description" => {
                            var_info.description = Some(value.trim_matches('"').to_string())
                        }
                        "alias_of" => var_info.alias_of = Some(value.trim_matches('"').to_string()),
                        "derivative_of" => {
                            var_info.derivative_of = Some(value.trim_matches('"').to_string())
                        }
                        _ => {} // Ignore unknown attributes
                    }
                }
            }
        }
        _ => return Err(syn::Error::new_spanned(attr, "Expected #[variable(...)]")),
    }

    Ok(())
}

/// Parse the content of a #[alias(...)] attribute
fn parse_alias_attribute_content(attr: &Attribute) -> syn::Result<AliasInfo> {
    let mut alias_info = AliasInfo {
        name: String::new(),
        causality: None,
        variability: None,
        initial: None,
        start: None,
        derivative: None,
        description: None,
    };

    match &attr.meta {
        Meta::List(meta_list) => {
            let content = meta_list.tokens.to_string();

            // Simple parser for key = value pairs
            for pair in content.split(',') {
                let pair = pair.trim();
                if let Some((key, value)) = pair.split_once('=') {
                    let key = key.trim();
                    let value = value.trim();

                    match key {
                        "name" => alias_info.name = value.trim_matches('"').to_string(),
                        "causality" => alias_info.causality = Some(value.to_string()),
                        "variability" => alias_info.variability = Some(value.to_string()),
                        "initial" => alias_info.initial = Some(value.to_string()),
                        "start" => alias_info.start = Some(value.to_string()),
                        "derivative" => {
                            alias_info.derivative = Some(value.trim_matches('"').to_string())
                        }
                        "description" => {
                            alias_info.description = Some(value.trim_matches('"').to_string())
                        }
                        _ => {} // Ignore unknown attributes
                    }
                }
            }
        }
        _ => return Err(syn::Error::new_spanned(attr, "Expected #[alias(...)]")),
    }

    if alias_info.name.is_empty() {
        return Err(syn::Error::new_spanned(attr, "Alias must have a name"));
    }

    Ok(alias_info)
}

/// Generate the ValueRef enum
fn generate_value_ref_enum(variables: &[VariableInfo]) -> TokenStream2 {
    let mut value_ref_variants = Vec::new();
    let mut from_u32_arms = Vec::new();
    let mut into_u32_arms = Vec::new();

    let mut vr_counter = 0u32;

    for var in variables.iter() {
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

        // Note: Derivative fields are explicitly declared by users and processed like any other field
    }

    quote! {
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
    }
}

/// Generate Model implementation
fn generate_model_data_impl(
    struct_name: &Ident,
    extended_vars: &ExtendedVariableInfo,
    struct_description: Option<String>,
) -> TokenStream2 {
    let set_start_values_body = generate_set_start_values(&extended_vars.user_variables);
    let float64_getter_cases = generate_float64_getter_cases(&extended_vars.all_variables);
    let float32_getter_cases = generate_float32_getter_cases(&extended_vars.all_variables);
    // IMPORTANT: Use all_variables for consistency between ValueRef and model_variables
    let model_variables_body = generate_model_variables(&extended_vars.all_variables);
    let model_structure_body = generate_model_structure(&extended_vars.all_variables);

    // Generate ModelExchange-specific functions - use only user variables for state access
    let get_continuous_states_body = generate_get_continuous_states(&extended_vars.user_variables);
    let set_continuous_states_body = generate_set_continuous_states(&extended_vars.user_variables);
    let get_derivatives_body = generate_get_derivatives(&extended_vars.user_variables);
    let number_of_continuous_states = count_continuous_states(&extended_vars.user_variables);

    // Generate the instantiation token at compile time
    let instantiation_token = generate_instantiation_token(&struct_name.to_string());

    // Use struct docstring if available, otherwise use default
    let model_description = struct_description
        .as_deref()
        .unwrap_or("Auto-generated FMU model");

    // Use struct docstring if available, otherwise use default
    let model_description = struct_description
        .as_deref()
        .unwrap_or("Auto-generated FMU model");

    quote! {
        impl ::fmi::fmi3::GetSet for #struct_name {
            type ValueRef = ::fmi::fmi3::binding::fmi3ValueReference;

            fn get_float64(
                &mut self,
                vrs: &[Self::ValueRef],
                values: &mut [f64],
            ) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
                for (vr, value) in vrs.iter().zip(values.iter_mut()) {
                    match ValueRef::from(*vr) {
                        #(#float64_getter_cases)*
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
                        #(#float32_getter_cases)*
                        _ => {} // Ignore unknown VRs for robustness
                    }
                }
                Ok(fmi::fmi3::Fmi3Res::OK)
            }
        }

        impl ::fmi_export::fmi3::Model for #struct_name {
            const MODEL_NAME: &'static str = stringify!(#struct_name);
            const MODEL_DESCRIPTION: &'static str = #model_description;
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
                // For this implementation, we assume one event indicator if the model
                // implements get_event_indicators in the UserModel trait
                // In a more sophisticated implementation, this could be derived from attributes
                1
            }

            fn model_variables() -> fmi::fmi3::schema::ModelVariables {
                #model_variables_body
            }

            fn model_structure() -> fmi::fmi3::schema::ModelStructure {
                #model_structure_body
            }
        }
    }
}

/// Generate the body of set_start_values
fn generate_set_start_values(variables: &[VariableInfo]) -> TokenStream2 {
    let mut assignments = Vec::new();

    for var in variables {
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

    quote! {
        #(#assignments)*
    }
}

/// Generate the body of calculate_values - this will delegate to UserModel::calculate_values
fn generate_calculate_values(_variables: &[VariableInfo]) -> TokenStream2 {
    // The generated implementation will delegate to the UserModel trait
    // Users implement UserModel::calculate_values with their specific logic
    quote! {
        // Delegate to UserModel implementation
        <Self as fmi_export::fmi3::UserModel>::calculate_values(self)
    }
}

/// Generate getter cases for float64 values
fn generate_float64_getter_cases(variables: &[VariableInfo]) -> Vec<TokenStream2> {
    let mut cases = Vec::new();

    for var in variables {
        if is_float64_type(&var.field_type) {
            let variant_name = format_ident!("{}", to_pascal_case(&var.name));
            let field_name = format_ident!("{}", &var.name);

            // Add case for the main variable
            cases.push(quote! {
                ValueRef::#variant_name => *value = self.#field_name,
            });

            // Add cases for aliases of this variable
            for alias in &var.aliases {
                let alias_variant_name = format_ident!("{}", to_pascal_case(&alias.name));

                cases.push(quote! {
                    ValueRef::#alias_variant_name => {
                        // For derivative aliases, we might want to call calculate_values first
                        let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self);
                        *value = self.#field_name;
                    },
                });
            }
        }
    }

    cases
}

/// Generate getter cases for float32 values
fn generate_float32_getter_cases(variables: &[VariableInfo]) -> Vec<TokenStream2> {
    let mut cases = Vec::new();

    for var in variables {
        if is_float32_type(&var.field_type) {
            let variant_name = format_ident!("{}", to_pascal_case(&var.name));

            if !var.name.starts_with("der_") {
                let field_name = format_ident!("{}", &var.name);
                cases.push(quote! {
                    ValueRef::#variant_name => *value = self.#field_name,
                });
            }
        }
    }

    cases
}

/// Check if a type is f64
fn is_float64_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "f64";
        }
    }
    false
}

/// Check if a type is f32
fn is_float32_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "f32";
        }
    }
    false
}

/// Generate the model_variables implementation
fn generate_model_variables(variables: &[VariableInfo]) -> TokenStream2 {
    let mut float64_vars = Vec::new();
    let mut float32_vars = Vec::new();
    let mut vr_counter = 0u32;

    for var in variables.iter() {
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

        // Parse variability
        let variability = match var.variability.as_deref() {
            Some("constant") => quote! { Some(fmi::fmi3::schema::Variability::Constant) },
            Some("fixed") => quote! { Some(fmi::fmi3::schema::Variability::Fixed) },
            Some("tunable") => quote! { Some(fmi::fmi3::schema::Variability::Tunable) },
            Some("discrete") => quote! { Some(fmi::fmi3::schema::Variability::Discrete) },
            Some("continuous") => quote! { Some(fmi::fmi3::schema::Variability::Continuous) },
            _ => {
                if is_float64_type(&var.field_type) || is_float32_type(&var.field_type) {
                    quote! { Some(fmi::fmi3::schema::Variability::Continuous) }
                } else {
                    quote! { Some(fmi::fmi3::schema::Variability::Discrete) }
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

                for check_var in variables {
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

    quote! {
        fmi::fmi3::schema::ModelVariables {
            float64: vec![#(#float64_vars),*],
            float32: vec![#(#float32_vars),*],
            ..Default::default()
        }
    }
}

/// Generate the model_structure implementation
fn generate_model_structure(variables: &[VariableInfo]) -> TokenStream2 {
    let mut outputs = Vec::new();
    let mut derivatives = Vec::new();
    let mut initial_unknowns = Vec::new();
    let mut event_indicators = Vec::new();

    let mut vr_counter = 0u32;

    for var in variables {
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

    quote! {
        fmi::fmi3::schema::ModelStructure {
            outputs: vec![#(#outputs),*],
            continuous_state_derivative: vec![#(#derivatives),*],
            initial_unknown: vec![#(#initial_unknowns),*],
            event_indicator: vec![#(#event_indicators),*],
            ..Default::default()
        }
    }
}

/// Convert snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    // Handle special characters for alias names like "der(h)" -> "DerH"
    let cleaned = s
        .replace("(", "_")
        .replace(")", "")
        .replace("-", "_")
        .replace(" ", "_")
        .replace(".", "_");

    cleaned
        .split('_')
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}

/// Generate an instantiation token at compile time using proper UUID v5
fn generate_instantiation_token(model_name: &str) -> String {
    // Use the same namespace UUID as used elsewhere in rust-fmi
    // This ensures consistency between compile-time and runtime token generation
    const RUST_FMI_NAMESPACE: uuid::Uuid = uuid::uuid!("6ba7b810-9dad-11d1-80b4-00c04fd430c8");

    // Generate UUID v5 based on the model name using our rust-fmi namespace
    let uuid = uuid::Uuid::new_v5(&RUST_FMI_NAMESPACE, model_name.as_bytes());

    // Format with curly braces as shown in FMI examples
    format!("{{{}}}", uuid)
}

/// Generate the get_continuous_states function body
fn generate_get_continuous_states(variables: &[VariableInfo]) -> TokenStream2 {
    let mut state_assignments = Vec::new();
    let mut index = 0usize;

    for var in variables {
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
        quote! {
            // No continuous states in this model
            Ok(fmi::fmi3::Fmi3Res::OK)
        }
    } else {
        quote! {
            #(#state_assignments)*
            Ok(fmi::fmi3::Fmi3Res::OK)
        }
    }
}

/// Generate the set_continuous_states function body
fn generate_set_continuous_states(variables: &[VariableInfo]) -> TokenStream2 {
    let mut state_assignments = Vec::new();
    let mut index = 0usize;

    for var in variables {
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
        quote! {
            // No continuous states in this model
            Ok(fmi::fmi3::Fmi3Res::OK)
        }
    } else {
        quote! {
            #(#state_assignments)*
            Ok(fmi::fmi3::Fmi3Res::OK)
        }
    }
}

/// Generate the get_derivatives function body
fn generate_get_derivatives(variables: &[VariableInfo]) -> TokenStream2 {
    let mut derivative_assignments = Vec::new();
    let mut state_variables = Vec::new();

    // Collect state variables in order
    for var in variables {
        if var.is_state {
            state_variables.push(var);
        }
    }

    if state_variables.is_empty() {
        return quote! {
            // No derivatives in this model
            Ok(fmi::fmi3::Fmi3Res::OK)
        };
    }

    // Generate assignments that find the derivative field for each state
    for (i, state_var) in state_variables.iter().enumerate() {
        // Find the field that contains the derivative for this state variable
        let derivative_name = format!("der({})", state_var.name);

        // Look for a field that has an alias matching the derivative name
        let mut derivative_field = None;
        for var in variables {
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
                    // Ensure derivatives are up-to-date before returning them
                    let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self);
                    derivatives[#i] = self.#field_name;
                }
            });
        } else {
            // Fallback to old behavior if no alias found
            let derivative_field_name = format_ident!("der_{}", state_var.name);
            derivative_assignments.push(quote! {
                if #i < derivatives.len() {
                    // Ensure derivatives are up-to-date before returning them
                    let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self);
                    derivatives[#i] = self.#derivative_field_name;
                }
            });
        }
    }

    quote! {
        // Get computed derivatives and populate the output array
        #(#derivative_assignments)*
        Ok(fmi::fmi3::Fmi3Res::OK)
    }
}

/// Count the number of continuous states
fn count_continuous_states(variables: &[VariableInfo]) -> usize {
    variables.iter().filter(|var| var.is_state).count()
}

/// Generate derivative field storage and access methods
fn generate_derivative_storage(
    struct_name: &Ident,
    extended_vars: &ExtendedVariableInfo,
) -> TokenStream2 {
    if extended_vars.derivative_variables.is_empty() {
        return quote! {}; // No derivatives to generate
    }

    // Generate storage fields for derivative values
    let mut derivative_fields = Vec::new();

    for der_var in &extended_vars.derivative_variables {
        let der_name = &der_var.name;
        let field_name = format_ident!("{}_computed", der_name);
        derivative_fields.push(quote! {
            #field_name: f64,
        });
    }

    quote! {
        // Extend the struct with derivative storage fields
        impl #struct_name {
            /// Internal storage for computed derivative values
            /// Derivative fields are declared explicitly in the struct definition
        }

        // Note: Derivative fields are declared by the user and processed by the derive macro
        // Users should not manually declare these fields
    }
}
