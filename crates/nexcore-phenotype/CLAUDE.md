# AI Guidance — nexcore-phenotype

Adversarial test fixture generator and safety verifier.

## Use When
- Generating negative test cases for data ingestion pipelines.
- Verifying the sensitivity of a `Ribosome` drift threshold.
- Stress-testing the "No-Panic" guarantee of a new domain module.
- Simulating "hallucinated" or malformed data to ensure system robustness.

## Grounding Patterns
- **Verification (κ)**: Always call `verify()` after generating a mutation to ensure it is actually measurable by the system.
- **Mutation Selection (∂)**: Use `mutate_all()` for comprehensive structural audits and `mutate_batch()` for high-volume fuzzing.
- **T1 Primitives**:
  - `∂ + μ`: Root primitives for strategy-based value generation.
  - `σ + κ`: Root primitives for sequential verification of drift.

## Maintenance SOPs
- **Expected Drifts**: When adding a new `Mutation` variant, you MUST update `expected_drift_types()` to ensure the verification loop remains accurate.
- **Normal Generation**: `generate_normal()` is provided for baseline comparison; avoid using it for "mutated" scenarios.
- **Thresholds**: Note that `verify()` uses a very low threshold (`0.01`) by default. If a mutation isn't detected at this level, the mutation logic is likely broken.

## Key Entry Points
- `src/lib.rs`: `mutate()`, `verify()`, and `Mutation` definitions.
- `src/grounding.rs`: T1 grounding for phenotype types.
