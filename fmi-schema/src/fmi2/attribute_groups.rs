use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Default, Debug, PartialEq, YaSerialize, YaDeserialize)]
pub struct RealAttributes {
    #[yaserde(attribute = true)]
    pub quantity: Option<String>,

    #[yaserde(attribute = true)]
    pub unit: Option<String>,

    /// Default display unit, provided the conversion of values in "unit" to values in
    /// "displayUnit" is defined in UnitDefinitions / Unit / DisplayUnit.
    #[yaserde(attribute = true, rename = "displayUnit")]
    pub display_unit: Option<String>,

    /// If relativeQuantity=true, offset for displayUnit must be ignored.
    #[yaserde(attribute = true, rename = "relativeQuantity")]
    pub relative_quantity: Option<bool>,

    #[yaserde(attribute = true, rename = "min")]
    pub min: Option<f64>,

    /// max >= min required
    #[yaserde(attribute = true, rename = "max")]
    pub max: Option<f64>,

    /// nominal >= min and <= max required
    #[yaserde(attribute = true, rename = "nominal")]
    pub nominal: Option<f64>,

    /// Set to true, e.g., for crank angle. If true and variable is a state, relative tolerance
    /// should be zero on this variable.
    #[yaserde(attribute = true, rename = "unbounded")]
    pub unbounded: Option<bool>,
}

#[derive(Default, Debug, PartialEq, YaSerialize, YaDeserialize)]
pub struct IntegerAttributes {
    pub quantity: Option<String>,

    #[yaserde(attribute = true, rename = "min")]
    pub min: Option<f64>,

    /// max >= min required
    #[yaserde(attribute = true, rename = "max")]
    pub max: Option<f64>,
}
