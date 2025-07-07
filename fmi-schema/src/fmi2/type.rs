use yaserde_derive::{YaDeserialize, YaSerialize};

use super::attribute_groups::{IntegerAttributes, RealAttributes};

#[derive(Debug, PartialEq, YaSerialize, YaDeserialize)]
pub enum SimpleTypeElement {
    #[yaserde(flatten = true)]
    Real(RealAttributes),
    #[yaserde(flatten = true)]
    Integer(IntegerAttributes),
    #[yaserde()]
    Boolean,
    #[yaserde()]
    String,
    #[yaserde()]
    Enumeration,
}

impl Default for SimpleTypeElement {
    fn default() -> Self {
        Self::Real(RealAttributes::default())
    }
}

#[derive(Default, Debug, PartialEq, YaSerialize, YaDeserialize)]
#[yaserde()]
/// Type attributes of a scalar variable
pub struct SimpleType {
    #[yaserde(flatten = true)]
    pub elem: SimpleTypeElement,

    #[yaserde(attribute = true)]
    /// Name of SimpleType element. "name" must be unique with respect to all other elements of the
    /// TypeDefinitions list. Furthermore, "name" of a SimpleType must be different to all
    /// "name"s of ScalarVariable.
    pub name: String,

    #[yaserde(attribute = true)]
    /// Description of the SimpleType
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::fmi2::{RealAttributes, SimpleTypeElement};

    use super::SimpleType;

    #[test]
    fn test_simple_type() {
        let xml = r#"
        <SimpleType name="Acceleration">
            <Real quantity="Acceleration" unit="m/s2"/>
        </SimpleType>"#;

        let simple_type: SimpleType = yaserde::de::from_str(xml).unwrap();
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
