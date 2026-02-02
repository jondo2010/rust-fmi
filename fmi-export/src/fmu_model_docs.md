## FmuModel derive reference

### Struct-level attributes

Use `#[model(...)]` on the struct to configure the generated FMI interfaces and
metadata.

```rust,ignore
#[derive(FmuModel, Default)]
#[model(
    description = "Example FMU",
    model_exchange = true,
    co_simulation = false,
    scheduled_execution = false,
    user_model = true,
)]
struct MyModel {
    #[variable(causality = Output, start = 1.0)]
    y: f64,
}
```

Supported keys:

- `description`: Optional string. Defaults to the struct docstring if omitted.
- `model_exchange`: Optional bool. Defaults to `true`.
- `co_simulation`: Optional bool. Defaults to `false`.
- `scheduled_execution`: Optional bool. Defaults to `false`.
- `user_model`: Optional bool. Defaults to `true`. Set `false` to provide your
  own `impl UserModel`.

Notes:

- All boolean flags must be explicit (`co_simulation = true`). Shorthand
  `co_simulation` is rejected.
- `#[model()]` with no arguments is valid and uses the defaults above.

### Field-level attributes

Use `#[variable(...)]` to include a field as an FMI variable. Use `#[alias(...)]`
for additional aliases. Both attributes accept the same keys.

```rust,ignore
#[derive(FmuModel, Default)]
struct MyModel {
    /// Height above ground
    #[variable(causality = Output, start = 1.0)]
    h: f64,

    /// Velocity of the ball
    #[variable(causality = Output, start = 0.0)]
    #[alias(name = "der(h)", causality = Local, derivative = h)]
    v: f64,
}
```

Supported keys for `#[variable(...)]` and `#[alias(...)]`:

- `skip`: Bool. When `true`, the field is ignored for FMI variables.
- `name`: String. Overrides the variable name (defaults to the field name).
- `description`: String. Overrides the field docstring.
- `causality`: One of `Parameter`, `CalculatedParameter`, `Input`, `Output`,
  `Local`, `Independent`, `Dependent`, `StructuralParameter`.
- `variability`: One of `Constant`, `Fixed`, `Tunable`, `Discrete`, `Continuous`.
- `start`: Rust expression used as the start value.
- `initial`: One of `Exact`, `Calculated`, `Approx`.
- `derivative`: Ident referencing another field. Marks this variable as the
  derivative of that field.
- `event_indicator`: Bool. When `true`, counts toward the FMI event indicator
  total.
- `interval_variability`: One of `Constant`, `Fixed`, `Tunable`, `Changing`,
  `Countdown`, `Triggered`.
- `clocks`: List of clock field idents that this variable belongs to.
- `max_size`: Integer. Max size for Binary variables.
- `mime_type`: String. MIME type for Binary variables.

Notes:

- Continuous state variables are inferred by `derivative` relationships.
- `clocks` must reference clock variables in the same model. The generated FMU
  resolves these to value references.

### Child components

Use `#[child(...)]` to reuse another `FmuModel` as a component and prefix its
variable names.

```rust,ignore
#[derive(FmuModel, Default)]
struct Parent {
    #[child(prefix = "bus")]
    bus: CanBus,
}
```

Supported keys:

- `prefix`: Optional string. Defaults to the field name. Child variables are
  named `<parent_prefix><prefix>.<child_variable>`.

Notes:

- Child fields should implement the `Model` trait (typically via `FmuModel`).
- `#[child]` only affects naming and metadata; it does not change runtime
  behavior of the child component.
