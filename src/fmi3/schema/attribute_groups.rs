use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct RealBaseAttributes {
    #[yaserde(attribute)]
    pub quantity: Option<String>,
    #[yaserde(attribute)]
    pub unit: Option<String>,
    #[yaserde(attribute, rename = "displayUnit")]
    pub display_unit: Option<String>,
    #[yaserde(attribute, rename = "relativeQuantity")]
    pub relative_quantity: bool,
    #[yaserde(attribute)]
    pub unbounded: bool,
}

//macro_rules! float_attributes {
//    ($name:ident, $type:ty) => {
//        #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
//        pub struct $name {
//            #[yaserde(attribute)]
//            pub min: Option<$type>,
//            #[yaserde(attribute)]
//            pub max: Option<$type>,
//            #[yaserde(attribute)]
//            pub nominal: Option<$type>,
//        }
//    };
//}
//
//float_attributes!(Float32Attributes, f32);
//float_attributes!(Float64Attributes, f64);

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Float32Attributes {
    #[yaserde(attribute)]
    pub min: Option<f32>,
    #[yaserde(attribute)]
    pub max: Option<f32>,
    #[yaserde(attribute)]
    pub nominal: Option<f32>,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Float64Attributes {
    #[yaserde(attribute)]
    pub min: Option<f64>,
    #[yaserde(attribute)]
    pub max: Option<f64>,
    #[yaserde(attribute)]
    pub nominal: Option<f64>,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
struct IntegerBaseAttributes {
    quantity: String,
}

macro_rules! integer_attributes {
    ($name:ident, $type:ty) => {
        #[derive(Clone, Default, Debug, PartialEq, YaSerialize, YaDeserialize)]
        pub struct $name {
            #[yaserde(attribute)]
            pub min: $type,
            #[yaserde(attribute)]
            pub max: $type,
        }
    };
}

integer_attributes!(Int8Attributes, i8);
integer_attributes!(UInt8Attributes, u8);
integer_attributes!(Int16Attributes, i16);
integer_attributes!(UInt16Attributes, u16);
integer_attributes!(Int32Attributes, i32);
integer_attributes!(UInt32Attributes, u32);
integer_attributes!(Int64Attributes, i64);
integer_attributes!(UInt64Attributes, u64);

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct RealVariableAttributes {
    #[yaserde(attribute)]
    pub derivative: Option<u32>,
    #[yaserde(attribute)]
    pub reinit: bool,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
struct EnumerationAttributes {
    #[yaserde(attribute)]
    pub min: i64,
    #[yaserde(attribute)]
    pub max: i64,
}
