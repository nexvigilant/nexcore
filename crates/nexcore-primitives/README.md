# nexcore-primitives

Universal computational primitives for the NexVigilant platform. This crate provides cross-domain abstractions for measurement, confidence, fidelity, and physics-derived dynamics, all formally grounded in the Lex Primitiva foundation.

## Intent
To provide a set of "ready-to-use" T2-P (Primitive) and T2-C (Composite) types that can be shared across all domain crates. It ensures that concepts like "Confidence" or "Fidelity" have a single, unified implementation.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **κ (Comparison)**: Used for confidence scoring and threshold comparisons.
- **→ (Causality)**: Primary for relay chains and causal data flows.
- **∂ (Boundary)**: Defines the boundaries for valid measurements.
- **N (Quantity)**: Represents scalar values in measurements and physics dynamics.

## Core Modules
- **Measurement**: Quantitative values with associated `Confidence` scores.
- **Relay**: The `Fidelity` model and `RelayChain` for information preservation tracking.
- **Chemistry**: Stoichiometric primitives for balancing data reactions.
- **Dynamics**: Physics-inspired primitives (Mass, Velocity, Acceleration) for progress tracking.
- **Quantum**: Foundational grounding for discrete states and amplitudes.

## SOPs for Use
### Using a Measured Value
```rust
use nexcore_primitives::{Measured, Confidence};

let val = Measured::new(42.0, Confidence::new(0.95));
println!("Value: {}, Confidence: {}", val.value, val.confidence);
```

### Building a Relay Chain
```rust
use nexcore_primitives::{RelayChain, RelayHop, Fidelity};

let mut chain = RelayChain::new();
chain.add_hop(RelayHop::new("Ingest", Fidelity::new(0.99)));
chain.add_hop(RelayHop::new("Process", Fidelity::new(0.92)));

println!("Total Fidelity: {}", chain.total_fidelity());
```

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
