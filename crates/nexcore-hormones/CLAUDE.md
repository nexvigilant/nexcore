# AI Guidance — nexcore-hormones

Persistent behavioral state modulators (Endocrine System).

## Use When
- Tracking long-term "mood" or disposition of the AI agent.
- Modulating risk tolerance based on recent error rates (Cortisol).
- Reinforcing successful interaction patterns (Dopamine).
- Detecting "Crisis Mode" after critical system failures (Adrenaline).
- Measuring session fatigue and recommending a rest (Melatonin).

## Grounding Patterns
- **Decay**: Always call `apply_decay()` at the start of a new session to simulate time passing.
- **Clamping**: Hormone levels are strictly clamped between `0.0` and `1.0`.
- **T1 Primitives**:
  - `ς + π`: Persistent state management.
  - `N + μ`: Quantified mapping of stimuli to shifts.

## Maintenance SOPs
- **Stimulus Addition**: When adding a new `Stimulus` variant, ensure it has a logical mapping to at least one primary hormone.
- **Thresholds**: Respect the standard thresholds: `>0.7` for "High/Active", `<0.3` for "Low/Depleted".
- **File Safety**: Use `EndocrineResult` for all I/O operations to handle missing `HOME` or corrupted JSON gracefully.

## Key Entry Points
- `src/lib.rs`: `EndocrineState`, `HormoneType`, and `Stimulus` definitions.
- `src/grounding.rs`: T1 grounding implementations.
