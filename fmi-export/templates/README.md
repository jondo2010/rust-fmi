# Rumoca → rust-fmi Template (Jinja)

This directory contains a `minijinja` template for generating Rust FMU code that targets
`rust-fmi` and the `fmi-export` crate.

The template can be used directly via Rumoca, but the **primary workflow** is to enable
the `rumoca` feature in `fmi-export` and call its helper API from `build.rs`. That API
embeds this template and keeps the user-facing integration stable.

## Quick Start

1) Create a new FMU crate:

   ```bash
   cargo fmi new my_fmu_crate
   ```

2) Generate Rust code from a Modelica model using the `fmi-export` Rumoca helper (recommended):

   ```rust
   let model_path = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
       .join("src/model.mo");
   println!("cargo:rerun-if-changed={}", model_path.display());
   fmi_export::rumoca::write_modelica_to_out_dir("MyModel", &model_path)
       .expect("render model.mo");
   ```

   Direct Rumoca usage is also supported:

   ```bash
   rumoca model.mo -m MyModel --template-file /path/to/rust-fmi.jinja > my_fmu_crate/src/lib.rs
   ```

3) Bundle the FMU:

   ```bash
   cargo fmi -p my_fmu_crate bundle
   ```

## What the Template Generates

- A Rust struct named after the Modelica model (`dae.model_name`).
- `#[derive(FmuModel, Default, Debug)]` and explicit ModelExchange configuration:

  ```rust
  #[model(model_exchange = true, co_simulation = false, scheduled_execution = false, user_model = false)]
  ```

- FMI variable metadata derived from Rumoca’s DAE:
  - `causality`, `variability`, `start`, `initial`, `description`.
  - Parameters and constants become `Parameter`/`Fixed` or `Parameter`/`Constant` as appropriate.
  - No `alias` attributes are emitted.

- Derivative mapping:
  - If a variable is defined as `v = der(h)`, the field `v` gets
    `#[variable(derivative = h)]` and is treated as the derivative of `h`.
  - If `der(x)` has no existing variable, a synthetic `der_x` field is generated
    and used as the derivative for `x`.

- Basic event handling:
  - `get_event_indicators` derives indicators from `when` conditions (`dae.fc`).
  - `event_update` applies `reinit` assignments (`dae.fr`) when conditions are true.
  - `pre(x)` is supported by snapshotting variables at the start of `event_update`
    using generated `_pre_*` locals.

## Known Limitations

- Only a subset of Modelica expressions is supported (literals, component refs, +/−/*//, comparisons, `der`, `pre`).
- Complex constructs (array comprehensions, multi-dimensional arrays, advanced control flow) are not yet supported.
- The generated `calculate_values` currently updates algebraics/discretes and synthetic
  derivatives; the solver uses generated metadata and state/derivative mapping for
  ModelExchange.

## Template Location

- Template file: `fmi-export/templates/rust-fmi.jinja`
- This README: `fmi-export/templates/README.md`
