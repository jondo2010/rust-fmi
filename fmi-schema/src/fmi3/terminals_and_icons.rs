use std::collections::{HashMap, HashSet};

use crate::{Error, fmi3::variable::AbstractVariableTrait};

use super::{Annotations, Fmi3ModelDescription};

#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(
    tag = "fmiTerminalsAndIcons",
    strict(unknown_attribute, unknown_element)
)]
pub struct Fmi3TerminalsAndIcons {
    #[xml(attr = "fmiVersion")]
    pub fmi_version: String,
    #[xml(child = "Terminals")]
    pub terminals: Option<Terminals>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
}

#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Terminals", strict(unknown_attribute, unknown_element))]
pub struct Terminals {
    #[xml(child = "Terminal")]
    pub terminals: Vec<Terminal>,
}

#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Terminal", strict(unknown_attribute, unknown_element))]
pub struct Terminal {
    #[xml(child = "TerminalMemberVariable")]
    pub terminal_member_variables: Vec<TerminalMemberVariable>,
    #[xml(child = "TerminalStreamMemberVariable")]
    pub terminal_stream_member_variables: Vec<TerminalStreamMemberVariable>,
    #[xml(child = "Terminal")]
    pub terminals: Vec<Terminal>,
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    #[xml(attr = "name")]
    pub name: String,
    #[xml(attr = "matchingRule")]
    pub matching_rule: String,
    #[xml(attr = "terminalKind")]
    pub terminal_kind: Option<String>,
    #[xml(attr = "description")]
    pub description: Option<String>,
}

#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(
    tag = "TerminalMemberVariable",
    strict(unknown_attribute, unknown_element)
)]
pub struct TerminalMemberVariable {
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    #[xml(attr = "variableName")]
    pub variable_name: String,
    #[xml(attr = "memberName")]
    pub member_name: Option<String>,
    #[xml(attr = "variableKind")]
    pub variable_kind: String,
}

#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(
    tag = "TerminalStreamMemberVariable",
    strict(unknown_attribute, unknown_element)
)]
pub struct TerminalStreamMemberVariable {
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
    #[xml(attr = "inStreamMemberName")]
    pub in_stream_member_name: String,
    #[xml(attr = "outStreamMemberName")]
    pub out_stream_member_name: String,
    #[xml(attr = "inStreamVariableName")]
    pub in_stream_variable_name: String,
    #[xml(attr = "outStreamVariableName")]
    pub out_stream_variable_name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum MatchingRule {
    Plug,
    Bus,
    Sequence,
    Other(String),
}

impl Terminal {
    pub fn matching_rule_kind(&self) -> MatchingRule {
        match self.matching_rule.as_str() {
            "plug" => MatchingRule::Plug,
            "bus" => MatchingRule::Bus,
            "sequence" => MatchingRule::Sequence,
            other => MatchingRule::Other(other.to_string()),
        }
    }

    pub fn validate_matching_rule(&self) -> Result<(), Error> {
        match self.matching_rule_kind() {
            MatchingRule::Plug | MatchingRule::Bus => {
                let mut seen = HashSet::new();
                for member in &self.terminal_member_variables {
                    let member_name = member.member_name.as_deref().ok_or_else(|| {
                        Error::Model(format!(
                            "Terminal '{}' requires memberName for matchingRule '{}'",
                            self.name, self.matching_rule
                        ))
                    })?;
                    if !seen.insert(member_name) {
                        return Err(Error::Model(format!(
                            "Terminal '{}' has duplicate memberName '{}' for matchingRule '{}'",
                            self.name, member_name, self.matching_rule
                        )));
                    }
                }
            }
            MatchingRule::Sequence | MatchingRule::Other(_) => {}
        }
        Ok(())
    }
}

pub struct ResolvedTerminals<'a> {
    pub terminals: Vec<ResolvedTerminal<'a>>,
}

pub struct ResolvedTerminal<'a> {
    pub terminal: &'a Terminal,
    pub members: Vec<ResolvedTerminalMemberVariable<'a>>,
    pub stream_members: Vec<ResolvedTerminalStreamMemberVariable<'a>>,
    pub terminals: Vec<ResolvedTerminal<'a>>,
}

pub struct ResolvedTerminalMemberVariable<'a> {
    pub member: &'a TerminalMemberVariable,
    pub variable: &'a dyn AbstractVariableTrait,
}

pub struct ResolvedTerminalStreamMemberVariable<'a> {
    pub member: &'a TerminalStreamMemberVariable,
    pub in_stream_variable: &'a dyn AbstractVariableTrait,
    pub out_stream_variable: &'a dyn AbstractVariableTrait,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TerminalResolutionError {
    MissingVariable {
        terminal_path: String,
        variable_name: String,
    },
}

pub fn resolve_terminals<'a>(
    terminals: &'a Fmi3TerminalsAndIcons,
    model: &'a Fmi3ModelDescription,
) -> Result<ResolvedTerminals<'a>, TerminalResolutionError> {
    let Some(root) = terminals.terminals.as_ref() else {
        return Ok(ResolvedTerminals {
            terminals: Vec::new(),
        });
    };

    let lookup = build_variable_lookup(&model.model_variables);
    let mut resolved = Vec::with_capacity(root.terminals.len());
    for terminal in &root.terminals {
        resolved.push(resolve_terminal(terminal, &lookup, terminal.name.as_str())?);
    }
    Ok(ResolvedTerminals {
        terminals: resolved,
    })
}

fn build_variable_lookup<'a>(
    model_variables: &'a super::variable::ModelVariables,
) -> HashMap<&'a str, &'a dyn AbstractVariableTrait> {
    let mut lookup = HashMap::new();
    for var in model_variables.iter_abstract() {
        lookup.insert(var.name(), var);
    }
    lookup
}

fn resolve_terminal<'a>(
    terminal: &'a Terminal,
    lookup: &HashMap<&'a str, &'a dyn AbstractVariableTrait>,
    terminal_path: &str,
) -> Result<ResolvedTerminal<'a>, TerminalResolutionError> {
    let mut members = Vec::with_capacity(terminal.terminal_member_variables.len());
    for member in &terminal.terminal_member_variables {
        let variable = *lookup.get(member.variable_name.as_str()).ok_or_else(|| {
            TerminalResolutionError::MissingVariable {
                terminal_path: terminal_path.to_string(),
                variable_name: member.variable_name.clone(),
            }
        })?;
        members.push(ResolvedTerminalMemberVariable { member, variable });
    }

    let mut stream_members = Vec::with_capacity(terminal.terminal_stream_member_variables.len());
    for member in &terminal.terminal_stream_member_variables {
        let in_stream_variable = *lookup
            .get(member.in_stream_variable_name.as_str())
            .ok_or_else(|| TerminalResolutionError::MissingVariable {
                terminal_path: terminal_path.to_string(),
                variable_name: member.in_stream_variable_name.clone(),
            })?;
        let out_stream_variable = *lookup
            .get(member.out_stream_variable_name.as_str())
            .ok_or_else(|| TerminalResolutionError::MissingVariable {
                terminal_path: terminal_path.to_string(),
                variable_name: member.out_stream_variable_name.clone(),
            })?;
        stream_members.push(ResolvedTerminalStreamMemberVariable {
            member,
            in_stream_variable,
            out_stream_variable,
        });
    }

    let mut terminals = Vec::with_capacity(terminal.terminals.len());
    for child in &terminal.terminals {
        let child_path = format!("{}/{}", terminal_path, child.name);
        terminals.push(resolve_terminal(child, lookup, &child_path)?);
    }

    Ok(ResolvedTerminal {
        terminal,
        members,
        stream_members,
        terminals,
    })
}
