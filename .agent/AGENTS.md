# Repository Guidelines

## Project Structure & Module Organization
This is a Rust workspace. Core crates live in top-level directories such as `fmi`, `fmi-schema`, `fmi-sys`, `fmi-sim`, `fmi-export`, `fmi-export-derive`, and `ssp-schema`. Shared tooling and supporting crates live in `xtask` and `fmi-xtask`, while `fmi-test-data` contains reference FMUs used in tests. Examples are under `examples/`, integration tests under `tests/`, and documentation and notes live in `docs/`, `DEVELOP.md`, and each crate's `README.md`.

## Build, Test, and Development Commands
Use the provided dev script for consistent workflows:
- `./dev.sh format` formats all code with rustfmt.
- `./dev.sh lint` runs clippy across targets/features with warnings as errors.
- `./dev.sh test` runs offline-safe unit tests for `fmi-schema` and `fmi-sim`.
- `./dev.sh build` builds all workspace crates.
- `./dev.sh check-all` runs formatting, lint, docs, and tests.

Manual equivalents include `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test --package fmi-schema --lib`, and `cargo build --all`.

## Coding Style & Naming Conventions
Formatting is enforced by `rustfmt.toml` (edition 2024). Use `cargo fmt --all` before commits. Lint with clippy; warnings are treated as errors. Follow Rust conventions: `snake_case` for modules/functions/variables, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants.

## Testing Guidelines
Prefer the offline-safe unit tests via `./dev.sh test`. Integration tests live in `tests/` and may require FMU assets from `fmi-test-data`. Name test functions descriptively (e.g., `loads_model_description`), and keep tests focused on one behavior.

## Commit & Pull Request Guidelines
Recent history favors Conventional Commits like `fix(fmi-schema): ...` and `chore(deps): ...`. Use `type(scope): summary` where possible; keep subjects short and imperative. PRs should include a concise description, linked issues if applicable, and the commands run (e.g., `./dev.sh pre-commit`). If changes affect FMUs or schemas, call that out explicitly.

## Configuration & Dependencies
Initialize submodules for FMI headers before building: `git submodule update --init --recursive`. A C compiler (`gcc` or `clang`) is required for `fmi-sys` builds.

## Agent-Specific Instructions
When writing complex features or significant refactors, use an ExecPlan (see `.agent/PLANS.md`) from design through implementation. Keep the plan updated as work proceeds.
