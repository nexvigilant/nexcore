# nexcore-tov-grounded

The runtime primitive layer for the Theory of Vigilance (ToV). This crate provides the concrete implementation of the core signal equations (S = U x R x T) and the foundational types (Bits, Uniqueness, Recognition, Temporal) used by the vigilance kernel.

## Intent
To provide a compilable, performance-oriented implementation of ToV primitives. It serves as the "physical" grounding for the formal theorems verified in `nexcore-tov-proofs`.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **N (Quantity)**: Represents fundamental units like `Bits` and `QuantityUnit`.
- **ς (State)**: Manages the `VigilanceSystem` state space and boundaries.
- **κ (Comparison)**: Used for `ComplexityChi` shell stability checks.
- **→ (Causality)**: Primary for the `Actuator` trait and response governance.

## Core Formulae
### Signal Strength (S)
`S = Uniqueness (U) x Recognition (R) x Temporal (T)`
Calculated in `Bits`.

### Stability Shells
Enforces architectural stability through "Magic Numbers":
- **Complexity**: `[2, 8, 20, 28, 50, 82, 126, 184, 258, 350]`
- **Connection**: `[2, 8, 18, 32, 50, 72, 98, 128, 162, 200]`

## SOPs for Use
### Calculating Signal Strength
```rust
use nexcore_tov_grounded::{SignalStrengthS, UniquenessU, RecognitionR, TemporalT, Bits};
let s = SignalStrengthS::calculate(UniquenessU(Bits(5.0)), RecognitionR(0.9), TemporalT(1.0));
```

### Checking Stability
```rust
use nexcore_tov_grounded::{StabilityShell, ComplexityChi, QuantityUnit};
let chi = ComplexityChi(QuantityUnit(8));
if chi.is_closed_shell() {
    println!("Architecture is stable.");
}
```

## Key Components
- **Actuator**: Trait for executing safety actions (Throttle, Alert, CircuitBreaker).
- **MetaVigilance**: Logic for self-monitoring the vigilance loop health.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
