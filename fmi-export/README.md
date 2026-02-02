# fmi-export

[<img alt="github" src="https://img.shields.io/github/stars/jondo2010/rust-fmi?style=for-the-badge&logo=github" height="20">](https://github.com/jondo2010/rust-fmi)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fmi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fmi-export)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fmi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fmi-export)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/jondo2010/rust-fmi/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/jondo2010/rust-fmi/actions?query=branch%3Amain)

A Rust interface to FMUs (Functional Mockup Units) that follow the FMI Standard. This crate provides necessary interfaces and utilities to construct FMUs.

This crate is part of [rust-fmi](https://github.com/jondo2010/rust-fmi).

See [http://www.fmi-standard.org](http://www.fmi-standard.org)

## Quick start: export an FMU

1) Define a `cdylib` model crate and derive `FmuModel`:

```rust,ignore
use fmi_export::FmuModel;

#[derive(FmuModel, Default, Debug)]
struct MyModel {
    #[variable(causality = Output, start = 1.0)]
    y: f64,
}
```

2) Export FMI symbols:

```rust,ignore
fmi_export::export_fmu!(MyModel);
```

3) Bundle the FMU with `xtask`:

```bash
cargo run --package xtask -- --package my-model bundle
```

## Building FMUs

This repository builds FMI 3.0 FMUs from pure Rust code. The FMI API interfacing boilerplate is generated with the
`FmuModel` derive macro. Automated packaging is handled by an `xtask` module.

### Minimal FMU setup

Your FMU crate must:

- Be a `cdylib`:

```toml
[lib]
crate-type = ["cdylib"]
```

- Derive `FmuModel` for your model struct
- Export FMI symbols via `export_fmu!`

Example skeleton:

```rust,ignore
use fmi_export::FmuModel;

#[derive(FmuModel, Default, Debug)]
struct MyModel {
    #[variable(causality = Output, start = 1.0)]
    y: f64,
}

fmi_export::export_fmu!(MyModel);
```

### Build an FMU (this repo)

From the repository root:

```bash
cargo run xtask -- --package can-triggered-output bundle
```

The FMU zip is written to:

```text
target/fmu/<model_identifier>.fmu
```

`<model_identifier>` is the Rust `cdylib` target name (for `can-triggered-output`, this is
`can_triggered_output`).

### Common options

- Build a release FMU:

```bash
cargo run --package xtask -- --package can-triggered-output bundle --release
```

- Build for a specific target:

```bash
cargo run --package xtask -- --package can-triggered-output bundle --target x86_64-unknown-linux-gnu
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
