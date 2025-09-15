#!/usr/bin/env bash

# To be used in: "coverage --lcov --output-path lcov.info"

set -ex

# Export environment variables for coverage
eval "$(cargo llvm-cov show-env --export-prefix)"

cargo llvm-cov clean --workspace

# Build and test with coverage
cargo build --locked --all-targets --all-features
cargo test --locked --all-features --workspace

# Build FMUs with coverage instrumentation (env vars are already set)
cargo xtask --package bouncing_ball bundle
cargo xtask --package dahlquist bundle
cargo xtask --package stair bundle
cargo xtask --package vanderpol bundle

cargo run --package fmi-sim -- --model target/fmu/bouncing_ball.fmu model-exchange --output-interval 0.1
cargo run --package fmi-sim -- --model target/fmu/dahlquist.fmu model-exchange
cargo run --package fmi-sim -- --model target/fmu/stair.fmu model-exchange
# Disabled until fmi-sim supports arrays
#cargo run --package fmi-sim -- --model target/fmu/vanderpol.fmu model-exchange

cargo llvm-cov report "$@"
