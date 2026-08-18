# Dead-Code Cleanup Design

## Goal

Remove code that no longer participates in the Rust FMI workspace while preserving supported behavior and making any public-API compatibility decisions explicit.

## Scope

The cleanup uses two passes.

### Pass 1: Conservative removal

Remove code only when repository evidence proves it cannot participate in a supported build. Candidates include:

- Rust source files that are not reachable from any crate module tree;
- items guarded unconditionally by `#[cfg(false)]`;
- private items with no callers after accounting for generated code, macros, tests, examples, and feature combinations;
- dead-code allowances whose protected value is required only for ownership or FFI lifetime safety remain in place and are documented rather than removed.

The already-reviewed `fmi/src/fmi3/model.rs` and `fmi/src/fmi3/model2.rs` belong to this pass. Removing `model.rs` also removes its disabled module declaration and disabled importer accessor.

### Pass 2: Public-API review

Review exported items with no workspace callers, but do not equate absence of internal callers with dead code. Each candidate is checked against:

- Graft callers and exhaustive indexed references;
- workspace tests, examples, documentation, and generated-code inputs;
- Git history and the current replacement API;
- published-crate compatibility and plausible downstream use.

Remove a public item only when it is demonstrably obsolete, unusable, or superseded and the replacement is clear. Ambiguous candidates remain unchanged and are listed in the pull-request description as possible follow-up work.

## Implementation boundaries

This change does not redesign FMI APIs, add deprecations, change runtime behavior, update dependencies, or clean up code merely for style. Generated bindings and submodule contents are out of scope. Each removal must have direct evidence recorded during the audit.

The implementation will use focused commits: first the conservative removals, then any independently justified public-API removals. If the second pass finds no removal meeting the evidence threshold, it will produce no code change and its candidates will be documented in the pull request.

## Verification

Verification consists of:

1. `cargo fmt --all -- --check`;
2. `cargo test --workspace --all-features`;
3. `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
4. targeted tests for any affected crate;
5. a final diff, public-API, and package-content review;
6. Graft graph refresh after the source removals.

Tests that create temporary projects may require network access to crates.io. Such failures are treated as environmental only after the same test passes with network access.

## Delivery

Push the isolated `chore/remove-dead-code` branch and open a pull request against `main`. The pull request will distinguish conservative removals, public-API decisions, deferred candidates, compatibility impact, and verification evidence.
