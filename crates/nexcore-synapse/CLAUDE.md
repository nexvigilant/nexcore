# AI Guidance — nexcore-synapse

Pattern reinforcement and synaptic learning engine.

## Use When
- Tracking the effectiveness of specific tool sequences or interaction patterns.
- Implementing "learned" preferences that persist across agent restarts.
- Measuring the decay of knowledge or patterns over time.
- Gating autonomous actions based on "consolidation" thresholds (only act when pattern is strong).

## Grounding Patterns
- **Learning Rates**: Use `AmplitudeConfig::FAST` for high-frequency low-risk patterns and `DEFAULT` for critical domain knowledge.
- **Decay**: Respect the `half_life`; patterns that aren't reinforced will eventually lose consolidation.
- **T1 Primitives**:
  - `ν + Σ`: Root primitives for signal accumulation.
  - `∝ + ∂`: Root primitives for decay and gating.

## Maintenance SOPs
- **Synapse ID**: Use descriptive, snake_case IDs for learning targets (e.g., `git_commit_style`, `rust_lint_preference`).
- **Pruning**: Periodically call `SynapseBank::prune_decayed()` to prevent memory bloat from obsolete patterns.
- **Dual-Write**: Note that synapses are often dual-written to `brain.db` via the `nexcore-brain` synapse adapter.

## Key Entry Points
- `src/lib.rs`: `Synapse`, `LearningSignal`, and `Amplitude` definitions.
- `src/grounding.rs`: T1 grounding for synaptic types.
