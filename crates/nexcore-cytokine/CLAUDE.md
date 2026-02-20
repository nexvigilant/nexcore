# AI Guidance — nexcore-cytokine

Biological-analog signaling for inter-crate communication and orchestration.

## Use When
- Implementing cross-crate event notifications (e.g., `brain` needs to tell `guardian` about an edit).
- Creating amplification cascades for safety triggers.
- Modeling agent specialization or lifecycle (birth/death).
- Implementing "chemotactic" search patterns in large data/codebases.

## Grounding Patterns
- **Family Selection**: 
  - Use `IL` families for general data flow.
  - Use `TNF` for "Stop/Abort" signals.
  - Use `IFN` for "Alert/Verify" signals.
- **T1 Primitives**:
  - `→ + μ`: Standard emission/reception.
  - `ρ + N`: Cascades and concentration-based thresholds.

## Maintenance SOPs
- **Async Handling**: Receptors handlers MUST be async and should avoid heavy blocking work; delegate to `tokio::spawn` if needed.
- **Fire-and-Forget**: Cytokines are not Request-Response. If you need a return value, use the `bus` to emit a "response" cytokine or use `nexcore-synapse`.
- **Potency**: Always specify `Potency` to allow the bus to prioritize delivery under heavy load.

## Key Entry Points
- `src/bus.rs`: The central signal relay.
- `src/receptor.rs`: Registration and binding logic.
- `src/types.rs`: `Cytokine` and `CytokineFamily` definitions.
