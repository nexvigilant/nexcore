# nexcore-tov-proofs

Formal theorem verification engine for the NexVigilant platform. This crate implements the **Curry-Howard Correspondence**, allowing logical propositions to be expressed as Rust types and proofs to be implemented as programs.

## Intent
To provide absolute certainty in safety-critical logic. If a proof function type-checks, the corresponding logical theorem is proven valid. This ensures that the axioms of the Theory of Vigilance (ToV) are sound and correctly implemented.

## The Correspondence
| Logic Concept | Rust Manifestation |
| :--- | :--- |
| **Proposition** | Type |
| **Proof** | Program / Function |
| **Conjunction (P ∧ Q)** | `And<P, Q>` |
| **Disjunction (P ∨ Q)** | `Or<P, Q>` |
| **Implication (P → Q)** | `fn(P) -> Q` |
| **Negation (¬P)** | `fn(P) -> Void` |

## T1 Grounding (Lex Primitiva)
Dominant Primitives:
- **→ (Causality)**: The primary primitive for logical implication and proof flow.
- **κ (Comparison)**: Used for comparing propositions and checking for logical consistency.
- **∅ (Void)**: Represents the "False" proposition (`Void`) in the correspondence.
- **∃ (Existence)**: Implements existential quantification (`Exists<Witness, Property>`).

## Core Logic Components
- **logic_prelude**: Fundamental types for intuitionistic logic (`And`, `Or`, `Not`, `Truth`).
- **inference_rules**: Standard rules like Modus Ponens, Conjunction Introduction, etc.
- **codex_compliance**: Proof patterns specifically for verifying Codex/T1 grounding rules.
- **kani_proofs**: (Optional) Model checking integration via the Kani framework.

## SOPs for Use
### Proving a Theorem
```rust
use nexcore_tov_proofs::prelude::*;

// Define premises and conclusion
struct P; struct Q;

// State theorem: P ∧ Q → P
fn theorem(premise: And<P, Q>) -> P {
    premise.left // Proof by conjunction elimination
}
// Compiles = Proven.
```

### Verifying a Proof
A proof is only valid if it contains NO `panic!()`, `todo!()`, `unsafe`, or infinite loops.

## License
Proprietary. Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
