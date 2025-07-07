use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "Annotations")]
pub struct Fmi3Annotations {
    pub annotation: Annotation,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Annotation {
    #[yaserde(attribute = true)]
    pub r#type: String,
}
