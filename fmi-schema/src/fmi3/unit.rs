use yaserde_derive::{YaDeserialize, YaSerialize};

use super::Annotations;

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde(rename = "Unit")]
pub struct Fmi3Unit {
    #[yaserde(attribute = true)]
    pub name: String,
    #[yaserde(rename = "BaseUnit")]
    pub base_unit: Option<BaseUnit>,
    #[yaserde(rename = "DisplayUnit")]
    pub display_unit: Vec<DisplayUnit>,
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct BaseUnit {
    #[yaserde(attribute = true, rename = "kg")]
    pub kg: Option<i32>,
    #[yaserde(attribute = true, rename = "m")]
    pub m: Option<i32>,
    #[yaserde(attribute = true, rename = "s")]
    pub s: Option<i32>,
    #[yaserde(attribute = true, rename = "A")]
    pub a: Option<i32>,
    #[yaserde(attribute = true, rename = "K")]
    pub k: Option<i32>,
    #[yaserde(attribute = true, rename = "mol")]
    pub mol: Option<i32>,
    #[yaserde(attribute = true, rename = "cd")]
    pub cd: Option<i32>,
    #[yaserde(attribute = true, rename = "rad")]
    pub rad: Option<i32>,
    #[yaserde(attribute = true, rename = "factor")]
    pub factor: Option<f64>,
    #[yaserde(attribute = true, rename = "offset")]
    pub offset: Option<f64>,
}

#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
pub struct DisplayUnit {
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
    #[yaserde(attribute = true, rename = "name")]
    pub name: String,
    #[yaserde(attribute = true, rename = "factor")]
    pub factor: Option<f64>,
    #[yaserde(attribute = true, rename = "offset")]
    pub offset: Option<f64>,
    #[yaserde(attribute = true, rename = "inverse")]
    pub inverse: Option<bool>,
}

#[test]
fn test_dependencies_kind() {
    let xml = r#"
    <Unit name="m/s2"> <BaseUnit m="1" s="-2"/> </Unit>
    "#;

    let unit: Fmi3Unit = yaserde::de::from_str(xml).unwrap();
    assert_eq!(unit.name, "m/s2");
    assert_eq!(
        unit.base_unit,
        Some(BaseUnit {
            m: Some(1),
            s: Some(-2),
            ..Default::default()
        })
    )
}
