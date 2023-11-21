use yaserde_derive::{YaDeserialize, YaSerialize};

use super::Annotations;

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub enum DependenciesKind {
    #[yaserde(rename = "dependent")]
    #[default]
    Dependent,
    #[yaserde(rename = "constant")]
    Constant,
    #[yaserde(rename = "fixed")]
    Fixed,
    #[yaserde(rename = "tunable")]
    Tunable,
    #[yaserde(rename = "discrete")]
    Discrete,
}

#[derive(Clone, Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct Fmi3Unknown {
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,

    #[yaserde(attribute, rename = "valueReference")]
    pub value_reference: u32,
    //#[yaserde(attribute, rename = "dependencies")]
    //pub dependencies: Vec<u32>,

    //#[yaserde(attribute, rename = "dependenciesKind")]
    //pub dependencies_kind: Option<DependenciesKind>,
}

#[test]
fn test_dependencies_kind() {
    let xml = r#"
    <Fmi3Unknown valueReference="0" dependencies="0 1 2" dependenciesKind="dependent" />
    "#;

    let dependencies_kind: Fmi3Unknown = yaserde::de::from_str(xml).unwrap();
    dbg!(dependencies_kind);
}
