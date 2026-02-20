# nexcore-pharmacovigilance

A comprehensive, WHO-grounded pharmacovigilance taxonomy encoded as typed Rust. This crate provides the formal semantic framework for the entire NexVigilant platform, classifying 120+ concepts across the four pillars of pharmacovigilance: Detection, Assessment, Understanding, and Prevention.

## Intent
To provide a stable, formal ontology for drug safety operations. It bridges the WHO definitions of pharmacovigilance with the Lex Primitiva symbolic foundation, enabling agents to reason about safety concepts across different levels of linguistic and mathematical complexity.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **κ (Comparison)**: The signature primitive for PV. It dominates the taxonomy, reflecting that PV is fundamentally about *systematic comparison* (observed vs. expected).
- **μ (Mapping)**: Used for mapping raw data to taxonomy concepts and clinical observations to regulatory categories.
- **σ (Sequence)**: Manages the temporal ordering of events and the progression through the WHO verbs (Detection → Assessment → Understanding → Prevention).
- **ς (State)**: Represents the state of safety knowledge at any given point in the lifecycle.

## WHO Pillar Complexity (Chomsky Hierarchy)
| WHO Pillar | Chomsky Level | Automaton | Complexity |
| :--- | :--- | :--- | :--- |
| **Detection** | Type-3 | Finite Automaton | O(N) linear scan |
| **Assessment** | Type-2 | Pushdown Automaton | Context-free |
| **Understanding**| Type-1 | Linear Bounded | Context-sensitive |
| **Prevention** | Type-0 | Turing Machine | Unrestricted |

## Core Components
- **Taxonomy**: A 4-tier hierarchy (T1 to T3) covering 120+ safety concepts.
- **TransferConfidence**: Metrics for predicting how well safety logic transfers to clinical, epidemiological, or regulatory domains.
- **ChomskyAnalyzer**: Classifies the computational complexity of different PV subsystems.

## SOPs for Use
### Querying the Taxonomy
```rust
use nexcore_pharmacovigilance::taxonomy_summary;
let summary = taxonomy_summary();
println!("Total T3 Concepts: {}", summary.t3);
```

### Checking Transfer Confidence
```rust
use nexcore_pharmacovigilance::transfer::{lookup_transfer, TransferDomain};
let confidence = lookup_transfer(TransferDomain::ClinicalTrials);
```

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
