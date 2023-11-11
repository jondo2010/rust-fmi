use yaserde_derive::{YaDeserialize, YaSerialize};

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde()]
pub struct Fmi3Annotations {
    #[yaserde(rename = "Annotation")]
    pub annotation: fmi_3_annotations::AnnotationType,
}

pub mod fmi_3_annotations {
    use super::*;
    
    #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
    #[yaserde()]
    pub struct AnnotationType {
        #[yaserde(attribute, rename = "type")]
        pub _type: String,
    }
}

// pub type Annotations = Fmi3Annotations;
