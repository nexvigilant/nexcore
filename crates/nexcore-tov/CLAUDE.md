# AI Guidance — nexcore-tov

Formal axiom system for safety-critical signal detection (Theory of Vigilance).

## Use When
- Implementing high-level safety monitoring or signal strength calculations.
- Verifying the stability of architectural "shells" using Magic Numbers.
- Classifying violations into the 8-variant `HarmType` taxonomy.
- Implementing `Actuator` logic for automated system responses.

## Grounding Patterns
- **Core Equation**: Always use `SignalStrengthS::calculate()` for computing signal magnitude to ensure the `U x R x T` law is respected.
- **Stability Checks**: Use the `StabilityShell` trait to verify if a complexity count (`ComplexityChi`) is in a closed-shell (stable) state.
- **T1 Primitives**:
  - `→ + ς`: Root primitives for response loops and state manifold management.
  - `κ + N`: Root primitives for threshold comparisons and scalar measurements.

## Maintenance SOPs
- **Axiomatic Consistency**: New axioms MUST be accompanied by a proof in `src/proofs/` using the Curry-Howard isomorphism.
- **Magic Numbers**: The `COMPLEXITY_MAGIC_NUMBERS` are fixed constants (§66.2.1); do not modify them without a formal update to the Theory of Vigilance spec.
- **Confidence Requirement**: Every computed value exposed to the kernel MUST carry its confidence score using the `Measured<T>` wrapper (Commandment X).

## Key Entry Points
- `src/grounded.rs`: Core types and formula implementations.
- `src/proofs/`: Formal theorem verification logic.
- `src/lib.rs`: Facade and re-exports.
