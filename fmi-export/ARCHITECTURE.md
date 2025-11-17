# FMI Export Architecture

## Overview

The FMI export system enables creating Functional Mockup Units (FMUs) from Rust code using an explicit, clean architecture. Users declare their model structure explicitly and implement physics equations in a simple trait.

### Core Design Pattern

```rust
#[derive(FmuModel, Default, Debug)]
#[model(model_exchange())]
struct VanDerPol {
    // Parameters
    #[variable(causality = Parameter, start = 1.0)]
    mu: f64,

    // State variables
    #[variable(causality = Output, state, start = 2.0)]
    x0: f64,

    #[variable(causality = Local, state, start = 0.0)]
    x1: f64,

    // Derivative fields
    #[variable(causality = Local, derivative_of = x0, start = 0.0)]
    der_x0: f64,

    #[variable(causality = Local, derivative_of = x1, start = 0.0)]
    der_x1: f64,
}

impl UserModel for VanDerPol {
    fn calculate_values(&mut self) -> fmi::fmi3::Fmi3Status {
        // Access both state variables and derivative fields directly
        self.der_x0 = self.x1;
        self.der_x1 = self.mu * ((1.0 - self.x0 * self.x0) * self.x1) - self.x0;
        fmi::fmi3::Fmi3Res::OK.into()
    }
}
```

## Architecture Components

### Wrapper Traits

- The `export_fmu` macro expands to implementations of the full FMI3 interface functions (declared extern "C" functions).
- Each exported function delegates to a wrapper method defined in the `Fmi3Common`, `Fmi3ModelExchange`, or `Fmi3CoSimulation` traits.
- These trait methods are 'low-level'. Their arguments and return types match the FMI specification exactly.

The default implementation of the wrapper trait methods:
1. Dereferences the raw FMI instance pointer to get a reference to `ModelInstance<Self, MC>`, where `Self: Model` and `MC: ModelContext`.
2. Corresponding higher-level trait methods are called on `fmi::fmi3::Common`, `fmi::fmi3::ModelExchange`, or `fmi::fmi3::CoSimulation` traits.
3.

## Cases
1. Model is defined with ME functions, and CS functions are no-ops. (The ModelDescription will indicate ME only.)
1. Model is defined with ME functions, and CS functions implemented with an embedded euler solver. (The ModelDescription will indicate both ME and CS.)
1. Model is defined with CS functions



### 1. UserModel Trait
Clean separation between user physics code and FMI protocol:

```rust
pub trait UserModel {
    /// Calculate values (derivatives, outputs, etc.)
    fn calculate_values(&mut self) -> Fmi3Status;

    /// Event update function for Model Exchange
    fn event_update(&mut self) -> Result<Fmi3Res, Fmi3Error>;

    /// Get event indicators for zero-crossing detection
    fn get_event_indicators(&mut self, indicators: &mut [f64]) -> Result<Fmi3Res, Fmi3Error>;
}
```

### 2. Derive Macro (FmuModel)
- Processes all declared fields (states, parameters, derivatives)
- Generates ValueRef enum with synchronized value references
- Creates FMI interface implementations with change detection
- Handles ModelDescription XML generation

### 3. Change Detection Pattern
Automatically calls `calculate_values()` before returning derivative values, following FMI Reference Implementation pattern:

```rust
ValueRef::DerX0 => {
    // Ensure derivatives are up-to-date (change detection)
    let _ = <Self as fmi_export::fmi3::UserModel>::calculate_values(self);
    *value = self.der_x0;
},
```

## User Experience

1. Mark state variables with the `state` flag
2. Declare corresponding derivative fields with `derivative_of = state_name`
3. Implement `UserModel::calculate_values()` with physics equations
4. Access both state variables and derivative fields directly in calculations

## Code Locations

- **Core traits**: `fmi-export/src/fmi3/mod.rs`
- **Procedural macro**: `fmi-export-derive/src/lib.rs`
- **Working examples**: `fmi-export/tests/van_der_pol.rs`, `fmi-export/tests/bouncing_ball.rs`
