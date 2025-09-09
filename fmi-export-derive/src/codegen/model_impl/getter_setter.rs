//! Getter and setter method generation for the Model trait

use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};

use crate::codegen::util;
use crate::model::{FieldAttributeOuter, Model};
use fmi::fmi3::schema;

/// Generator for all getter/setter method implementations in the Model trait
pub struct GetterSetterGen<'a> {
    model: &'a Model,
}

impl<'a> GetterSetterGen<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }

    /// Check if the model has any variables of the given type
    fn has_variables_of_type(&self, variable_type: schema::VariableType) -> bool {
        self.model.fields.iter().any(|field| {
            if field.field_type.r#type == variable_type {
                // Check if field has a variable or alias attribute
                field.attrs.iter().any(|attr| {
                    matches!(
                        attr,
                        FieldAttributeOuter::Variable(_) | FieldAttributeOuter::Alias(_)
                    )
                })
            } else {
                false
            }
        })
    }
}

impl ToTokens for GetterSetterGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = &self.model.ident;
        let value_ref_enum_name = format_ident!("{}ValueRef", struct_name);

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
            // Always generate Float64 methods since Time VR is always available
            // For other types, only generate if the model actually has variables of this type
            let should_generate = *variable_type == schema::VariableType::FmiFloat64
                || self.has_variables_of_type(*variable_type);

            if !should_generate {
                continue;
            }

            let get_method_name = format_ident!("{}", get_method);
            let set_method_name = format_ident!("{}", set_method);
            let rust_type: syn::Type = syn::parse_str(rust_type)
                .expect("Failed to parse rust type - this is a bug in the code generator");

            let getter_cases = TypeGetterGen::new(self.model, *variable_type);
            let setter_cases = TypeSetterGen::new(self.model, *variable_type);

            tokens.extend(quote! {
                fn #get_method_name(
                    &mut self,
                    vrs: &[Self::ValueRef],
                    values: &mut [#rust_type],
                    context: &::fmi_export::fmi3::ModelContext<Self>,
                ) -> Result<fmi::fmi3::Fmi3Res, fmi::fmi3::Fmi3Error> {
                    let mut value_index = 0;
                    for vr in vrs.iter() {
                        match #value_ref_enum_name::from(*vr) {
                            #getter_cases
                            _ => {
                                context.log(
                                    fmi::fmi3::Fmi3Error::Error,
                                    <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                    format_args!("Unknown value reference {} in getter", vr)
                                );
                                return Err(fmi::fmi3::Fmi3Error::Error);
                            }
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
                    let mut value_index = 0;
                    for vr in vrs.iter() {
                        match #value_ref_enum_name::from(*vr) {
                            #setter_cases
                            _ => {
                                context.log(
                                    fmi::fmi3::Fmi3Error::Error,
                                    <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                    format_args!("Unknown value reference {} in setter", vr)
                                );
                                return Err(fmi::fmi3::Fmi3Error::Error);
                            }
                        }
                    }
                    Ok(fmi::fmi3::Fmi3Res::OK)
                }
            });
        }

        // Generate special string methods with different signatures
        if self.has_variables_of_type(schema::VariableType::FmiString) {
            let string_getter_cases =
                TypeGetterGen::new(self.model, schema::VariableType::FmiString);
            let string_setter_cases =
                TypeSetterGen::new(self.model, schema::VariableType::FmiString);

            tokens.extend(quote! {
                fn get_string(
                    &mut self,
                    vrs: &[Self::ValueRef],
                    values: &mut [std::ffi::CString],
                    context: &::fmi_export::fmi3::ModelContext<Self>,
                ) -> Result<(), fmi::fmi3::Fmi3Error> {
                    let mut value_index = 0;
                    for vr in vrs.iter() {
                        match #value_ref_enum_name::from(*vr) {
                            #string_getter_cases
                            _ => {
                                context.log(
                                    fmi::fmi3::Fmi3Error::Error,
                                    <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                    format_args!("Unknown value reference {} in string getter", vr)
                                );
                                return Err(fmi::fmi3::Fmi3Error::Error);
                            }
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
                    let mut value_index = 0;
                    for vr in vrs.iter() {
                        match #value_ref_enum_name::from(*vr) {
                            #string_setter_cases
                            _ => {
                                context.log(
                                    fmi::fmi3::Fmi3Error::Error,
                                    <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                    format_args!("Unknown value reference {} in string setter", vr)
                                );
                                return Err(fmi::fmi3::Fmi3Error::Error);
                            }
                        }
                    }
                    Ok(())
                }
            });
        }

        // Generate binary methods (placeholder implementations)
        if self.has_variables_of_type(schema::VariableType::FmiBinary) {
            tokens.extend(quote! {
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
        let struct_name = &self.model.ident;
        let value_ref_enum_name = format_ident!("{}ValueRef", struct_name);

        // Special handling for Float64 to include Time VR
        if self.variable_type == schema::VariableType::FmiFloat64 {
            tokens.extend(quote! {
                #value_ref_enum_name::Time => {
                    if value_index < values.len() {
                        values[value_index] = context.time();
                        value_index += 1;
                    } else {
                        context.log(
                            fmi::fmi3::Fmi3Error::Error,
                            <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                            format_args!("Value array index {} out of bounds for Time variable", value_index)
                        );
                        return Err(fmi::fmi3::Fmi3Error::Error);
                    }
                },
            });
        }

        for field in &self.model.fields {
            if field.field_type.r#type == self.variable_type {
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
                                #value_ref_enum_name::#variant_name => {
                                    if value_index < values.len() {
                                        match std::ffi::CString::new(self.#field_name.clone()) {
                                            Ok(cstring) => values[value_index] = cstring,
                                            Err(_) => {
                                                context.log(
                                                    fmi::fmi3::Fmi3Error::Error,
                                                    <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                                    format_args!("String variable {} contains null bytes and cannot be converted to CString", stringify!(#field_name))
                                                );
                                                return Err(fmi::fmi3::Fmi3Error::Error);
                                            }
                                        }
                                        value_index += 1;
                                    } else {
                                        context.log(
                                            fmi::fmi3::Fmi3Error::Error,
                                            <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                            format_args!("Value array index {} out of bounds for string variable {}", value_index, stringify!(#field_name))
                                        );
                                        return Err(fmi::fmi3::Fmi3Error::Error);
                                    }
                                },
                            });
                    } else {
                        // Handle both scalar and array types
                        if field.field_type.dimensions.is_empty() {
                            // Scalar field
                            tokens.extend(quote! {
                                #value_ref_enum_name::#variant_name => {
                                    if value_index < values.len() {
                                        values[value_index] = self.#field_name;
                                        value_index += 1;
                                    } else {
                                        context.log(
                                            fmi::fmi3::Fmi3Error::Error,
                                            <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                            format_args!("Value array index {} out of bounds for variable {}", value_index, stringify!(#field_name))
                                        );
                                        return Err(fmi::fmi3::Fmi3Error::Error);
                                    }
                                },
                            });
                        } else {
                            // Array field - copy all elements
                            tokens.extend(quote! {
                                #value_ref_enum_name::#variant_name => {
                                    let array_len = self.#field_name.len();
                                    if value_index + array_len <= values.len() {
                                        values[value_index..value_index + array_len].copy_from_slice(&self.#field_name);
                                        value_index += array_len;
                                    } else {
                                        context.log(
                                            fmi::fmi3::Fmi3Error::Error,
                                            <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                            format_args!("Value array too small for array variable {}: need {} elements, have {} available",
                                                       stringify!(#field_name), array_len, values.len() - value_index)
                                        );
                                        return Err(fmi::fmi3::Fmi3Error::Error);
                                    }
                                },
                            });
                        }
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
                                        #value_ref_enum_name::#alias_variant_name => {
                                            if value_index < values.len() {
                                                match std::ffi::CString::new(self.#field_name.clone()) {
                                                    Ok(cstring) => values[value_index] = cstring,
                                                    Err(_) => {
                                                        context.log(
                                                            fmi::fmi3::Fmi3Error::Error,
                                                            <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                                            format_args!("String variable {} (alias {}) contains null bytes and cannot be converted to CString", stringify!(#field_name), stringify!(#alias_variant_name))
                                                        );
                                                        return Err(fmi::fmi3::Fmi3Error::Error);
                                                    }
                                                }
                                                value_index += 1;
                                            } else {
                                                context.log(
                                                    fmi::fmi3::Fmi3Error::Error,
                                                    <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                                    format_args!("Value array index {} out of bounds for string alias {} of variable {}", value_index, stringify!(#alias_variant_name), stringify!(#field_name))
                                                );
                                                return Err(fmi::fmi3::Fmi3Error::Error);
                                            }
                                        },
                                    });
                            } else {
                                // Handle both scalar and array types
                                if field.field_type.dimensions.is_empty() {
                                    // Scalar field
                                    tokens.extend(quote! {
                                        #value_ref_enum_name::#alias_variant_name => {
                                            if value_index < values.len() {
                                                values[value_index] = self.#field_name;
                                                value_index += 1;
                                            } else {
                                                context.log(
                                                    fmi::fmi3::Fmi3Error::Error,
                                                    <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                                    format_args!("Value array index {} out of bounds for alias {} of variable {}", value_index, stringify!(#alias_variant_name), stringify!(#field_name))
                                                );
                                                return Err(fmi::fmi3::Fmi3Error::Error);
                                            }
                                        },
                                    });
                                } else {
                                    // Array field - copy all elements
                                    tokens.extend(quote! {
                                        #value_ref_enum_name::#alias_variant_name => {
                                            let array_len = self.#field_name.len();
                                            if value_index + array_len <= values.len() {
                                                values[value_index..value_index + array_len].copy_from_slice(&self.#field_name);
                                                value_index += array_len;
                                            } else {
                                                context.log(
                                                    fmi::fmi3::Fmi3Error::Error,
                                                    <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                                    format_args!("Value array too small for array alias {} of variable {}: need {} elements, have {} available",
                                                               stringify!(#alias_variant_name), stringify!(#field_name), array_len, values.len() - value_index)
                                                );
                                                return Err(fmi::fmi3::Fmi3Error::Error);
                                            }
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
        let struct_name = &self.model.ident;
        let value_ref_enum_name = format_ident!("{}ValueRef", struct_name);

        // Special handling for Float64: Time VR should not be settable
        if self.variable_type == schema::VariableType::FmiFloat64 {
            tokens.extend(quote! {
                #value_ref_enum_name::Time => {
                    // Time is read-only, cannot be set through fmi3SetFloat64
                    // Time is set through fmi3SetTime function instead
                    return Err(fmi::fmi3::Fmi3Error::Error);
                },
            });
        }

        for field in &self.model.fields {
            if field.field_type.r#type == self.variable_type {
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
                            #value_ref_enum_name::#variant_name => {
                                if value_index < values.len() {
                                    self.#field_name = values[value_index].to_string_lossy().to_string();
                                    value_index += 1;
                                } else {
                                    context.log(
                                        fmi::fmi3::Fmi3Error::Error,
                                        <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                        format_args!("Value array index {} out of bounds for string variable {}", value_index, stringify!(#field_name))
                                    );
                                    return Err(fmi::fmi3::Fmi3Error::Error);
                                }
                            },
                        });
                    } else {
                        // Handle both scalar and array types
                        if field.field_type.dimensions.is_empty() {
                            // Scalar field
                            tokens.extend(quote! {
                                #value_ref_enum_name::#variant_name => {
                                    if value_index < values.len() {
                                        self.#field_name = values[value_index];
                                        value_index += 1;
                                    } else {
                                        context.log(
                                            fmi::fmi3::Fmi3Error::Error,
                                            <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                            format_args!("Value array index {} out of bounds for variable {}", value_index, stringify!(#field_name))
                                        );
                                        return Err(fmi::fmi3::Fmi3Error::Error);
                                    }
                                },
                            });
                        } else {
                            // Array field - copy all elements
                            tokens.extend(quote! {
                                #value_ref_enum_name::#variant_name => {
                                    let array_len = self.#field_name.len();
                                    if value_index + array_len <= values.len() {
                                        self.#field_name.copy_from_slice(&values[value_index..value_index + array_len]);
                                        value_index += array_len;
                                    } else {
                                        context.log(
                                            fmi::fmi3::Fmi3Error::Error,
                                            <Self::LoggingCategory as ::fmi_export::fmi3::ModelLoggingCategory>::error_category(),
                                            format_args!("Value array too small for array variable {}: need {} elements, have {} available",
                                                       stringify!(#field_name), array_len, values.len() - value_index)
                                        );
                                        return Err(fmi::fmi3::Fmi3Error::Error);
                                    }
                                },
                            });
                        }
                    }
                }
                // Note: Aliases (especially derivatives) typically shouldn't be settable
            }
        }
    }
}
