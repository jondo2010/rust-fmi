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

impl<'a> GetSetImpl<'a> {
    pub fn new(struct_name: &'a Ident, model: &'a Model) -> Self {
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

struct Float64GetterGen<'a>(&'a Model);

impl<'a> Float64GetterGen<'a> {
    fn new(model: &'a Model) -> Self {
        Self(model)
    }
}

impl ToTokens for Float64GetterGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        for field in &self.0.fields {
            if let Ok(vtype) = rust_type_to_variable_type(&field.ty) {
                if vtype == schema::VariableType::FmiFloat64 {
                    // Add case for main variable
                    let has_variable = field
                        .attrs
                        .iter()
                        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
                    if has_variable {
                        let variant_name =
                            format_ident!("{}", util::to_pascal_case(&field.ident.to_string()));
                        let field_name = &field.ident;

                        tokens.extend(quote! {
                            ValueRef::#variant_name => *value = self.#field_name,
                        });
                    }

                    // Add cases for aliases of this variable
                    for attr in &field.attrs {
                        if let FieldAttributeOuter::Alias(alias_attr) = attr {
                            if let Some(alias_name) = &alias_attr.name {
                                let alias_variant_name = util::generate_variant_name(alias_name);
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
                if vtype == schema::VariableType::FmiFloat64 {
                    // Only generate setter for main variable (not aliases)
                    let has_variable = field
                        .attrs
                        .iter()
                        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
                    if has_variable {
                        let variant_name =
                            format_ident!("{}", util::to_pascal_case(&field.ident.to_string()));
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
                if vtype == schema::VariableType::FmiFloat32 {
                    // Add case for main variable
                    let has_variable = field
                        .attrs
                        .iter()
                        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
                    if has_variable {
                        let variant_name =
                            format_ident!("{}", util::to_pascal_case(&field.ident.to_string()));
                        let field_name = &field.ident;

                        tokens.extend(quote! {
                            ValueRef::#variant_name => *value = self.#field_name,
                        });
                    }

                    // Add cases for aliases of this variable
                    for attr in &field.attrs {
                        if let FieldAttributeOuter::Alias(alias_attr) = attr {
                            if let Some(alias_name) = &alias_attr.name {
                                let alias_variant_name = util::generate_variant_name(alias_name);
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
                if vtype == schema::VariableType::FmiFloat32 {
                    // Only generate setter for main variable (not aliases)
                    let has_variable = field
                        .attrs
                        .iter()
                        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
                    if has_variable {
                        let variant_name =
                            format_ident!("{}", util::to_pascal_case(&field.ident.to_string()));
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
