# rust-fmi: A Rust FMI (Functional Mockup Interface) Library

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## MCP / rust-analyzer prompt snippet

Use this in the system or preamble when working with the rust-analyzer MCP server:

```
Always prefer MCP tools over guessing. For any codebase question, call `search` (use regex/includePattern) and then `read` the relevant files; do not manually grep or speculate. Batch related queries when possible. If a tool fails, adjust the query and retry before concluding data is missing. Only cite files/lines that were fetched via tools.
```

## Working Effectively

- **CRITICAL**: Initialize git submodules FIRST before any build operation:
  - `git submodule update --init --recursive` -- REQUIRED: FMI standard headers are in submodules
- Bootstrap and build the project:
  - Ensure C compiler is available: `gcc --version` or `clang --version`
  - `cargo check --all` -- takes ~75 seconds. NEVER CANCEL. Set timeout to 120+ seconds.
  - `cargo build --all` -- takes ~49 seconds. NEVER CANCEL. Set timeout to 90+ seconds.
- **CRITICAL**: Unit tests that work offline:
  - `cargo test --package fmi-schema --lib` -- takes ~12 seconds including compilation. Tests XML schema parsing (34 tests pass)
  - `cargo test --package fmi-sim --lib` -- takes ~7 seconds. Tests simulation internals (3 tests pass)
- **WARNING**: Full test suite requires internet access:
  - `cargo test --all` -- FAILS in restricted environments due to TLS certificate issues downloading Reference-FMUs
  - Integration tests download test data from GitHub and will fail offline
- **IMPORTANT**: When using types from the `fmi-schema` or `fmi-sys` crates in other crates, prefer to use them through the re-exports in the `fmi` crate: `use fmi::fmi3::{schema, binding}`, and with prefix.

## Validation

- ALWAYS manually validate any FMI-related code changes by running unit tests for the specific crate.
- ALWAYS run `cargo check --all` before committing to ensure compilation succeeds.
- Test the fmi-sim CLI tool to validate simulation functionality:
  - `cargo run -p fmi-sim -- --help` -- verify CLI interface works
  - `cargo run -p fmi-sim -- --model /nonexistent/file.fmu co-simulation --help` -- test subcommand help
- Build documentation to check for doc issues:
  - `cargo doc --package fmi-schema --no-deps` -- takes ~4 seconds, generates docs with warnings
- **Validation Scenarios After Changes**:
  - For FMI schema changes: Run `cargo test --package fmi-schema --lib` to validate XML parsing
  - For FMI core changes: Run `cargo test --package fmi --lib` (if unit tests exist)
  - For simulation changes: Run `cargo test --package fmi-sim --lib` to validate solver/interpolation logic
  - For bindings changes: Run `cargo build --package fmi-sys` to validate C bindings compilation
- **DO NOT** attempt to run examples or integration tests without internet access - they will fail

## Repository Structure

This is a Rust workspace with 6 main crates:

| Crate           | Purpose                                        | Key Features                                |
| --------------- | ---------------------------------------------- | ------------------------------------------- |
| `fmi`           | Core FMI library for importing/executing FMUs | FMI 2.0/3.0 support, model importing       |
| `fmi-sys`       | Raw Rust bindings to FMI C API               | Uses bindgen, requires C compiler          |
| `fmi-schema`    | XML parsing of FMU Model Description         | Handles FMI 2.0/3.0 XML schemas            |
| `fmi-sim`       | FMU simulation CLI tool                       | ME/CS/SE simulation modes                   |
| `fmi-test-data` | Reference FMUs for testing                    | Downloads test data from GitHub             |
| `xtask`         | Development automation for FMU building      | Bundle, inspect, multi-platform builds     |

## Common Tasks

The following are tested commands and expected behaviors:

### Build Commands (with timing)
```bash
# Essential preparation
git submodule update --init --recursive  # ~30 seconds, downloads FMI headers

# Core build commands
cargo check --all        # ~75 seconds - NEVER CANCEL. Set timeout to 120+ seconds.
cargo build --all        # ~49 seconds - NEVER CANCEL. Set timeout to 90+ seconds.
cargo build --all --release  # ~163 seconds (2m 42s) - NEVER CANCEL. Set timeout to 300+ seconds.

# Working unit tests
cargo test --package fmi-schema --lib  # ~12 seconds including compilation, 34 tests pass
cargo test --package fmi-sim --lib     # ~7 seconds, 3 tests pass
```

### CLI Tool Usage
```bash
# FMI simulation CLI
cargo run -p fmi-sim -- --help                    # Show main help
cargo run -p fmi-sim -- --model file.fmu --help   # Show model-specific options

# Subcommands available:
# - model-exchange: Perform ModelExchange simulation
# - co-simulation: Perform CoSimulation simulation
```

### XTask Development Tool
The `xtask` crate provides development automation for FMU building and inspection:

```bash
# Show available commands
cargo run -p xtask -- --help

# Bundle a package as FMU for native platform
cargo run -p xtask -- bundle --package dahlquist
cargo run -p xtask -- bundle --package dahlquist --release

# Bundle for specific target platform
cargo run -p xtask -- bundle --package dahlquist --target x86_64-unknown-linux-gnu

# Bundle for multiple platforms (creates multi-platform FMU)
cargo run -p xtask -- bundle-multi --package dahlquist
cargo run -p xtask -- bundle-multi --package dahlquist --targets "aarch64-apple-darwin,x86_64-unknown-linux-gnu"

# Inspect ModelData extracted from dylib (debugging FMU export)
cargo run -p xtask -- inspect --package dahlquist
cargo run -p xtask -- inspect --package stair --target x86_64-unknown-linux-gnu --release

# All commands support:
# --package/-p <PACKAGE>  # Required: name of example package to process
# --target <TARGET>       # Optional: specific target platform
# --release/-r            # Optional: build in release mode
```

**XTask Command Examples**:
- Available example packages: `dahlquist`, `stair`, `vanderpol`, `bouncing_ball`
- Default multi-platform targets: `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-gnu`, `aarch64-apple-darwin`
- Inspect command shows raw XML for ModelVariables and ModelStructure extracted from dylib symbols
- Generated FMUs are placed in `target/fmu/` directory

### Documentation
```bash
cargo doc --package fmi-schema --no-deps  # Generate docs, ~4 seconds
# Generates to target/doc/fmi_schema/index.html
```

### Key Project Files
```
Cargo.toml              # Workspace configuration
fmi-sys/                # C bindings with submodules
├── fmi-standard2/      # FMI 2.0 headers (submodule)
├── fmi-standard3/      # FMI 3.0 headers (submodule)
└── build.rs           # Bindgen build script
fmi-sim/
├── examples/          # Contains bouncing_ball.rs example
└── src/main.rs       # CLI application entry point
xtask/
├── src/
│   ├── main.rs        # CLI entry point
│   ├── commands/      # Command implementations (bundle, inspect, etc.)
│   ├── fmu_builder.rs # Core FMU building logic
│   ├── extractor.rs   # ModelData extraction from dylibs
│   └── platform.rs    # Platform mapping for cross-compilation
└── Cargo.toml        # Development tool dependencies
examples/              # Example FMU packages
├── dahlquist/         # Simple ODE example
├── stair/             # Step function example
├── vanderpol/         # Van der Pol oscillator
└── bouncing_ball/     # Physics simulation example
.github/workflows/ci.yml  # CI configuration
rustfmt.toml          # Code formatting config (requires nightly for full features)
```

## Critical Warnings

- **NEVER CANCEL** build commands - they require significant time for compilation
- **ALWAYS** initialize submodules before building - required for FMI headers
- **DO NOT** expect examples or integration tests to work offline - they require GitHub downloads
- **DO NOT** run full clippy or rustfmt until known issues are resolved
- **ALWAYS** use timeouts of 120+ seconds for check operations, 300+ seconds for release builds
- **INTERNET ACCESS REQUIRED** for integration tests and examples (downloads Reference-FMUs)
- **BUILD TIMING**: Debug builds ~49s, Release builds ~163s (2m 42s) - plan accordingly

## Commit Guidelines

- **REQUIRED**: All commits MUST follow the [Conventional Commits](https://www.conventionalcommits.org/) specification
- **Format**: `<type>[optional scope]: <description>`
- **Types**:
  - `feat`: New features
  - `fix`: Bug fixes
  - `docs`: Documentation changes
  - `style`: Formatting, missing semi-colons, etc (no code change)
  - `refactor`: Code change that neither fixes bug nor adds feature
  - `test`: Adding missing tests or correcting existing tests
  - `chore`: Changes to build process or auxiliary tools
  - `perf`: Performance improvements
  - `ci`: Changes to CI configuration files and scripts
- **Examples**:
  - `feat(fmi-sim): add support for FMI 3.0 simulation`
  - `fix(fmi-schema): resolve XML parsing error for empty elements`
  - `docs: update README with installation instructions`
  - `test(fmi-schema): add unit tests for variable parsing`
  - `chore: update dependencies to latest versions`

## Troubleshooting

**Build failures**: Ensure git submodules are initialized and C compiler is available
**Test failures**: Most likely due to network restrictions - focus on unit tests only
**Examples fail**: Require internet access to download test FMUs from GitHub
