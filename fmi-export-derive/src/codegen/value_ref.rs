use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};

use crate::{Model, model::FieldAttributeOuter};

use super::util;

/// Generate the ValueRef enum
pub struct ValueRefEnum<'a> {
    model: &'a Model,
}

impl<'a> ValueRefEnum<'a> {
    pub fn new(model: &'a Model) -> Self {
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
                let variant_name =
                    format_ident!("{}", util::to_pascal_case(&field.ident.to_string()));

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

                    let variant_name = util::generate_variant_name(alias_name);

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
