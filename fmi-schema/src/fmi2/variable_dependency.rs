use std::{fmt::Display, str::FromStr};

use crate::utils::AttrList;

/// Dependency of scalar Unknown from Knowns in Continuous-Time and Event Mode (ModelExchange), and
/// at Communication Points (CoSimulation): Unknown=f(Known_1, Known_2, ...).
/// The Knowns are "inputs", "continuous states" and "independent variable" (usually time)".
#[derive(Default, PartialEq, Debug)]
pub struct Fmi2VariableDependency {
    /// ScalarVariable index of Unknown
    pub index: u32,

    /// Defines the dependency of the Unknown (directly or indirectly via auxiliary variables) on
    /// the Knowns in Continuous-Time and Event Mode ([`super::ModelExchange`]) and at
    /// Communication Points ([`super::CoSimulation`]).
    ///
    /// If not present, it must be assumed that the Unknown depends on all Knowns. If present as
    /// empty list, the Unknown depends on none of the Knowns. Otherwise the Unknown depends on
    /// the Knowns defined by the given [`super::ScalarVariable`] indices. The indices are
    /// ordered according to size, starting with the smallest index.
    pub dependencies: Vec<u32>,

    /// If not present, it must be assumed that the Unknown depends on the Knowns without a
    /// particular structure. Otherwise, the corresponding Known v enters the equation as:
    ///
    /// * [`DependenciesKind::Dependent`]:  no particular structure, f(v)
    /// * [`DependenciesKind::Constant`]:   constant factor, c*v (only for Real variablse)
    /// * [`DependenciesKind::Fixed`]:      fixed factor, p*v (only for Real variables)
    /// * [`DependenciesKind::Tunable`]:    tunable factor, p*v (only for Real variables)
    /// * [`DependenciesKind::Discrete`]:   discrete factor, d*v (only for Real variables)
    ///
    /// If [`Self::dependencies_kind`] is present, [`Self::dependencies`] must be present and must
    /// have the same number of list elements.
    pub dependencies_kind: Vec<DependenciesKind>,
}

// Custom implementation of XmlRead and XmlWrite for Fmi2VariableDependency to allow multiple tags.
impl<'__input> ::hard_xml::XmlRead<'__input> for Fmi2VariableDependency {
    fn from_reader(reader: &mut ::hard_xml::XmlReader<'__input>) -> ::hard_xml::XmlResult<Self> {
        use ::hard_xml::XmlError;
        use ::hard_xml::xmlparser::{ElementEnd, Token};

        let mut __self_index = None;
        let mut __self_dependencies = Vec::new();
        let mut __self_dependencies_kind = Vec::new();

        let tag = reader
            .find_element_start(None)?
            .expect("Expected start element");
        let _ = reader.next().unwrap()?;

        while let Some((__key, __value)) = reader.find_attribute()? {
            match __key {
                "index" => {
                    __self_index = Some(
                        <u32 as std::str::FromStr>::from_str(&__value)
                            .map_err(|e| XmlError::FromStr(e.into()))?,
                    );
                }
                "dependencies" => {
                    let attr_list = <AttrList<u32> as std::str::FromStr>::from_str(&__value)
                        .map_err(|e| XmlError::FromStr(e.into()))?;
                    __self_dependencies = attr_list.0;
                }
                "dependenciesKind" => {
                    let attr_list =
                        <AttrList<DependenciesKind> as std::str::FromStr>::from_str(&__value)
                            .map_err(|e| XmlError::FromStr(e.into()))?;
                    __self_dependencies_kind = attr_list.0;
                }
                key => {
                    return Err(XmlError::UnknownField {
                        name: "Fmi2VariableDependency".to_owned(),
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
            return Ok(Fmi2VariableDependency {
                index: __self_index.ok_or(XmlError::MissingField {
                    name: "Fmi2VariableDependency".to_owned(),
                    field: "index".to_owned(),
                })?,
                dependencies: __self_dependencies,
                dependencies_kind: __self_dependencies_kind,
            });
        }

        if let Some(__tag) = reader.find_element_start(Some(tag))? {
            return Err(XmlError::UnknownField {
                name: "Fmi2VariableDependency".to_owned(),
                field: __tag.to_owned(),
            });
        }

        Ok(Fmi2VariableDependency {
            index: __self_index.ok_or(XmlError::MissingField {
                name: "Fmi2VariableDependency".to_owned(),
                field: "index".to_owned(),
            })?,
            dependencies: __self_dependencies,
            dependencies_kind: __self_dependencies_kind,
        })
    }
}

impl ::hard_xml::XmlWrite for Fmi2VariableDependency {
    fn to_writer<W: std::io::Write>(
        &self,
        writer: &mut ::hard_xml::XmlWriter<W>,
    ) -> ::hard_xml::XmlResult<()> {
        writer.write_element_start("Unknown")?;
        writer.write_attribute("index", &format!("{}", self.index))?;

        if !self.dependencies.is_empty() {
            writer.write_attribute(
                "dependencies",
                &format!("{}", AttrList(self.dependencies.clone())),
            )?;
        }

        if !self.dependencies_kind.is_empty() {
            writer.write_attribute(
                "dependenciesKind",
                &format!("{}", AttrList(self.dependencies_kind.clone())),
            )?;
        }

        writer.write_element_end_empty()?;
        Ok(())
    }
}

#[derive(Clone, Default, PartialEq, Debug)]
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
