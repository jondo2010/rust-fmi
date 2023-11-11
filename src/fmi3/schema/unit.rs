use yaserde_derive::{YaDeserialize, YaSerialize};
use super::{Annotations};

//use fmi3Annotation.xsd  ;
#[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
#[yaserde()]
pub struct Fmi3Unit {
    #[yaserde(rename = "BaseUnit")]
    pub base_unit: Option<fmi_3_unit::BaseUnitType>,

    #[yaserde(rename = "DisplayUnit")]
    pub display_unit: Vec<fmi_3_unit::DisplayUnitType>,

    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,

    #[yaserde(attribute, rename = "name")]
    pub name: String,
}

pub mod fmi_3_unit {
    use super::*;
    
    #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
    #[yaserde()]
    pub struct BaseUnitType {
        #[yaserde(attribute, rename = "kg")]
        pub kg: Option<i32>,

        #[yaserde(attribute, rename = "m")]
        pub m: Option<i32>,

        #[yaserde(attribute, rename = "s")]
        pub s: Option<i32>,

        #[yaserde(attribute, rename = "A")]
        pub a: Option<i32>,

        #[yaserde(attribute, rename = "K")]
        pub k: Option<i32>,

        #[yaserde(attribute, rename = "mol")]
        pub mol: Option<i32>,

        #[yaserde(attribute, rename = "cd")]
        pub cd: Option<i32>,

        #[yaserde(attribute, rename = "rad")]
        pub rad: Option<i32>,

        #[yaserde(attribute, rename = "factor")]
        pub factor: Option<f64>,

        #[yaserde(attribute, rename = "offset")]
        pub offset: Option<f64>,
    }

    #[derive(Default, PartialEq, Debug, YaSerialize, YaDeserialize)]
    #[yaserde()]
    pub struct DisplayUnitType {
        #[yaserde(rename = "Annotations")]
        pub annotations: Option<Annotations>,

        #[yaserde(attribute, rename = "name")]
        pub name: String,

        #[yaserde(attribute, rename = "factor")]
        pub factor: Option<f64>,

        #[yaserde(attribute, rename = "offset")]
        pub offset: Option<f64>,

        #[yaserde(attribute, rename = "inverse")]
        pub inverse: Option<bool>,
    }
}

