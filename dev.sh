#!/bin/bash
# Development script for rust-fmi project
# Runs common development tasks including formatting, linting, and testing

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

echo_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

echo_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if git submodules are initialized
check_submodules() {
    echo_status "Checking git submodules..."
    if ! git submodule status | grep -q "^+\|^-"; then
        echo_status "Git submodules are up to date"
    else
        echo_warning "Git submodules need to be updated"
        git submodule update --init --recursive
    fi
}

# Function to format code
format() {
    echo_status "Formatting code..."
    cargo fmt --all
    echo_status "Code formatting complete"
}

# Function to check formatting
check_format() {
    echo_status "Checking code formatting..."
    echo_warning "Note: Formatting check may show warnings for nightly-only rustfmt options"
    if cargo fmt --all --check; then
        echo_status "Code formatting is correct"
    else
        echo_warning "Code formatting differences found."
        echo_warning "This may be due to rustfmt.toml containing nightly-only options."
        echo_warning "Consider running 'cargo fmt --all' with the stable toolchain."
        return 1
    fi
}

# Function to run clippy
lint() {
    echo_status "Running clippy..."
    cargo clippy --all-targets --all-features -- -D warnings
    echo_status "Clippy checks complete"
}

# Function to run tests
test() {
    echo_status "Running tests..."
    # Run only unit tests that work offline per the instructions
    cargo test --package fmi-schema --lib
    cargo test --package fmi-sim --lib
    echo_status "Unit tests complete"
}

# Function to build the project
build() {
    echo_status "Building project..."
    cargo build --all
    echo_status "Build complete"
}

# Function to check documentation
docs() {
    echo_status "Checking documentation..."
    cargo doc --all --no-deps --document-private-items
    echo_status "Documentation check complete"
}

# Function to run all checks
check_all() {
    echo_status "Running all quality checks..."
    check_submodules
    check_format
    lint
    docs
    test
    echo_status "All checks passed!"
}

# Function to build FMU examples
build_fmu_examples() {
    echo_status "Building FMU examples..."
    check_submodules
    
    # Build the xtask first
    echo_status "Building xtask..."
    cargo build --package xtask
    
    # Build example FMUs
    echo_status "Building bouncing_ball_fmu example..."
    cargo run --package xtask -- build-fmu \
        --crate-path fmi-export \
        --example bouncing_ball_fmu \
        --output target/fmu \
        --release
    
    echo_status "FMU examples built successfully!"
    echo_status "Output location: target/fmu/"
}

# Function to build multi-platform FMUs
build_fmu_multi() {
    echo_status "Building multi-platform FMU examples..."
    check_submodules
    
    # Build the xtask first
    echo_status "Building xtask..."
    cargo build --package xtask
    
    # Check available targets
    local targets="x86_64-unknown-linux-gnu"
    if command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
        targets="$targets,x86_64-pc-windows-gnu"
        echo_status "Windows cross-compilation available"
    else
        echo_warning "Windows cross-compilation not available (install mingw-w64)"
    fi
    
    # Build multi-platform FMU
    echo_status "Building bouncing_ball_fmu for platforms: $targets"
    cargo run --package xtask -- build-fmu-multi \
        --crate-path fmi-export \
        --example bouncing_ball_fmu \
        --targets "$targets" \
        --output target/fmu \
        --release
    
    echo_status "Multi-platform FMU built successfully!"
    echo_status "Output location: target/fmu/"
}

# Function to prepare for commit
pre_commit() {
    echo_status "Running pre-commit checks..."
    check_submodules
    format
    lint
    test
    echo_status "Ready to commit!"
}

# Function to show help
show_help() {
    echo "rust-fmi development script"
    echo ""
    echo "Usage: ./dev.sh [command]"
    echo ""
    echo "Commands:"
    echo "  format       Format all code"
    echo "  check-format Check if code is formatted"
    echo "  lint         Run clippy linting"
    echo "  test         Run unit tests (offline-safe)"
    echo "  build        Build the project"
    echo "  docs         Check documentation"
    echo "  check-all    Run all quality checks"
    echo "  pre-commit   Format, lint, and test (ideal before committing)"
    echo "  build-fmu    Build FMU examples"
    echo "  build-fmu-multi Build multi-platform FMU examples"
    echo "  help         Show this help message"
    echo ""
    echo "FMU Building:"
    echo "  The build-fmu commands use the xtask system to create"
    echo "  FMU packages from the examples in fmi-export."
    echo ""
}

# Main script logic
case "${1:-help}" in
    format)
        check_submodules
        format
        ;;
    check-format)
        check_format
        ;;
    lint)
        check_submodules
        lint
        ;;
    test)
        check_submodules
        test
        ;;
    build)
        check_submodules
        build
        ;;
    docs)
        check_submodules
        docs
        ;;
    check-all)
        check_all
        ;;
    pre-commit)
        pre_commit
        ;;
    build-fmu)
        build_fmu_examples
        ;;
    build-fmu-multi)
        build_fmu_multi
        ;;
    help|--help|-h)
        show_help
        ;;
    *)
        echo_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac
