# AI Guidance — nexcore-tov-proofs

Theorem verification and Curry-Howard proof engine.

## Use When
- Proving the soundness of a new safety axiom or regulatory rule.
- Implementing compile-time checks for Codex/T1 compliance.
- Reasoning about logical implications in complex PV decision loops.
- Verifying the "Falsehood" of a state via negation (fn(P) -> Void).

## Grounding Patterns
- **Constructive Only**: Remember that this engine implements *intuitionistic* logic. You cannot use the Law of Excluded Middle (P ∨ ¬P) or Double Negation Elimination (¬¬P → P).
- **Proof-as-Program**: Every function body IS the proof. Keep it concise and use the `inference_rules` module to delegate standard steps.
- **T1 Primitives**:
  - `→ + κ`: Root primitives for implication and equality/comparison proofs.
  - `∅ + ∃`: Root primitives for contradiction (False) and existential proofs.

## Maintenance SOPs
- **Zero Panic**: Never use `unwrap()` or `expect()` in a proof function; it invalidates the formal guarantee.
- **No Unsafe**: Strictly enforce `#![forbid(unsafe_code)]`.
- **Termination**: Ensure all proof functions are guaranteed to terminate (no recursion without a clear base case).
- **Module Hygiene**: Place new theorems in `src/proofs/` to keep them separate from the core engine.

## Key Entry Points
- `src/logic_prelude.rs`: The atomic logical types.
- `src/inference_rules.rs`: Reusable building blocks for proof construction.
- `src/codex_compliance.rs`: Axioms for the NexCore type hierarchy.
