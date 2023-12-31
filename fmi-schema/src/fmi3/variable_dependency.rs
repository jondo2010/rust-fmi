use yaserde_derive::{YaDeserialize, YaSerialize};

use super::Annotations;

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
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

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(tag = "Fmi3Unknown")]
pub struct Fmi3Unknown {
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
    #[yaserde(attribute, rename = "valueReference")]
    pub value_reference: u32,
    #[yaserde(attribute, rename = "dependencies")]
    pub dependencies: Vec<u32>,
    #[yaserde(attribute, rename = "dependenciesKind")]
    pub dependencies_kind: Vec<DependenciesKind>,
}

#[test]
fn test_dependencies_kind() {
    let xml = r#"
    <Fmi3Unknown valueReference="1" dependencies="0 1 2" dependenciesKind="dependent constant fixed" />
    "#;

    let x: Fmi3Unknown = yaserde::de::from_str(xml).unwrap();
    assert_eq!(x.value_reference, 1);
    assert_eq!(x.dependencies, vec![0, 1, 2]);
    assert_eq!(
        x.dependencies_kind,
        vec![
            DependenciesKind::Dependent,
            DependenciesKind::Constant,
            DependenciesKind::Fixed
        ]
    );
}
