use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    Attribute, Data, DataStruct, DeriveInput, FieldsNamed, Ident, Meta, Type, parse_macro_input,
};

/// Main derive macro for FMU models
#[proc_macro_derive(FmuModel, attributes(model, variable))]
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

    // Extract fields and their variable attributes
    let fields = extract_fields(input)?;
    let variables = parse_variable_attributes(&fields)?;

    // Generate value reference enum
    let value_ref_enum = generate_value_ref_enum(&variables);

    // Generate Model implementation
    let model_data_impl = generate_model_data_impl(struct_name, &variables);

    Ok(quote! {
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
}

/// Parse variable attributes from fields
fn parse_variable_attributes(fields: &FieldsNamed) -> syn::Result<Vec<VariableInfo>> {
    let mut variables = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let field_type = field.ty.clone();

        // Look for #[variable(...)] attribute
        let mut var_info = VariableInfo {
            name: field_name,
            field_type,
            causality: None,
            variability: None,
            initial: None,
            start: None,
            is_state: false,
            description: None,
        };

        for attr in &field.attrs {
            if attr.path().is_ident("variable") {
                parse_variable_attribute_content(attr, &mut var_info)?;
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
                        _ => {} // Ignore unknown attributes
                    }
                }
            }
        }
        _ => return Err(syn::Error::new_spanned(attr, "Expected #[variable(...)]")),
    }

    Ok(())
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

        // If this is a state variable, also add its derivative
        if var.is_state {
            let der_variant_name = format_ident!("Der{}", to_pascal_case(&var.name));

            value_ref_variants.push(quote! {
                #der_variant_name = #vr_counter
            });

            from_u32_arms.push(quote! {
                #vr_counter => ValueRef::#der_variant_name
            });

            into_u32_arms.push(quote! {
                ValueRef::#der_variant_name => #vr_counter
            });

            vr_counter += 1;
        }
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
fn generate_model_data_impl(struct_name: &Ident, variables: &[VariableInfo]) -> TokenStream2 {
    let set_start_values_body = generate_set_start_values(variables);
    let calculate_values_body = generate_calculate_values(variables);
    let float64_getter_cases = generate_float64_getter_cases(variables);
    let float32_getter_cases = generate_float32_getter_cases(variables);

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
            const MODEL_DESCRIPTION: &'static str = "Auto-generated FMU model"; //TODO: pull this from the struct docstring

            fn set_start_values(&mut self) {
                #set_start_values_body
            }

            fn calculate_values(&mut self) -> fmi::fmi3::Fmi3Status {
                #calculate_values_body
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
            if let Ok(value) = start_value.parse::<f64>() {
                assignments.push(quote! {
                    self.#field_name = #value;
                });
            }
        }
    }

    quote! {
        #(#assignments)*
    }
}

/// Generate the body of calculate_values (state derivatives)
fn generate_calculate_values(_variables: &[VariableInfo]) -> TokenStream2 {
    // For now, return OK - users will implement their own calculation logic
    // In the future, this could support simple derivative definitions in attributes
    quote! {
        // Implement your calculation logic here
        // This method is called to calculate derived values and state derivatives
        fmi::fmi3::Fmi3Res::OK.into()
    }
}

/// Generate getter cases for float64 values
fn generate_float64_getter_cases(variables: &[VariableInfo]) -> Vec<TokenStream2> {
    let mut cases = Vec::new();

    for var in variables {
        if is_float64_type(&var.field_type) {
            let variant_name = format_ident!("{}", to_pascal_case(&var.name));
            let field_name = format_ident!("{}", &var.name);

            cases.push(quote! {
                ValueRef::#variant_name => *value = self.#field_name,
            });

            // NOTE: We don't generate derivative getters here since the macro doesn't add derivative fields.
            // In a real implementation, this would need to be addressed differently.
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
            let field_name = format_ident!("{}", &var.name);

            cases.push(quote! {
                ValueRef::#variant_name => *value = self.#field_name,
            });
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

/// Convert snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
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
