# fmi-export

[<img alt="github" src="https://img.shields.io/github/stars/jondo2010/rust-fmi?style=for-the-badge&logo=github" height="20">](https://github.com/jondo2010/rust-fmi)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fmi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fmi-export)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fmi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fmi-export)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/jondo2010/rust-fmi/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/jondo2010/rust-fmi/actions?query=branch%3Amain)

A Rust interface to FMUs (Functional Mockup Units) that follow the FMI Standard. This crate provides necessary interfaces and utilities to construct FMUs.

See [http://www.fmi-standard.org](http://www.fmi-standard.org)

## Quick start: export an FMU

1) Define a `cdylib` model crate and derive `FmuModel`:

```
use fmi_export::FmuModel;

#[derive(FmuModel, Default, Debug)]
struct MyModel {
    #[variable(causality = Output, start = 1.0)]
    y: f64,
}
```

2) Export FMI symbols:

```
fmi_export::export_fmu!(MyModel);
```

3) Bundle the FMU with `xtask`:

```
cargo run --package xtask -- --package my-model bundle
```

See `docs/building-fmus.md` for a full walkthrough and output location details.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
