# A native Rust interface to FMI

[<img alt="github" src="https://img.shields.io/github/stars/jondo2010/rust-fmi?style=for-the-badge&logo=github" height="20">](https://github.com/jondo2010/rust-fmi)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fmi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fmi)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fmi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fmi)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/jondo2010/rust-fmi/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/jondo2010/rust-fmi/actions?query=branch%3Amain)
[<img alt="codecov" src="https://img.shields.io/codecov/c/github/jondo2010/rust-fmi?token=G99W0WOGWG&style=for-the-badge" height="20">](https://codecov.io/gh/jondo2010/rust-fmi)

A Rust interface to FMUs (Functional Mockup Units) that follow the FMI Standard.

See [http://www.fmi-standard.org](http://www.fmi-standard.org)

This repository is composed of the following crates:

| Crate           | Description                                        | Latest API Docs                                | README                        |
| --------------- | -------------------------------------------------- | ---------------------------------------------- | ----------------------------- |
| `fmi`           | Core functionality for importing and excuting FMUs | [docs.rs](https://docs.rs/fmi/latest)          | [(README)][fmi-readme]        |
| `fmi-sys`       | Raw generated Rust bindings to the FMI API         | [docs.rs](https://docs.rs/fmi-sys/latest)      | [(README)][fmi-sys-readme]    |
| `fmi-schema`    | XML parsing of the FMU Model Description           | [docs.rs](https://docs.rs/fmi-schema/latest)   | [(README)][fmi-schema-readme] |
| `fmi-sim`       | Work-in-progress FMU Simulation master             | [docs.rs](https://docs.rs/fmi-sim/latest)      | [(README)][fmi-sim-readme]    |
| `fmi-test-data` | Reference FMUs for testing                         | [docs.rs](https//docs.rs/fmi-test-data/latest) | [(README)][fmi-test-data]     |

## Development

This project includes development tools to ensure code quality and consistent formatting.

### Prerequisites

```bash
# Initialize git submodules (required for FMI headers)
git submodule update --init --recursive

# Ensure you have a C compiler available
gcc --version  # or clang --version
```

### Development Script

Use the included development script for common tasks:

```bash
# Format all code
./dev.sh format

# Check formatting without making changes
./dev.sh check-format

# Run clippy linting
./dev.sh lint

# Run unit tests (offline-safe)
./dev.sh test

# Build the project
./dev.sh build

# Check documentation
./dev.sh docs

# Run all quality checks
./dev.sh check-all

# Prepare for commit (format, lint, test)
./dev.sh pre-commit
```

### Manual Commands

You can also run these commands directly:

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run unit tests (work offline)
cargo test --package fmi-schema --lib
cargo test --package fmi-sim --lib

# Build (takes ~75 seconds for check, ~49 seconds for build)
cargo check --all
cargo build --all
```

### CI/CD

The project includes two GitHub Actions workflows:

- **`ci.yml`**: Main CI pipeline with build and test across multiple platforms
- **`quality.yml`**: Code quality checks including formatting, clippy, documentation, and security audit

Both workflows enforce:
- Code formatting via `cargo fmt --check`
- Linting via clippy with warnings as errors
- Documentation builds without warnings
- Security audits

## License

Licensed under either of
 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[fmi-readme]: fmi/README.md
[fmi-schema-readme]: fmi-schema/README.md
[fmi-sys-readme]: fmi-sys/README.md
[fmi-sim-readme]: fmi-sim/README.md
[fmi-test-data]: fmi-test-data/README.md
