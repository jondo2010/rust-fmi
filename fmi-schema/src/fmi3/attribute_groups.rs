use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct RealBaseAttributes {
    #[yaserde(attribute)]
    pub quantity: Option<String>,
    #[yaserde(attribute)]
    pub unit: Option<String>,
    #[yaserde(attribute, rename = "displayUnit")]
    pub display_unit: Option<String>,
    #[yaserde(attribute, rename = "relativeQuantity")]
    pub relative_quantity: bool,
    #[yaserde(attribute, rename = "unbounded")]
    pub unbounded: bool,
}

macro_rules! float_attrs {
    ($name:ident, $type:ty) => {
        #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
        pub struct $name {
            #[yaserde(attr)]
            pub min: Option<$type>,
            #[yaserde(attr)]
            pub max: Option<$type>,
            #[yaserde(attr)]
            pub nominal: Option<$type>,
        }
    };
}
float_attrs!(Float32Attributes, f32);
float_attrs!(Float64Attributes, f64);

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct IntegerBaseAttributes {
    #[yaserde(attribute)]
    quantity: String,
}

macro_rules! integer_attrs {
    ($name:ident, $type:ty) => {
        #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
        #[yaserde(rename = "$name")]
        pub struct $name {
            #[yaserde(attribute)]
            pub min: $type,
            #[yaserde(attribute)]
            pub max: $type,
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
    #[yaserde(attribute)]
    pub derivative: Option<u32>,
    #[yaserde(attribute)]
    pub reinit: bool,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
struct EnumerationAttributes {
    #[yaserde(attribute)]
    pub min: i64,
    #[yaserde(attribute)]
    pub max: i64,
}
