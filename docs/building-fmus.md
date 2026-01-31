# Building FMUs

This repository builds FMI 3.0 FMUs from Rust `cdylib` crates using the `xtask` tooling.

## Prerequisites

- Rust toolchain installed
- A C compiler (`clang` or `gcc`) for `fmi-sys`
- FMI headers submodule initialized

Run once in the repo root:

    git submodule update --init --recursive

## Minimal FMU setup

Your FMU crate must:

- Be a `cdylib`:

    [lib]
    crate-type = ["cdylib"]

- Derive `FmuModel` for your model struct
- Export FMI symbols via `export_fmu!`

Example skeleton:

    use fmi_export::FmuModel;

    #[derive(FmuModel, Default, Debug)]
    struct MyModel {
        #[variable(causality = Output, start = 1.0)]
        y: f64,
    }

    fmi_export::export_fmu!(MyModel);

## Build an FMU (this repo)

From the repository root:

    cargo run --package xtask -- --package can-triggered-output bundle

The FMU zip is written to:

    target/fmu/<model_identifier>.fmu

`<model_identifier>` is the Rust `cdylib` target name (for `can-triggered-output`, this is `can_triggered_output`).

## Common options

- Build a release FMU:

    cargo run --package xtask -- --package can-triggered-output bundle --release

- Build for a specific target:

    cargo run --package xtask -- --package can-triggered-output bundle --target x86_64-unknown-linux-gnu

