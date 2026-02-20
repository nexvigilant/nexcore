# AI Guidance — nexcore-foundry

Dual-pipeline (Builder + Analyst) assembly line architecture.

## Use When
- Implementing structured workflows for code or artifact generation.
- Coordinating multiple agents through stage-gated sequences.
- Building bridges between construction and analysis tasks.
- Measuring alignment of strategic goals across a delivery pipeline.

## Grounding Patterns
- **Station ID**: Always refer to the exact `StationId` (B1, B2, B3, A1, A2, A3) to ensure alignment with the VDAG spec.
- **Run Mode**: Respect the `Foreground` (blocking) vs `Background` (async) distinction for each station.
- **T1 Primitives**:
  - `σ + →`: Root primitives for sequential pipeline causality.
  - `κ + μ`: Root primitives for validation and signal mapping.

## Maintenance SOPs
- **Agent Mapping**: When adding a new station, ensure a corresponding `StationConfig` constructor is added with a unique `agent_name`.
- **Validation Gates**: The B3 station MUST implement a "Red/Green" logic based on `ValidatedDeliverable`.
- **Bridge Contracts**: Bridges should never lose information; always verify round-trip fidelity if the bridge is "Codifying."

## Key Entry Points
- `src/station.rs`: Station identifiers and the 14-stage VDAG order.
- `src/artifact.rs`: Schemas for data flowing through the assembly line.
- `src/governance.rs`: SMART goal cascade and quality gate logic.
