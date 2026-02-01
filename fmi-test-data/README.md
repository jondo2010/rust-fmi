# fmi-test-data

[<img alt="github" src="https://img.shields.io/github/stars/jondo2010/rust-fmi?style=for-the-badge&logo=github" height="20">](https://github.com/jondo2010/rust-fmi)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fmi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fmi)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fmi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fmi)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/jondo2010/rust-fmi/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/jondo2010/rust-fmi/actions?query=branch%3Amain)

Utilities for fetching test data from Modelica's Reference-FMUs repository.

This crate provides easy access to the official [Modelica Reference FMUs](https://github.com/modelica/Reference-FMUs) for testing and validation purposes. It automatically downloads and caches the FMU archive, providing convenient methods to access individual FMUs.

## Features

- **Automatic Download**: Downloads and caches Reference FMUs from the official GitHub repository
- **Version Management**: Easy upgrade path with centralized version constants (currently v0.0.39)
- **FMI Support**: Works with both FMI 2.0 and FMI 3.0 standards
- **Multiple Access Methods**: Load FMUs directly into memory or extract to temporary files
- **Archive Exploration**: List all available FMUs in the archive
- **Integrity Verification**: SHA256 checksum validation of downloaded archives

## Usage

```rust
use fmi_test_data::ReferenceFmus;
use fmi::traits::FmiImport;

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// Create a new instance (downloads archive if needed)
let mut reference_fmus = ReferenceFmus::new()?;

// Load a specific FMU
let fmu: fmi::fmi3::import::Fmi3Import = reference_fmus.get_reference_fmu("BouncingBall")?;

// List all available FMUs
let available_fmus = reference_fmus.list_available_fmus()?;
println!("Available FMUs: {:?}", available_fmus);

// Extract FMU to a temporary file
let temp_file = reference_fmus.extract_reference_fmu("BouncingBall", fmi::schema::MajorVersion::FMI3)?;
# Ok(())
# }
```

## Available FMUs

The Reference FMUs package includes several test models such as:

- **BouncingBall**: A simple bouncing ball simulation
- **Dahlquist**: A test equation for numerical solvers
- **VanDerPol**: Van der Pol oscillator
- **Feedthrough**: Simple input/output feedthrough
- **Clocks**: Clock-based simulation (FMI 3.0)
- And many more...

Use `ReferenceFmus::list_available_fmus()` to get the complete list.

## Version Management

The crate uses version constants for easy upgrades:

```rust
use fmi_test_data::ReferenceFmus;

// Check the current Reference FMUs version
println!("Using Reference FMUs version: {}", ReferenceFmus::version());

// Access version constants
println!("Archive: {}", fmi_test_data::REF_ARCHIVE);
println!("URL: {}", fmi_test_data::REF_URL);
```

To upgrade to a newer version, simply update the `REF_FMU_VERSION` constant in the source code.

## Testing

The crate includes comprehensive tests for all functionality:

```bash
cargo test -p fmi-test-data
```

## References

See [https://github.com/modelica/Reference-FMUs](https://github.com/modelica/Reference-FMUs)

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
