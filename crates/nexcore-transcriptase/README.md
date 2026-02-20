# nexcore-transcriptase

Bidirectional Data ↔ Schema engine for the NexVigilant Core kernel. It implements a **Reverse Transcriptase** model for structured data, enabling schema inference from JSON records, range-based merging, and automated synthesis of boundary violations.

## Intent
To bridge the gap between unstructured data observations and formal structural knowledge. It allows AI agents to "learn" the expected schema of a data source by observing examples, and then use that learned schema to detect anomalies or generate valid synthetic test data.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **μ (Mapping)**: The core primitive for mapping JSON values to `SchemaKind` types.
- **σ (Sequence)**: Management of the pipeline from observation to merged schema.
- **κ (Comparison)**: Used for merging schemas (widening ranges) and synthesizing violations.
- **∂ (Boundary)**: Explicit generation of `SchemaViolation` assertions based on observed ranges.

## Pipeline
1. **observe()**: Ingests a JSON record and infers a local schema.
2. **merge()**: Combines local schemas into a global representation, widening ranges.
3. **synthesize_violations()**: Generates assertions about what SHOULD NOT be true (boundary crossings).
4. **generate()**: Produces synthetic JSON records that conform to the observed schema.

## SOPs for Use
### Inferring and Merging Schemas
```rust
use nexcore_transcriptase::{Engine, TranscriptionOutput};

let mut engine = Engine::new();
engine.observe(&serde_json::json!({"score": 10, "drug": "A"}));
engine.observe(&serde_json::json!({"score": 90, "drug": "B"}));

let schema = engine.schema().unwrap();
// score is now Int { min: 10, max: 90, sum: 100 }
```

### Synthesizing Violations
```rust
let violations = nexcore_transcriptase::synthesize_violations(&schema);
for v in violations {
    println!("Boundary check: {}", v.assertion); // e.g. "score < 10"
}
```

## Key Components
- **Schema**: The recursive structural representation of observed data.
- **SchemaKind**: Enumeration of supported types (Record, Array, Int, Str, etc.).
- **Fidelity**: Metrics for tracking round-trip serialization correctness.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
