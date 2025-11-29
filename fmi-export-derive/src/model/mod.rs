use attribute_derive::FromAttr;
use proc_macro_error2::emit_error;

mod field_attr;
pub use field_attr::{FieldAttribute, FieldAttributeOuter};

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

/// Co-Simulation configuration options
#[derive(Debug, attribute_derive::FromAttr, PartialEq, Clone, Default)]
#[attribute(ident = co_simulation)]
pub struct CoSimulationAttr {
    /// Embedded solver configuration
    #[attribute(optional)]
    pub embedded_solver: Option<EmbeddedSolverAttr>,
}

/// Embedded solver configuration for Co-Simulation wrapper
#[derive(Debug, attribute_derive::FromAttr, PartialEq, Clone)]
#[attribute(ident = embedded_solver)]
pub struct EmbeddedSolverAttr {
    /// Fixed step size for the embedded solver
    #[attribute(optional)]
    pub step_size: Option<f64>,
}

/// StructAttribute represents the attributes that can be applied to the model struct
#[derive(Debug, attribute_derive::FromAttr, PartialEq, Clone, Default)]
#[attribute(ident = model)]
pub struct StructAttr {
    /// Optional model description (defaults to the struct docstring)
    #[attribute(optional)]
    pub description: Option<String>,
    
    /// Enable Model Exchange interface
    #[attribute(optional)]
    pub model_exchange: Option<bool>,
    
    /// Enable Co-Simulation interface (with optional configuration)
    #[attribute(optional)]
    pub co_simulation: Option<CoSimulationAttr>,
    
    /// Enable Scheduled Execution interface
    #[attribute(optional)]
    pub scheduled_execution: Option<bool>,
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
    pub rust_type: syn::Type,
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

impl Model {
    /// Returns an iterator over all continuous state fields.
    /// A field is considered a continuous state if another field has it as a derivative.
    pub fn iter_continuous_states(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter(move |field| {
            let field_name = field.ident.to_string();
            self.is_continuous_state(&field_name)
        })
    }

    /// Checks if a field with the given name is a continuous state variable.
    /// A field is a continuous state if another field has it as a derivative.
    pub fn is_continuous_state(&self, field_name: &str) -> bool {
        self.fields.iter().any(|other_field| {
            other_field.attrs.iter().any(|attr| {
                let derivative_ref = match attr {
                    FieldAttributeOuter::Variable(var_attr) => &var_attr.derivative,
                    FieldAttributeOuter::Alias(alias_attr) => &alias_attr.derivative,
                    _ => return false,
                };

                derivative_ref.as_ref().map(|d| d.to_string()) == Some(field_name.to_string())
            })
        })
    }

    /// Counts the total number of continuous state elements (including array elements).
    pub fn count_continuous_states(&self) -> usize {
        todo!()
        //self.iter_continuous_states()
        //    .map(|field| field.field_type.total_elements())
        //    .sum()
    }

    /// Iterator over all fields that are derivatives.
    /// A field is considered a derivative if it has a derivative attribute.
    pub fn iter_derivatives(&self) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter(|field| self.is_derivative(field))
    }

    /// Checks if a field is a derivative variable.
    /// A field is a derivative if it has a derivative attribute.
    pub fn is_derivative(&self, field: &Field) -> bool {
        field.attrs.iter().any(|attr| match attr {
            FieldAttributeOuter::Variable(var_attr) => var_attr.derivative.is_some(),
            FieldAttributeOuter::Alias(alias_attr) => alias_attr.derivative.is_some(),
            _ => false,
        })
    }

    /// Counts the total number of derivative elements (including array elements).
    #[allow(dead_code)]
    pub fn count_derivatives(&self) -> usize {
        todo!()
        //self.iter_derivatives()
        //    .map(|field| field.field_type.total_elements())
        //    .sum()
    }

    /// Get the parsed model attribute, if present
    pub fn get_model_attr(&self) -> Option<&StructAttr> {
        self.attrs.iter().find_map(|attr| match attr {
            StructAttrOuter::Model(model_attr) => Some(model_attr),
            _ => None,
        })
    }

    /// Check if Model Exchange is supported
    pub fn supports_model_exchange(&self) -> bool {
        self.get_model_attr()
            .and_then(|attr| attr.model_exchange)
            .unwrap_or(true) // Default to true if not specified
    }

    /// Check if Co-Simulation is supported
    pub fn supports_co_simulation(&self) -> bool {
        self.get_model_attr()
            .and_then(|attr| attr.co_simulation.as_ref())
            .is_some()
    }

    /// Check if Scheduled Execution is supported
    pub fn supports_scheduled_execution(&self) -> bool {
        self.get_model_attr()
            .and_then(|attr| attr.scheduled_execution)
            .unwrap_or(false)
    }

    /// Get the Co-Simulation mode
    pub fn cs_mode(&self) -> &'static str {
        if !self.supports_co_simulation() {
            return "::fmi_export::fmi3::CSMode::NotSupported";
        }

        let has_embedded_solver = self
            .get_model_attr()
            .and_then(|attr| attr.co_simulation.as_ref())
            .and_then(|cs| cs.embedded_solver.as_ref())
            .is_some();

        if has_embedded_solver {
            "::fmi_export::fmi3::CSMode::Wrapper"
        } else {
            "::fmi_export::fmi3::CSMode::Direct"
        }
    }

    /// Get the fixed solver step size if using embedded solver
    pub fn fixed_solver_step(&self) -> Option<f64> {
        self.get_model_attr()
            .and_then(|attr| attr.co_simulation.as_ref())
            .and_then(|cs| cs.embedded_solver.as_ref())
            .and_then(|solver| solver.step_size)
    }
}

impl TryFrom<syn::Field> for Field {
    type Error = String;
    fn try_from(field: syn::Field) -> Result<Self, String> {
        use attribute_derive::Attribute;
        let attrs = field
            .attrs
            .iter()
            .filter_map(|attr| match attr.meta.path().get_ident() {
                Some(ident) if ident == "doc" => {
                    parse_doc_attribute(attr).map(FieldAttributeOuter::Docstring)
                }

                Some(ident) if ident == "variable" => {
                    match FieldAttribute::from_attribute(attr).map(FieldAttributeOuter::Variable) {
                        Ok(attr) => Some(attr),
                        Err(e) => {
                            emit_error!(attr, format!("{e}"));
                            None
                        }
                    }
                }

                Some(ident) if ident == "alias" => {
                    match FieldAttribute::from_attribute(attr).map(FieldAttributeOuter::Alias) {
                        Ok(attr) => Some(attr),
                        Err(e) => {
                            emit_error!(attr, format!("{e}"));
                            None
                        }
                    }
                }

                _ => None,
            })
            .collect();

        Ok(Self {
            ident: field.ident.expect("Expected named field"),
            rust_type: field.ty,
            attrs,
        })
    }
}

/// Check for variable name conflicts with the built-in "time" variable
fn check_time_variable_conflicts(fields: &[Field]) {
    for field in fields {
        let field_name = field.ident.to_string();
        if field_name.to_lowercase() == "time" {
            emit_error!(field.ident, "'time' is a reserved name.");
        }

        // Check alias names too
        for attr in &field.attrs {
            if let FieldAttributeOuter::Alias(alias_attr) = attr {
                if let Some(alias_name) = &alias_attr.name {
                    if alias_name.to_lowercase() == "time" {
                        emit_error!(field.ident, "'time' is a reserved name.");
                    }
                }
            }
        }
    }
}

impl From<syn::DeriveInput> for Model {
    fn from(item: syn::DeriveInput) -> Self {
        if let syn::Data::Struct(struct_data) = item.data {
            let attrs = build_attrs(item.attrs);
            let fields = build_fields(struct_data.fields);

            // Check for time variable name conflicts
            check_time_variable_conflicts(&fields);

            Self {
                ident: item.ident,
                attrs,
                fields,
            }
        } else {
            emit_error!(item, "FmuModel can only be derived for structs");
            Self {
                ident: item.ident,
                attrs: vec![],
                fields: vec![],
            }
        }
    }
}

pub fn build_attrs(attrs: Vec<syn::Attribute>) -> Vec<StructAttrOuter> {
    attrs
        .into_iter()
        .filter_map(|attr| match attr.meta.path().get_ident() {
            Some(ident) if ident == "doc" => {
                parse_doc_attribute(&attr).map(StructAttrOuter::Docstring)
            }

            Some(ident) if ident == "model" => match StructAttr::from_attribute(attr.clone()) {
                Ok(attr) => Some(StructAttrOuter::Model(attr)),
                Err(e) => {
                    emit_error!(attr, format!("{e}"));
                    None
                }
            },

            _ => None,
        })
        .collect()
}

/// Check if a field has any FMU-relevant attributes (variable or alias)
fn has_fmu_attributes(field: &syn::Field) -> bool {
    field.attrs.iter().any(|attr| {
        attr.meta
            .path()
            .get_ident()
            .map(|ident| ident == "variable" || ident == "alias")
            .unwrap_or(false)
    })
}

pub fn build_fields(fields: syn::Fields) -> Vec<Field> {
    match fields {
        syn::Fields::Named(syn::FieldsNamed { named, .. }) => named
            .into_iter()
            .filter(|field| has_fmu_attributes(field)) // Only process fields with FMU attributes
            .filter_map(|ref field| match Field::try_from(field.clone()) {
                Ok(field) => Some(field),
                Err(e) => {
                    emit_error!(field, format!("{e}"));
                    None
                }
            })
            .collect(),
        _ => {
            emit_error!(fields, "Expected named fields in the struct");
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use attribute_derive::FromAttr;
    use fmi::fmi3::schema;

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
        let fields = build_fields(input.fields);

        assert_eq!(fields.len(), 2, "There should be 2 fields");
        assert_eq!(
            fields[0].attrs,
            vec![
                FieldAttributeOuter::Docstring("Test1".to_string()),
                FieldAttributeOuter::Variable(FieldAttribute {
                    causality: Some(schema::Causality::Output.into()),
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
                    causality: Some(schema::Causality::Output.into()),
                    start: Some(syn::parse_quote!(0.0)),
                    ..Default::default()
                }),
                FieldAttributeOuter::Alias(FieldAttribute {
                    name: Some("der(h)".to_string()),
                    description: Some("Derivative of h".to_string()),
                    causality: Some(schema::Causality::Local.into()),
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
        let field = Field::try_from(input).unwrap();
        assert_eq!(
            field.fold_description(),
            "This is a field description".to_string(),
            "Field description should match the docstring"
        );

        let input: syn::Field = syn::parse_quote! {
            #[variable(causality = Output, start = 1.0)]
            height: f64
        };
        let field = Field::try_from(input).unwrap();
        assert_eq!(
            field.fold_description(),
            "".to_string(),
            "Field description should be empty when no docstring"
        );
    }

    #[test]
    fn test_fields_without_fmu_attributes_are_ignored() {
        let input: syn::ItemStruct = syn::parse_quote! {
            struct TestModel {
                /// Height above ground (state output) - has FMU attribute
                #[variable(causality = Output, start = 1.0)]
                h: f64,

                /// User variable without FMU attributes - should be ignored
                internal_state: Vec<bool>,

                /// Another user variable - should be ignored
                helper_data: std::collections::HashMap<String, i32>,

                /// Velocity - has FMU attribute
                #[variable(causality = Output, start = 0.0)]
                v: f64,
            }
        };
        let fields = build_fields(input.fields);

        // Only fields with FMU attributes should be included
        assert_eq!(
            fields.len(),
            2,
            "Only fields with FMU attributes should be processed"
        );
        assert_eq!(fields[0].ident.to_string(), "h");
        assert_eq!(fields[1].ident.to_string(), "v");
    }
}
