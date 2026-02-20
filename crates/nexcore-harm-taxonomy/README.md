# nexcore-harm-taxonomy

The systematic taxonomy of harm types for the NexVigilant platform, derived from the Theory of Vigilance (ToV §9). This crate connects specific harm types to the conservation laws they violate, the hierarchical levels at which they manifest, and appropriate intervention strategies.

## Intent
To provide a formal framework for classifying and reasoning about system failures. It uses a combinatorial approach (Proposition 9.0.1) to define eight core harm types (A-H) and one extension (Type I for autonomous agents), ensuring an exhaustive classification of all possible harm events.

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **κ (Comparison)**: Used for classifying harm events based on binary characteristics and mapping them to conservation laws.
- **Σ (Sum)**: Represents the combinatorial aggregation of multiplicity, temporal, and determinism factors.
- **∂ (Boundary)**: Defines the manifestation levels and safety boundaries for each harm type.
- **→ (Causality)**: Manages the propagation of harm across hierarchical levels (Axiom 5).
- **N (Quantity)**: Computes propagation probabilities and manifestation thresholds.

## The Eight Harm Types (§9.1)
| Type | Name | Primary Mechanism | Manifestation Levels |
| :--- | :--- | :--- | :--- |
| **A** | Acute | Law 1 (Mass): rapid accumulation | 4-6 |
| **B** | Cumulative | Law 1 (Mass): chronic exposure | 5-7 |
| **C** | Off-Target | Law 2 (Energy): unintended binding | 3-5 |
| **D** | Cascade | Law 4 (Flux): imbalance propagation | 4-7 |
| **E** | Idiosyncratic | θ-space: unusual susceptibility | 3-6 |
| **F** | Saturation | Law 8 (Capacity): limit exceeded | 3-5 |
| **G** | Interaction | Law 5 (Catalyst): competitive inhibition | 4-6 |
| **H** | Population | θ-distribution: disparate impact | 6-8 |

## SOPs for Use
### Classifying a Harm Event
```rust
use nexcore_harm_taxonomy::{classify_harm_event, PerturbationMultiplicity, TemporalProfile, ResponseDeterminism};

let result = classify_harm_event(
    PerturbationMultiplicity::Single,
    TemporalProfile::Acute,
    ResponseDeterminism::Deterministic,
);
println!("Primary Harm Type: {}", result.primary_type.name());
```

### Verifying Exhaustiveness
Use `verify_exhaustiveness()` to ensure that the currently defined types cover all theoretical 2³ combinations of characteristics.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
