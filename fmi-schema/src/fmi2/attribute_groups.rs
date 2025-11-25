#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Real")]
pub struct RealAttributes {
    #[xml(attr = "quantity")]
    pub quantity: Option<String>,

    #[xml(attr = "unit")]
    pub unit: Option<String>,

    /// Default display unit, provided the conversion of values in "unit" to values in
    /// "displayUnit" is defined in UnitDefinitions / Unit / DisplayUnit.
    #[xml(attr = "displayUnit")]
    pub display_unit: Option<String>,

    /// If relativeQuantity=true, offset for displayUnit must be ignored.
    #[xml(attr = "relativeQuantity")]
    pub relative_quantity: Option<bool>,

    #[xml(attr = "min")]
    pub min: Option<f64>,

    /// max >= min required
    #[xml(attr = "max")]
    pub max: Option<f64>,

    /// nominal >= min and <= max required
    #[xml(attr = "nominal")]
    pub nominal: Option<f64>,

    /// Set to true, e.g., for crank angle. If true and variable is a state, relative tolerance
    /// should be zero on this variable.
    #[xml(attr = "unbounded")]
    pub unbounded: Option<bool>,
}

#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Integer")]
pub struct IntegerAttributes {
    #[xml(attr = "quantity")]
    pub quantity: Option<String>,

    #[xml(attr = "min")]
    pub min: Option<f64>,

    /// max >= min required
    #[xml(attr = "max")]
    pub max: Option<f64>,
}
