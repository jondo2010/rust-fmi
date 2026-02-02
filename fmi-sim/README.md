# fmi-sim

[<img alt="github" src="https://img.shields.io/github/stars/jondo2010/rust-fmi?style=for-the-badge&logo=github" height="20">](https://github.com/jondo2010/rust-fmi)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fmi.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/fmi-sim)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fmi-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/fmi-sim)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/jondo2010/rust-fmi/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/jondo2010/rust-fmi/actions?query=branch%3Amain)

A pure-Rust FMI simulator framework. This crate is a work-in-progress.

This crate is part of [rust-fmi](https://github.com/jondo2010/rust-fmi).

## Scope

The purpose of `fmi-sim` is to simulate a single `FMI 2.0` or `FMI 3.0` FMU in ME/CS/SE modes as a way to drive testing and API completeness of the `rust-fmi` crates. The simulation algorithms are heavily inspired by those in [fmusim](https://github.com/modelica/Reference-FMUs/tree/main/fmusim).

## Running

```bash
➜ cargo run -p fmi-sim -- --help
    Finished dev [unoptimized + debuginfo] target(s) in 0.08s
     Running `target/debug/fmi-sim --help`
Error: A pure Rust FMI simulator

Usage: fmi-sim [OPTIONS] --model <MODEL> <COMMAND>

Commands:
  model-exchange  Perform a ModelExchange simulation
  co-simulation   Perform a CoSimulation simulation
  help            Print this message or the help of the given subcommand(s)

Options:
      --model <MODEL>              The FMU model to read
  -i, --input-file <INPUT_FILE>    Name of the CSV file name with input data
  -o, --output-file <OUTPUT_FILE>  Simulation result output CSV file name. Default is to use standard output
  -c <SEPARATOR>                   Separator to be used in CSV input/output [default: ,]
  -m                               Mangle variable names to avoid quoting (needed for some CSV importing applications, but not according to the CrossCheck rules)
  -h, --help                       Print help
  -V, --version                    Print version
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
