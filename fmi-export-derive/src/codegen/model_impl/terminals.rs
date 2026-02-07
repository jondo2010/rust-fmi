use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};

use crate::model::{Field, FieldAttributeOuter, Model};

pub struct BuildTerminalsGen<'a> {
    model: &'a Model,
}

impl<'a> BuildTerminalsGen<'a> {
    pub fn new(model: &'a Model) -> Self {
        Self { model }
    }
}

impl ToTokens for BuildTerminalsGen<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = &self.model.ident;
        let terminal_name = self.model.ident.to_string();
        let terminal_name_lit = syn::LitStr::new(&terminal_name, self.model.ident.span());
        let prefix_binding = quote! { prefix.unwrap_or("") };

        if self.model.get_terminal_attr().is_some() {
            tokens.extend(quote! {
                let terminal = <#struct_name as ::fmi_export::fmi3::TerminalProvider>::terminal(#terminal_name_lit, prefix);
                terminals.push(terminal);
            });
            return;
        }

        let mut child_tokens = Vec::new();
        for field in &self.model.fields {
            if self.is_child_field(field) {
                let field_type = &field.rust_type;
                let field_name = field.ident.to_string();
                let child_prefix_lit = self.child_prefix_literal(field, &field_name);
                if let Some(terminal_attr) = self.terminal_attr(field) {
                    let child_terminal_name = terminal_attr
                        .name
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| field_name.clone());
                    let child_terminal_name_lit =
                        syn::LitStr::new(&child_terminal_name, field.ident.span());
                    child_tokens.push(quote! {
                        let child_prefix = format!("{}{}.", #prefix_binding, #child_prefix_lit);
                        let child_terminal = if child_prefix.is_empty() {
                            <#field_type as ::fmi_export::fmi3::TerminalProvider>::terminal(#child_terminal_name_lit, None)
                        } else {
                            <#field_type as ::fmi_export::fmi3::TerminalProvider>::terminal(#child_terminal_name_lit, Some(child_prefix.as_str()))
                        };
                        terminals.push(child_terminal);
                    });
                } else {
                    child_tokens.push(quote! {
                        let child_prefix = format!("{}{}.", #prefix_binding, #child_prefix_lit);
                        <#field_type as ::fmi_export::fmi3::Model>::build_terminals(
                            terminals,
                            Some(child_prefix.as_str()),
                        );
                    });
                }
            } else if self.has_no_variable_attributes(field) {
                let field_type = &field.rust_type;
                child_tokens.push(quote! {
                    <#field_type as ::fmi_export::fmi3::Model>::build_terminals(
                        terminals,
                        Some(#prefix_binding),
                    );
                });
            }
        }

        tokens.extend(quote! {
            #(#child_tokens)*
        });
    }
}

impl BuildTerminalsGen<'_> {
    fn has_no_variable_attributes(&self, field: &Field) -> bool {
        !field
            .attrs
            .iter()
            .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)))
    }

    fn is_child_field(&self, field: &Field) -> bool {
        field
            .attrs
            .iter()
            .any(|attr| matches!(attr, FieldAttributeOuter::Child(_)))
    }

    fn child_prefix_literal(&self, field: &Field, default_name: &str) -> syn::LitStr {
        let prefix = field.attrs.iter().find_map(|attr| {
            if let FieldAttributeOuter::Child(child_attr) = attr {
                child_attr.prefix.as_deref()
            } else {
                None
            }
        });
        let prefix = prefix.unwrap_or(default_name);
        syn::LitStr::new(prefix, field.ident.span())
    }

    fn terminal_attr<'a>(
        &self,
        field: &'a Field,
    ) -> Option<&'a crate::model::TerminalAttribute> {
        field.attrs.iter().find_map(|attr| {
            if let FieldAttributeOuter::Terminal(terminal_attr) = attr {
                Some(terminal_attr)
            } else {
                None
            }
        })
    }
}

pub struct TerminalProviderImpl<'a> {
    struct_name: &'a syn::Ident,
    model: &'a Model,
}

impl<'a> TerminalProviderImpl<'a> {
    pub fn new(struct_name: &'a syn::Ident, model: &'a Model) -> Self {
        Self { struct_name, model }
    }
}

impl ToTokens for TerminalProviderImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let struct_name = self.struct_name;
        let prefix_binding = quote! { prefix.unwrap_or("") };

        let terminal_attr = self.model.get_terminal_attr();
        let include_members = terminal_attr.is_some();
        let struct_name_lit =
            syn::LitStr::new(&self.model.ident.to_string(), self.model.ident.span());
        let terminal_name_override = terminal_attr
            .and_then(|attr| attr.name.as_deref())
            .map(|name| syn::LitStr::new(name, self.model.ident.span()));
        let matching_rule_override = terminal_attr
            .and_then(|attr| attr.matching_rule.as_deref())
            .map(|rule| syn::LitStr::new(rule, self.model.ident.span()));
        let terminal_kind_override = terminal_attr
            .and_then(|attr| attr.terminal_kind.as_deref())
            .map(|kind| syn::LitStr::new(kind, self.model.ident.span()));

        let terminal_name_tokens = terminal_name_override
            .map(|lit| {
                quote! {
                    if provided == #struct_name_lit { #lit } else { provided }
                }
            })
            .unwrap_or_else(|| quote! { provided });
        let matching_rule_tokens = matching_rule_override
            .map(|lit| quote! { #lit })
            .unwrap_or_else(|| quote! { "bus" });
        let terminal_kind_tokens = terminal_kind_override
            .map(|lit| quote! { Some(#lit.to_string()) })
            .unwrap_or_else(|| quote! { None });

        let mut member_tokens = Vec::new();
        if include_members {
            for field in &self.model.fields {
                for attr in &field.attrs {
                    if let FieldAttributeOuter::Variable(var_attr) = attr {
                        if var_attr.skip {
                            continue;
                        }
                        let variable_name = var_attr
                            .name
                            .as_ref()
                            .cloned()
                            .unwrap_or_else(|| field.ident.to_string());
                        let variable_name_lit =
                            syn::LitStr::new(&variable_name, field.ident.span());
                        member_tokens.push(quote! {
                            let variable_name = format!("{}{}", #prefix_binding, #variable_name_lit);
                            terminal_member_variables.push(::fmi::schema::fmi3::TerminalMemberVariable {
                                variable_name: variable_name.clone(),
                                member_name: Some(variable_name),
                                variable_kind: "signal".to_string(),
                                ..Default::default()
                            });
                        });
                    }
                }
            }
        }

        let mut child_tokens = Vec::new();
        for field in &self.model.fields {
            if self.is_child_field(field) || self.has_no_variable_attributes(field) {
                let field_type = &field.rust_type;
                let field_name = field.ident.to_string();
                let child_terminal_name = self
                    .terminal_attr(field)
                    .and_then(|attr| attr.name.as_ref())
                    .cloned()
                    .unwrap_or_else(|| field_name.clone());
                let child_terminal_name_lit =
                    syn::LitStr::new(&child_terminal_name, field.ident.span());
                let child_prefix_lit = self.child_prefix_literal(field, &field_name);
                child_tokens.push(quote! {
                    let child_prefix = format!("{}{}.", #prefix_binding, #child_prefix_lit);
                    let child_terminal = if child_prefix.is_empty() {
                        <#field_type as ::fmi_export::fmi3::TerminalProvider>::terminal(#child_terminal_name_lit, None)
                    } else {
                        <#field_type as ::fmi_export::fmi3::TerminalProvider>::terminal(#child_terminal_name_lit, Some(child_prefix.as_str()))
                    };
                    terminals.push(child_terminal);
                });
            }
        }

        tokens.extend(quote! {
            #[automatically_derived]
            impl ::fmi_export::fmi3::TerminalProvider for #struct_name {
                fn terminal(name: &str, prefix: Option<&str>) -> ::fmi::schema::fmi3::Terminal {
                    let terminal_name = {
                        let provided = name;
                        #terminal_name_tokens
                    };
                    let mut terminal_member_variables = Vec::new();
                    #(#member_tokens)*

                    let mut terminals = Vec::new();
                    #(#child_tokens)*

                    ::fmi::schema::fmi3::Terminal {
                        name: terminal_name.to_string(),
                        matching_rule: #matching_rule_tokens.to_string(),
                        terminal_kind: #terminal_kind_tokens,
                        terminal_member_variables,
                        terminals,
                        ..Default::default()
                    }
                }
            }
        });
    }
}

impl TerminalProviderImpl<'_> {
    fn has_no_variable_attributes(&self, field: &Field) -> bool {
        !field
            .attrs
            .iter()
            .any(|attr| matches!(attr, FieldAttributeOuter::Variable(_)))
    }

    fn is_child_field(&self, field: &Field) -> bool {
        field
            .attrs
            .iter()
            .any(|attr| matches!(attr, FieldAttributeOuter::Child(_)))
    }

    fn child_prefix_literal(&self, field: &Field, default_name: &str) -> syn::LitStr {
        let prefix = field.attrs.iter().find_map(|attr| {
            if let FieldAttributeOuter::Child(child_attr) = attr {
                child_attr.prefix.as_deref()
            } else {
                None
            }
        });
        let prefix = prefix.unwrap_or(default_name);
        syn::LitStr::new(prefix, field.ident.span())
    }

    fn terminal_attr<'a>(
        &self,
        field: &'a Field,
    ) -> Option<&'a crate::model::TerminalAttribute> {
        field.attrs.iter().find_map(|attr| {
            if let FieldAttributeOuter::Terminal(terminal_attr) = attr {
                Some(terminal_attr)
            } else {
                None
            }
        })
    }
}
