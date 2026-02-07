//! Tests for FMI 3.0 terminals and icons schema.

#[test]
#[cfg(feature = "fmi3")]
fn test_fmi3_terminals_and_icons_parse() {
    use fmi_schema::fmi3::Fmi3TerminalsAndIcons;

    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI3TerminalsAndIcons.xml"))
        .unwrap();
    let xml_content = std::fs::read_to_string(test_file).unwrap();
    let terminals: Fmi3TerminalsAndIcons = fmi_schema::deserialize(&xml_content).unwrap();

    assert_eq!(terminals.fmi_version, "3.0");
    let root = terminals.terminals.as_ref().unwrap();
    assert_eq!(root.terminals.len(), 1);
    let powertrain = &root.terminals[0];
    assert_eq!(powertrain.name, "Powertrain");
    assert_eq!(
        powertrain.terminal_kind.as_deref(),
        Some("org.fmi-ls-bus.network-terminal")
    );
    assert_eq!(
        powertrain.matching_rule,
        "org.fmi-ls-bus.transceiver"
    );
    assert_eq!(powertrain.terminal_member_variables.len(), 2);
    assert_eq!(
        powertrain.terminal_member_variables[0].variable_name,
        "bus.Tx_Data"
    );
    assert_eq!(
        powertrain.terminal_member_variables[0]
            .member_name
            .as_deref(),
        Some("Tx_Data")
    );
    assert_eq!(powertrain.terminal_stream_member_variables.len(), 1);
    assert_eq!(powertrain.terminals.len(), 1);
    let config = &powertrain.terminals[0];
    assert_eq!(config.name, "Configuration");
    assert_eq!(config.matching_rule, "bus");
}

#[test]
#[cfg(feature = "fmi3")]
fn test_fmi3_terminals_and_icons_roundtrip() {
    use fmi_schema::fmi3::Fmi3TerminalsAndIcons;

    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI3TerminalsAndIcons.xml"))
        .unwrap();
    let xml_content = std::fs::read_to_string(test_file).unwrap();
    let terminals: Fmi3TerminalsAndIcons = fmi_schema::deserialize(&xml_content).unwrap();

    let xml = fmi_schema::serialize(&terminals, false).unwrap();
    let reparsed: Fmi3TerminalsAndIcons = fmi_schema::deserialize(&xml).unwrap();

    assert_eq!(terminals, reparsed);
}

#[test]
#[cfg(feature = "fmi3")]
fn test_matching_rule_validation() {
    use fmi_schema::{Error, fmi3::{Terminal, TerminalMemberVariable}};

    let plug_terminal = Terminal {
        name: "Plug".to_string(),
        matching_rule: "plug".to_string(),
        terminal_member_variables: vec![TerminalMemberVariable {
            variable_name: "x".to_string(),
            member_name: None,
            variable_kind: "signal".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert!(matches!(
        plug_terminal.validate_matching_rule(),
        Err(Error::Model(_))
    ));

    let bus_terminal = Terminal {
        name: "Bus".to_string(),
        matching_rule: "bus".to_string(),
        terminal_member_variables: vec![
            TerminalMemberVariable {
                variable_name: "x".to_string(),
                member_name: Some("same".to_string()),
                variable_kind: "signal".to_string(),
                ..Default::default()
            },
            TerminalMemberVariable {
                variable_name: "y".to_string(),
                member_name: Some("same".to_string()),
                variable_kind: "signal".to_string(),
                ..Default::default()
            },
        ],
        ..Default::default()
    };
    assert!(matches!(
        bus_terminal.validate_matching_rule(),
        Err(Error::Model(_))
    ));

    let sequence_terminal = Terminal {
        name: "Sequence".to_string(),
        matching_rule: "sequence".to_string(),
        terminal_member_variables: vec![TerminalMemberVariable {
            variable_name: "x".to_string(),
            member_name: None,
            variable_kind: "signal".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert!(sequence_terminal.validate_matching_rule().is_ok());

    let custom_terminal = Terminal {
        name: "Custom".to_string(),
        matching_rule: "org.example.rule".to_string(),
        terminal_member_variables: vec![TerminalMemberVariable {
            variable_name: "x".to_string(),
            member_name: None,
            variable_kind: "signal".to_string(),
            ..Default::default()
        }],
        ..Default::default()
    };
    assert!(custom_terminal.validate_matching_rule().is_ok());
}

#[test]
#[cfg(feature = "fmi3")]
fn test_terminals_resolution() {
    use fmi_schema::fmi3::{
        Fmi3ModelDescription, Fmi3TerminalsAndIcons, FmiFloat64, ModelVariables, Terminal,
        TerminalMemberVariable, TerminalResolutionError, Terminals, Variable,
        resolve_terminals,
    };

    let model = Fmi3ModelDescription {
        fmi_version: "3.0".to_string(),
        model_name: "Test".to_string(),
        instantiation_token: "token".to_string(),
        model_variables: ModelVariables {
            variables: vec![
                Variable::Float64(FmiFloat64 {
                    name: "bus.Tx_Data".to_string(),
                    value_reference: 1,
                    ..Default::default()
                }),
                Variable::Float64(FmiFloat64 {
                    name: "bus.Rx_Data".to_string(),
                    value_reference: 2,
                    ..Default::default()
                }),
                Variable::Float64(FmiFloat64 {
                    name: "bus.Config".to_string(),
                    value_reference: 3,
                    ..Default::default()
                }),
            ],
        },
        ..Default::default()
    };

    let terminals = Fmi3TerminalsAndIcons {
        fmi_version: "3.0".to_string(),
        terminals: Some(Terminals {
            terminals: vec![Terminal {
                name: "Powertrain".to_string(),
                matching_rule: "bus".to_string(),
                terminal_member_variables: vec![
                    TerminalMemberVariable {
                        variable_name: "bus.Tx_Data".to_string(),
                        member_name: Some("Tx_Data".to_string()),
                        variable_kind: "signal".to_string(),
                        ..Default::default()
                    },
                    TerminalMemberVariable {
                        variable_name: "bus.Rx_Data".to_string(),
                        member_name: Some("Rx_Data".to_string()),
                        variable_kind: "signal".to_string(),
                        ..Default::default()
                    },
                ],
                terminals: vec![Terminal {
                    name: "Configuration".to_string(),
                    matching_rule: "bus".to_string(),
                    terminal_member_variables: vec![TerminalMemberVariable {
                        variable_name: "bus.Config".to_string(),
                        member_name: Some("Config".to_string()),
                        variable_kind: "signal".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }),
        ..Default::default()
    };

    let resolved = resolve_terminals(&terminals, &model).unwrap();
    assert_eq!(resolved.terminals.len(), 1);
    assert_eq!(resolved.terminals[0].members.len(), 2);
    assert_eq!(resolved.terminals[0].terminals.len(), 1);

    let missing_model = Fmi3ModelDescription {
        model_variables: ModelVariables {
            variables: vec![
                Variable::Float64(FmiFloat64 {
                    name: "bus.Tx_Data".to_string(),
                    value_reference: 1,
                    ..Default::default()
                }),
                Variable::Float64(FmiFloat64 {
                    name: "bus.Rx_Data".to_string(),
                    value_reference: 2,
                    ..Default::default()
                }),
            ],
        },
        ..model
    };

    let err = match resolve_terminals(&terminals, &missing_model) {
        Ok(_) => panic!("Expected resolution to fail for missing variable"),
        Err(err) => err,
    };
    assert_eq!(
        err,
        TerminalResolutionError::MissingVariable {
            terminal_path: "Powertrain/Configuration".to_string(),
            variable_name: "bus.Config".to_string(),
        }
    );
}
