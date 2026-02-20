# AI Guidance — nexcore-transcriptase

Bidirectional Data ↔ Schema synthesis engine.

## Use When
- Discovering the schema of an unknown data source through observation.
- Detecting anomalies or boundary violations in structured JSON data.
- Generating synthetic test data that conforms to real-world observations.
- Maintaining schema contracts between disparate system components.

## Grounding Patterns
- **Range Widening**: Remember that merging schemas always widens ranges (Int, Float, Str length); it never restricts them.
- **Violation Synthesis (∂)**: Use synthesized violations as unit test cases for new data handlers.
- **T1 Primitives**:
  - `μ + σ`: Root primitives for the inference pipeline.
  - `κ + ∂`: Root primitives for validation and boundary checking.

## Maintenance SOPs
- **Mixed Types**: If a field's `SchemaKind` becomes `Mixed`, it usually indicates a data quality issue or a highly dynamic API.
- **Fidelity**: Always run `check_fidelity()` when implementing new custom types to ensure lossless serialization.
- **Midpoint Generation**: `generate()` is deterministic and uses observed midpoints. For varied test data, use `generate_batch()` and apply manual jitter if needed.

## Key Entry Points
- `src/lib.rs`: `Engine`, `Schema`, and `SchemaKind` definitions.
- `src/grounding.rs`: T1 grounding for transcriptase types.
