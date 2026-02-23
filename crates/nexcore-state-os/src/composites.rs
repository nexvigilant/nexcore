//! # Composite Types
//!
//! Compound types composed from this crate's primitives.
//!
//! STOS composites span the 15 layer engines plus the kernel:
//!
//! | Composite        | Layer    | Tier | Dominant Primitive |
//! |------------------|----------|------|--------------------|
//! | `StateRegistry`  | STOS-ST  | T2-C | ς (State) |
//! | `TransitionEngine`| STOS-TR | T2-C | → (Causality) |
//! | `BoundaryManager`| STOS-BD  | T2-C | ∂ (Boundary) |
//! | `GuardEvaluator` | STOS-GD  | T2-C | κ (Comparison) |
//! | `CountMetrics`   | STOS-CT  | T2-C | N (Quantity) |
//! | `StateKernel`    | Kernel   | T3   | All 15 (orchestrator) |
//! | `MachineInstance`| Machine  | T3   | ς + → + ∂ |
//!
// Currently empty — composites will be added as the crate evolves.
