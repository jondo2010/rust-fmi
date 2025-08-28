# fmi-schema

[<img alt="github" src="https://img.shields.io/github/stars/jondo2010/rust-fmi?style=for-the-badge&logo=github" height="20">](https://github.com/jondo2010/rust-fmi)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fmi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fmi-schema)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fmi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fmi-schema)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/jondo2010/rust-fmi/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/jondo2010/rust-fmi/actions?query=branch%3Amain)

XML schema support for FMI 2.0 and 3.0. This crate is part of [rust-fmi](https://github.com/jondo2010/rust-fmi).

The reference XSI can be found at [https://fmi-standard.org/downloads](https://fmi-standard.org/downloads).

## Determining the FMI version

FMI 2.0 and 3.0 have different XML schemas.

The FMI version can initially be determined in a non-specific way by using [`minimal::ModelDescription`].

## Example

The [`FmiModelDescription`] trait is implemented for both FMI2 and FMI3, and has serialize/deserialize methods.

```rust,no_run
#[cfg(feature = "fmi3")]
{
  // deserialize an XML string into a model:
  let xml = std::fs::read_to_string("tests/FMI3.xml").unwrap();
  let model = fmi_schema::fmi3::Fmi3ModelDescription::deserialize(xml.as_str()).unwrap();
  // now serialize it back again:
  let xml = model.serialize().unwrap();
}
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
