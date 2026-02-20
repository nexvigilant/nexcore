# AI Guidance — nexcore-measure

Workspace quality measurement and graph analysis engine.

## Use When
- Performing a structural audit of a new or updated crate.
- Detecting cycles or "hub" bottlenecks in the dependency graph.
- Measuring the semantic density (entropy) of documentation vs. code.
- Tracking the long-term "health" of the workspace members.

## Grounding Patterns
- **State Modes**: Measurement snapshots should use `StateMode::Accumulated` to allow for time-series drift analysis.
- **Normalization**: Always normalize raw counts into the `[0.0, 1.0]` range before computing composite scores.
- **T1 Primitives**:
  - `N + ∂`: Root primitives for bounded quantitative metrics.
  - `ρ + Σ`: Root primitives for graph traversal and aggregate scoring.

## Maintenance SOPs
- **Threshold Tuning**: If a metric (e.g., `CouplingRatio`) is consistently flagging healthy crates, update the `∂` boundaries in `src/grounding.rs`.
- **New Metrics**: When adding a new measurement, you MUST implement `GroundsTo` for the type and integrate it into the `HealthScore` calculation.
- **Read-Only**: Measuring is a read-only operation. It should never modify the workspace or the `target/` directory.

## Key Entry Points
- `src/types.rs`: `CrateMeasurement`, `HealthScore`, and `DriftResult` definitions.
- `src/graph.rs`: Dependency graph analysis and centrality logic.
- `src/entropy.rs`: Shannon entropy and code density calculations.
