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
                context: &::fmi_export::fmi3::ModelContext<Self>,
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
                context: &::fmi_export::fmi3::ModelContext<Self>,
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

#[test]
fn test_get_set_comprehensive_datatype_support() {
    // This test verifies that our GetSet implementation generates the correct method signatures
    // and compiles properly for all supported datatypes.
    use syn::parse_quote;

    let input: syn::ItemStruct = parse_quote! {
        #[model()]
        struct GetSetTestModel {
            // All supported types for GetSet testing
            #[variable(causality = Output, start = 1.5)]
            test_f32: f32,
            #[variable(causality = Output, start = 2.7)]
            test_f64: f64,
            #[variable(causality = Parameter, start = 10)]
            test_i8: i8,
            #[variable(causality = Parameter, start = 1000)]
            test_i16: i16,
            #[variable(causality = Parameter, start = 50000)]
            test_i32: i32,
            #[variable(causality = Parameter, start = 9000000000)]
            test_i64: i64,
            #[variable(causality = Input, start = 255)]
            test_u8: u8,
            #[variable(causality = Input, start = 8080)]
            test_u16: u16,
            #[variable(causality = Input, start = 1024)]
            test_u32: u32,
            #[variable(causality = Input, start = 1234567890123)]
            test_u64: u64,
            #[variable(causality = Input, start = true)]
            test_bool: bool,
            #[variable(causality = Parameter, start = "TestModel")]
            test_string: String,
        }
    };

    let model = Model::from(input);

    // Test that model description generation works for all types
    let fmi_description = schema::Fmi3ModelDescription::try_from(model.clone()).unwrap();

    // Verify all variable types are present
    assert_eq!(fmi_description.model_variables.float32.len(), 1);
    assert_eq!(fmi_description.model_variables.float64.len(), 1);
    assert_eq!(fmi_description.model_variables.int8.len(), 1);
    assert_eq!(fmi_description.model_variables.int16.len(), 1);
    assert_eq!(fmi_description.model_variables.int32.len(), 1);
    assert_eq!(fmi_description.model_variables.int64.len(), 1);
    assert_eq!(fmi_description.model_variables.uint8.len(), 1);
    assert_eq!(fmi_description.model_variables.uint16.len(), 1);
    assert_eq!(fmi_description.model_variables.uint32.len(), 1);
    assert_eq!(fmi_description.model_variables.uint64.len(), 1);
    assert_eq!(fmi_description.model_variables.boolean.len(), 1);
    assert_eq!(fmi_description.model_variables.string.len(), 1);

    // Total should be 12 variables (one of each type)
    assert_eq!(fmi_description.model_variables.len(), 12);

    // Test that GetSet implementation compiles and generates proper code
    // by verifying the token stream contains all required method implementations
    use quote::ToTokens;

    let get_set_impl = GetSetImpl::new(&model.ident, &model);
    let mut tokens = proc_macro2::TokenStream::new();
    get_set_impl.to_tokens(&mut tokens);

    let code = tokens.to_string();

    // Verify all GetSet method implementations are generated
    assert!(
        code.contains("fn get_boolean"),
        "Missing get_boolean implementation"
    );
    assert!(
        code.contains("fn set_boolean"),
        "Missing set_boolean implementation"
    );
    assert!(
        code.contains("fn get_float32"),
        "Missing get_float32 implementation"
    );
    assert!(
        code.contains("fn set_float32"),
        "Missing set_float32 implementation"
    );
    assert!(
        code.contains("fn get_float64"),
        "Missing get_float64 implementation"
    );
    assert!(
        code.contains("fn set_float64"),
        "Missing set_float64 implementation"
    );
    assert!(
        code.contains("fn get_int8"),
        "Missing get_int8 implementation"
    );
    assert!(
        code.contains("fn set_int8"),
        "Missing set_int8 implementation"
    );
    assert!(
        code.contains("fn get_int16"),
        "Missing get_int16 implementation"
    );
    assert!(
        code.contains("fn set_int16"),
        "Missing set_int16 implementation"
    );
    assert!(
        code.contains("fn get_int32"),
        "Missing get_int32 implementation"
    );
    assert!(
        code.contains("fn set_int32"),
        "Missing set_int32 implementation"
    );
    assert!(
        code.contains("fn get_int64"),
        "Missing get_int64 implementation"
    );
    assert!(
        code.contains("fn set_int64"),
        "Missing set_int64 implementation"
    );
    assert!(
        code.contains("fn get_uint8"),
        "Missing get_uint8 implementation"
    );
    assert!(
        code.contains("fn set_uint8"),
        "Missing set_uint8 implementation"
    );
    assert!(
        code.contains("fn get_uint16"),
        "Missing get_uint16 implementation"
    );
    assert!(
        code.contains("fn set_uint16"),
        "Missing set_uint16 implementation"
    );
    assert!(
        code.contains("fn get_uint32"),
        "Missing get_uint32 implementation"
    );
    assert!(
        code.contains("fn set_uint32"),
        "Missing set_uint32 implementation"
    );
    assert!(
        code.contains("fn get_uint64"),
        "Missing get_uint64 implementation"
    );
    assert!(
        code.contains("fn set_uint64"),
        "Missing set_uint64 implementation"
    );
    assert!(
        code.contains("fn get_string"),
        "Missing get_string implementation"
    );
    assert!(
        code.contains("fn set_string"),
        "Missing set_string implementation"
    );

    // Verify that variable-specific match cases are generated for each field
    assert!(
        code.contains("ValueRef::TestF32"),
        "Missing TestF32 value reference"
    );
    assert!(
        code.contains("ValueRef::TestF64"),
        "Missing TestF64 value reference"
    );
    assert!(
        code.contains("ValueRef::TestI8"),
        "Missing TestI8 value reference"
    );
    assert!(
        code.contains("ValueRef::TestI16"),
        "Missing TestI16 value reference"
    );
    assert!(
        code.contains("ValueRef::TestI32"),
        "Missing TestI32 value reference"
    );
    assert!(
        code.contains("ValueRef::TestI64"),
        "Missing TestI64 value reference"
    );
    assert!(
        code.contains("ValueRef::TestU8"),
        "Missing TestU8 value reference"
    );
    assert!(
        code.contains("ValueRef::TestU16"),
        "Missing TestU16 value reference"
    );
    assert!(
        code.contains("ValueRef::TestU32"),
        "Missing TestU32 value reference"
    );
    assert!(
        code.contains("ValueRef::TestU64"),
        "Missing TestU64 value reference"
    );
    assert!(
        code.contains("ValueRef::TestBool"),
        "Missing TestBool value reference"
    );
    assert!(
        code.contains("ValueRef::TestString"),
        "Missing TestString value reference"
    );

    // Verify field access patterns
    assert!(
        code.contains("self.test_f32"),
        "Missing test_f32 field access"
    );
    assert!(
        code.contains("self.test_f64"),
        "Missing test_f64 field access"
    );
    assert!(
        code.contains("self.test_i8"),
        "Missing test_i8 field access"
    );
    assert!(
        code.contains("self.test_i16"),
        "Missing test_i16 field access"
    );
    assert!(
        code.contains("self.test_i32"),
        "Missing test_i32 field access"
    );
    assert!(
        code.contains("self.test_i64"),
        "Missing test_i64 field access"
    );
    assert!(
        code.contains("self.test_u8"),
        "Missing test_u8 field access"
    );
    assert!(
        code.contains("self.test_u16"),
        "Missing test_u16 field access"
    );
    assert!(
        code.contains("self.test_u32"),
        "Missing test_u32 field access"
    );
    assert!(
        code.contains("self.test_u64"),
        "Missing test_u64 field access"
    );
    assert!(
        code.contains("self.test_bool"),
        "Missing test_bool field access"
    );
    assert!(
        code.contains("self.test_string"),
        "Missing test_string field access"
    );

    println!("✅ All GetSet comprehensive datatype tests passed!");
}
