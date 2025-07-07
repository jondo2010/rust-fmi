use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct RealBaseAttributes {
    #[yaserde(attribute = true)]
    pub quantity: Option<String>,
    #[yaserde(attribute = true)]
    pub unit: Option<String>,
    #[yaserde(attribute = true, rename = "displayUnit")]
    pub display_unit: Option<String>,
    #[yaserde(attribute = true, rename = "relativeQuantity")]
    pub relative_quantity: Option<bool>,
    #[yaserde(attribute = true, rename = "unbounded")]
    pub unbounded: Option<bool>,
}

macro_rules! float_attrs {
    ($name:ident, $type:ty) => {
        #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
        pub struct $name {
            #[yaserde(attribute = true)]
            pub min: Option<$type>,
            #[yaserde(attribute = true)]
            pub max: Option<$type>,
            #[yaserde(attribute = true)]
            pub nominal: Option<$type>,
        }
    };
}
float_attrs!(Float32Attributes, f32);
float_attrs!(Float64Attributes, f64);

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct IntegerBaseAttributes {
    #[yaserde(attribute = true)]
    quantity: Option<String>,
}

macro_rules! integer_attrs {
    ($name:ident, $type:ty) => {
        #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
        #[yaserde(rename = "$name")]
        pub struct $name {
            #[yaserde(attribute = true)]
            pub min: Option<$type>,
            #[yaserde(attribute = true)]
            pub max: Option<$type>,
        }
    };
}

integer_attrs!(Int8Attributes, i8);
integer_attrs!(UInt8Attributes, u8);
integer_attrs!(Int16Attributes, i16);
integer_attrs!(UInt16Attributes, u16);
integer_attrs!(Int32Attributes, i32);
integer_attrs!(UInt32Attributes, u32);
integer_attrs!(Int64Attributes, i64);
integer_attrs!(UInt64Attributes, u64);

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct RealVariableAttributes {
    /// If present, then the variable with this attribute is the derivative of the variable with value reference given in derivative.
    #[yaserde(attribute = true)]
    pub derivative: Option<u32>,
    /// Only used in Model Exchange, ignored for the other interface types. May only be present for a continuous-time state.
    /// If `true`, state may be reinitialized by the FMU in Event Mode.
    /// If `false`, state will not be reinitialized.
    #[yaserde(attribute = true)]
    pub reinit: Option<bool>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
struct EnumerationAttributes {
    #[yaserde(attribute = true)]
    pub min: Option<i64>,
    #[yaserde(attribute = true)]
    pub max: Option<i64>,
}
