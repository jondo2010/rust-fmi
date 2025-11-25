use super::annotation::Fmi3Annotations;

pub trait BaseTypeTrait {
    fn name(&self) -> &str;
    fn description(&self) -> Option<&str>;
}

macro_rules! declare_float_type {
    ($name: ident, $tag: expr, $type: ty) => {
        #[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
        #[xml(tag = $tag, strict(unknown_attribute, unknown_element))]
        pub struct $name {
            // TypeDefinitionBase
            #[xml(attr = "name")]
            pub name: String,
            #[xml(attr = "description")]
            pub description: Option<String>,
            #[xml(child = "Annotations")]
            pub annotations: Option<Fmi3Annotations>,
            // RealBaseAttributes
            #[xml(attr = "quantity")]
            pub quantity: Option<String>,
            #[xml(attr = "unit")]
            pub unit: Option<String>,
            #[xml(attr = "displayUnit")]
            pub display_unit: Option<String>,
            #[xml(attr = "relativeQuantity")]
            pub relative_quantity: Option<bool>,
            #[xml(attr = "unbounded")]
            pub unbounded: Option<bool>,
            // FloatAttributes
            #[xml(attr = "min")]
            pub min: Option<$type>,
            #[xml(attr = "max")]
            pub max: Option<$type>,
            #[xml(attr = "nominal")]
            pub nominal: Option<$type>,
        }

        impl BaseTypeTrait for $name {
            fn name(&self) -> &str {
                &self.name
            }

            fn description(&self) -> Option<&str> {
                self.description.as_deref()
            }
        }
    };
}

macro_rules! declare_int_type {
    ($name: ident, $tag: expr, $type: ty) => {
        #[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
        #[xml(tag = $tag, strict(unknown_attribute, unknown_element))]
        pub struct $name {
            // TypeDefinitionBase
            #[xml(attr = "name")]
            pub name: String,
            #[xml(attr = "description")]
            pub description: Option<String>,
            #[xml(child = "Annotations")]
            pub annotations: Option<Fmi3Annotations>,
            // IntegerBaseAttributes
            #[xml(attr = "quantity")]
            quantity: Option<String>,
            // IntAttributes
            #[xml(attr = "min")]
            pub min: Option<$type>,
            #[xml(attr = "max")]
            pub max: Option<$type>,
        }
    };
}

declare_float_type!(Float32Type, "Float32Type", f32);
declare_float_type!(Float64Type, "Float64Type", f64);
declare_int_type!(Int8Type, "Int8Type", i8);
declare_int_type!(UInt8Type, "UInt8Type", u8);
declare_int_type!(Int16Type, "Int16Type", i16);
declare_int_type!(UInt16Type, "UInt16Type", u16);
declare_int_type!(Int32Type, "Int32Type", i32);
declare_int_type!(UInt32Type, "UInt32Type", u32);
declare_int_type!(Int64Type, "Int64Type", i64);
declare_int_type!(UInt64Type, "UInt64Type", u64);

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "BooleanType", strict(unknown_attribute, unknown_element))]
pub struct BooleanType {
    // TypeDefinitionBase
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "description")]
    pub description: Option<String>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Fmi3Annotations>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "StringType", strict(unknown_attribute, unknown_element))]
pub struct StringType {
    // TypeDefinitionBase
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "description")]
    pub description: Option<String>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Fmi3Annotations>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "BinaryType", strict(unknown_attribute, unknown_element))]
pub struct BinaryType {
    // TypeDefinitionBase
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "description")]
    pub description: Option<String>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Fmi3Annotations>,
    // BinaryType specific attributes
    #[xml(attr = "mimeType")]
    pub mime_type: Option<String>,
    #[xml(attr = "maxSize")]
    pub max_size: Option<u64>,
}

#[derive(PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Item", strict(unknown_attribute, unknown_element))]
pub struct EnumerationItem {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "value")]
    pub value: i64,
    #[xml(attr = "description")]
    pub description: Option<String>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Fmi3Annotations>,
}

#[derive(PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "EnumerationType", strict(unknown_attribute, unknown_element))]
pub struct EnumerationType {
    // TypeDefinitionBase
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "description")]
    pub description: Option<String>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Fmi3Annotations>,
    // IntegerBaseAttributes
    #[xml(attr = "quantity")]
    pub quantity: Option<String>,
    // Items
    #[xml(child = "Item")]
    pub items: Vec<EnumerationItem>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "ClockType", strict(unknown_attribute, unknown_element))]
pub struct ClockType {
    // TypeDefinitionBase
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "description")]
    pub description: Option<String>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Fmi3Annotations>,
    // ClockAttributes
    #[xml(attr = "canBeDeactivated")]
    pub can_be_deactivated: Option<bool>,
    #[xml(attr = "priority")]
    pub priority: Option<u32>,
    #[xml(attr = "intervalVariability")]
    pub interval_variability: Option<String>,
    #[xml(attr = "intervalDecimal")]
    pub interval_decimal: Option<f64>,
    #[xml(attr = "shiftDecimal")]
    pub shift_decimal: Option<f64>,
    #[xml(attr = "supportsFraction")]
    pub supports_fraction: Option<bool>,
    #[xml(attr = "resolution")]
    pub resolution: Option<u64>,
    #[xml(attr = "intervalCounter")]
    pub interval_counter: Option<u64>,
    #[xml(attr = "shiftCounter")]
    pub shift_counter: Option<u64>,
}

#[derive(PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
pub enum TypeDefinition {
    #[xml(tag = "Float32Type")]
    Float32(Float32Type),
    #[xml(tag = "Float64Type")]
    Float64(Float64Type),
    #[xml(tag = "Int8Type")]
    Int8(Int8Type),
    #[xml(tag = "UInt8Type")]
    UInt8(UInt8Type),
    #[xml(tag = "Int16Type")]
    Int16(Int16Type),
    #[xml(tag = "UInt16Type")]
    UInt16(UInt16Type),
    #[xml(tag = "Int32Type")]
    Int32(Int32Type),
    #[xml(tag = "UInt32Type")]
    UInt32(UInt32Type),
    #[xml(tag = "Int64Type")]
    Int64(Int64Type),
    #[xml(tag = "UInt64Type")]
    UInt64(UInt64Type),
    #[xml(tag = "BooleanType")]
    Boolean(BooleanType),
    #[xml(tag = "StringType")]
    String(StringType),
    #[xml(tag = "BinaryType")]
    Binary(BinaryType),
    #[xml(tag = "EnumerationType")]
    Enumeration(EnumerationType),
    #[xml(tag = "ClockType")]
    Clock(ClockType),
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "TypeDefinitions", strict(unknown_attribute, unknown_element))]
pub struct TypeDefinitions {
    #[xml(
        child = "Float32Type",
        child = "Float64Type",
        child = "Int8Type",
        child = "UInt8Type",
        child = "Int16Type",
        child = "UInt16Type",
        child = "Int32Type",
        child = "UInt32Type",
        child = "Int64Type",
        child = "UInt64Type",
        child = "BooleanType",
        child = "StringType",
        child = "BinaryType",
        child = "EnumerationType",
        child = "ClockType"
    )]
    pub type_definitions: Vec<TypeDefinition>,
}

#[test]
fn test_type_definitions() {
    let xml = r#"<TypeDefinitions>
        <Float32Type name="speed" unit="m/s" min="0.0" max="100.0" nominal="50.0"/>
        <Int16Type name="count" quantity="count" min="0" max="1000"/>
        <Float64Type name="Position" quantity="Position" unit="m"/>
    </TypeDefinitions>"#;

    let types: TypeDefinitions = hard_xml::XmlRead::from_str(xml).unwrap();
    assert_eq!(types.type_definitions.len(), 3);
}
