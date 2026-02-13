# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust library implementing the **Curry-Howard correspondence** for automated theorem verification through the type system. Logical propositions are represented as types, and proofs are programs. If a function compiles without `panic!`, `unsafe`, or infinite loops, the corresponding logical proposition is proven valid within intuitionistic logic.

## Build and Test Commands

```bash
# Check compilation (primary verification method - compilation = proof)
cargo check

# Build the library
cargo build

# Run tests (verifies proof functions compile and type-check)
cargo test

# Run a specific test
cargo test test_name

# Run tests in a specific module
cargo test propositional::
cargo test predicate::

# Check with all lints
cargo clippy
```

## Architecture

### Core Correspondence

| Logic | Rust Type |
|-------|-----------|
| True (⊤) | `()` |
| False (⊥) | `Void` (empty enum) |
| P ∧ Q | `And<P, Q>` |
| P ∨ Q | `Or<P, Q>` |
| P → Q | `fn(P) -> Q` |
| ¬P | `fn(P) -> Void` |
| ∀x. P(x) | `fn<T>(...) -> P<T>` (generics) |
| ∃x. P(x) | `Exists<Witness, Property>` |

### Module Structure

```
src/
├── lib.rs              # Crate root, module declarations, prelude re-exports
├── logic_prelude.rs    # Core types (Void, And, Or, Exists, Proof, Not) and inference rules
├── inference_rules.rs  # Standard inference rules (commutativity, associativity, dilemmas)
├── proof_patterns.rs   # Reusable proof templates (chains, cases, quantifier patterns)
└── proofs/
    ├── mod.rs          # Proofs module declarations
    ├── propositional.rs # Propositional logic theorems
    ├── predicate.rs    # Predicate logic with quantifiers (∀ and ∃)
    └── examples.rs     # Practical examples demonstrating proof techniques
```

### Key Design Principles

1. **Proofs are compile-time** - No runtime dependencies; verification happens at compilation
2. **Soundness constraints** - Lints deny `unsafe_code`, `panic`, `unwrap_used`, `expect_used`
3. **Intuitionistic logic** - Law of Excluded Middle (P ∨ ¬P) and Double Negation Elimination (¬¬P → P) are NOT provable

## Writing New Proofs

1. Define atomic propositions as zero-sized structs: `struct MyProp;`
2. Write the theorem as a function signature with premises as parameters and conclusion as return type
3. Implement the body using pattern matching, constructors, and function application
4. If it compiles without escape hatches, the proof is valid

```rust
use crate::logic_prelude::*;

// THEOREM: (P ∧ Q) → P
fn and_elim<P, Q>(pq: And<P, Q>) -> P {
    pq.left
}
```

## Proof Validation (CLAUDE.md Integration)

When validating proofs presented by users, apply the protocol from the project's CLAUDE.md:

1. **Premise Extraction** - List explicit and implicit assumptions
2. **Inference Audit** - Verify each step follows via named rule (Modus Ponens, Hypothetical Syllogism, etc.)
3. **Counter-Model Attempt** - Try to find a scenario where premises hold but conclusion fails
4. **Edge Cases** - Check degenerate cases (empty sets, zero, boundaries)

Output format:
```
CLAIM: [statement]
PREMISES: [list]
IMPLICIT ASSUMPTIONS: [hidden requirements]
INFERENCE CHAIN: [step → step with rules]
COUNTER-MODEL: [found/impossible]
VERDICT: [VALID / INVALID / NEEDS CLARIFICATION]
CONFIDENCE: [High/Medium/Low]
```

## What Cannot Be Proven (Intuitionistic Limitations)

- Law of Excluded Middle: `P ∨ ¬P`
- Double Negation Elimination: `¬¬P → P`
- Peirce's Law: `((P → Q) → P) → P`
- De Morgan (one direction): `¬(P ∧ Q) → (¬P ∨ ¬Q)`

If a proof requires these, it needs classical logic axioms.

## Documentation

See `docs/reference.md` for the complete reference including:
- Theoretical foundations and BHK interpretation
- Complete correspondence table
- Translation methodology
- Proof patterns and templates
- Validation methodology and formal protocol
- Worked examples
- Appendices (inference rules, glossary, fallacies)
