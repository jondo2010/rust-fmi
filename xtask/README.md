# FMU Builder - xtask

This directory contains the xtask infrastructure for building FMU (Functional Mockup Interface) packages from Rust examples.

## Overview

The xtask system automates the process of:
1. Building dynamic libraries from Rust FMU examples
2. Creating the proper FMU directory structure
3. Generating model description XML files
4. Packaging everything into a compliant FMU ZIP file
5. Supporting multiple target platforms in a single FMU

## Usage

### Building for Single Platform

Build an FMU for the current platform:

```bash
cargo run --package xtask -- build-fmu \
  --crate-path fmi-export \
  --example bouncing_ball_fmu \
  --output target/fmu
```

Build for a specific target:

```bash
cargo run --package xtask -- build-fmu \
  --crate-path fmi-export \
  --example bouncing_ball_fmu \
  --target x86_64-pc-windows-gnu \
  --output target/fmu \
  --release
```

### Building for Multiple Platforms

Build an FMU containing binaries for multiple platforms:

```bash
cargo run --package xtask -- build-fmu-multi \
  --crate-path fmi-export \
  --example bouncing_ball_fmu \
  --targets x86_64-unknown-linux-gnu,x86_64-pc-windows-gnu,x86_64-apple-darwin \
  --output target/fmu \
  --release
```

### Supported Platforms

The following Rust target -> FMI platform mappings are supported:

| Rust Target                  | FMI Platform     | Library Extension |
|-----------------------------|------------------|-------------------|
| x86_64-unknown-linux-gnu    | x86_64-linux     | .so               |
| aarch64-unknown-linux-gnu   | aarch64-linux    | .so               |
| i686-unknown-linux-gnu      | x86-linux        | .so               |
| x86_64-pc-windows-gnu       | x86_64-windows   | .dll              |
| x86_64-pc-windows-msvc      | x86_64-windows   | .dll              |
| i686-pc-windows-gnu         | x86-windows      | .dll              |
| i686-pc-windows-msvc        | x86-windows      | .dll              |
| x86_64-apple-darwin         | x86_64-darwin    | .dylib            |
| aarch64-apple-darwin        | aarch64-darwin   | .dylib            |

## FMU Structure

The generated FMUs follow the FMI 3.0 standard structure:

```
example.fmu
├── modelDescription.xml          # Model metadata and interface
├── binaries/                    # Platform-specific dynamic libraries
│   ├── x86_64-linux/
│   │   └── example.so
│   ├── x86_64-windows/
│   │   └── example.dll
│   └── x86_64-darwin/
│       └── example.dylib
├── sources/                     # Source code and build information
│   ├── buildDescription.xml
│   └── README.md
└── documentation/               # Optional documentation
    └── index.md
```

## Prerequisites

### Cross-compilation Setup

To build for multiple platforms, you need to install the appropriate Rust targets:

```bash
# Linux targets
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add i686-unknown-linux-gnu

# Windows targets
rustup target add x86_64-pc-windows-gnu
rustup target add i686-pc-windows-gnu

# macOS targets (only available on macOS)
rustup target add x86_64-apple-darwin
rustup target add aarch64-apple-darwin
```

### Cross-compilation Toolchains

For cross-compilation to work, you may need additional toolchains:

#### Windows (from Linux/macOS)
```bash
# On Ubuntu/Debian
sudo apt-get install gcc-mingw-w64

# On macOS with Homebrew
brew install mingw-w64
```

#### Linux ARM (from x86_64)
```bash
# On Ubuntu/Debian
sudo apt-get install gcc-aarch64-linux-gnu
```

## Options

### Common Options

- `--crate-path`: Path to the crate containing the FMU example
- `--example`: Name of the example to build (must be configured as cdylib in Cargo.toml)
- `--output`: Output directory for the FMU file (default: `target/fmu`)
- `--release`: Build in release mode for optimized binaries
- `--model-identifier`: Override the model identifier (defaults to example name)

### Single Platform Options

- `--target`: Specific Rust target to build for (defaults to host target)

### Multi-Platform Options

- `--targets`: Comma-separated list of targets to build for

## Integration with User Projects

To use this xtask system in your own Rust FMI projects:

1. **Copy the xtask directory** to your project root
2. **Add xtask to your workspace** in `Cargo.toml`:
   ```toml
   [workspace]
   members = ["xtask"]
   ```
3. **Configure your examples** as cdylib in your crate's `Cargo.toml`:
   ```toml
   [[example]]
   name = "my_fmu"
   crate-type = ["cdylib"]
   ```
4. **Use the build commands** as shown above

## Future Enhancements

- [ ] Automatic model description generation from Rust derive macros
- [ ] Support for FMI 2.0 in addition to FMI 3.0
- [ ] Validation of generated FMUs
- [ ] Integration with CI/CD pipelines
- [ ] Support for Co-Simulation and Scheduled Execution modes
- [ ] Resource file handling
- [ ] Advanced model metadata extraction

## Troubleshooting

### Build Failures

1. **Missing targets**: Install required Rust targets with `rustup target add <target>`
2. **Cross-compilation issues**: Ensure appropriate cross-compilation toolchains are installed
3. **Library not found**: Check that the example is configured with `crate-type = ["cdylib"]`

### FMU Validation

To validate generated FMUs, you can use tools like:
- FMPy for Python
- FMU Compliance Checker
- Simulation tools that support FMI 3.0

### Platform-Specific Issues

- **Windows**: Ensure MinGW-w64 is installed for cross-compilation
- **macOS**: Some targets may only be available when building on macOS
- **Linux ARM**: May require additional system libraries for cross-compilation
