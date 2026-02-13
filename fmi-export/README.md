# fmi-export

[<img alt="github" src="https://img.shields.io/github/stars/jondo2010/rust-fmi?style=for-the-badge&logo=github" height="20">](https://github.com/jondo2010/rust-fmi)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fmi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fmi-export)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fmi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fmi-export)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/jondo2010/rust-fmi/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/jondo2010/rust-fmi/actions?query=branch%3Amain)

A Rust interface to FMUs (Functional Mockup Units) that follow the FMI Standard. This crate provides necessary interfaces and utilities to construct FMUs.

This crate is part of [rust-fmi](https://github.com/jondo2010/rust-fmi).

See [http://www.fmi-standard.org](http://www.fmi-standard.org)

## Quick start: Build an FMU

1) Install the `cargo-fmi` subcommand:

```bash
cargo install cargo-fmi
```

2) Create a new crate for your model using `cargo-fmi`:

```bash
cargo fmi new my-model
```

This will generate a `cdylib` crate with a sample model struct deriving `FmuModel`:

```rust,ignore
use fmi_export::FmuModel;

#[derive(FmuModel, Default, Debug)]
struct MyModel {
    #[variable(causality = Output, start = 1.0)]
    y: f64,
}
```

The `export_fmu!` macro will generate the required FMI-API exports.

```rust,ignore
fmi_export::export_fmu!(MyModel);
```

3) Build and bundle the FMU with `cargo-fmi`:

```bash
cargo fmi --package my-model bundle
```

## Building FMUs

This repository builds FMI 3.0 FMUs from pure Rust code, and is driven by the `FmuModel`
derive macro.

The FMI API interfacing boilerplate is generated, and automated packaging is handled by the
`cargo-fmi` subcommand.

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

### Build an example FMU (this repo)

From the repository root:

```bash
cargo fmi --package can-triggered-output bundle
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
cargo fmi --package can-triggered-output bundle --release
```

- Build for a specific target:

```bash
cargo fmi --package can-triggered-output bundle --target x86_64-unknown-linux-gnu
```

## Building FMUs from Modelica models

Using the [rumoca](https://crates.io/crates/rumoca) crate, `fmi-export` can generate
Rust code from Modelica models.

The template can be used directly via Rumoca, but the **primary workflow** is to enable
the `rumoca` feature in `fmi-export` and call its helper API from `build.rs`.

### Quick Start

1) Add `fmi-export` with the `rumoca` feature to your `Cargo.toml`:

```toml
[build-dependencies]
fmi-export = { version = "0.1.1", features = ["rumoca"] }
```

2) Invoke the Rumoca compiler in your crates' `build.rs`:

```rust,ignore
let model_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
    .join("src/model.mo");
println!("cargo:rerun-if-changed={}", model_path.display());
fmi_export::rumoca::write_modelica_to_out_dir("MyModel", &model_path)
    .expect("render model.mo");
```

Direct Rumoca usage is also supported (path shown is for this repo checkout):

```bash
rumoca model.mo -m MyModel --template-file fmi-export/templates/rust-fmi.jinja > my_fmu_crate/src/lib.rs
```

If you're outside this repo, point `--template-file` at a local copy of
`rust-fmi.jinja` from the `fmi-export/templates` directory.

See [templates/README.md](templates/README.md) and the examples for further details.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
