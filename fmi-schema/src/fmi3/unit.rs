use super::Annotations;

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Unit", strict(unknown_attribute, unknown_element))]
pub struct Fmi3Unit {
    #[xml(attr = "name")]
    pub name: String,
    #[xml(child = "BaseUnit")]
    pub base_unit: Option<BaseUnit>,
    #[xml(child = "DisplayUnit")]
    pub display_unit: Vec<DisplayUnit>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "BaseUnit", strict(unknown_attribute, unknown_element))]
pub struct BaseUnit {
    #[xml(attr = "kg")]
    pub kg: Option<i32>,
    #[xml(attr = "m")]
    pub m: Option<i32>,
    #[xml(attr = "s")]
    pub s: Option<i32>,
    #[xml(attr = "a")]
    pub a: Option<i32>,
    #[xml(attr = "k")]
    pub k: Option<i32>,
    #[xml(attr = "mol")]
    pub mol: Option<i32>,
    #[xml(attr = "cd")]
    pub cd: Option<i32>,
    #[xml(attr = "rad")]
    pub rad: Option<i32>,
    #[xml(attr = "factor")]
    pub factor: Option<f64>,
    #[xml(attr = "offset")]
    pub offset: Option<f64>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "DisplayUnit", strict(unknown_attribute, unknown_element))]
pub struct DisplayUnit {
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "factor")]
    pub factor: Option<f64>,
    #[xml(attr = "offset")]
    pub offset: Option<f64>,
    #[xml(attr = "inverse")]
    pub inverse: Option<bool>,
}

#[test]
fn test_dependencies_kind() {
    use hard_xml::{XmlRead, XmlWrite};

    let xml = r#"<Unit name="m/s2"><BaseUnit m="1" s="-2"/></Unit>"#;
    let unit = Fmi3Unit::from_str(xml).unwrap();

    assert_eq!(unit.name, "m/s2");
    assert_eq!(
        unit.base_unit,
        Some(BaseUnit {
            m: Some(1),
            s: Some(-2),
            ..Default::default()
        })
    );

    let xml_out = unit.to_string().unwrap();
    assert_eq!(xml_out, xml);
}

#[test]
fn test_display_unit() {
    use hard_xml::XmlRead;

    let xml = r#"<DisplayUnit name="km/h" factor="0.2777777777777778" offset="0"/>"#;

    let display_unit = DisplayUnit::from_str(xml).unwrap();

    assert_eq!(display_unit.name, "km/h");
    assert_eq!(display_unit.factor, Some(0.2777777777777778));
    assert_eq!(display_unit.offset, Some(0.0));
}
