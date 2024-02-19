# A native Rust interface to FMI

[<img alt="github" src="https://img.shields.io/github/stars/jondo2010/rust-fmi?style=for-the-badge&logo=github" height="20">](https://github.com/jondo2010/rust-fmi)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fmi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fmi)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fmi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fmi)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/jondo2010/rust-fmi/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/jondo2010/rust-fmi/actions?query=branch%3Amain)

A Rust interface to FMUs (Functional Mockup Units) that follow the FMI Standard.

See [http://www.fmi-standard.org](http://www.fmi-standard.org)

This repository is composed of the following crates:

| Crate        | Description                                        | Latest API Docs                              | README                        |
| ------------ | -------------------------------------------------- | -------------------------------------------- | ----------------------------- |
| `fmi`        | Core functionality for importing and excuting FMUs | [docs.rs](https://docs.rs/fmi/latest)        | [(README)][fmi-readme]        |
| `fmi-schema` | XML parsing of the FMU Model Description           | [docs.rs](https://docs.rs/fmi-schema/latest) | [(README)][fmi-schema-readme] |
| `fmi-sys`    | Raw generated Rust bindings to the FMI API         | [docs.rs](https://docs.rs/fmi-sys/latest)    | [(README)][fmi-sys-readme]    |
| `fmi-sim`    | Work-in-progress FMU Simulation master             | Not Yet Published                            | [(README)][fmi-sim-readme]    |

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