# AI Guidance — nexcore-pharmacovigilance

WHO-grounded pharmacovigilance taxonomy and semantic framework.

## Use When
- Classifying data into formal PV categories (Detection, Assessment, Understanding, Prevention).
- Reasoning about the computational complexity (Chomsky level) of a safety task.
- Calculating transfer confidence for safety logic across different domains.
- Implementing high-level "Understanding" or "Prevention" logic that requires context-sensitivity.

## Grounding Patterns
- **Comparison (κ)**: Favor concepts that use the `κ` primitive when implementing signal detection or assessment logic to maintain alignment with the core PV signature.
- **WHO Alignment**: All high-level workflows SHOULD be mapped to the four WHO pillars to ensure regulatory and semantic compliance.
- **T1 Primitives**:
  - `κ + μ`: Root primitives for signal comparison and classification mapping.
  - `σ + ς`: Root primitives for temporal sequencing of cases and state management.

## Maintenance SOPs
- **Taxonomy Expansion**: When adding a new concept (T3), ensure it is added to the correct pillar module (e.g., `detection.rs`, `prevention.rs`) and implemented with a `primitive_composition`.
- **Complexity Guard**: If implementing a new subsystem, check its `who_pillar_complexity()` to ensure the architecture (e.g., Finite Automaton vs. Turing Machine) matches the task's complexity requirements.
- **TC Matrix**: Update the `transfer_matrix` when new empirical data on cross-domain accuracy becomes available.

## Key Entry Points
- `src/lib.rs`: Taxonomy summary and primary re-exports.
- `src/lex.rs`: Symbols and tier definitions.
- `src/chomsky.rs`: Automated complexity classification.
- `src/transfer.rs`: Cross-domain transfer confidence logic.
