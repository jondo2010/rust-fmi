use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "Annotations")]
pub struct Fmi3Annotations {
    //#[xml(child = "Annotation")]
    pub annotation: Annotation,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Annotation {
    #[yaserde(attribute = "type")]
    pub r#type: String,
}
