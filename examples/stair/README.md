# Stair FMU Example

This example is a Rust port of the Stair reference FMU from the FMI standard.

## Model Description

The Stair model implements a discrete event system that:

- Increments an integer counter every second
- Starts with `counter = 1` at time 0
- Schedules time events at t = 1, 2, 3, ... seconds
- Terminates the simulation when `counter >= 10`

## Variables

- `time`: Independent time variable (built-in, VR 0)
- `counter`: Discrete output variable that counts seconds (VR 1)

## Mathematical Model

```
counter(t) = floor(t) + 1  for t ∈ [0, 9]
```

The model terminates when `counter >= 10`, which occurs at `t = 9`.

## Reference

This port is based on the Stair reference FMU from the FMI 3.0 standard reference implementations.
