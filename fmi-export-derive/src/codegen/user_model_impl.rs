use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

use crate::model::Model;

pub struct UserModelImpl<'a> {
    struct_name: &'a syn::Ident,
    model: &'a Model,
}

impl<'a> UserModelImpl<'a> {
    pub fn new(struct_name: &'a syn::Ident, model: &'a Model) -> Self {
        Self { struct_name, model }
    }
}

impl ToTokens for UserModelImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        if !self.model.auto_user_model() {
            return;
        }

        let struct_name = self.struct_name;
        tokens.extend(quote! {
            impl ::fmi_export::fmi3::UserModel for #struct_name {
                type LoggingCategory = ::fmi_export::fmi3::DefaultLoggingCategory;
            }
        });
    }
}
