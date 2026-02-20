# AI Guidance — nexcore-vigilance

Domain monolith for ToV axioms, Guardian-AV, and consolidated PV modules.

## Use When
- Implementing core pharmacovigilance logic or safety signal detection.
- Extending the Theory of Vigilance (ToV) axiom system.
- Interfacing with the homeostatic Guardian loops.
- Accessing re-exported PVOS types.

## Grounding Patterns
- **Safety Margin**: Always use `SafetyMargin::calculate(prr, ror, ic, eb, n)` to compute the distance to harm.
- **Harm Types**: Reference `HarmType` (A-H) when classifying any signal violation.
- **T1 Primitive Usage**:
  - `ς + ∂`: For any state-based boundary check.
  - `μ + →`: For mapping external data to causal chains.

## Maintenance SOPs
- **Module Expansion**: When adding sub-domains (e.g., `betting`, `bioinfo`), ensure they re-export their types into `nexcore-vigilance`'s facade.
- **Test Mandate**: All new axioms MUST have a deterministic test case in `src/tests/` or the local module's `tests` module.
- **No Unsafe/Panic**: Strictly enforce the `#![forbid(unsafe_code)]` and `#![deny(clippy::unwrap_used)]` rules. Use `Result` or `Option` for all boundary-crossing operations.

## Key Entry Points
- `src/tov/mod.rs`: ToV definitions and the 8 harm types.
- `src/pvos/mod.rs`: Re-exports of the PVOS substrate.
- `src/lib.rs`: The top-level module registry.
