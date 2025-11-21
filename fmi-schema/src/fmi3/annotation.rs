#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[xml(tag = "Annotations", strict(unknown_attribute, unknown_element))]
pub struct Fmi3Annotations {
    #[xml(child = "Annotation")]
    pub annotations: Vec<Annotation>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
#[xml(tag = "Annotation", strict(unknown_attribute, unknown_element))]
pub struct Annotation {
    #[xml(attr = "type")]
    pub r#type: String,
    #[xml(text)]
    pub content: String,
}
