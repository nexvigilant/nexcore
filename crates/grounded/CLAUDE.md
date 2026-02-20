# AI Guidance — grounded

Epistemological substrate and evidence-based reasoning engine.

## Use When
- Implementing critical decision logic that must account for uncertainty.
- Building evidence chains to justify a safety intervention.
- Running experimental loops (Hypothesis → Reality Check).
- Enforcing exhaustive confidence-band matching for high-stakes tool calls.

## Grounding Patterns
- **Explicit Uncertainty**: Never return a raw value if the result depends on probabilistic reasoning; use `Uncertain<T>`.
- **Evidence-First (σ)**: A conclusion without an `EvidenceChain` has an implicit `Confidence` of 0.0.
- **T1 Primitives**:
  - `× + N`: Root primitives for uncertainty composition.
  - `→ + π`: Root primitives for experimental causality and learning persistence.

## Maintenance SOPs
- **Band Invariant**: `ConfidenceBand` transitions (e.g., from Medium to High) MUST be backed by at least one new `EvidenceStep`.
- **Store Safety**: The `SqliteStore` is the standard for cross-session learning. Ensure migrations preserve the `EvidenceChain` integrity.
- **Macro Usage**: Use the `uncertain_match!` macro to ensure that "Low" and "Negligible" confidence cases are never ignored.

## Key Entry Points
- `src/uncertain.rs`: The `Uncertain<T>` type and band logic.
- `src/feedback.rs`: The `GroundedLoop` and experimental framework.
- `src/chain.rs`: Sequential evidence tracking.
