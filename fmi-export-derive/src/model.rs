use proc_macro_error2::emit_error;

/// Helper function to extract docstring from a syn::Attribute
/// Follows DRY principles by centralizing doc attribute parsing logic
fn parse_doc_attribute(attr: &syn::Attribute) -> Option<String> {
    if attr.meta.path().is_ident("doc") {
        attr.meta.require_name_value().ok().and_then(|name_value| {
            if let syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(lit_str),
                ..
            }) = &name_value.value
            {
                let doc_line = lit_str.value();
                Some(doc_line.strip_prefix(' ').unwrap_or(&doc_line).to_string())
            } else {
                None
            }
        })
    } else {
        None
    }
}

/// StructAttribute represents the attributes that can be applied to the model struct
#[derive(Debug, attribute_derive::FromAttr, PartialEq, Clone)]
#[attribute(ident = model)]
pub struct StructAttr {
    /// Optional model description (defaults to the struct docstring)
    pub description: Option<String>,
}

/// FieldAttribute represents the attributes that can be applied to a model struct field
#[derive(Default, Debug, attribute_derive::FromAttr, PartialEq, Clone)]
#[attribute(ident = variable, aliases = [alias])]
#[attribute(error(missing_field = "`{field}` was not specified"))]
pub struct FieldAttribute {
    /// Optional custom name for the variable (defaults to field name)
    pub name: Option<String>,
    /// Optional description (overriding the field docstring)
    pub description: Option<String>,
    #[attribute(example = "Parameter")]
    pub causality: Option<syn::Ident>,
    pub variability: Option<syn::Ident>,
    pub start: Option<syn::Expr>,
    /// Indicate the initial value determination (exact, calculated, approx)
    pub initial: Option<syn::Ident>,
    /// Indicate that this variable is the derivative of another variable
    pub derivative: Option<syn::Ident>,
    /// Indicate that this variable is a state variable
    pub state: Option<bool>,
    /// Indicate that this variable is an event indicator
    pub event_indicator: Option<bool>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldAttributeOuter {
    Docstring(String),
    Variable(FieldAttribute),
    Alias(FieldAttribute),
}

#[derive(Debug, PartialEq, Clone)]
pub enum StructAttrOuter {
    Docstring(String),
    Model(StructAttr),
}

/// Representation of an FmuModel field with it's parsed attributes
#[derive(Debug, PartialEq, Clone)]
pub struct Field {
    pub ident: syn::Ident,
    pub ty: syn::Type,
    pub attrs: Vec<FieldAttributeOuter>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Model {
    pub ident: syn::Ident,
    pub attrs: Vec<StructAttrOuter>,
    pub fields: Vec<Field>,
}

impl Field {
    /// Extract the description from field docstrings
    pub fn fold_description(&self) -> String {
        self.attrs
            .iter()
            .find_map(|attr| {
                if let FieldAttributeOuter::Docstring(doc) = attr {
                    Some(doc.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "".to_string())
    }
}

impl From<syn::Field> for Field {
    fn from(field: syn::Field) -> Self {
        use attribute_derive::Attribute;
        let attrs = field
            .attrs
            .iter()
            .filter_map(|attr| {
                match attr.meta.path().get_ident() {
                    Some(ident) if ident == "doc" => {
                        parse_doc_attribute(attr).map(FieldAttributeOuter::Docstring)
                    }

                    Some(ident) if ident == "variable" => {
                        FieldAttribute::from_attribute(attr)
                            //.map_err(|e| emit_error!(field, "{e}"))
                            .map_err(|e| panic!("{e}"))
                            .ok()
                            .map(FieldAttributeOuter::Variable)
                    }

                    Some(ident) if ident == "alias" => {
                        FieldAttribute::from_attribute(attr)
                            //.map_err(|e| emit_error!(field, "{e}"))
                            .map_err(|e| panic!("{e}"))
                            .ok()
                            .map(FieldAttributeOuter::Alias)
                    }

                    _ => None,
                }
            })
            .collect();

        Self {
            ident: field.ident.expect("Expected named field"),
            ty: field.ty,
            attrs,
        }
    }
}

impl From<syn::ItemStruct> for Model {
    fn from(item: syn::ItemStruct) -> Self {
        use attribute_derive::Attribute;
        let attrs = item
            .attrs
            .iter()
            .filter_map(|attr| {
                match attr.meta.path().get_ident() {
                    Some(ident) if ident == "doc" => {
                        parse_doc_attribute(attr).map(StructAttrOuter::Docstring)
                    }

                    Some(ident) if ident == "model" => {
                        StructAttr::from_attribute(attr)
                            //.map_err(|e| emit_error!(item, "{e}"))
                            .map_err(|e| panic!("{e}"))
                            .ok()
                            .map(StructAttrOuter::Model)
                    }

                    _ => None,
                }
            })
            .collect();

        let fields = match item.fields {
            syn::Fields::Named(named) => named.named.into_iter().map(Field::from).collect(),
            _ => {
                emit_error!(item, "Expected named fields in the struct");
                vec![]
            }
        };

        Self {
            ident: item.ident,
            attrs,
            fields,
        }
    }
}

impl Model {
    /// Heuristically gather the model description from the 'description' attribute if present,
    /// otherwise use the struct docstring
    pub fn fold_description(&self) -> String {
        // First, look for explicit description in model attributes
        let explicit_description = self.attrs.iter().find_map(|attr| {
            if let StructAttrOuter::Model(model_attr) = attr {
                model_attr.description.clone()
            } else {
                None
            }
        });

        // If no explicit description, look for docstring
        if explicit_description.is_some() {
            explicit_description.unwrap_or_else(|| "".to_string())
        } else {
            self.attrs
                .iter()
                .find_map(|attr| {
                    if let StructAttrOuter::Docstring(doc) = attr {
                        Some(doc.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use attribute_derive::FromAttr;

    #[test]
    fn test_attribute() {
        let input: syn::Attribute = syn::parse_quote! {
            #[variable(causality = Parameter, variability = Fixed, start = -9.81)]
        };
        let _attr = FieldAttribute::from_attribute(input).unwrap();

        let input: syn::Attribute = syn::parse_quote! {
            #[variable(causality = Output, start = 0.0)]
        };
        let _attr = FieldAttribute::from_attribute(input).unwrap();
    }

    #[test]
    fn test_fields_and_attributes() {
        let input: syn::ItemStruct = syn::parse_quote! {
            struct TestModel {
                /// Test1
                #[variable(causality = Output, start = 1.0)]
                h: f64,

                /// Test2
                #[variable(causality = Output, start = 0.0)]
                #[alias(name="der(h)", description = "Derivative of h", causality = Local, derivative=h)]
                v: f64,
            }
        };

        let fields = match input.fields {
            syn::Fields::Named(named) => {
                named.named.into_iter().map(Field::from).collect::<Vec<_>>()
            }
            _ => panic!("Expected named fields"),
        };

        assert_eq!(fields.len(), 2, "There should be 2 fields");
        assert_eq!(
            fields[0].attrs,
            vec![
                FieldAttributeOuter::Docstring("Test1".to_string()),
                FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(syn::parse_quote!(Output)),
                    start: Some(syn::parse_quote!(1.0)),
                    ..Default::default()
                })
            ],
            "First field should have 2 attributes: docstring and variable"
        );
        assert_eq!(
            fields[1].attrs,
            vec![
                FieldAttributeOuter::Docstring("Test2".to_string()),
                FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(syn::parse_quote!(Output)),
                    start: Some(syn::parse_quote!(0.0)),
                    ..Default::default()
                }),
                FieldAttributeOuter::Alias(FieldAttribute {
                    name: Some("der(h)".to_string()),
                    description: Some("Derivative of h".to_string()),
                    causality: Some(syn::parse_quote!(Local)),
                    derivative: Some(syn::parse_quote!(h)),
                    ..Default::default()
                })
            ],
            "Second field should have 3 attributes: docstring, variable, and alias"
        );
    }

    #[test]
    fn test_field_description() {
        let input: syn::Field = syn::parse_quote! {
            /// This is a field description
            #[variable(causality = Output, start = 1.0)]
            height: f64
        };
        let field = Field::from(input);
        assert_eq!(
            field.fold_description(),
            "This is a field description".to_string(),
            "Field description should match the docstring"
        );

        let input: syn::Field = syn::parse_quote! {
            #[variable(causality = Output, start = 1.0)]
            height: f64
        };
        let field = Field::from(input);
        assert_eq!(
            field.fold_description(),
            "".to_string(),
            "Field description should be empty when no docstring"
        );
    }

    #[test]
    fn test_model_description() {
        let input: syn::ItemStruct = syn::parse_quote! {
            /// This is a test model
            #[model()]
            struct TestModel {
            }
        };
        let model = Model::from(input);
        assert_eq!(
            model.fold_description(),
            "This is a test model".to_string(),
            "Model description should match the docstring"
        );

        let input: syn::ItemStruct = syn::parse_quote! {
            /// This is a test model
            #[model(description = "Custom model description")]
            struct TestModel {
            }
        };
        let model = Model::from(input);
        assert_eq!(
            model.fold_description(),
            "Custom model description".to_string(),
            "Model description should match the custom description"
        );
    }
}
