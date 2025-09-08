//! Custom implementations of YaSerialize and YaDeserialize for Dimension
use yaserde::{
    __xml::{attribute::OwnedAttribute, namespace::Namespace, reader::XmlEvent, writer},
    Visitor,
    de::Deserializer,
};

use std::str::FromStr;

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

impl yaserde::YaDeserialize for Dimension {
    fn deserialize<R: std::io::Read>(reader: &mut Deserializer<R>) -> Result<Self, String> {
        let (named_element, struct_namespace) =
            if let XmlEvent::StartElement { name, .. } = reader.peek()?.to_owned() {
                (name.local_name.to_owned(), name.namespace.clone())
            } else {
                (String::from("Dimension"), Option::None)
            };
        let start_depth = reader.depth();
        yaserde::__derive_debug!(
            "Struct {} @ {}: start to parse {:?}",
            stringify!(Dimension),
            start_depth,
            named_element
        );
        if reader.depth() == 0 {
            if let Some(namespace) = struct_namespace {
                match namespace.as_str() {
                    bad_namespace => {
                        let msg = format!(
                            "bad namespace for {}, found {}",
                            named_element, bad_namespace
                        );
                        return Err(msg);
                    }
                }
            }
        }
        let mut __start_value = None;
        let mut __value_reference_value = None;

        #[allow(non_snake_case, non_camel_case_types)]
        struct __Visitor_Attribute_Start_;

        impl<'de> yaserde::Visitor<'de> for __Visitor_Attribute_Start_ {
            type Value = u64;
            fn visit_u64(self, v: &str) -> Result<Self::Value, String> {
                u64::from_str(v).map_err(|e| e.to_string())
            }
        }

        #[allow(non_snake_case, non_camel_case_types)]
        struct __Visitor_Attribute_ValueReference_;

        impl<'de> yaserde::Visitor<'de> for __Visitor_Attribute_ValueReference_ {
            type Value = u32;
            fn visit_u32(self, v: &str) -> Result<Self::Value, String> {
                u32::from_str(v).map_err(|e| e.to_string())
            }
        }
        let mut depth = 0;

        loop {
            let event = reader.peek()?.to_owned();
            yaserde::__derive_trace!(
                "Struct {} @ {}: matching {:?}",
                stringify!(Dimension),
                start_depth,
                event,
            );
            match event {
                XmlEvent::StartElement {
                    ref name,
                    ref attributes,
                    ..
                } => {
                    let namespace = name.namespace.clone().unwrap_or_default();
                    if depth == 0 && name.local_name == "Dimension" && namespace.as_str() == "" {
                        let _event = reader.next_event()?;
                    } else {
                        match (namespace.as_str(), name.local_name.as_str()) {
                            _ => {
                                let _event = reader.next_event()?;
                                if depth > 0 {
                                    reader.skip_element(|_event| {})?;
                                }
                            }
                        }
                    }
                    if depth == 0 {
                        for attr in attributes {
                            if attr.name.local_name == "start" {
                                let visitor = __Visitor_Attribute_Start_ {};
                                let value = visitor.visit_u64(&attr.value)?;
                                __start_value = Some(value);
                            }
                        }
                        for attr in attributes {
                            if attr.name.local_name == "valueReference" {
                                let visitor = __Visitor_Attribute_ValueReference_ {};
                                let value = visitor.visit_u32(&attr.value)?;
                                __value_reference_value = Some(value);
                            }
                        }
                    }
                    depth += 1;
                }
                XmlEvent::EndElement { ref name } => {
                    if name.local_name == named_element && reader.depth() == start_depth + 1 {
                        break;
                    }
                    let _event = reader.next_event()?;
                    depth -= 1;
                }
                XmlEvent::EndDocument => {
                    if false {
                        break;
                    }
                }
                XmlEvent::Characters(_text_content) => {
                    let _event = reader.next_event()?;
                }
                event => {
                    return Result::Err(format!("unknown event {:?}", event));
                }
            }
        }
        yaserde::__derive_debug!(
            "Struct {} @ {}: success",
            stringify!(Dimension),
            start_depth
        );

        match (__start_value, __value_reference_value) {
            (Some(_), Some(_)) => Result::Err(
                "Dimension cannot have both 'start' and 'valueReference' attributes".to_string(),
            ),
            (None, None) => Result::Err(
                "Dimension must have either 'start' or 'valueReference' attribute".to_string(),
            ),
            (Some(start), None) => Ok(Dimension::Fixed(start)),
            (None, Some(value_reference)) => Ok(Dimension::Variable(value_reference)),
        }
    }
}

impl yaserde::YaSerialize for Dimension {
    #[allow(unused_variables)]
    fn serialize<W: std::io::Write>(
        &self,
        writer: &mut yaserde::ser::Serializer<W>,
    ) -> Result<(), String> {
        if !writer.skip_start_end() {
            match self {
                Dimension::Fixed(fixed) => {
                    let val = fixed.to_string();
                    let event = writer::XmlEvent::start_element("Dimension").attr("start", &val);
                    writer.write(event).map_err(|e| e.to_string())?;
                }
                Dimension::Variable(variable) => {
                    let val = variable.to_string();
                    let event =
                        writer::XmlEvent::start_element("Dimension").attr("valueReference", &val);
                    writer.write(event).map_err(|e| e.to_string())?;
                }
            };

            writer
                .write(writer::XmlEvent::end_element())
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    fn serialize_attributes(
        &self,
        _source_attributes: Vec<OwnedAttribute>,
        _source_namespace: Namespace,
    ) -> Result<(Vec<OwnedAttribute>, Namespace), String> {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use yaserde_derive::{YaDeserialize, YaSerialize};

    #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
    pub struct TestVar {
        #[yaserde(rename = "Dimension")]
        pub dimensions: Vec<Dimension>,
    }
    #[test]
    fn test_dim_de() {
        let xml = r#"<TestVar>
    <Dimension valueReference="2"/>
    <Dimension start="2"/>
    </TestVar>"#;
        let var: TestVar = yaserde::de::from_str(xml).unwrap();

        assert_eq!(var.dimensions.len(), 2);
        assert_eq!(var.dimensions[0], Dimension::Variable(2));
        assert_eq!(var.dimensions[1], Dimension::Fixed(2));
    }

    #[test]
    fn test_dim_roundtrip() {
        let var = TestVar {
            dimensions: vec![Dimension::Fixed(2), Dimension::Variable(0)],
        };
        let serialized = yaserde::ser::to_string(&var).unwrap();
        let deserialized: TestVar = yaserde::de::from_str(&serialized).unwrap();
        assert_eq!(var, deserialized);
    }
}
