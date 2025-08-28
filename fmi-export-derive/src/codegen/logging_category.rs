use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote};

use crate::codegen::util;
use crate::model::Model;

/// Generate a LoggingCategory enum for the model
pub struct LoggingCategoryEnum<'a> {
    model: &'a Model,
}

impl<'a> LoggingCategoryEnum<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl ToTokens for LoggingCategoryEnum<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = &self.model.ident;
        let enum_name = format_ident!("{}LoggingCategory", struct_name);

        // Check if the model has logging categories defined
        let log_categories = match self.model.log_categories() {
            Some(categories) => categories.collect::<Vec<_>>(),
            None => {
                // If no categories are defined, generate a minimal enum with just Default
                tokens.extend(quote! {
                    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
                    pub enum #enum_name {
                        Default,
                    }

                    impl ::std::fmt::Display for #enum_name {
                        fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                            match self {
                                Self::Default => write!(f, "default"),
                            }
                        }
                    }

                    impl ::std::str::FromStr for #enum_name {
                        type Err = String;

                        fn from_str(s: &str) -> Result<Self, Self::Err> {
                            match s {
                                "default" => Ok(Self::Default),
                                _ => Err(format!("Unknown logging category: {}", s)),
                            }
                        }
                    }

                    impl Default for #enum_name {
                        fn default() -> Self {
                            Self::Default
                        }
                    }

                    impl ::fmi_export::fmi3::ModelLoggingCategory for #enum_name {
                        fn all_categories() -> impl Iterator<Item = Self> {
                            [Self::Default].iter().copied()
                        }
                    }
                });
                return;
            }
        };

        // Generate enum variants from logging categories
        let mut variants = Vec::new();
        let mut display_arms = Vec::new();
        let mut from_str_arms = Vec::new();
        let mut all_categories_items = Vec::new();

        for category in &log_categories {
            let variant_name = format_ident!("{}", util::to_pascal_case(&category.name));
            let category_str = &category.name;

            // Generate the variant with optional doc attribute
            let variant = if let Some(ref description) = category.descr {
                quote! {
                    #[doc = #description]
                    #variant_name
                }
            } else {
                quote! {
                    #variant_name
                }
            };

            variants.push(variant);

            display_arms.push(quote! {
                Self::#variant_name => write!(f, #category_str)
            });

            from_str_arms.push(quote! {
                #category_str => Ok(Self::#variant_name)
            });

            all_categories_items.push(quote! {
                Self::#variant_name
            });
        }

        // Use the first category as default if available
        let default_variant = if !variants.is_empty() {
            let first_variant = &log_categories[0];
            let first_variant_name = format_ident!("{}", util::to_pascal_case(&first_variant.name));
            quote! { Self::#first_variant_name }
        } else {
            quote! { Self::Default }
        };

        tokens.extend(quote! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
            pub enum #enum_name {
                #(#variants,)*
            }

            impl ::std::fmt::Display for #enum_name {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    match self {
                        #(#display_arms,)*
                    }
                }
            }

            impl ::std::str::FromStr for #enum_name {
                type Err = String;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    match s {
                        #(#from_str_arms,)*
                        _ => Err(format!("Unknown logging category: {}", s)),
                    }
                }
            }

            impl Default for #enum_name {
                fn default() -> Self {
                    #default_variant
                }
            }

            impl ::fmi_export::fmi3::ModelLoggingCategory for #enum_name {
                fn all_categories() -> impl Iterator<Item = Self> {
                    [#(#all_categories_items),*].iter().copied()
                }
            }
        });
    }
}
