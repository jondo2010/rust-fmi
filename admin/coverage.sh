#!/usr/bin/env bash

# To be used in: "coverage --lcov --output-path lcov.info"

set -ex

# Export environment variables for coverage
eval `cargo llvm-cov show-env --export-prefix "$@"`

cargo llvm-cov clean --workspace

cargo build --locked --all-targets --all-features
cargo test --locked --all-features --workspace

cargo xtask bundle --package bouncing_ball
cargo xtask bundle --package dahlquist
cargo xtask bundle --package stair
cargo xtask bundle --package vanderpol

cargo run --package fmi-sim -- --model target/fmu/bouncing_ball.fmu model-exchange
cargo run --package fmi-sim -- --model target/fmu/dahlquist.fmu model-exchange
cargo run --package fmi-sim -- --model target/fmu/stair.fmu model-exchange
# Disabled until fmi-sim supports arrays
#cargo run --package fmi-sim -- --model target/fmu/vanderpol.fmu model-exchange

cargo llvm-cov report "$@"
