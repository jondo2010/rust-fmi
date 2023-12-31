This crate provides a Rust representation of the FMI schema.

The refernce XSI can be found at https://fmi-standard.org/downloads.

## Determining the FMI version

FMI 2.0 and 3.0 have different XML schemas.

The FMI version can initially be determined in a non-specific way by using [`minimal::ModelDescription`].

## Example

```rust,no_run
let md = fmi_schema::fmi3::FmiModelDescription::from_str(
    std::fs::read_to_string("tests/FMI3.xml").unwrap().as_str(),
)
.unwrap();
println!("{}", md.model_name);
```