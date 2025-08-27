//! Generate the GetSet trait implementation

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};
use syn::Ident;

use crate::codegen::util;
use crate::model::{FieldAttributeOuter, Model};
use crate::model_description::rust_type_to_variable_type;
use fmi::fmi3::schema;

pub struct GetSetImpl<'a> {
    struct_name: &'a Ident,
    model: &'a Model,
}

/// Configuration for a getter/setter method pair
struct MethodConfig {
    variable_type: schema::VariableType,
    rust_type: TokenStream2,
    get_method_name: &'static str,
    set_method_name: &'static str,
    return_type: TokenStream2,
}

impl MethodConfig {
    fn new(
        variable_type: schema::VariableType,
        rust_type: TokenStream2,
        get_method_name: &'static str,
        set_method_name: &'static str,
    ) -> Self {
        let return_type = if matches!(variable_type, schema::VariableType::FmiString) {
            quote! { Result<(), fmi::fmi3::Fmi3Error> }
        } else {
            quote! { Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> }
        };

        Self {
            variable_type,
            rust_type,
            get_method_name,
            set_method_name,
            return_type,
        }
    }
}

impl<'a> GetSetImpl<'a> {
    pub fn new(struct_name: &'a Ident, model: &'a Model) -> Self {
        Self { struct_name, model }
    }

    /// Generate a getter/setter method pair for a specific type
    fn generate_method_pair(&self, config: &MethodConfig) -> TokenStream2 {
        let get_method_name = format_ident!("{}", config.get_method_name);
        let set_method_name = format_ident!("{}", config.set_method_name);
        let rust_type = &config.rust_type;
        let return_type = &config.return_type;
        
        let getter_cases = TypeGetterGen::new(self.model, config.variable_type);
        let setter_cases = TypeSetterGen::new(self.model, config.variable_type);

        let success_value = if matches!(config.variable_type, schema::VariableType::FmiString) {
            quote! { Ok(()) }
        } else {
            quote! { Ok(fmi::fmi3::Fmi3Res::OK) }
        };

        quote! {
            fn #get_method_name(
                &mut self,
                vrs: &[Self::ValueRef],
                values: &mut [#rust_type],
            ) -> #return_type {
                for (vr, value) in vrs.iter().zip(values.iter_mut()) {
                    match ValueRef::from(*vr) {
                        #getter_cases
                        _ => {} // Ignore unknown VRs for robustness
                    }
                }
                #success_value
            }

            fn #set_method_name(
                &mut self,
                vrs: &[Self::ValueRef],
                values: &[#rust_type],
            ) -> #return_type {
                for (vr, value) in vrs.iter().zip(values.iter()) {
                    match ValueRef::from(*vr) {
                        #setter_cases
                        _ => {} // Ignore unknown VRs for robustness
                    }
                }
                #success_value
            }
        }
    }

    /// Generate all method configurations for supported types
    fn get_method_configs() -> Vec<MethodConfig> {
        vec![
            MethodConfig::new(
                schema::VariableType::FmiBoolean,
                quote! { bool },
                "get_boolean",
                "set_boolean",
            ),
            MethodConfig::new(
                schema::VariableType::FmiFloat32,
                quote! { f32 },
                "get_float32",
                "set_float32",
            ),
            MethodConfig::new(
                schema::VariableType::FmiFloat64,
                quote! { f64 },
                "get_float64",
                "set_float64",
            ),
            MethodConfig::new(
                schema::VariableType::FmiInt8,
                quote! { i8 },
                "get_int8",
                "set_int8",
            ),
            MethodConfig::new(
                schema::VariableType::FmiInt16,
                quote! { i16 },
                "get_int16",
                "set_int16",
            ),
            MethodConfig::new(
                schema::VariableType::FmiInt32,
                quote! { i32 },
                "get_int32",
                "set_int32",
            ),
            MethodConfig::new(
                schema::VariableType::FmiInt64,
                quote! { i64 },
                "get_int64",
                "set_int64",
            ),
            MethodConfig::new(
                schema::VariableType::FmiUInt8,
                quote! { u8 },
                "get_uint8",
                "set_uint8",
            ),
            MethodConfig::new(
                schema::VariableType::FmiUInt16,
                quote! { u16 },
                "get_uint16",
                "set_uint16",
            ),
            MethodConfig::new(
                schema::VariableType::FmiUInt32,
                quote! { u32 },
                "get_uint32",
                "set_uint32",
            ),
            MethodConfig::new(
                schema::VariableType::FmiUInt64,
                quote! { u64 },
                "get_uint64",
                "set_uint64",
            ),
            MethodConfig::new(
                schema::VariableType::FmiString,
                quote! { std::ffi::CString },
                "get_string",
                "set_string",
            ),
        ]
    }
}

impl ToTokens for GetSetImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = self.struct_name;

        // Generate all getter/setter method pairs
        let method_configs = Self::get_method_configs();
        let method_pairs: Vec<TokenStream2> = method_configs
            .iter()
            .map(|config| self.generate_method_pair(config))
            .collect();

        tokens.extend(quote! {
            impl ::fmi::fmi3::GetSet for #struct_name {
                type ValueRef = ::fmi::fmi3::binding::fmi3ValueReference;

                #(#method_pairs)*
            }
        });
    }
}

/// Generic getter generator for any variable type
struct TypeGetterGen<'a> {
    model: &'a Model,
    variable_type: schema::VariableType,
}

impl<'a> TypeGetterGen<'a> {
    fn new(model: &'a Model, variable_type: schema::VariableType) -> Self {
        Self {
            model,
            variable_type,
        }
    }
}

impl ToTokens for TypeGetterGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for field in &self.model.fields {
            if let Ok(vtype) = rust_type_to_variable_type(&field.ty) {
                if vtype == self.variable_type {
                    // Add case for main variable
                    let has_variable = field
                        .attrs
                        .iter()
                        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
                    if has_variable {
                        let variant_name =
                            format_ident!("{}", util::to_pascal_case(&field.ident.to_string()));
                        let field_name = &field.ident;

                        // Special handling for string types
                        if self.variable_type == schema::VariableType::FmiString {
                            tokens.extend(quote! {
                                ValueRef::#variant_name => {
                                    *value = std::ffi::CString::new(self.#field_name.clone()).unwrap_or_default();
                                },
                            });
                        } else {
                            tokens.extend(quote! {
                                ValueRef::#variant_name => *value = self.#field_name,
                            });
                        }
                    }

                    // Add cases for aliases of this variable
                    for attr in &field.attrs {
                        if let FieldAttributeOuter::Alias(alias_attr) = attr {
                            if let Some(alias_name) = &alias_attr.name {
                                let alias_variant_name = util::generate_variant_name(alias_name);
                                let field_name = &field.ident;

                                // Special handling for string types
                                if self.variable_type == schema::VariableType::FmiString {
                                    tokens.extend(quote! {
                                        ValueRef::#alias_variant_name => {
                                            let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self);
                                            *value = std::ffi::CString::new(self.#field_name.clone()).unwrap_or_default();
                                        },
                                    });
                                } else {
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
}

/// Generic setter generator for any variable type
struct TypeSetterGen<'a> {
    model: &'a Model,
    variable_type: schema::VariableType,
}

impl<'a> TypeSetterGen<'a> {
    fn new(model: &'a Model, variable_type: schema::VariableType) -> Self {
        Self {
            model,
            variable_type,
        }
    }
}

impl ToTokens for TypeSetterGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for field in &self.model.fields {
            if let Ok(vtype) = rust_type_to_variable_type(&field.ty) {
                if vtype == self.variable_type {
                    // Only generate setter for main variable (not aliases)
                    let has_variable = field
                        .attrs
                        .iter()
                        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
                    if has_variable {
                        let variant_name =
                            format_ident!("{}", util::to_pascal_case(&field.ident.to_string()));
                        let field_name = &field.ident;

                        // Special handling for string types
                        if self.variable_type == schema::VariableType::FmiString {
                            tokens.extend(quote! {
                                ValueRef::#variant_name => {
                                    self.#field_name = value.to_string_lossy().to_string();
                                },
                            });
                        } else {
                            tokens.extend(quote! {
                                ValueRef::#variant_name => self.#field_name = *value,
                            });
                        }
                    }
                    // Note: Aliases (especially derivatives) typically shouldn't be settable
                }
            }
        }
    }
}
