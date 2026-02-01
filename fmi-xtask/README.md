# fmi-xtask

[<img alt="github" src="https://img.shields.io/github/stars/jondo2010/rust-fmi?style=for-the-badge&logo=github" height="20">](https://github.com/jondo2010/rust-fmi)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fmi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fmi-xtask)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fmi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fmi-xtask)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/jondo2010/rust-fmi/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/jondo2010/rust-fmi/actions?query=branch%3Amain)

The `xtask` infrastructure for building FMU (Functional Mockup Interface) packages from Rust crates.

## Overview

The xtask system automates the process of:
1. Building dynamic libraries from Rust FMU dylib crates
2. Creating the proper FMU directory structure
3. Generating model description XML files
4. Packaging everything into a compliant FMU ZIP file

## Usage

### Building for Single Platform

Build an FMU for the current platform:

```bash
cargo run --package xtask -- --package bouncing_ball bundle
```

The FMU zip is written to:

```text
target/fmu/<model_identifier>.fmu
```

`<model_identifier>` is the `cdylib` target name (hyphens become underscores).

Build for a specific target:

```bash
cargo run --package xtask -- \
  --package bouncing_ball \
  bundle \
  --target x86_64-pc-windows-gnu \
  --release
```

### End-to-end example (this repo)

```bash
cargo run --package xtask -- --package can-triggered-output bundle
```

See the `fmi-export` README for a complete walkthrough.

## Integration with User Projects

To use this xtask system in your own Rust FMI projects:

1. **Copy the xtask directory** to your project root
2. **Add xtask to your workspace** in `Cargo.toml`:
  ```toml
  [workspace]
  members = ["xtask"]
  ```
3. **Configure your crate** as cdylib in `Cargo.toml`:
  ```toml
  [lib]
  crate-type = ["cdylib"]
  ```
4. **Use the bundle command** as shown above

### Supported Platforms

The following Rust target -> FMI platform mappings are supported:

| Rust Target                 | FMI Platform     |
|-----------------------------|------------------|
| i686-unknown-linux-gnu      | x86-linux        |
| x86_64-unknown-linux-gnu    | x86_64-linux     |
| aarch64-unknown-linux-gnu   | aarch64-linux    |
| x86_64-pc-windows-gnu       | x86_64-windows   |
| x86_64-pc-windows-msvc      | x86_64-windows   |
| i686-pc-windows-gnu         | x86-windows      |
| i686-pc-windows-msvc        | x86-windows      |
| x86_64-apple-darwin         | x86_64-darwin    |
| aarch64-apple-darwin        | aarch64-darwin   |

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
