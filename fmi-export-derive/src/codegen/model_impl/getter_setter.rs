//! Getter and setter method generation for the Model trait

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};

use crate::codegen::util;
use crate::model::{FieldAttributeOuter, Model};
use crate::util::rust_type_to_variable_type;
use fmi::fmi3::schema;

/// Generator for all getter/setter method implementations in the Model trait
pub struct GetterSetterGen<'a> {
    model: &'a Model,
}

impl<'a> GetterSetterGen<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl ToTokens for GetterSetterGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        // Configuration for all supported types
        let method_configs = [
            (
                "bool",
                "get_boolean",
                "set_boolean",
                schema::VariableType::FmiBoolean,
            ),
            (
                "f32",
                "get_float32",
                "set_float32",
                schema::VariableType::FmiFloat32,
            ),
            (
                "f64",
                "get_float64",
                "set_float64",
                schema::VariableType::FmiFloat64,
            ),
            ("i8", "get_int8", "set_int8", schema::VariableType::FmiInt8),
            (
                "i16",
                "get_int16",
                "set_int16",
                schema::VariableType::FmiInt16,
            ),
            (
                "i32",
                "get_int32",
                "set_int32",
                schema::VariableType::FmiInt32,
            ),
            (
                "i64",
                "get_int64",
                "set_int64",
                schema::VariableType::FmiInt64,
            ),
            (
                "u8",
                "get_uint8",
                "set_uint8",
                schema::VariableType::FmiUInt8,
            ),
            (
                "u16",
                "get_uint16",
                "set_uint16",
                schema::VariableType::FmiUInt16,
            ),
            (
                "u32",
                "get_uint32",
                "set_uint32",
                schema::VariableType::FmiUInt32,
            ),
            (
                "u64",
                "get_uint64",
                "set_uint64",
                schema::VariableType::FmiUInt64,
            ),
        ];

        // Generate standard getter/setter methods
        for (rust_type, get_method, set_method, variable_type) in &method_configs {
            let get_method_name = format_ident!("{}", get_method);
            let set_method_name = format_ident!("{}", set_method);
            let rust_type: syn::Type = syn::parse_str(rust_type).unwrap();

            let getter_cases = TypeGetterGen::new(self.model, *variable_type);
            let setter_cases = TypeSetterGen::new(self.model, *variable_type);

            tokens.extend(quote! {
                fn #get_method_name(
                    &mut self,
                    vrs: &[Self::ValueRef],
                    values: &mut [#rust_type],
                    context: &::fmi_export::fmi3::ModelContext<Self>,
                ) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
                    for (vr, value) in vrs.iter().zip(values.iter_mut()) {
                        match ValueRef::from(*vr) {
                            #getter_cases
                            _ => {} // Ignore unknown VRs for robustness
                        }
                    }
                    Ok(fmi::fmi3::Fmi3Res::OK)
                }

                fn #set_method_name(
                    &mut self,
                    vrs: &[Self::ValueRef],
                    values: &[#rust_type],
                    context: &::fmi_export::fmi3::ModelContext<Self>,
                ) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
                    for (vr, value) in vrs.iter().zip(values.iter()) {
                        match ValueRef::from(*vr) {
                            #setter_cases
                            _ => {} // Ignore unknown VRs for robustness
                        }
                    }
                    Ok(fmi::fmi3::Fmi3Res::OK)
                }
            });
        }

        // Generate special string methods with different signatures
        let string_getter_cases = TypeGetterGen::new(self.model, schema::VariableType::FmiString);
        let string_setter_cases = TypeSetterGen::new(self.model, schema::VariableType::FmiString);

        tokens.extend(quote! {
            fn get_string(
                &mut self,
                vrs: &[Self::ValueRef],
                values: &mut [std::ffi::CString],
                context: &::fmi_export::fmi3::ModelContext<Self>,
            ) -> Result<(), fmi::fmi3::Fmi3Error> {
                for (vr, value) in vrs.iter().zip(values.iter_mut()) {
                    match ValueRef::from(*vr) {
                        #string_getter_cases
                        _ => {} // Ignore unknown VRs for robustness
                    }
                }
                Ok(())
            }

            fn set_string(
                &mut self,
                vrs: &[Self::ValueRef],
                values: &[std::ffi::CString],
                context: &::fmi_export::fmi3::ModelContext<Self>,
            ) -> Result<(), fmi::fmi3::Fmi3Error> {
                for (vr, value) in vrs.iter().zip(values.iter()) {
                    match ValueRef::from(*vr) {
                        #string_setter_cases
                        _ => {} // Ignore unknown VRs for robustness
                    }
                }
                Ok(())
            }

            fn get_binary(
                &mut self,
                vrs: &[Self::ValueRef],
                values: &mut [&mut [u8]],
                context: &::fmi_export::fmi3::ModelContext<Self>,
            ) -> Result<Vec<usize>, fmi::fmi3::Fmi3Error> {
                // Binary not implemented for now - return empty sizes
                Ok(vec![0; vrs.len()])
            }

            fn set_binary(
                &mut self,
                vrs: &[Self::ValueRef],
                values: &[&[u8]],
                context: &::fmi_export::fmi3::ModelContext<Self>,
            ) -> Result<(), fmi::fmi3::Fmi3Error> {
                // Binary not implemented for now
                Ok(())
            }
        });
    }
}

/// Generic getter generator for any variable type
pub struct TypeGetterGen<'a> {
    model: &'a Model,
    variable_type: schema::VariableType,
}

impl<'a> TypeGetterGen<'a> {
    pub fn new(model: &'a Model, variable_type: schema::VariableType) -> Self {
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
                                            let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self, context);
                                            *value = std::ffi::CString::new(self.#field_name.clone()).unwrap_or_default();
                                        },
                                    });
                                } else {
                                    tokens.extend(quote! {
                                        ValueRef::#alias_variant_name => {
                                            let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self, context);
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
pub struct TypeSetterGen<'a> {
    model: &'a Model,
    variable_type: schema::VariableType,
}

impl<'a> TypeSetterGen<'a> {
    pub fn new(model: &'a Model, variable_type: schema::VariableType) -> Self {
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
