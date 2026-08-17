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

#[cfg(test)]
mod tests {
    use hard_xml::{XmlRead, XmlWrite};

    use super::*;

    #[test]
    fn float_base_type_trait_exposes_name_and_description() {
        let float32 = Float32Type {
            name: "speed".into(),
            description: Some("shaft speed".into()),
            ..Default::default()
        };
        let float64 = Float64Type {
            name: "position".into(),
            ..Default::default()
        };

        assert_eq!(float32.name(), "speed");
        assert_eq!(float32.description(), Some("shaft speed"));
        assert_eq!(float64.name(), "position");
        assert_eq!(float64.description(), None);
    }

    #[test]
    fn all_type_definitions_parse_and_round_trip() {
        let xml = r#"<TypeDefinitions>
            <Float32Type name="f32" description="single" quantity="q" unit="m" displayUnit="cm" relativeQuantity="true" unbounded="false" min="-1.5" max="2.5" nominal="1.0"/>
            <Float64Type name="f64" min="-2" max="4" nominal="0.5"/>
            <Int8Type name="i8" quantity="count" min="-8" max="7"/>
            <UInt8Type name="u8" min="0" max="8"/>
            <Int16Type name="i16" min="-16" max="16"/>
            <UInt16Type name="u16" min="0" max="16"/>
            <Int32Type name="i32" min="-32" max="32"/>
            <UInt32Type name="u32" min="0" max="32"/>
            <Int64Type name="i64" min="-64" max="64"/>
            <UInt64Type name="u64" min="0" max="64"/>
            <BooleanType name="enabled" description="switch"/>
            <StringType name="label" description="caption"/>
            <BinaryType name="payload" mimeType="application/octet-stream" maxSize="4096"/>
            <EnumerationType name="state" quantity="mode">
                <Item name="off" value="0" description="disabled"/>
                <Item name="on" value="1"/>
            </EnumerationType>
            <ClockType name="tick" canBeDeactivated="true" priority="2" intervalVariability="constant" intervalDecimal="0.25" shiftDecimal="0.5" supportsFraction="true" resolution="1000" intervalCounter="4" shiftCounter="1"/>
        </TypeDefinitions>"#;

        let parsed = TypeDefinitions::from_str(xml).unwrap();
        assert_eq!(parsed.type_definitions.len(), 15);
        assert!(matches!(
            &parsed.type_definitions[0],
            TypeDefinition::Float32(value)
                if value.description.as_deref() == Some("single")
                    && value.relative_quantity == Some(true)
                    && value.min == Some(-1.5)
        ));
        assert!(matches!(
            &parsed.type_definitions[1],
            TypeDefinition::Float64(value)
                if value.name == "f64" && value.min == Some(-2.0) && value.max == Some(4.0)
        ));
        assert!(matches!(
            &parsed.type_definitions[2],
            TypeDefinition::Int8(value)
                if value.name == "i8" && value.min == Some(-8) && value.max == Some(7)
        ));
        assert!(matches!(
            &parsed.type_definitions[3],
            TypeDefinition::UInt8(value)
                if value.name == "u8" && value.min == Some(0) && value.max == Some(8)
        ));
        assert!(matches!(
            &parsed.type_definitions[4],
            TypeDefinition::Int16(value)
                if value.name == "i16" && value.min == Some(-16) && value.max == Some(16)
        ));
        assert!(matches!(
            &parsed.type_definitions[5],
            TypeDefinition::UInt16(value)
                if value.name == "u16" && value.min == Some(0) && value.max == Some(16)
        ));
        assert!(matches!(
            &parsed.type_definitions[6],
            TypeDefinition::Int32(value)
                if value.name == "i32" && value.min == Some(-32) && value.max == Some(32)
        ));
        assert!(matches!(
            &parsed.type_definitions[7],
            TypeDefinition::UInt32(value)
                if value.name == "u32" && value.min == Some(0) && value.max == Some(32)
        ));
        assert!(matches!(
            &parsed.type_definitions[8],
            TypeDefinition::Int64(value)
                if value.name == "i64" && value.min == Some(-64) && value.max == Some(64)
        ));
        assert!(matches!(
            &parsed.type_definitions[9],
            TypeDefinition::UInt64(value)
                if value.name == "u64" && value.min == Some(0) && value.max == Some(64)
        ));
        assert!(matches!(
            &parsed.type_definitions[10],
            TypeDefinition::Boolean(value)
                if value.name == "enabled" && value.description.as_deref() == Some("switch")
        ));
        assert!(matches!(
            &parsed.type_definitions[11],
            TypeDefinition::String(value)
                if value.name == "label" && value.description.as_deref() == Some("caption")
        ));
        assert!(matches!(
            &parsed.type_definitions[12],
            TypeDefinition::Binary(value)
                if value.mime_type.as_deref() == Some("application/octet-stream")
                    && value.max_size == Some(4096)
        ));
        assert!(matches!(
            &parsed.type_definitions[13],
            TypeDefinition::Enumeration(value)
                if value.items.len() == 2
                    && value.items[0].value == 0
                    && value.items[1].name == "on"
        ));
        assert!(matches!(
            &parsed.type_definitions[14],
            TypeDefinition::Clock(value)
                if value.priority == Some(2)
                    && value.interval_decimal == Some(0.25)
                    && value.resolution == Some(1000)
        ));

        let serialized = parsed.to_string().unwrap();
        let reparsed = TypeDefinitions::from_str(&serialized).unwrap();
        assert_eq!(reparsed, parsed);
    }

    #[test]
    fn type_defaults_are_empty() {
        assert!(TypeDefinitions::default().type_definitions.is_empty());
        assert!(Float32Type::default().name.is_empty());
        assert!(Float64Type::default().description.is_none());
        assert!(Int8Type::default().min.is_none());
        assert!(UInt8Type::default().max.is_none());
        assert!(Int16Type::default().name.is_empty());
        assert!(UInt16Type::default().name.is_empty());
        assert!(Int32Type::default().name.is_empty());
        assert!(UInt32Type::default().name.is_empty());
        assert!(Int64Type::default().name.is_empty());
        assert!(UInt64Type::default().name.is_empty());
        assert!(BooleanType::default().description.is_none());
        assert!(StringType::default().annotations.is_none());
        assert!(BinaryType::default().max_size.is_none());
        assert!(ClockType::default().interval_decimal.is_none());
    }

    #[test]
    fn strict_type_schema_rejects_unknown_attributes() {
        let xml =
            r#"<TypeDefinitions><Float64Type name="x" unsupported="true"/></TypeDefinitions>"#;

        assert!(TypeDefinitions::from_str(xml).is_err());
    }
}
