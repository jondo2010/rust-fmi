# Quillum

Quillum is an FMU Simulator built on a Reactor Architecture using `boomerang`

## Goal

Build a **deterministic, composable FMI simulator** that supports:

* Multiple FMUs
* Co-Simulation (CS) and Model-Exchange (ME)
* Explicit solvers
* Clear time and causality semantics

The core idea is to use a **reactor-based coordination model** where *everything that owns timing or causality is a reactor*.

---

## Core Principle

> **Reactors define coordination and visibility in logical time.**
> **Computation happens inside those boundaries.**

Time advancement, scheduling, and dataflow are explicit and analyzable.

---

## Reactor Roles

All components are reactors, but with different semantics.

### 1. Discrete Reactor

**Examples**

* CS FMUs
* CSV / time-series inputs
* Arrow recorders

**Semantics**

* Fires on logical ticks
* Owns its execution
* Atomic per tick

```
Inputs → doStep / emit → Outputs
```

---

### 2. Continuous Reactor (ME FMU)

**Examples**

* FMI Model-Exchange FMUs

**Semantics**

* Passive
* Never fires
* Provides equations to a solver

```
state, inputs ──► derivatives / outputs
```

Defines:

* Continuous state
* Input/output ports
* Evaluation callbacks

---

### 3. Solver Reactor

**Examples**

* Fixed-step Runge–Kutta
* Variable-step CVODE

**Semantics**

* Fires once per macro step
* Owns continuous time advancement
* Coordinates ME reactors

```
[t → t+Δt]
  iterate:
    evaluate ME reactors
    solve coupled system
```

---

## Time Model

### Two Time Domains

| Domain                | Meaning                                |
| --------------------- | -------------------------------------- |
| Logical (macro) time  | Reactor scheduling, I/O, logging       |
| Physical (micro) time | Solver iterations, continuous dynamics |

Only **solver reactors advance physical time**.

---

## Execution Cycle

For each logical step `t`:

```
1. Input Discrete Reactors fire
2. Solver Reactor fires
   - Integrates ME reactors from t → t+Δt
3. Output / Logging Reactors fire
4. Advance logical time
```

ME reactors are only active during step 2.

---

## Dataflow Model

* Typed ports connect reactors
* No shared memory
* All coupling is explicit
* Algebraic loops exist **only inside solver scope**

---

## Graph Structure

```
InputReactor
     ↓
SolverReactor ──► ContinuousReactors (ME FMUs)
     ↓
OutputReactor
```

Discrete scheduling graph excludes ME reactors.

---

## Validation Rules

* CS reactors must not observe ME state mid-step
* ME reactors interact only via solver reactors
* Algebraic loops must be solvable within solver
* Logical causality must be acyclic

Fail fast at build time.

---

## Benefits

* Deterministic execution
* Explicit time ownership
* Clean ME + CS integration
* Composable solvers
* Aligns with FMI 3.0 clocked partitions
* Matches Lingua Franca semantics
* Rust-friendly ownership & traits

---

## Mental Model

* **Reactors** = coordination & causality
* **Solvers** = time advancement
* **ME FMUs** = equations
* **CS FMUs** = self-stepping components

> *Reactors orchestrate; solvers compute.*
