# Development Guide

This project includes development tools to ensure code quality and consistent formatting.

## Prerequisites

```bash
# Initialize git submodules (required for FMI headers)
git submodule update --init --recursive

# Ensure you have a C compiler available
gcc --version  # or clang --version
```

## Development Script

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

## Manual Commands

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

## CI/CD

The project includes two GitHub Actions workflows:

- **`ci.yml`**: Main CI pipeline with build and test across multiple platforms
- **`quality.yml`**: Code quality checks including formatting, clippy, documentation, and security audit

Both workflows enforce:
- Code formatting via `cargo fmt --check`
- Linting via clippy with warnings as errors
- Documentation builds without warnings
- Security audits
