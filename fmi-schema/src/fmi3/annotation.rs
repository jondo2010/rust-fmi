use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "Annotations")]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Fmi3Annotations {
    pub annotation: Annotation,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Annotation {
    #[yaserde(attribute = true)]
    pub r#type: String,
}
