# AI Guidance — nexcore-hormones

Thin re-export wrapper for `nexcore-hormone-types`. All types, grounding
implementations, and tests live in `nexcore-hormone-types`. This crate
exists for backward compatibility — downstream consumers that depend on
`nexcore-hormones` continue to work unchanged via `pub use nexcore_hormone_types::*`.

## Use When
- Existing code imports `nexcore_hormones::*` — no migration needed.
- New code should depend on `nexcore-hormone-types` directly when possible.
- The `sessionstart_endocrine_loader` binary lives here.

## Grounding Patterns
- **Decay**: Always call `apply_decay()` at the start of a new session to simulate time passing.
- **Clamping**: Hormone levels are strictly clamped between `0.0` and `1.0`.
- **T1 Primitives**:
  - `ς + π`: Persistent state management.
  - `N + μ`: Quantified mapping of stimuli to shifts.

## Maintenance SOPs
- **Stimulus Addition**: When adding a new `Stimulus` variant, edit `nexcore-hormone-types/src/lib.rs`. Ensure it has a logical mapping to at least one primary hormone.
- **Thresholds**: Respect the standard thresholds: `>0.7` for "High/Active", `<0.3` for "Low/Depleted".
- **File Safety**: Use `EndocrineResult` for all I/O operations to handle missing `HOME` or corrupted JSON gracefully.

## Key Entry Points
- `src/lib.rs`: Re-exports all types from `nexcore-hormone-types`.
- `src/bin/sessionstart_endocrine_loader.rs`: Session-start binary for loading endocrine state.
- Types and grounding: see `nexcore-hormone-types/src/lib.rs` and `nexcore-hormone-types/src/grounding.rs`.
