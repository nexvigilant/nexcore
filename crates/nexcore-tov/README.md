# nexcore-tov

The formal axiom system for safety-critical signal detection in the NexVigilant platform. This crate implements the **Theory of Vigilance (ToV)**, providing a mathematical foundation for measuring uniqueness, recognition, and temporal relevance to compute signal strength.

## Intent
To provide a rigorous, verifiable framework for safety monitoring. It translates high-level vigilance concepts (rarity, detection accuracy, decay) into a formal calculus (S = U x R x T) and uses Curry-Howard proofs to verify theorem correctness.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **→ (Causality)**: The primary primitive for the Actuator loop and response governance.
- **κ (Comparison)**: Used for evaluating signal strength against stability shells.
- **N (Quantity)**: Represents scalar units (Bits, QuantityUnit) and measured values.
- **ς (State)**: Manages the Safety Manifold interior and system state space.

## Core Formulae
### The Core Signal Equation (§20)
`S = U x R x T`
- **U (Uniqueness)**: Rarity measure `-log2 P(C|H0)` in Bits.
- **R (Recognition)**: Detection sensitivity and accuracy `[0, 1]`.
- **T (Temporal)**: Decaying relevance factor `[0, 1]`.

### Safety Margin (§9.2)
`d(s) = (threshold - s) / threshold`
Safe states are within the interior of the Safety Manifold where `d(s) > 0`.

## Core Components
- **StabilityShell**: Enforces architectural invariants using "Magic Numbers" (§66.2.1).
- **HarmType**: The 8-variant taxonomy for classifying boundary violations (Acute to Hidden).
- **Actuator**: Trait for executing system responses to pull the state back into the manifold.
- **MetaVigilance**: Monitoring the health and integrity of the vigilance loop itself.

## SOPs for Use
### Computing Signal Strength
```rust
use nexcore_tov::{SignalStrengthS, UniquenessU, RecognitionR, TemporalT, Bits};

let u = UniquenessU(Bits(10.0));
let r = RecognitionR(0.95);
let t = TemporalT(0.8);

let s = SignalStrengthS::calculate(u, r, t);
println!("Signal Strength: {} bits", s.0.0);
```

### Verifying Proofs
Theorem verification is handled at compile-time via the `proofs` module, mapping theorems to types and proofs to programs.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
