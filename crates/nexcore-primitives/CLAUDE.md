# AI Guidance — nexcore-primitives

Universal cross-domain primitives (T2-P and T2-C).

## Use When
- Implementing types that require explicit `Confidence` or `Fidelity` tracking.
- Modeling data pipelines as `RelayChain` sequences.
- Using time-evolution dynamics (Phasor, EnvironmentalCoupling) for quantum-analog simulation.
- Balancing "data reactions" using stoichiometric chemistry analogs.
- Quantifying information entropy or distribution divergence (KL, mutual information).
- Building cross-domain transfer patterns: feedback loops, circuit breakers, homeostasis, decay.

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
- `src/measurement.rs`: Re-exports `Confidence` and `Measured<T>` from `nexcore-constants`.
- `src/relay.rs`: `Fidelity`, `RelayChain`, `RelayHop`, and `RelayVerification` (5-axiom verification).
- `src/chemistry/`: 14 stoichiometric primitives (Arrhenius, Michaelis-Menten, Hill, Nernst, Langmuir, Eyring, Gibbs, Henderson-Hasselbalch, Beer-Lambert, half-life, equilibrium, inhibition, dependency rate-law, aggregation pipeline).
- `src/quantum.rs`: 13 quantum primitive types across 4 layers (Wave, Operators, State, Domain).
- `src/dynamics.rs`: Time-evolution operators — `Phasor`, `EnvironmentalCoupling`, `Interaction`, `Observer`.
- `src/entropy.rs`: Shannon entropy, KL divergence, mutual information, joint entropy, information loss.
- `src/transfer.rs`: Cross-domain T2-P patterns — `FeedbackLoop`, `CircuitBreaker`, `Homeostasis`, `DecayFunction`, `RateLimiter`, `NegativeEvidence`, `ThreatSignature`, and others.
- `src/spatial_bridge.rs`: `stem-math` `Orient` / `Neighborhood` trait impls for transfer primitives.
- `src/grounding.rs`: `GroundsTo` implementations for all 84 public types.
