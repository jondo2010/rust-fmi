use super::attribute_groups::{IntegerAttributes, RealAttributes};

#[derive(Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
pub enum SimpleTypeElement {
    #[xml(tag = "Real")]
    Real(RealAttributes),
    #[xml(tag = "Integer")]
    Integer(IntegerAttributes),
    #[xml(tag = "Boolean")]
    Boolean,
    #[xml(tag = "String")]
    String,
    #[xml(tag = "Enumeration")]
    Enumeration,
}

impl Default for SimpleTypeElement {
    fn default() -> Self {
        Self::Real(RealAttributes::default())
    }
}

#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "SimpleType", strict(unknown_attribute, unknown_element))]
/// Type attributes of a scalar variable
pub struct SimpleType {
    /// Name of SimpleType element. "name" must be unique with respect to all other elements of the
    /// TypeDefinitions list. Furthermore, "name" of a SimpleType must be different to all
    /// "name"s of ScalarVariable.
    #[xml(attr = "name")]
    pub name: String,

    /// Description of the SimpleType
    #[xml(attr = "description")]
    pub description: Option<String>,

    #[xml(child = "Real", child = "Integer", child = "Boolean", child = "String", child = "Enumeration")]
    pub elem: SimpleTypeElement,
}

#[cfg(test)]
mod tests {
    use hard_xml::XmlRead;

    use crate::fmi2::{RealAttributes, SimpleTypeElement};

    use super::SimpleType;

    #[test]
    fn test_simple_type() {
        let xml = r#"
        <SimpleType name="Acceleration">
            <Real quantity="Acceleration" unit="m/s2"/>
        </SimpleType>"#;

        let simple_type = SimpleType::from_str(xml).unwrap();
        assert_eq!(simple_type.name, "Acceleration");
        assert_eq!(simple_type.description, None);
        assert_eq!(
            simple_type.elem,
            SimpleTypeElement::Real(RealAttributes {
                quantity: Some("Acceleration".to_owned()),
                unit: Some("m/s2".to_owned()),
                ..Default::default()
            })
        );
    }
}
