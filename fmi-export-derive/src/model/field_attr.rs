use fmi::fmi3::schema;

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
    pub causality: Option<Causality>,
    pub variability: Option<Variability>,
    pub start: Option<syn::Expr>,
    /// Indicate the initial value determination (exact, calculated, approx)
    pub initial: Option<Initial>,
    /// Indicate that this variable is the derivative of another variable
    pub derivative: Option<syn::Ident>,
    /// Indicate that this variable is a state variable
    pub state: Option<bool>,
    /// Indicate that this variable is an event indicator
    pub event_indicator: Option<bool>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Causality(pub schema::Causality);

impl From<schema::Causality> for Causality {
    fn from(causality: schema::Causality) -> Self {
        Causality(causality)
    }
}

impl From<Causality> for schema::Causality {
    fn from(causality: Causality) -> Self {
        causality.0
    }
}

impl attribute_derive::parsing::AttributeBase for Causality {
    type Partial = Self;
}

impl attribute_derive::parsing::AttributeValue for Causality {
    fn parse_value(
        input: syn::parse::ParseStream,
    ) -> syn::Result<attribute_derive::parsing::SpannedValue<Self::Partial>> {
        let causality_id: syn::Ident = input.parse()?;
        let causality = match (&causality_id).to_string().as_str() {
            "Parameter" => schema::Causality::Parameter,
            "Input" => schema::Causality::Input,
            "Output" => schema::Causality::Output,
            "Local" => schema::Causality::Local,
            "Independent" => schema::Causality::Independent,
            "CalculatedParameter" => schema::Causality::CalculatedParameter,
            _ => {
                return Err(syn::Error::new(
                    causality_id.span(),
                    format!("Unknown causality: {causality_id}"),
                ));
            }
        };

        Ok(attribute_derive::parsing::SpannedValue::new(
            Causality(causality),
            causality_id.span(),
        ))
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Variability(pub schema::Variability);

impl From<schema::Variability> for Variability {
    fn from(variability: schema::Variability) -> Self {
        Variability(variability)
    }
}

impl From<Variability> for schema::Variability {
    fn from(variability: Variability) -> Self {
        variability.0
    }
}

impl attribute_derive::parsing::AttributeBase for Variability {
    type Partial = Self;
}

impl attribute_derive::parsing::AttributeValue for Variability {
    fn parse_value(
        input: syn::parse::ParseStream,
    ) -> syn::Result<attribute_derive::parsing::SpannedValue<Self::Partial>> {
        let variability_id: syn::Ident = input.parse()?;
        let variability = match variability_id.to_string().as_str() {
            "Constant" => schema::Variability::Constant,
            "Fixed" => schema::Variability::Fixed,
            "Tunable" => schema::Variability::Tunable,
            "Discrete" => schema::Variability::Discrete,
            "Continuous" => schema::Variability::Continuous,
            _ => {
                return Err(syn::Error::new(
                    variability_id.span(),
                    format!("Invalid variability '{}'", variability_id),
                ));
            }
        };
        Ok(attribute_derive::parsing::SpannedValue::new(
            Variability(variability),
            variability_id.span(),
        ))
    }
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct Initial(schema::Initial);

impl From<Initial> for schema::Initial {
    fn from(initial: Initial) -> Self {
        initial.0
    }
}

impl attribute_derive::parsing::AttributeBase for Initial {
    type Partial = Self;
}

impl attribute_derive::parsing::AttributeValue for Initial {
    fn parse_value(
        input: syn::parse::ParseStream,
    ) -> syn::Result<attribute_derive::parsing::SpannedValue<Self::Partial>> {
        let initial_id: syn::Ident = input.parse()?;
        let initial = match initial_id.to_string().as_str() {
            "Exact" => schema::Initial::Exact,
            "Calculated" => schema::Initial::Calculated,
            "Approx" => schema::Initial::Approx,
            _ => {
                return Err(syn::Error::new(
                    initial_id.span(),
                    format!("Invalid initial value '{}'", initial_id),
                ));
            }
        };
        Ok(attribute_derive::parsing::SpannedValue::new(
            Initial(initial),
            initial_id.span(),
        ))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldAttributeOuter {
    Docstring(String),
    Variable(FieldAttribute),
    Alias(FieldAttribute),
}
