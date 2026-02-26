# nexcore-primitives

Universal computational primitives for the NexVigilant platform. This crate provides cross-domain abstractions for measurement, confidence, fidelity, relay chains, chemistry, quantum dynamics, entropy, and transfer patterns, all formally grounded in the Lex Primitiva foundation.

## Intent
To provide a set of "ready-to-use" T2-P (Primitive) and T2-C (Composite) types that can be shared across all domain crates. It ensures that concepts like "Confidence" or "Fidelity" have a single, unified implementation.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **κ (Comparison)**: Used for confidence scoring and threshold comparisons.
- **→ (Causality)**: Primary for relay chains and causal data flows.
- **∂ (Boundary)**: Defines the boundaries for valid measurements.
- **N (Quantity)**: Represents scalar values in measurements and physics dynamics.

## Core Modules
- **measurement**: Re-exports `Confidence` and `Measured<T>` from `nexcore-constants` — the single source of truth.
- **relay**: The `Fidelity` model and `RelayChain` for information preservation tracking across pipeline hops.
- **chemistry**: Stoichiometric primitives (Arrhenius, Michaelis-Menten, Hill, Nernst, Langmuir, Eyring, etc.) mapped to PV applications.
- **quantum**: Wave, operator, state, and domain quantum primitives (Qubit, Superposition, Entanglement, Decoherence, etc.).
- **dynamics**: Time-evolution operators for quantum primitives (Phasor, EnvironmentalCoupling, Observer).
- **entropy**: Shannon entropy, KL divergence, mutual information, and information loss quantification.
- **transfer**: Cross-domain T2-P patterns (FeedbackLoop, CircuitBreaker, Homeostasis, DecayFunction, etc.).
- **spatial_bridge**: `stem-math` spatial trait implementations for transfer primitives.
- **grounding**: `GroundsTo` implementations for all public types.

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

// RelayChain::new(f_min) requires the minimum acceptable fidelity
let mut chain = RelayChain::new(0.80);
// RelayHop::new(stage, fidelity, threshold) requires all three args
chain.add_hop(RelayHop::new("Ingest", Fidelity::new(0.99), 0.0));
chain.add_hop(RelayHop::new("Process", Fidelity::new(0.92), 1.0));

println!("Total Fidelity: {}", chain.total_fidelity());
assert!(chain.verify_preservation()); // F_total >= f_min
```

### Using Hill Cooperativity (Chemistry)
```rust
use nexcore_primitives::chemistry::{CooperativeBinding, classify_cooperativity};

let binding = CooperativeBinding::new(10.0, 2.5).unwrap();
let response = binding.response(10.0).unwrap(); // 0.5 at K₀.₅
println!("Cooperativity: {:?}", binding.classify());
```

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
