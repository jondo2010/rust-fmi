# Implement FMI3 TerminalsAndIcons (Non-Graphical)

This ExecPlan is a living document. The sections `Progress`, `Surprises & Discoveries`, `Decision Log`, and `Outcomes & Retrospective` must be kept up to date as work proceeds.

PLANS.md is not present in the repo. This document follows the bundled ExecPlan requirements.

## Purpose / Big Picture

After this change, the `fmi-schema` crate can parse and serialize the FMI 3.0 `fmiTerminalsAndIcons` XML file that describes terminals and terminal member variables. A user can deserialize an XML file (for example, the `fmi-sys/fmi-ls-bus/docs/examples/X_network4FMI_terminalsAndIcons_lowCut.xml` sample) into Rust structs and then serialize back to equivalent XML without losing terminal structure. The graphical elements (icons and coordinates) are intentionally omitted for now, so behavior is limited to non-graphical terminal data.

## Progress

- [x] (2026-02-06 00:00Z) Drafted ExecPlan for non-graphical `fmiTerminalsAndIcons` support in `fmi-schema`.
- [x] (2026-02-07 01:05Z) Add new FMI3 module types for `fmiTerminalsAndIcons` and terminal member variables (non-graphical subset).
- [x] (2026-02-07 01:07Z) Wire new module into `fmi-schema/src/fmi3/mod.rs` exports and crate documentation.
- [x] (2026-02-07 01:12Z) Implement matching rule helpers and validation per FMI 3.0 spec (plug/bus/sequence).
- [x] (2026-02-07 01:18Z) Add parsing and serialization tests using an example terminals XML (non-graphical).
- [x] (2026-02-07 01:25Z) Design and add an ergonomic "integrated" API to resolve terminal member variables against `ModelVariables`.
- [x] (2026-02-07 01:33Z) Run `fmi-schema` tests with `--features fmi3` and confirm round-trip behavior.

## Surprises & Discoveries

- Observation: 
  Evidence: 

## Decision Log

- Decision: Implement only non-graphical elements (`Terminals`, `Terminal`, `TerminalMemberVariable`, `TerminalStreamMemberVariable`, `Annotations`) and omit `GraphicalRepresentation`, `Icon`, and `TerminalGraphicalRepresentation`.
  Rationale: The request explicitly asks to skip the graphical portion for now; keeping strict XML parsing means documents with graphical elements will be rejected until a later update.
  Date/Author: 2026-02-06 / Codex
- Decision: Represent schema attributes such as `terminalKind`, `matchingRule`, and `variableKind` as raw `String` values instead of enums.
  Rationale: The XSD defines them as `xs:normalizedString` without a fixed vocabulary; using `String` avoids premature constraints and matches existing style.
  Date/Author: 2026-02-06 / Codex
- Decision: Place new types in `fmi-schema/src/fmi3/terminals_and_icons.rs` and re-export them from `fmi-schema/src/fmi3/mod.rs`.
  Rationale: Keeps FMI3 schema types organized by feature while matching current module layout.
  Date/Author: 2026-02-06 / Codex
- Decision: Add matching rule helpers that interpret `matchingRule` values while preserving raw strings in the XML model.
  Rationale: The spec defines behavior for `plug`, `bus`, and `sequence`; helpers let callers validate data without constraining the schema.
  Date/Author: 2026-02-06 / Codex
- Decision: Provide an optional integrated resolution API that maps terminal member references to `ModelVariables` by name.
  Rationale: Users need an ergonomic way to traverse terminals and access the underlying variables without manually matching names and handling missing links.
  Date/Author: 2026-02-07 / Codex

## Outcomes & Retrospective

- Completed: Added FMI3 terminals parsing/serialization, matching rule helpers, and integrated resolution API with tests.
- Remaining: Graphical representation support (icons/coordinates) remains out of scope for this iteration.
- Lessons: Keeping resolution logic separate from XML structs makes the API ergonomic without constraining schema evolution.

## Context and Orientation

The `fmi-schema` crate is the Rust representation of FMI XML schemas. FMI 3.0 types live in `fmi-schema/src/fmi3/`, with `fmi-schema/src/fmi3/mod.rs` re-exporting them. Parsing and serialization are implemented via `hard_xml` derives on structs with `#[xml(...)]` attributes. Annotations for FMI 3.0 are defined in `fmi-schema/src/fmi3/annotation.rs` as `Annotations` and `Annotation`.

The target schema is defined in `fmi-sys/fmi-standard3/schema/fmi3TerminalsAndIcons.xsd` and `fmi-sys/fmi-standard3/schema/fmi3Terminal.xsd`. The root element is `fmiTerminalsAndIcons` with an `fmiVersion` attribute and optional `Terminals` and `Annotations` elements. Terminals are recursive and can contain terminal member variables and nested terminals. The graphical elements (`GraphicalRepresentation`, `Icon`, and `TerminalGraphicalRepresentation`) are explicitly excluded in this iteration.

Relevant example XML files are in `fmi-sys/fmi-ls-bus/docs/examples/` and do not include graphical elements. These are useful sources for tests and expected structure.

Key terms:
- Terminals and Icons file: The optional FMI 3.0 XML file `terminalsAndIcons.xml` that describes terminal groupings and member variables for an FMU.
- Terminal: A named container with attributes like `matchingRule` and optional `terminalKind` that can include member variables and nested terminals.
- Terminal member variable: A reference to a model variable within a terminal, defined by `variableName`, optional `memberName`, and `variableKind`.
- Terminal stream member variable: A pair of stream variables defined by in/out stream names and variable names.
- Graphical portion: The `GraphicalRepresentation`, `Icon`, and `TerminalGraphicalRepresentation` elements and their coordinate attributes, which are skipped in this plan.

## Plan of Work

Create a new FMI 3.0 module file, `fmi-schema/src/fmi3/terminals_and_icons.rs`, that defines Rust structs mirroring the non-graphical subset of the XSD. The root struct will be `Fmi3TerminalsAndIcons` with an `fmi_version` attribute and optional `Terminals` and `Annotations` children. `Terminals` will contain a list of `Terminal` elements. `Terminal` will be recursive and include lists of `TerminalMemberVariable`, `TerminalStreamMemberVariable`, nested `Terminal` elements, and optional `Annotations`. Each member variable struct will map attribute names directly to string fields, with `Annotations` optional where permitted.

Keep `hard_xml` in strict mode for unknown elements and unknown attributes to align with other schema modules. Because the graphical elements are omitted, any document containing them will currently fail to parse; this is a deliberate limitation to revisit later.

Wire the new module into `fmi-schema/src/fmi3/mod.rs` via `mod terminals_and_icons;` and `pub use terminals_and_icons::*;` so users can import `Fmi3TerminalsAndIcons` alongside existing FMI3 types. No changes are needed in `fmi-schema/src/lib.rs` beyond the re-export.

Add tests under `fmi-schema/tests/` that deserialize a terminals XML file and verify the parsed structure. Use a trimmed, non-graphical example based on `fmi-sys/fmi-ls-bus/docs/examples/X_network4FMI_terminalsAndIcons_lowCut.xml` to validate nested terminals and member variables. Add a serialization round-trip test that serializes the struct back to XML and re-parses it to ensure no data is lost.

Implement matching rule helpers on `Terminal` to interpret `matchingRule` per FMI 3.0: `plug` and `bus` require `memberName` on each `TerminalMemberVariable` and those `memberName` values must be unique within that terminal, while `sequence` allows missing `memberName` and does not require uniqueness checks. Unknown or custom `matchingRule` values should be preserved as raw strings but return a distinct enum variant so downstream code can decide how to behave. These helpers apply only to the `<TerminalMemberVariable>` entries at the same terminal level; nested terminals may have different rules.

Provide an integrated API that lets a user resolve terminal member variable references against the FMI 3.0 `ModelVariables` list. This API should accept a `&Fmi3ModelDescription` (or explicit `&ModelVariables`) plus a `&Fmi3TerminalsAndIcons` and produce a tree of resolved terminal views where each member variable includes a reference to the matching model variable (or a structured error if missing). Keep this API additive and separate from the XML structs so parsing/serialization remains pure.

## Concrete Steps

All commands should be run from `/Users/johhug01/Source/rust-fmi`.

1) Add the new module file.

   Create `fmi-schema/src/fmi3/terminals_and_icons.rs` with the following structs (field names and XML tags must match):

     - `Fmi3TerminalsAndIcons` with `fmi_version: String`, `terminals: Option<Terminals>`, and `annotations: Option<Annotations>`.
     - `Terminals` with `terminals: Vec<Terminal>`.
     - `Terminal` with attributes `name: String`, `matching_rule: String`, `terminal_kind: Option<String>`, `description: Option<String>`, and child lists for `TerminalMemberVariable`, `TerminalStreamMemberVariable`, nested `Terminal`, plus optional `Annotations`.
     - `TerminalMemberVariable` with attributes `variable_name: String`, `member_name: Option<String>`, `variable_kind: String`, plus optional `Annotations`.
     - `TerminalStreamMemberVariable` with attributes `in_stream_member_name: String`, `out_stream_member_name: String`, `in_stream_variable_name: String`, `out_stream_variable_name: String`, plus optional `Annotations`.

   Use `#[xml(tag = "...", strict(unknown_attribute, unknown_element))]` on each struct and `#[xml(attr = "...")]` / `#[xml(child = "...")]` for the fields, matching the XSD element and attribute names.

2) Export the module.

   Update `fmi-schema/src/fmi3/mod.rs` to include:

     - `mod terminals_and_icons;`
     - `pub use terminals_and_icons::*;`

3) Add test fixtures and tests.

   Create `fmi-schema/tests/FMI3TerminalsAndIcons.xml` with a small non-graphical example that includes:
   - Root `<fmiTerminalsAndIcons fmiVersion="3.0">`
   - `<Terminals>` containing at least one `<Terminal>` with nested `<Terminal>` elements
   - `<TerminalMemberVariable>` and `<TerminalStreamMemberVariable>` entries

   Add `fmi-schema/tests/test_fmi3_terminals_and_icons.rs` with two tests (both behind `#[cfg(feature = "fmi3")]`):
   - Parse the XML file and assert the root `fmi_version`, number of terminals, and expected attributes on the nested terminal/member variables.
   - Serialize the parsed struct back to XML and ensure it can be deserialized again with equivalent structure (compare key fields).

4) Implement matching rule helpers and add tests.

   In `fmi-schema/src/fmi3/terminals_and_icons.rs`, add:
   - A `MatchingRule` enum with variants `Plug`, `Bus`, `Sequence`, and `Other(String)`.
   - A `matching_rule_kind(&self) -> MatchingRule` method on `Terminal` that maps the raw `matchingRule` string.
   - A `validate_matching_rule(&self) -> Result<(), Error>` method that checks:
     - For `Plug` and `Bus`: every `TerminalMemberVariable` has `memberName` and all `memberName` values are unique.
     - For `Sequence`: no `memberName` requirements are enforced.
     - For `Other`: return `Ok(())` without validation so custom rules can be handled externally.

   Add test cases for these helpers:
   - `plug` with missing `memberName` yields an error.
   - `bus` with duplicate `memberName` yields an error.
   - `sequence` accepts missing `memberName`.
   - `Other("org.example.rule")` returns `Ok(())`.

5) Add integrated resolution API and tests.

   Add a new module `fmi-schema/src/fmi3/terminals_resolved.rs` (or extend `terminals_and_icons.rs` with a `resolved` sub-module) that defines:
   - `ResolvedTerminals` and `ResolvedTerminal` types that mirror the terminal tree but replace `variable_name` strings with references to `ModelVariables` entries (use `&'a Variable` or `&'a dyn Fmi3VariableLike` depending on existing patterns).
   - A resolver function, e.g. `resolve_terminals(terminals: &Fmi3TerminalsAndIcons, model: &Fmi3ModelDescription) -> Result<ResolvedTerminals<'_>, Error>`.
   - A dedicated error enum for resolution failures that includes terminal path and missing variable name.

   Resolution rules:
   - Use `variableName` to look up the model variable by its exact name (matching `ModelVariables` names).
   - For nested terminals, preserve the terminal hierarchy and include a string path (e.g., `Powertrain/Configuration`) in errors.
   - The resolver should not enforce `matchingRule` semantics beyond optional validation helper calls; keep matching rule validation separate so users can opt in.

   Add tests:
   - Successful resolution against a known `ModelVariables` list (can build a tiny `Fmi3ModelDescription` with a few variables).
   - Failure case: unresolved variable name yields an error that includes the missing name and terminal path.

6) Run tests.

   - `cargo test -p fmi-schema --features fmi3 test_fmi3_terminals_and_icons`
   - `cargo test -p fmi-schema --features fmi3`

   Expected output: all tests pass, and the new tests validate nested terminals and member variable attributes.

## Validation and Acceptance

Acceptance criteria:
- `Fmi3TerminalsAndIcons` can deserialize a non-graphical terminals XML file and preserve terminal/member attributes.
- Serialization round-trip succeeds: serialize -> deserialize yields equivalent terminals and member variables.
- The new types are publicly re-exported from `fmi-schema::fmi3`.
- Matching rule helpers enforce required `memberName` and uniqueness for `plug` and `bus`, and do not block `sequence` or custom rules.
- The integrated resolution API can map terminal member variables to `ModelVariables` entries and reports missing variables with actionable errors.
- Existing FMI3 tests still pass with `--features fmi3`.

Validation commands (run from `/Users/johhug01/Source/rust-fmi`):
- `cargo test -p fmi-schema --features fmi3 test_fmi3_terminals_and_icons`
- `cargo test -p fmi-schema --features fmi3`

## Idempotence and Recovery

All changes are additive. Re-running the tests is safe. If parsing errors occur, confirm the XML fixture does not contain graphical elements and that attribute names match the XSD. Rolling back is as simple as removing the new module and tests.

## Artifacts and Notes

Example minimal XML fixture (non-graphical) to include in `fmi-schema/tests/FMI3TerminalsAndIcons.xml`:

    <?xml version="1.0" encoding="UTF-8"?>
    <fmiTerminalsAndIcons fmiVersion="3.0">
      <Terminals>
        <Terminal terminalKind="org.fmi-ls-bus.network-terminal" name="Powertrain" matchingRule="org.fmi-ls-bus.transceiver">
          <TerminalMemberVariable variableKind="signal" variableName="bus.Tx_Data" memberName="Tx_Data"/>
          <TerminalStreamMemberVariable inStreamMemberName="Rx_Data" outStreamMemberName="Tx_Data"
            inStreamVariableName="bus.Rx_Data" outStreamVariableName="bus.Tx_Data"/>
          <Terminal terminalKind="org.fmi-ls-bus.network-terminal.configuration" name="Configuration" matchingRule="bus">
            <TerminalMemberVariable variableKind="signal" variableName="bus.Config"/>
          </Terminal>
        </Terminal>
      </Terminals>
    </fmiTerminalsAndIcons>

## Interfaces and Dependencies

New FMI3 interfaces:

In `fmi-schema/src/fmi3/terminals_and_icons.rs`, define:

    #[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
    #[xml(tag = "fmiTerminalsAndIcons", strict(unknown_attribute, unknown_element))]
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

    #[derive(Debug, PartialEq, Eq)]
    pub enum MatchingRule {
        Plug,
        Bus,
        Sequence,
        Other(String),
    }

    impl Terminal {
        pub fn matching_rule_kind(&self) -> MatchingRule;
        pub fn validate_matching_rule(&self) -> Result<(), crate::Error>;
    }

    pub struct ResolvedTerminals<'a> { /* holds resolved terminal tree */ }
    pub struct ResolvedTerminal<'a> { /* includes resolved member variables */ }

    pub enum TerminalResolutionError {
        MissingVariable { terminal_path: String, variable_name: String },
    }

    pub fn resolve_terminals<'a>(
        terminals: &'a Fmi3TerminalsAndIcons,
        model: &'a Fmi3ModelDescription,
    ) -> Result<ResolvedTerminals<'a>, TerminalResolutionError>;

    #[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
    #[xml(tag = "TerminalMemberVariable", strict(unknown_attribute, unknown_element))]
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
    #[xml(tag = "TerminalStreamMemberVariable", strict(unknown_attribute, unknown_element))]
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

Dependencies:
- No new dependencies are required beyond existing `hard_xml` and `fmi-schema` types.

## Plan Change Note

Added explicit matching rule helper and validation steps, plus an integrated resolution API for mapping terminal member variables to `ModelVariables` for ergonomic access.
