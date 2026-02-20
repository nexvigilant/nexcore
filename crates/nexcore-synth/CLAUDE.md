# AI Guidance — nexcore-synth

Autonomous primitive synthesis and evolution engine.

## Use When
- Discovering new fundamental patterns in unfamiliar data domains.
- Automating the creation of T2-P or T2-C primitives from raw observations.
- Implementing self-evolving logic that adapts to environment drift.
- Testing the limits of Lex Primitiva coverage in new workspaces.

## Grounding Patterns
- **Evolution Loop (ρ)**: Always follow the 4-stage loop: Analyze → Infer → Map → Compose.
- **Novelty Gating**: Ensure the `evolve()` call uses inputs with sufficient statistical entropy to avoid duplicate primitive generation.
- **T1 Primitives**:
  - `ρ + Σ`: Root primitives for the iterative synthesis of new types.
  - `μ + ν`: Root primitives for feature mapping and statistical analysis.

## Maintenance SOPs
- **Confidence Scoring**: Always check `candidate.confidence` before promoting a synth-result to the global registry. Thresholds <0.7 require manual review.
- **Derivation Path**: Maintain the `derivation_path` string to ensure all synthesized logic is traceable back to its statistical and structural origins.
- **Dependency Invariant**: `nexcore-synth` depends on `antitransformer` and `transcriptase`. Ensure these remain leaf-level or domain-agnostic to prevent cycles.

## Key Entry Points
- `src/lib.rs`: `SynthEngine` and `evolve()` implementation.
- `src/grounding.rs`: T1 grounding for synth types.
