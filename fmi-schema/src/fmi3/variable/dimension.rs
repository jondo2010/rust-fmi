//! Custom implementations of YaSerialize and YaDeserialize for Dimension
use hard_xml::{XmlRead, XmlWrite};

#[derive(Clone, PartialEq, Debug)]
pub enum Dimension {
    /// Defines a constant unsigned 64-bit integer size for this dimension. The variability of the
    /// dimension size is constant in this case.
    Fixed(u64),
    /// If the present, it defines the size of this dimension to be the value of the variable with
    /// the value reference given by the `value_reference` attribute. The referenced variable
    /// must be a variable of type `UInt64`, and must either be a constant (i.e. with
    /// variability = constant) or a structural parameter (i.e. with causality =
    /// structuralParameter). The variability of the dimension size is in this case the variability
    /// of the referenced variable. A structural parameter must be a variable of type `UInt64`
    /// only if it is referenced in `Dimension`.
    Variable(u32),
}

impl Dimension {
    pub fn as_fixed(&self) -> Option<u64> {
        match self {
            Dimension::Fixed(size) => Some(*size),
            Dimension::Variable(_) => None,
        }
    }

    pub fn as_variable(&self) -> Option<u32> {
        match self {
            Dimension::Fixed(_) => None,
            Dimension::Variable(value_reference) => Some(*value_reference),
        }
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self::Fixed(1)
    }
}

#[derive(hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Dimension")]
struct Inner {
    #[xml(attr = "start")]
    start: Option<u64>,
    #[xml(attr = "valueReference")]
    value_reference: Option<u32>,
}

impl<'a> XmlRead<'a> for Dimension {
    fn from_reader(reader: &mut hard_xml::XmlReader<'a>) -> hard_xml::XmlResult<Self> {
        let Inner {
            start,
            value_reference,
        } = Inner::from_reader(reader)?;

        match (start, value_reference) {
            (Some(_), Some(_)) => Err(hard_xml::XmlExtendedError::DuplicateAttribute(
                "Dimension cannot have both 'start' and 'valueReference' attributes".into(),
            )
            .into()),
            (None, None) => Err(hard_xml::XmlError::MissingField {
                name: "Dimension".into(),
                field: "either 'start' or 'valueReference'".into(),
            }),
            (Some(start), None) => Ok(Dimension::Fixed(start)),
            (None, Some(value_reference)) => Ok(Dimension::Variable(value_reference)),
        }
    }
}

impl XmlWrite for Dimension {
    fn to_writer<W: std::io::Write>(
        &self,
        writer: &mut hard_xml::XmlWriter<W>,
    ) -> hard_xml::XmlResult<()> {
        match self {
            Dimension::Fixed(fixed) => Inner {
                start: Some(*fixed),
                value_reference: None,
            },
            Dimension::Variable(variable) => Inner {
                start: None,
                value_reference: Some(*variable),
            },
        }
        .to_writer(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
    #[xml(tag = "TestVar")]
    pub struct TestVar {
        #[xml(child = "Dimension")]
        pub dimensions: Vec<Dimension>,
    }
    #[test]
    fn test_dim_de() {
        let _ = env_logger::builder()
            .is_test(true)
            .format_timestamp(None)
            .try_init();

        let xml = r#"<TestVar>
    <Dimension valueReference="2"/>
    <Dimension start="2"/>
    </TestVar>"#;
        let var = TestVar::from_str(xml).unwrap();

        assert_eq!(var.dimensions.len(), 2);
        assert_eq!(var.dimensions[0], Dimension::Variable(2));
        assert_eq!(var.dimensions[1], Dimension::Fixed(2));
    }

    #[test]
    fn test_dim_roundtrip() {
        let var = TestVar {
            dimensions: vec![Dimension::Fixed(2), Dimension::Variable(0)],
        };
        let serialized = var.to_string().unwrap();
        let deserialized = TestVar::from_str(&serialized).unwrap();
        assert_eq!(var, deserialized);
    }
}
