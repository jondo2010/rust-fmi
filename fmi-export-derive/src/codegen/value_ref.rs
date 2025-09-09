use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};

use crate::{Model, model::FieldAttributeOuter};
use fmi::fmi3::schema;

use super::util;

/// Generate the ValueRef enum
pub struct ValueRefEnum<'a> {
    model: &'a Model,
    model_variables: &'a schema::ModelVariables,
}

impl<'a> ValueRefEnum<'a> {
    pub fn new(model: &'a Model, model_variables: &'a schema::ModelVariables) -> Self {
        Self {
            model,
            model_variables,
        }
    }
}

impl ToTokens for ValueRefEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = &self.model.ident;
        let value_ref_enum_name = format_ident!("{}ValueRef", struct_name);
        
        let mut value_ref_variants = Vec::new();
        let mut from_u32_arms = Vec::new();
        let mut into_u32_arms = Vec::new();

        // Always add Time variant with VR 0 first
        value_ref_variants.push(quote! {
            Time = 0
        });
        from_u32_arms.push(quote! {
            0 => #value_ref_enum_name::Time
        });
        into_u32_arms.push(quote! {
            #value_ref_enum_name::Time => 0
        });

        // Collect all variables from the model description and create a mapping
        // from field name to value reference
        let mut field_to_vr = std::collections::HashMap::new();
        let mut alias_to_vr = std::collections::HashMap::new();

        // Build mapping from the model description (source of truth)
        for variable in self.model_variables.iter_abstract() {
            let var_name = variable.name();
            let vr = variable.value_reference();

            // Skip VR 0 as it's reserved for Time
            if vr == 0 {
                continue;
            }

            // Try to match this variable to a field or alias in the model
            for field in &self.model.fields {
                let field_name = field.ident.to_string();

                // Check if this is the main field variable
                if var_name == field_name {
                    let has_variable = field
                        .attrs
                        .iter()
                        .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
                    if has_variable {
                        field_to_vr.insert(field_name.clone(), vr);
                    }
                }

                // Check if this is an alias variable
                for attr in &field.attrs {
                    if let FieldAttributeOuter::Alias(alias_attr) = attr {
                        let alias_name = alias_attr.name.as_deref().unwrap_or(&field_name);
                        if var_name == alias_name {
                            alias_to_vr.insert(alias_name.to_string(), vr);
                        }
                    }
                }
            }
        }

        // Generate enum variants based on the model fields, using VRs from model description
        for field in &self.model.fields {
            let field_name = field.ident.to_string();

            // First, add the main field variable if it exists
            let has_variable = field
                .attrs
                .iter()
                .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)));
            if has_variable {
                if let Some(&vr) = field_to_vr.get(&field_name) {
                    let variant_name = format_ident!("{}", util::to_pascal_case(&field_name));

                    value_ref_variants.push(quote! {
                        #variant_name = #vr
                    });

                    from_u32_arms.push(quote! {
                        #vr => #value_ref_enum_name::#variant_name
                    });

                    into_u32_arms.push(quote! {
                        #value_ref_enum_name::#variant_name => #vr
                    });
                }
            }

            // Then add any alias variables with their custom names
            for attr in &field.attrs {
                if let FieldAttributeOuter::Alias(alias_attr) = attr {
                    let alias_name = alias_attr.name.as_deref().unwrap_or(&field_name);

                    if let Some(&vr) = alias_to_vr.get(alias_name) {
                        let variant_name = util::generate_variant_name(alias_name);

                        value_ref_variants.push(quote! {
                            #variant_name = #vr
                        });

                        from_u32_arms.push(quote! {
                            #vr => #value_ref_enum_name::#variant_name
                        });

                        into_u32_arms.push(quote! {
                            #value_ref_enum_name::#variant_name => #vr
                        });
                    }
                }
            }
        }

        tokens.extend(quote! {
            #[repr(u32)]
            #[derive(Clone, Copy, Debug, PartialEq, Eq)]
            enum #value_ref_enum_name {
                #(#value_ref_variants,)*
            }

            impl From<fmi::fmi3::binding::fmi3ValueReference> for #value_ref_enum_name {
                fn from(value: fmi::fmi3::binding::fmi3ValueReference) -> Self {
                    match value {
                        #(#from_u32_arms,)*
                        _ => panic!("Invalid value reference: {}", value),
                    }
                }
            }

            impl From<#value_ref_enum_name> for fmi::fmi3::binding::fmi3ValueReference {
                fn from(value: #value_ref_enum_name) -> Self {
                    match value {
                        #(#into_u32_arms,)*
                    }
                }
            }
        });
    }
}
