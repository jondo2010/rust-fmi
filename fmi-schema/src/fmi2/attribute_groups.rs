use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct RealAttributes {
    #[yaserde(attribute)]
    pub quantity: Option<String>,

    #[yaserde(attribute)]
    pub unit: Option<String>,

    /// Default display unit, provided the conversion of values in "unit" to values in "displayUnit" is defined in UnitDefinitions / Unit / DisplayUnit.
    #[yaserde(attribute, rename = "displayUnit")]
    pub display_unit: Option<String>,

    /// If relativeQuantity=true, offset for displayUnit must be ignored.
    #[yaserde(attribute, rename = "relativeQuantity")]
    pub relative_quantity: bool,

    #[yaserde(attribute, rename = "min")]
    pub min: Option<f64>,

    /// max >= min required
    #[yaserde(attribute, rename = "max")]
    pub max: Option<f64>,

    /// nominal >= min and <= max required
    #[yaserde(attribute, rename = "nominal")]
    pub nominal: Option<f64>,

    /// Set to true, e.g., for crank angle. If true and variable is a state, relative tolerance should be zero on this variable.
    #[yaserde(attribute, rename = "unbounded")]
    pub unbounded: bool,
}

#[derive(Default, Debug, YaSerialize, YaDeserialize)]
pub struct IntegerAttributes {
    pub quantity: Option<String>,

    #[yaserde(attribute, rename = "min")]
    pub min: Option<f64>,

    /// max >= min required
    #[yaserde(attribute, rename = "max")]
    pub max: Option<f64>,
}
