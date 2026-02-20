# grounded

A foundational library for epistemological grounding and evidence-based reasoning in the NexVigilant Core kernel. It provides the **GROUNDED** framework (Giving Reasoning Observable, Unified, Nested, Developmental, Evidence-based Dynamics), enabling AI agents to explicitly track uncertainty and verify reasoning against reality.

## Intent
To move beyond "blind" reasoning towards a system where every conclusion carries a verifiable confidence score and is backed by a structured `EvidenceChain`. It enforces a feedback loop between internal hypotheses and external world observations.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **× (Product)**: The primary primitive for the composition of uncertainty and data.
- **N (Quantity)**: Represents scalar confidence values [0.0, 1.0].
- **∂ (Boundary)**: Defines confidence bands (High, Medium, Low) for decision gating.
- **→ (Causality)**: Manages the `GroundedLoop` (Hypothesis → Experiment → Outcome).
- **π (Persistence)**: Durable storage of `EvidenceChain` and learning history.

## Core Components
- **Uncertain<T>**: A wrapper that forces explicit handling of uncertainty at the type level.
- **ConfidenceBand**: Categorical classification of confidence for exhaustive matching.
- **GroundedLoop**: The primary control loop for experimental reasoning.
- **EvidenceChain**: A sequential record of reasoning steps and supporting evidence.
- **SqliteStore**: A persistent store for long-term experience and learning.

## SOPs for Use
### Handling Uncertain Values
```rust
use grounded::{Uncertain, ConfidenceBand};

let result: Uncertain<u32> = Uncertain::new(42, 0.95);

match result.band() {
    ConfidenceBand::High => println!("Proceed with confidence."),
    ConfidenceBand::Medium => println!("Requires secondary verification."),
    _ => println!("Reject or re-evaluate."),
}
```

### Running a Grounded Loop
1. Define a `Hypothesis` (expected outcome).
2. Execute an `Experiment` (external action).
3. Observe the `Outcome` and verify against the hypothesis.
4. Record the `Learning` to persistent storage.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
