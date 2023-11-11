use yaserde_derive::{YaDeserialize, YaSerialize};

use super::{Annotations};

//use fmi3Annotation.xsd  ;
#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde()]
pub struct Fmi3Unknown {
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,

    #[yaserde(attribute, rename = "valueReference")]
    pub value_reference: u32,

    #[yaserde(attribute, rename = "dependencies")]
    pub dependencies: Option<String>,

    #[yaserde(attribute, rename = "dependenciesKind")]
    pub dependencies_kind: Option<String>,
}