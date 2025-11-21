use std::{fmt::Display, str::FromStr};

use crate::utils::AttrList;

use super::Annotations;

#[derive(Default, PartialEq, Debug)]
pub enum DependenciesKind {
    #[default]
    Dependent,
    Constant,
    Fixed,
    Tunable,
    Discrete,
}

impl FromStr for DependenciesKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dependent" => Ok(DependenciesKind::Dependent),
            "constant" => Ok(DependenciesKind::Constant),
            "fixed" => Ok(DependenciesKind::Fixed),
            "tunable" => Ok(DependenciesKind::Tunable),
            "discrete" => Ok(DependenciesKind::Discrete),
            _ => Err(format!("Invalid DependenciesKind: {}", s)),
        }
    }
}

impl Display for DependenciesKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            DependenciesKind::Dependent => "dependent",
            DependenciesKind::Constant => "constant",
            DependenciesKind::Fixed => "fixed",
            DependenciesKind::Tunable => "tunable",
            DependenciesKind::Discrete => "discrete",
        };
        write!(f, "{}", s)
    }
}

#[derive(Default, PartialEq, Debug, /*hard_xml::XmlRead*/ hard_xml::XmlWrite)]
#[xml(tag = "Fmi3Unknown", strict(unknown_attribute, unknown_element))]
pub struct Fmi3Unknown {
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    #[xml(attr = "valueReference")]
    pub value_reference: u32,
    #[xml(attr = "dependencies")]
    pub dependencies: Option<AttrList<u32>>,
    #[xml(attr = "dependenciesKind")]
    pub dependencies_kind: Option<AttrList<DependenciesKind>>,
}

impl<'__input> ::hard_xml::XmlRead<'__input> for Fmi3Unknown {
    fn from_reader(
        mut reader: &mut ::hard_xml::XmlReader<'__input>,
    ) -> ::hard_xml::XmlResult<Self> {
        use ::hard_xml::XmlError;
        use ::hard_xml::xmlparser::{ElementEnd, Token, Tokenizer};
        ::hard_xml::log_start_reading!(Fmi3Unknown_);
        let mut __self_annotations = None;
        let mut __self_value_reference = None;
        let mut __self_dependencies = None;
        let mut __self_dependencies_kind = None;

        let tag = reader
            .find_element_start(None)?
            .expect("Expected start element");
        let _ = reader.next().unwrap()?; // Move to attributes or end

        while let Some((__key, __value)) = reader.find_attribute()? {
            match __key {
                "valueReference" => {
                    ::hard_xml::log_start_reading_field!(Fmi3Unknown_, value_reference);
                    if !false && __self_value_reference.is_some() {
                        return ::std::result::Result::Err(
                            ::hard_xml::XmlExtendedError::DuplicateAttribute(
                                ("valueReference").into(),
                            )
                            .into(),
                        );
                    }
                    __self_value_reference = Some(
                        <u32 as std::str::FromStr>::from_str(&__value)
                            .map_err(|e| XmlError::FromStr(e.into()))?,
                    );
                    ::hard_xml::log_finish_reading_field!(Fmi3Unknown_, value_reference);
                }
                "dependencies" => {
                    ::hard_xml::log_start_reading_field!(Fmi3Unknown_, dependencies);
                    if !false && __self_dependencies.is_some() {
                        return ::std::result::Result::Err(
                            ::hard_xml::XmlExtendedError::DuplicateAttribute(
                                ("dependencies").into(),
                            )
                            .into(),
                        );
                    }
                    __self_dependencies = Some(
                        <AttrList<u32> as std::str::FromStr>::from_str(&__value)
                            .map_err(|e| XmlError::FromStr(e.into()))?,
                    );
                    ::hard_xml::log_finish_reading_field!(Fmi3Unknown_, dependencies);
                }
                "dependenciesKind" => {
                    ::hard_xml::log_start_reading_field!(Fmi3Unknown_, dependencies_kind);
                    if !false && __self_dependencies_kind.is_some() {
                        return ::std::result::Result::Err(
                            ::hard_xml::XmlExtendedError::DuplicateAttribute(
                                ("dependenciesKind").into(),
                            )
                            .into(),
                        );
                    }
                    __self_dependencies_kind = Some(
                        <AttrList<DependenciesKind> as std::str::FromStr>::from_str(&__value)
                            .map_err(|e| XmlError::FromStr(e.into()))?,
                    );
                    ::hard_xml::log_finish_reading_field!(Fmi3Unknown_, dependencies_kind);
                }
                key => {
                    return Err(XmlError::UnknownField {
                        name: stringify!(Fmi3Unknown_).to_owned(),
                        field: key.to_owned(),
                    });
                }
            }
        }
        if let Token::ElementEnd {
            end: ElementEnd::Empty,
            ..
        } = reader.next().unwrap()?
        {
            let __res = Fmi3Unknown {
                annotations: __self_annotations,
                value_reference: __self_value_reference.ok_or(XmlError::MissingField {
                    name: stringify!(Fmi3Unknown_).to_owned(),
                    field: stringify!(value_reference).to_owned(),
                })?,
                dependencies: __self_dependencies,
                dependencies_kind: __self_dependencies_kind,
            };
            ::hard_xml::log_finish_reading!(Fmi3Unknown_);
            return Ok(__res);
        }
        while let Some(__tag) = reader.find_element_start(Some(tag))? {
            match __tag {
                "Annotations" => {
                    ::hard_xml::log_start_reading_field!(Fmi3Unknown_, annotations);
                    if !true && __self_annotations.is_some() {
                        return ::std::result::Result::Err(
                            ::hard_xml::XmlExtendedError::DuplicateElement(
                                __tag.to_string().into(),
                            )
                            .into(),
                        );
                    }
                    __self_annotations =
                        Some(<Annotations as ::hard_xml::XmlRead>::from_reader(reader)?);
                    ::hard_xml::log_finish_reading_field!(Fmi3Unknown_, annotations);
                }
                tag => {
                    return Err(XmlError::UnknownField {
                        name: stringify!(Fmi3Unknown_).to_owned(),
                        field: tag.to_owned(),
                    });
                }
            }
        }
        let __res = Fmi3Unknown {
            annotations: __self_annotations,
            value_reference: __self_value_reference.ok_or(XmlError::MissingField {
                name: stringify!(Fmi3Unknown_).to_owned(),
                field: stringify!(value_reference).to_owned(),
            })?,
            dependencies: __self_dependencies,
            dependencies_kind: __self_dependencies_kind,
        };
        ::hard_xml::log_finish_reading!(Fmi3Unknown_);
        return Ok(__res);
    }
}

#[derive(PartialEq, Debug, hard_xml::XmlRead, hard_xml::XmlWrite)]
pub enum VariableDependency {
    #[xml(tag = "Output")]
    Output(Fmi3Unknown),
    #[xml(tag = "ContinuousStateDerivative")]
    ContinuousStateDerivative(Fmi3Unknown),
    #[xml(tag = "ClockedState")]
    ClockedState(Fmi3Unknown),
    #[xml(tag = "InitialUnknown")]
    InitialUnknown(Fmi3Unknown),
    #[xml(tag = "EventIndicator")]
    EventIndicator(Fmi3Unknown),
}

#[test]
fn test_dependencies_kind() {
    let _ = env_logger::builder()
        .is_test(true)
        .format_timestamp(None)
        .try_init();

    use hard_xml::{XmlRead, XmlWrite};
    let xml = r#"<Output valueReference="1" dependencies="0 1 2" dependenciesKind="dependent constant fixed"/>"#;

    let var = VariableDependency::from_str(xml).unwrap();

    assert_eq!(
        var,
        VariableDependency::Output(Fmi3Unknown {
            value_reference: 1,
            dependencies: Some(AttrList(vec![0, 1, 2])),
            dependencies_kind: Some(AttrList(vec![
                DependenciesKind::Dependent,
                DependenciesKind::Constant,
                DependenciesKind::Fixed
            ])),
            ..Default::default()
        })
    );

    let xml_out = var.to_string().unwrap();
    assert_eq!(xml_out, xml);
}
