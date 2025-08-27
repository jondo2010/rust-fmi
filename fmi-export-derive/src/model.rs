use attribute_derive::FromAttr;
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
pub struct StructAttribute {
    /// Optional model description (defaults to the struct docstring)
    pub description: Option<String>,
    /// ModelExchange interface configuration
    pub model_exchange: Option<ModelExchangeAttribute>,
    /// CoSimulation interface configuration
    pub co_simulation: Option<CoSimulationAttribute>,
}

/// ModelExchange interface capabilities that can be specified in attributes
#[derive(Default, Debug, attribute_derive::FromAttr, PartialEq, Clone)]
pub struct ModelExchangeAttribute {
    pub needs_completed_integrator_step: Option<bool>,
    pub provides_evaluate_discrete_states: Option<bool>,
    pub needs_execution_tool: Option<bool>,
    pub can_be_instantiated_only_once_per_process: Option<bool>,
    pub can_get_and_set_fmu_state: Option<bool>,
    pub can_serialize_fmu_state: Option<bool>,
    pub provides_directional_derivatives: Option<bool>,
    pub provides_adjoint_derivatives: Option<bool>,
    pub provides_per_element_dependencies: Option<bool>,
}

/// CoSimulation interface capabilities that can be specified in attributes
#[derive(Default, Debug, attribute_derive::FromAttr, PartialEq, Clone)]
pub struct CoSimulationAttribute {
    pub can_handle_variable_communication_step_size: Option<bool>,
    pub fixed_internal_step_size: Option<f64>,
    pub max_output_derivative_order: Option<u32>,
    pub recommended_intermediate_input_smoothness: Option<i32>,
    pub provides_intermediate_update: Option<bool>,
    pub might_return_early_from_do_step: Option<bool>,
    pub can_return_early_after_intermediate_update: Option<bool>,
    pub has_event_mode: Option<bool>,
    pub provides_evaluate_discrete_states: Option<bool>,
    pub needs_execution_tool: Option<bool>,
    pub can_be_instantiated_only_once_per_process: Option<bool>,
    pub can_get_and_set_fmu_state: Option<bool>,
    pub can_serialize_fmu_state: Option<bool>,
    pub provides_directional_derivatives: Option<bool>,
    pub provides_adjoint_derivatives: Option<bool>,
    pub provides_per_element_dependencies: Option<bool>,
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
    /// Indicate that this variable is the derivative of another variable
    pub derivative: Option<syn::Ident>,
    /// Indicate that this variable is a state variable
    pub state: Option<bool>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldAttributeOuter {
    Docstring(String),
    Variable(FieldAttribute),
    Alias(FieldAttribute),
}

#[derive(Debug, PartialEq, Clone)]
pub enum StructAttributeOuter {
    Docstring(String),
    Model(StructAttribute),
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
    pub attrs: Vec<StructAttributeOuter>,
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
        let attrs = item
            .attrs
            .iter()
            .filter_map(|attr| {
                match attr.meta.path().get_ident() {
                    Some(ident) if ident == "doc" => {
                        parse_doc_attribute(attr).map(StructAttributeOuter::Docstring)
                    }

                    Some(ident) if ident == "model" => {
                        StructAttribute::from_attribute(attr)
                            //.map_err(|e| emit_error!(item, "{e}"))
                            .map_err(|e| panic!("{e}"))
                            .ok()
                            .map(StructAttributeOuter::Model)
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
            if let StructAttributeOuter::Model(model_attr) = attr {
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
                    if let StructAttributeOuter::Docstring(doc) = attr {
                        Some(doc.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "".to_string())
        }
    }

    /// Extract the interface type from model attributes
    pub fn interface_type(&self) -> Option<String> {
        self.attrs.iter().find_map(|attr| {
            if let StructAttributeOuter::Model(model_attr) = attr {
                if model_attr.model_exchange.is_some() {
                    Some("ModelExchange".to_string())
                } else if model_attr.co_simulation.is_some() {
                    Some("CoSimulation".to_string())
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    /// Extract the ModelExchange configuration from model attributes
    pub fn model_exchange(&self) -> Option<&ModelExchangeAttribute> {
        self.attrs.iter().find_map(|attr| {
            if let StructAttributeOuter::Model(model_attr) = attr {
                model_attr.model_exchange.as_ref()
            } else {
                None
            }
        })
    }

    /// Extract the CoSimulation configuration from model attributes
    pub fn co_simulation(&self) -> Option<&CoSimulationAttribute> {
        self.attrs.iter().find_map(|attr| {
            if let StructAttributeOuter::Model(model_attr) = attr {
                model_attr.co_simulation.as_ref()
            } else {
                None
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use attribute_derive::FromAttr;

    use super::*;

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

    #[test]
    fn test_model_attributes() {
        let input: syn::ItemStruct = syn::parse_quote! {
            #[model(model_exchange(needs_execution_tool))]
            struct TestModel {
                #[variable(causality = Output, start = 1.0)]
                h: f64,
            }
        };
        let model = Model::from(input);
        assert_eq!(
            model.attrs,
            vec![StructAttributeOuter::Model(StructAttribute {
                description: None,
                model_exchange: Some(ModelExchangeAttribute {
                    needs_execution_tool: Some(true),
                    ..Default::default()
                }),
                co_simulation: None,
            })],
            "Model should have one attribute with no description"
        );
        assert_eq!(model.fields.len(), 1, "Model should have one field");
        let expected_ident: syn::Ident = syn::parse_quote!(h);
        assert_eq!(
            model.fields[0].ident, expected_ident,
            "Field name should be 'h'"
        );
    }
}
