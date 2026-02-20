# nexcore-phenotype

Adversarial test fixture generator for the NexVigilant Core kernel. It implements the **Phenotype** model, taking a schema (genotype) and producing intentionally mutated JSON values (phenotypes) to verify the robustness of drift detection and safety boundaries.

## Intent
To provide an automated, biologically-inspired mechanism for stress-testing data handlers and drift detectors. By generating phenotypes that exhibit specific "mutations" (e.g., type swaps, range expansions), the system can empirically verify its own ability to detect and mitigate data drift.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **∂ (Boundary)**: The primary primitive for selecting and applying specific mutation strategies.
- **μ (Mapping)**: Maps a baseline schema to a mutated JSON value.
- **σ (Sequence)**: Management of mutation batches for exhaustive testing.
- **κ (Comparison)**: Validates that applied mutations actually trigger the expected drift types.

## Mutation Types
| Mutation | AI Agent Function | Expected Drift |
| :--- | :--- | :--- |
| **TypeMismatch** | fundemental type change (Int → Str) | `TypeMismatch` |
| **AddField** | injecting extra fields into records | `ExtraField` |
| **RemoveField** | deleting required fields from records | `MissingField` |
| **RangeExpand** | pushing numbers beyond observed limits | `RangeExpansion` |
| **StructureSwap** | replacing entire objects with primitives | `TypeMismatch` |

## SOPs for Use
### Generating a Mutated Phenotype
```rust
use nexcore_phenotype::{mutate, Mutation};

let phenotype = mutate(&schema, Mutation::TypeMismatch);
// phenotype.data now contains a JSON value with incorrect types
```

### Verifying Drift Detection
```rust
let is_detected = nexcore_phenotype::verify(&schema, &phenotype);
if is_detected {
    println!("Success: Ribosome correctly detected the mutation.");
}
```

## Key Components
- **Phenotype**: The container for mutated data and its expected drift metadata.
- **Mutation Engine**: The core logic for applying various structural and scalar transformations.
- **Verification Loop**: Integration with `nexcore-ribosome` to confirm drift detection efficacy.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
