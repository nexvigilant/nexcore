# AI Guidance — nexcore-primitives

Universal cross-domain primitives (T2-P and T2-C).

## Use When
- Implementing types that require explicit `Confidence` or `Fidelity` tracking.
- Modeling data pipelines as `RelayChain` sequences.
- Using physics-inspired dynamics (velocity, mass) to measure system performance.
- Balancing "data reactions" using stoichiometric chemistry analogs.

## Grounding Patterns
- **Multiplicative Decay**: Remember that `Fidelity` and `Confidence` are generally multiplicative; total value is the product of all components in a chain.
- **Tiering**: All types in this crate are at least T2. Never add a T1 universal primitive here; use `nexcore-lex-primitiva`.
- **T1 Primitives**:
  - `κ + →`: Root primitives for measured causal chains.
  - `∂ + N`: Root primitives for bounded quantities.

## Maintenance SOPs
- **Lossless Relay**: When adding a new `RelayHop`, ensure the `Fidelity` calculation accurately reflects the potential for information loss in that stage.
- **Quantum Grounding**: Use the `quantum` module for any type that requires discrete, amplitude-based state management (interfacing with `nexcore-synapse`).
- **No Domain Logic**: This crate MUST remain domain-agnostic. Never add PV or Skill-specific logic here.

## Key Entry Points
- `src/measurement.rs`: `Confidence` and `Measured` value wrappers.
- `src/relay.rs`: `Fidelity` and `RelayChain` implementation.
- `src/chemistry/`: Stoichiometry and balancing logic.
