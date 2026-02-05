# Van der Pol Oscillator FMU

This is a Rust implementation of the Van der Pol oscillator FMU, ported from the Reference FMUs.

## Model Description

The Van der Pol oscillator is a non-conservative oscillator with non-linear damping. It's described by the second-order differential equation:

```
d²x/dt² - μ(1 - x²)dx/dt + x = 0
```

This is implemented as a system of first-order ODEs:
- `der(x0) = x1`
- `der(x1) = μ(1 - x0²)x1 - x0`

## Variables

- **time**: Independent variable
- **x0**: The first state variable (position), output, start value = 2.0
- **der(x0)**: Derivative of x0, local variable
- **x1**: The second state variable (velocity), output, start value = 0.0
- **der(x1)**: Derivative of x1, local variable
- **mu**: Parameter controlling nonlinearity, fixed parameter, start value = 1.0

## Building and Running

```bash
# Build the FMU
cargo fmi --package vanderpol bundle

# Run simulation
cargo run -p fmi-sim -- --model target/fmu/vanderpol.fmu model-exchange
```
