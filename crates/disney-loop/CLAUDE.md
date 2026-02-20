# AI Guidance — disney-loop

Forward-only causal discovery pipeline.

## Use When
- Implementing discovery loops for new compounds or knowledge elements.
- Ensuring that an automated reasoning process never reverts to an inferior previous state.
- Aggregating "Novelty Scores" across multiple research domains.
- Persisting irreversible state transitions in a JSON-based ledger.

## Grounding Patterns
- **Irreversibility (σ)**: The Disney Loop is a "one-way street." Never implement logic that reads a `ρ(t+1)` state back into a `ρ(t)` position within the same execution cycle.
- **Novelty over Noise**: The `Curiosity Search` (∃) stage should prioritize high-entropy discoveries that trigger `LexPrimitiva::Existence`.
- **T1 Primitives**:
  - `ρ + ∂`: Root primitives for iterative gating.
  - `∃ + ν`: Root primitives for discovery and frequency tracking.

## Maintenance SOPs
- **DataFrame Integrity**: All columns used for filtering (`direction`) or aggregation (`novelty_score`, `discovery`) MUST exist in the input Polars DataFrame.
- **Fail-Safe**: If the `anti-regression-gate` filters out 100% of the input, the pipeline should return `DisneyError::EmptyPipeline` rather than an empty state file.
- **Traceability**: Every run of the `sink_new_state` should be logged with the number of surviving records to maintain an audit trail of the discovery velocity.

## Key Entry Points
- `src/lib.rs`: The main pipeline stages and Polars transformations.
- `src/humanize.rs`: Logic for converting raw scores into human-readable discovery reports.
