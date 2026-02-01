# fmi-export

[<img alt="github" src="https://img.shields.io/github/stars/jondo2010/rust-fmi?style=for-the-badge&logo=github" height="20">](https://github.com/jondo2010/rust-fmi)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fmi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fmi-export)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fmi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fmi-export)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/jondo2010/rust-fmi/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/jondo2010/rust-fmi/actions?query=branch%3Amain)

A Rust interface to FMUs (Functional Mockup Units) that follow the FMI Standard. This crate provides necessary interfaces and utilities to construct FMUs.

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

## FmuModel derive reference

### Struct-level attributes

Use `#[model(...)]` on the struct to configure the generated FMI interfaces and
metadata.

```rust,ignore
#[derive(FmuModel, Default)]
#[model(
    description = "Example FMU",
    model_exchange = true,
    co_simulation = false,
    scheduled_execution = false,
    user_model = true,
)]
struct MyModel {
    #[variable(causality = Output, start = 1.0)]
    y: f64,
}
```

Supported keys:

- `description`: Optional string. Defaults to the struct docstring if omitted.
- `model_exchange`: Optional bool. Defaults to `true`.
- `co_simulation`: Optional bool. Defaults to `false`.
- `scheduled_execution`: Optional bool. Defaults to `false`.
- `user_model`: Optional bool. Defaults to `true`. Set `false` to provide your
  own `impl UserModel`.

Notes:

- All boolean flags must be explicit (`co_simulation = true`). Shorthand
  `co_simulation` is rejected.
- `#[model()]` with no arguments is valid and uses the defaults above.

### Field-level attributes

Use `#[variable(...)]` to include a field as an FMI variable. Use `#[alias(...)]`
for additional aliases. Both attributes accept the same keys.

```rust,ignore
#[derive(FmuModel, Default)]
struct MyModel {
    /// Height above ground
    #[variable(causality = Output, start = 1.0)]
    h: f64,

    /// Velocity of the ball
    #[variable(causality = Output, start = 0.0)]
    #[alias(name = "der(h)", causality = Local, derivative = h)]
    v: f64,
}
```

Supported keys for `#[variable(...)]` and `#[alias(...)]`:

- `skip`: Bool. When `true`, the field is ignored for FMI variables.
- `name`: String. Overrides the variable name (defaults to the field name).
- `description`: String. Overrides the field docstring.
- `causality`: One of `Parameter`, `CalculatedParameter`, `Input`, `Output`,
  `Local`, `Independent`, `Dependent`, `StructuralParameter`.
- `variability`: One of `Constant`, `Fixed`, `Tunable`, `Discrete`, `Continuous`.
- `start`: Rust expression used as the start value.
- `initial`: One of `Exact`, `Calculated`, `Approx`.
- `derivative`: Ident referencing another field. Marks this variable as the
  derivative of that field.
- `event_indicator`: Bool. When `true`, counts toward the FMI event indicator
  total.
- `interval_variability`: One of `Constant`, `Fixed`, `Tunable`, `Changing`,
  `Countdown`, `Triggered`.
- `clocks`: List of clock field idents that this variable belongs to.
- `max_size`: Integer. Max size for Binary variables.
- `mime_type`: String. MIME type for Binary variables.

Notes:

- Continuous state variables are inferred by `derivative` relationships.
- `clocks` must reference clock variables in the same model. The generated FMU
  resolves these to value references.

### Child components

Use `#[child(...)]` to reuse another `FmuModel` as a component and prefix its
variable names.

```rust,ignore
#[derive(FmuModel, Default)]
struct Parent {
    #[child(prefix = "bus")]
    bus: CanBus,
}
```

Supported keys:

- `prefix`: Optional string. Defaults to the field name. Child variables are
  named `<parent_prefix><prefix>.<child_variable>`.

Notes:

- Child fields should implement the `Model` trait (typically via `FmuModel`).
- `#[child]` only affects naming and metadata; it does not change runtime
  behavior of the child component.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
