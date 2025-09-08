# Dahlquist FMU Example

This is a Rust implementation of the Dahlquist test equation FMU, ported from the Reference FMUs provided by the FMI standard.

## Model Description

The Dahlquist model implements a simple first-order linear ordinary differential equation:

```
der(x) = -k * x
```

Where:
- `x` is the state variable (initial value: 1.0)
- `k` is a parameter (initial value: 1.0)
- `der(x)` is the time derivative of x

This is a classic test equation used in numerical analysis to evaluate ODE solvers.

## Variables

| Name   | Value Reference | Causality   | Variability | Initial | Start Value | Description |
|--------|----------------|-------------|-------------|---------|-------------|-------------|
| `time` | 0              | independent | continuous  | -       | -           | Simulation time |
| `x`    | 1              | output      | continuous  | exact   | 1.0         | State variable |
| `der(x)` | 2            | local       | continuous  | calculated | -         | Derivative of x |
| `k`    | 3              | parameter   | fixed       | exact   | 1.0         | Parameter |

## Building

Build and package the Dahlquist FMU:

```bash
cargo xtask bundle dahlquist
```

## Model Behavior

With the default values (x₀ = 1.0, k = 1.0), the analytical solution is:

```
x(t) = e^(-t)
```

The state variable `x` exponentially decays from 1.0 towards 0.0 over time.
