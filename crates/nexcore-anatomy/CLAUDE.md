# AI Guidance — nexcore-anatomy

Structural workspace anatomy and layer enforcement.

## Use When
- Auditing the dependency graph for new architectural violations.
- Planning the extraction of a module into a standalone crate (analyzing blast radius).
- Measuring the complexity of a crate using the Chomsky classification.
- Identifying high-fan-in bottlenecks that require stabilization.

## Grounding Patterns
- **Layer Compliance**: Always verify that a crate is assigned to the correct `Layer`. Service crates must never be dependencies of Domain crates.
- **Topological Order (σ)**: Respect the ordering provided by Kahn's algorithm; it determines the safe build and test sequence.
- **T1 Primitives**:
  - `σ + ρ`: Root primitives for graph traversal and ordering.
  - `∂ + μ`: Root primitives for boundary enforcement and classification.

## Maintenance SOPs
- **Violation Rules**: If the architectural rules change (e.g., adding a new layer), you MUST update the `LayerMap` logic in `src/layer.rs`.
- **Chomsky Mapping**: When a new generator type is added to the system (e.g., a new macro engine), update the `chomsky.rs` profile to include it.
- **No Circularity**: The dependency graph MUST remain a DAG. Any `Recursion` that leads back to the origin is a `Critical` violation.

## Key Entry Points
- `src/graph.rs`: `DependencyGraph` construction and Kahn's algorithm.
- `src/layer.rs`: Layer definitions and boundary violation logic.
- `src/blast_radius.rs`: Impact analysis for crate changes.
