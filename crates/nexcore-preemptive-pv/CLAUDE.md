# AI Guidance — nexcore-preemptive-pv

Preemptive signal detection and intervention engine.

## Use When
- Moving beyond simple disproportionality (PRR/ROR) to predictive safety models.
- Evaluating the potential for signal emergence using energy-based analogies (Gibbs).
- Modeling the temporal growth of a signal using Hill amplification.
- Deciding whether to intervene based on the cost of action vs. the risk of irreversible harm (Omega).

## Grounding Patterns
- **Three-Tier Pipeline**: Always consider the tier hierarchy (Reactive → Predictive → Preemptive) when communicating signal status.
- **Noise Floor (eta)**: Use the `noise` module to correct for baseline reporting noise before taking high-stakes decisions.
- **T1 Primitives**:
  - `→ + κ`: Root primitives for the preemptive decision logic.
  - `∝ + σ`: Root primitives for trajectory and growth modeling.

## Maintenance SOPs
- **Model Parameters**: The `GibbsParams` and `NoiseParams` should be derived from empirical historical data wherever possible.
- **Irreversibility (Omega)**: Ensure that `Seriousness` weights reflect the formal ToV definitions of irreversible harm (§34).
- **Intervention Testing**: Cross-validate the `InterventionResult` against the `Guardian` system's actual actuation success rates.

## Key Entry Points
- `src/preemptive.rs`: The main Tier 3 decision engine.
- `src/predictive.rs`: Tier 2 signal feasibility and trajectory logic.
- `src/gibbs.rs`: Thermodynamic analogies for signal emergence.
- `src/lib.rs`: Common types and re-exports.
