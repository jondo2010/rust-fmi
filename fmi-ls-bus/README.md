# fmi-ls-bus

[<img alt="crates.io" src="https://img.shields.io/crates/v/fmi-ls-bus.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fmi-ls-bus)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fmi--ls--bus-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fmi-ls-bus)

Rust bindings for the FMI-LS-BUS interface, with a safe, ergonomic wrapper around
buffer operations that is binary compatible with the C implementation.

This crate is part of [rust-fmi](https://github.com/jondo2010/rust-fmi).

## Features

- `can`: Enable CAN-related LS-BUS operations.
- `fmi-export`: Enable integrations used by `fmi-export`.

## Usage

```rust
use fmi_ls_bus::FmiLsBus;

let mut bus = FmiLsBus::new();
let mut buffer = [0u8; 256];

bus.reset();
let _start = FmiLsBus::start(&buffer);
```

## Minimum supported Rust version

This crate follows the workspace MSRV policy.

## License

Licensed under either of the following, at your option:

- Apache License, Version 2.0
- MIT license

## License

Licensed under either of
 * Apache License, Version 2.0
   ([LICENSE-APACHE](../LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](../LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
