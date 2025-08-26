//! Parsing module for extracting model information from attributes

use convert_case::{Case, Casing};
use proc_macro_error2::{Diagnostic, Level};
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Expr, ExprLit, FieldsNamed, Lit, Meta,
    MetaList, MetaNameValue, Type, spanned::Spanned,
};

use crate::model::{AliasInfo, ModelInfo, VariableInfo};

/// Parse the derive macro input into a structured model representation
pub fn parse_model_input(input: &DeriveInput) -> Result<ModelInfo, Diagnostic> {
    let struct_name = &input.ident;

    // Parse the model attribute to determine FMI interface type
    let interface_type = parse_model_attribute(&input.attrs)
        .map_err(|e| Diagnostic::spanned(input.span(), Level::Error, e.to_string()))?;

    // Extract struct docstring for model description
    let struct_description = extract_docstring(&input.attrs);

    // Extract fields and their variable attributes
    let fields = extract_fields(input)
        .map_err(|e| Diagnostic::spanned(input.span(), Level::Error, e.to_string()))?;

    let variables = parse_variable_attributes(fields)?;

    Ok(ModelInfo {
        name: struct_name.to_string(),
        interface_type,
        description: struct_description,
        variables,
    })
}

/// Parse the #[model(...)] attribute
fn parse_model_attribute(attrs: &[Attribute]) -> syn::Result<String> {
    for attr in attrs {
        if attr.path().is_ident("model") {
            match &attr.meta {
                Meta::List(MetaList { tokens, .. }) => {
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
fn extract_docstring(attrs: &[Attribute]) -> Option<String> {
    let mut docstring_parts = Vec::new();

    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let Meta::NameValue(MetaNameValue {
                value: Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }),
                ..
            }) = &attr.meta
            {
                let doc_line = lit_str.value();
                let cleaned = doc_line.strip_prefix(' ').unwrap_or(&doc_line);
                docstring_parts.push(cleaned.to_string());
            }
        }
    }

    if docstring_parts.is_empty() {
        None
    } else {
        Some(docstring_parts.join(" ").trim().to_string())
    }
}

/// Parse variable attributes from fields
fn parse_variable_attributes(fields: &FieldsNamed) -> Result<Vec<VariableInfo>, Diagnostic> {
    let mut variables = Vec::new();

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let field_type = field.ty.clone();

        // Extract docstring from field attributes
        let field_docstring = extract_docstring(&field.attrs);

        // Look for #[variable(...)] and #[alias(...)] attributes
        let mut var_info = VariableInfo {
            name: field_name,
            field_type,
            causality: None,
            variability: None,
            initial: None,
            start: None,
            is_state: false,
            description: field_docstring,
            alias_of: None,
            derivative_of: None,
            aliases: Vec::new(),
        };

        for attr in &field.attrs {
            if attr.path().is_ident("variable") {
                parse_variable_attribute_content(attr, &mut var_info)
                    .map_err(|e| Diagnostic::spanned(attr.span(), Level::Error, e.to_string()))?;
            } else if attr.path().is_ident("alias") {
                let alias_info = parse_alias_attribute_content(attr)
                    .map_err(|e| Diagnostic::spanned(attr.span(), Level::Error, e.to_string()))?;
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

/// Convert snake_case to PascalCase for enum variants
pub fn to_pascal_case(s: &str) -> String {
    // Handle special characters for alias names like "der(h)" -> "DerH"
    let cleaned = s
        .replace("(", "_")
        .replace(")", "")
        .replace("-", "_")
        .replace(" ", "_")
        .replace(".", "_");

    cleaned.to_case(Case::Pascal)
}

/// Check if a type is f64
pub fn is_float64_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "f64";
        }
    }
    false
}

/// Check if a type is f32
pub fn is_float32_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "f32";
        }
    }
    false
}
