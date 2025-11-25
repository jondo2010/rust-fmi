#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Unit", strict(unknown_attribute, unknown_element))]
/// Unit definition (with respect to SI base units) and default display units
pub struct Fmi2Unit {
    #[xml(attr = "name")]
    pub name: String,
    /// BaseUnit_value = factor*Unit_value + offset
    #[xml(child = "BaseUnit")]
    pub base_unit: Option<BaseUnit>,
    #[xml(child = "DisplayUnit")]
    pub display_unit: Vec<DisplayUnit>,
}

#[derive(Default, PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "BaseUnit", strict(unknown_attribute, unknown_element))]
pub struct BaseUnit {
    /// Exponent of SI base unit "kg"
    #[xml(attr = "kg")]
    pub kg: Option<i32>,
    /// Exponent of SI base unit "m"
    #[xml(attr = "m")]
    pub m: Option<i32>,
    #[xml(attr = "s")]
    pub s: Option<i32>,
    #[xml(attr = "A")]
    pub a: Option<i32>,
    #[xml(attr = "K")]
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
    use hard_xml::XmlRead;

    let xml = r#"
    <Unit name="m/s2"><BaseUnit m="1" s="-2"/></Unit>
    "#;

    let unit: Fmi2Unit = Fmi2Unit::from_str(xml).unwrap();
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
