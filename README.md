# fmi

[<img alt="github" src="https://img.shields.io/github/stars/jondo2010/rust-fmi?style=for-the-badge&logo=github" height="20">](https://github.com/jondo2010/rust-fmi)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fmi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fmi)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fmi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fmi)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/jondo2010/rust-fmi/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/jondo2010/rust-fmi/actions?query=branch%3Amain)
[<img alt="codecov" src="https://img.shields.io/codecov/c/github/jondo2010/rust-fmi?token=G99W0WOGWG&style=for-the-badge" height="20">](https://codecov.io/gh/jondo2010/rust-fmi)

A Rust interface to FMUs (Functional Mockup Units) that follow the FMI Standard.

See [http://www.fmi-standard.org](http://www.fmi-standard.org)

## Importing FMUs

The `fmi` crate implements a Rust interface to FMUs (Functional Mockup Units) that follow the FMI
Standard. This version of the library supports FMI 2.0 and 3.0.

### Loading an FMI 2.0 FMU

```rust,no_run
use fmi::{fmi2::import::Fmi2Import, import, traits::{FmiImport, FmiInstance}};

// Load an FMU from a file path
let import: Fmi2Import = import::from_path("path/to/model.fmu").unwrap();
assert_eq!(import.model_description().fmi_version, "2.0");

// Create a Model Exchange instance
let me = import.instantiate_me("inst1", false, true).unwrap();
assert_eq!(me.get_version(), "2.0");
```

### Loading an FMI 3.0 FMU

```rust,no_run
use fmi::{fmi3::{import::Fmi3Import, Fmi3Model}, import, traits::{FmiImport, FmiInstance}};

// Load an FMU from a file path
let import: Fmi3Import = import::from_path("path/to/model.fmu").unwrap();
assert_eq!(import.model_description().fmi_version, "3.0");

// Create a Model Exchange instance
let me = import.instantiate_me("inst1", false, true).unwrap();
assert_eq!(me.get_version(), "3.0");
```

### Checking FMU version before loading

```rust,no_run
use fmi::{import, schema::{MajorVersion, traits::FmiModelDescription}};

// Peek at the FMU metadata without fully extracting it
let model_desc = import::peek_descr_path("path/to/model.fmu").unwrap();
let version = model_desc.major_version().unwrap();
match version {
    MajorVersion::FMI2 => {
        // Load as FMI 2.0
        let import: fmi::fmi2::import::Fmi2Import = import::from_path("path/to/model.fmu").unwrap();
        // ... use import
    }
    MajorVersion::FMI3 => {
        // Load as FMI 3.0
        let import: fmi::fmi3::import::Fmi3Import = import::from_path("path/to/model.fmu").unwrap();
        // ... use import
    }
    _ => panic!("Unsupported FMI version"),
}
```

## Exporting FMUs

For exporting FMUs, use the `fmi-export` crate, which provides the traits and helper types for
building FMUs in Rust. See the `fmi-export` documentation on
[docs.rs](https://docs.rs/fmi-export/latest).

See the [`fmi-export` README][fmi-export-readme] for the step-by-step workflow and expected output paths.


## Repository Structure

This repository is composed of the following crates:

| Crate               | Description                                         | Latest API Docs                                     | README                        |
| ------------------- | --------------------------------------------------- | --------------------------------------------------- | ----------------------------- |
| `fmi`               | Core functionality for importing and executing FMUs | [docs.rs](https://docs.rs/fmi/latest)               | [README][fmi-readme]          |
| `fmi-sys`           | Raw generated Rust bindings to the FMI API          | [docs.rs](https://docs.rs/fmi-sys/latest)           | [README][fmi-sys-readme]      |
| `fmi-schema`        | XML parsing of the FMU Model Description            | [docs.rs](https://docs.rs/fmi-schema/latest)        | [README][fmi-schema-readme]   |
| `fmi-sim`           | Work-in-progress FMU Simulation master              | [docs.rs](https://docs.rs/fmi-sim/latest)           | [README][fmi-sim-readme]      |
| `fmi-test-data`     | Reference FMUs for testing                          | [docs.rs](https://docs.rs/fmi-test-data/latest)     | [README][fmi-test-data-readme]|
| `fmi-export`        | Types and traits necessary for exporting FMUs       | [docs.rs](https://docs.rs/fmi-export/latest)        | [README][fmi-export-readme]   |
| `fmi-export-derive` | Procedural macros for `fmi-export`                  | [docs.rs](https://docs.rs/fmi-export-derive/latest) | [README][fmi-export-derive-readme] |
| `fmi-ls-bus`        | FMI-LS-BUS support                                  | [docs.rs](https://docs.rs/fmi-ls-bus/latest)        | [README][fmi-ls-bus-readme]   |
| `cargo-fmi`         | Cargo subcommand for FMI packaging                  | n/a                                                 | [README][cargo-fmi-readme]    |

## Development

For development information, build instructions, and contribution guidelines, see [DEVELOP.md][develop-readme].

## License

Licensed under either of
 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

[develop-readme]: https://github.com/jondo2010/rust-fmi/blob/main/DEVELOP.md
[fmi-readme]: https://github.com/jondo2010/rust-fmi/blob/main/fmi/README.md
[fmi-schema-readme]: https://github.com/jondo2010/rust-fmi/blob/main/fmi-schema/README.md
[fmi-sys-readme]: https://github.com/jondo2010/rust-fmi/blob/main/fmi-sys/README.md
[fmi-sim-readme]: https://github.com/jondo2010/rust-fmi/blob/main/fmi-sim/README.md
[fmi-test-data-readme]: https://github.com/jondo2010/rust-fmi/blob/main/fmi-test-data/README.md
[fmi-export-readme]: https://github.com/jondo2010/rust-fmi/blob/main/fmi-export/README.md
[fmi-export-derive-readme]: https://github.com/jondo2010/rust-fmi/blob/main/fmi-export-derive/README.md
[fmi-ls-bus-readme]: https://github.com/jondo2010/rust-fmi/blob/main/fmi-ls-bus/README.md
[cargo-fmi-readme]: https://github.com/jondo2010/rust-fmi/blob/main/cargo-fmi/README.md
