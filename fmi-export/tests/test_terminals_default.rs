use fmi_export::{
    FmuModel,
    fmi3::{Binary, Clock, Model, TerminalProvider},
};

#[derive(FmuModel, Default)]
#[terminal(name = "Child", matching_rule = "bus")]
struct Child {
    #[variable(causality = Output, start = 0.0)]
    y: f64,
}

#[derive(FmuModel, Default)]
#[terminal(name = "Root", matching_rule = "bus", terminal_kind = "demo.kind")]
struct Root {
    #[variable(causality = Input, start = 0.0)]
    a: f64,
    #[variable(causality = Input, start = 0)]
    b: i32,
    #[variable(causality = Input, start = false)]
    c: bool,
    #[variable(causality = Input, start = "".to_string())]
    d: String,
    #[variable(causality = Input, max_size = 16, start = b"")]
    e: Binary,
    #[variable(causality = Input, interval_variability = Triggered)]
    clk: Clock,

    #[child(prefix = "Child")]
    #[terminal(name = "ChildTerm")]
    child: Child,
}

#[test]
fn terminals_include_all_variables_and_children() {
    let metadata = Root::build_toplevel_metadata();
    let terminals = metadata.terminals.expect("terminals present");
    let root_terminals = terminals.terminals.expect("root terminals");
    assert_eq!(root_terminals.terminals.len(), 1);

    let root_terminal = &root_terminals.terminals[0];
    assert_eq!(root_terminal.name, "Root");
    assert_eq!(root_terminal.matching_rule, "bus");
    assert_eq!(root_terminal.terminal_kind.as_deref(), Some("demo.kind"));

    let mut member_names: Vec<_> = root_terminal
        .terminal_member_variables
        .iter()
        .map(|member| (member.variable_name.as_str(), member.member_name.as_deref()))
        .collect();
    member_names.sort_by_key(|(name, _)| *name);

    assert_eq!(
        member_names,
        vec![
            ("a", Some("a")),
            ("b", Some("b")),
            ("c", Some("c")),
            ("clk", Some("clk")),
            ("d", Some("d")),
            ("e", Some("e")),
        ]
    );

    assert_eq!(root_terminal.terminals.len(), 1);
    let child_terminal = &root_terminal.terminals[0];
    assert_eq!(child_terminal.name, "ChildTerm");
    assert_eq!(
        child_terminal.terminal_member_variables[0].variable_name,
        "Child.y"
    );
    assert_eq!(
        child_terminal.terminal_member_variables[0]
            .member_name
            .as_deref(),
        Some("Child.y")
    );

    let child_terminal_provider = <Child as TerminalProvider>::terminal("ignored", Some("Child."));
    assert_eq!(child_terminal_provider.terminal_member_variables.len(), 1);
}

#[derive(FmuModel, Default)]
struct PassiveChild {
    #[variable(causality = Output, start = 1.0)]
    z: f64,
}

#[derive(FmuModel, Default)]
struct RootWithoutTerminal {
    #[child(prefix = "Inner")]
    inner: PassiveChild,
}

#[test]
fn terminals_are_opt_in_at_struct_level() {
    let metadata = RootWithoutTerminal::build_toplevel_metadata();
    assert!(metadata.terminals.is_none());
}

#[derive(FmuModel, Default)]
#[terminal(name = "ActiveChild")]
struct ActiveChild {
    #[variable(causality = Output, start = 2.0)]
    value: f64,
}

#[derive(FmuModel, Default)]
struct RootWithActiveChild {
    #[child(prefix = "Active")]
    #[terminal(name = "ActiveTerminal")]
    inner: ActiveChild,
}

#[test]
fn terminals_still_discover_children_recursively() {
    let metadata = RootWithActiveChild::build_toplevel_metadata();
    let terminals = metadata.terminals.expect("terminals present");
    let root_terminals = terminals.terminals.expect("root terminals");

    assert_eq!(root_terminals.terminals.len(), 1);
    let active_terminal = &root_terminals.terminals[0];
    assert_eq!(active_terminal.name, "ActiveTerminal");
    assert_eq!(active_terminal.terminal_member_variables.len(), 1);
    assert_eq!(
        active_terminal.terminal_member_variables[0].variable_name,
        "Active.value"
    );
}
