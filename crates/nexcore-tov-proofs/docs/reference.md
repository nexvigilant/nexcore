# Curry-Howard Proof System: Complete Reference

**Version:** 1.0.0
**Author:** NexVigilant LLC
**Domain:** Logic, Type Theory, Formal Verification
**Last Updated:** January 2026

---

## Executive Summary

This document provides a complete framework for:
1. **Theorem Proving in Rust** — Using the Curry-Howard correspondence to encode proofs as programs
2. **Proof Validation** — Systematic methodology for validating logical arguments

**Core Principle:** Under the Curry-Howard correspondence, *propositions are types* and *proofs are programs*. A Rust function that compiles (without `panic!`, `unsafe`, or infinite loops) represents a valid proof within intuitionistic logic.

---

## Table of Contents

### Part I: Theory
1. [The Curry-Howard Correspondence](#1-the-curry-howard-correspondence)
2. [Classical vs. Intuitionistic Logic](#2-classical-vs-intuitionistic-logic)

### Part II: Implementation
3. [The Correspondence Table](#3-the-correspondence-table)
4. [Core Type Definitions](#4-core-type-definitions)
5. [Translation Methodology](#5-translation-methodology)
6. [Proof Patterns](#6-proof-patterns)

### Part III: Validation
7. [Validation Methodology](#7-validation-methodology)
8. [Formal Validation Protocol](#8-formal-validation-protocol)

### Part IV: Reference
9. [Worked Examples](#9-worked-examples)
10. [Limitations](#10-limitations)
11. [Appendices](#11-appendices)

---

# Part I: Theory

## 1. The Curry-Howard Correspondence

### 1.1 The Core Insight

The Curry-Howard correspondence, discovered independently by Haskell Curry (1934) and William Howard (1969), reveals that logic and computation are two views of the same structure:

| Logic | Type Theory | Computation |
|-------|-------------|-------------|
| Proposition | Type | Specification |
| Proof | Term (Program) | Implementation |
| Provability | Type inhabitation | Constructibility |
| Proof normalization | Program evaluation | Execution |

**Key insight:** A type is "inhabited" if and only if there exists a program of that type. An inhabited type corresponds to a provable proposition.

### 1.2 What "Proof" Means Computationally

1. **The type signature is the claim** — `fn theorem(premise: P) -> Q` claims "P implies Q"
2. **The function body is the proof** — The implementation demonstrates how to transform evidence of P into evidence of Q
3. **Compilation is verification** — If the code compiles, the transformation is valid
4. **Execution is proof normalization** — Running the code corresponds to simplifying the proof

### 1.3 The BHK Interpretation

Under the Brouwer-Heyting-Kolmogorov interpretation (constructive logic):

| Proposition | What Constitutes a Proof |
|-------------|-------------------------|
| P ∧ Q | A pair (proof of P, proof of Q) |
| P ∨ Q | Either a proof of P or a proof of Q, with a tag indicating which |
| P → Q | A method transforming any proof of P into a proof of Q |
| ∀x. P(x) | A method producing a proof of P(a) for any given a |
| ∃x. P(x) | A pair (witness a, proof of P(a)) |
| ⊥ (falsity) | No proof exists |
| ¬P | A method transforming any proof of P into a proof of ⊥ |

This is precisely how Rust's type system works.

---

## 2. Classical vs. Intuitionistic Logic

Rust's type system embodies **intuitionistic (constructive) logic**, not classical logic.

### 2.1 What Cannot Be Proven in Rust

| Principle | Classical | Intuitionistic | Why |
|-----------|-----------|----------------|-----|
| Law of Excluded Middle | P ∨ ¬P ✓ | ✗ | Requires deciding P without evidence |
| Double Negation Elimination | ¬¬P → P ✓ | ✗ | Cannot extract witness from ¬¬P |
| Peirce's Law | ((P→Q)→P)→P ✓ | ✗ | No constructive interpretation |
| De Morgan (one direction) | ¬(P∧Q) → ¬P∨¬Q ✓ | ✗ | Requires deciding which fails |

### 2.2 What This Means Practically

- **Proof by contradiction is limited** — You can prove ¬P by deriving ⊥ from P, but you cannot prove P by deriving ⊥ from ¬P
- **Existence requires witnesses** — To prove ∃x.P(x), you must provide a specific x
- **Disjunction requires commitment** — To prove P ∨ Q, you must prove one specific disjunct

### 2.3 When You Need Classical Logic

If your proof requires classical principles, you have three options:
1. **Add as axiom** — Explicitly declare LEM or DNE as an assumption
2. **Restructure** — Many proofs can be rewritten constructively
3. **Use a classical prover** — Coq, Isabelle/HOL, or Z3 support classical reasoning

---

# Part II: Implementation

## 3. The Correspondence Table

### 3.1 Complete Mapping

| Logic | Rust | Notes |
|-------|------|-------|
| **True (⊤)** | `()` | Always inhabited—trivially provable |
| **False (⊥)** | `enum Void {}` | Never inhabited—unprovable |
| **Conjunction (P ∧ Q)** | `And<P, Q>` or `(P, Q)` | Product types |
| **Disjunction (P ∨ Q)** | `Or<P, Q>` | Sum types (enum) |
| **Implication (P → Q)** | `fn(P) -> Q` | Function types |
| **Negation (¬P)** | `fn(P) -> Void` | Function to impossibility |
| **Universal (∀x. P(x))** | `fn<T>(...) -> P<T>` | Generics |
| **Existential (∃x. P(x))** | `Exists<W, P>` | Witness + proof struct |
| **Biconditional (P ↔ Q)** | `(fn(P)->Q, fn(Q)->P)` | Pair of implications |

### 3.2 Inference Rules as Functions

| Rule | Logic | Rust |
|------|-------|------|
| Modus Ponens | P, P→Q ⊢ Q | `f(p)` |
| Modus Tollens | ¬Q, P→Q ⊢ ¬P | `\|p\| not_q(f(p))` |
| And-Introduction | P, Q ⊢ P∧Q | `And::intro(p, q)` |
| And-Elimination | P∧Q ⊢ P | `pq.left` |
| Or-Introduction | P ⊢ P∨Q | `Or::Left(p)` |
| Or-Elimination | P∨Q, P→R, Q→R ⊢ R | `match pq { ... }` |
| Hypothetical Syllogism | P→Q, Q→R ⊢ P→R | `\|p\| qr(pq(p))` |
| Ex Falso | ⊥ ⊢ P | `match void {}` |

---

## 4. Core Type Definitions

```rust
//! Core types for Curry-Howard proofs

/// Falsity (⊥) - The uninhabited type
#[derive(Debug, Clone, Copy)]
pub enum Void {}

impl Void {
    /// Ex falso quodlibet: from falsity, anything follows
    pub fn absurd<T>(self) -> T {
        match self {}
    }
}

/// Truth (⊤)
pub type Truth = ();

/// Conjunction (P ∧ Q)
#[derive(Debug, Clone, Copy)]
pub struct And<P, Q> {
    pub left: P,
    pub right: Q,
}

impl<P, Q> And<P, Q> {
    pub fn intro(p: P, q: Q) -> Self {
        And { left: p, right: q }
    }
}

/// Disjunction (P ∨ Q)
#[derive(Debug, Clone, Copy)]
pub enum Or<P, Q> {
    Left(P),
    Right(Q),
}

impl<P, Q> Or<P, Q> {
    pub fn elim<R>(self, f: impl FnOnce(P) -> R, g: impl FnOnce(Q) -> R) -> R {
        match self {
            Or::Left(p) => f(p),
            Or::Right(q) => g(q),
        }
    }
}

/// Negation (¬P) - use only for non-capturing refutations
pub type Not<P> = fn(P) -> Void;

/// Existential (∃x. P(x))
#[derive(Debug, Clone)]
pub struct Exists<Witness, Property> {
    pub witness: Witness,
    pub proof: Property,
}

impl<W, P> Exists<W, P> {
    pub fn intro(witness: W, proof: P) -> Self {
        Exists { witness, proof }
    }
}
```

---

## 5. Translation Methodology

### 5.1 The Translation Algorithm

```
INPUT:  A logical proposition in natural language or formal notation
OUTPUT: A Rust type signature

PROCEDURE:
1. IDENTIFY atomic propositions → type parameters
2. IDENTIFY the main connective (outermost operator)
3. RECURSIVELY translate sub-propositions
4. ASSEMBLE using the correspondence table
5. Premises → function parameters
6. Conclusion → return type
```

### 5.2 Translation Examples

| Natural Language | Formal | Rust Signature |
|-----------------|--------|----------------|
| "If it rains, ground is wet" | Rain → Wet | `fn(Rain) -> Wet` |
| "Either P or Q holds" | P ∨ Q | `Or<P, Q>` |
| "If P and Q, then R" | (P ∧ Q) → R | `fn(And<P, Q>) -> R` |
| "P is impossible" | ¬P | `fn(P) -> Void` |
| "Something satisfies P" | ∃x.P(x) | `Exists<X, P>` |

### 5.3 Translation Checklist

Before implementing:
- [ ] Every atomic proposition is a type parameter
- [ ] Every premise is a function parameter
- [ ] The conclusion is the return type
- [ ] No `panic!`, `todo!`, `unreachable!`, `unsafe`

---

## 6. Proof Patterns

### 6.1 Direct Proof

```rust
/// (P ∧ Q) → P
fn and_elim<P, Q>(pq: And<P, Q>) -> P {
    pq.left
}
```

### 6.2 Proof by Cases

```rust
/// P ∨ Q, P → R, Q → R ⊢ R
fn by_cases<P, Q, R>(
    pq: Or<P, Q>,
    case_p: impl FnOnce(P) -> R,
    case_q: impl FnOnce(Q) -> R,
) -> R {
    match pq {
        Or::Left(p) => case_p(p),
        Or::Right(q) => case_q(q),
    }
}
```

### 6.3 Chain of Implications

```rust
/// P → Q, Q → R ⊢ P → R
fn chain<P, Q, R>(
    pq: impl Fn(P) -> Q,
    qr: impl Fn(Q) -> R,
) -> impl Fn(P) -> R {
    move |p| qr(pq(p))
}
```

### 6.4 Proof of Negation

```rust
/// Show P leads to contradiction, derive ¬P
fn prove_negation<P>(
    derive_contradiction: impl FnOnce(P) -> Void,
    p: P,
) -> Void {
    derive_contradiction(p)
}
```

### 6.5 Existential Introduction

```rust
/// Provide witness and proof
fn exists_intro<W, P>(witness: W, proof: P) -> Exists<W, P> {
    Exists::intro(witness, proof)
}
```

---

# Part III: Validation

## 7. Validation Methodology

### 7.1 The Problem of Agreement Bias

Language models exhibit systematic tendency toward agreement. Effective logical validation requires explicit techniques to activate formal reasoning.

### 7.2 Validation Levels

| Level | Name | Best For |
|-------|------|----------|
| 1 | Devil's Advocate | Finding weak points in plans/proposals |
| 2 | Formal Logician | Academic work, reasoning chains |
| 3 | Symbolic Translation | Mathematical proofs, formal logic |
| 4 | Curry-Howard | Machine-verified proofs |
| 5 | Full Protocol | Critical proofs requiring highest rigor |

### 7.3 Quick Validation Checklist

For any proof or argument:
1. **Premise Extraction** — List all explicit and implicit assumptions
2. **Inference Audit** — Does each step follow via a named rule?
3. **Counter-Model Attempt** — Can premises be true while conclusion is false?
4. **Edge Cases** — Empty sets, zero, boundaries checked?

---

## 8. Formal Validation Protocol

### 8.1 Phase 1: Extraction

Convert natural language to formal notation:
```
CLAIM: [what is being proven]
PREMISES: [P1, P2, ...]
IMPLICIT ASSUMPTIONS: [hidden requirements]
```

### 8.2 Phase 2: Formalization

Translate to logical notation:
```
FORMAL STATEMENT: P₁, P₂, ... ⊢ C
DOMAIN: [what are the objects and predicates]
```

### 8.3 Phase 3: Inference Audit

For each step, identify the rule:
```
STEP 1: [statement] — by [rule name] from [previous steps]
STEP 2: [statement] — by [rule name] from [previous steps]
...
```

### 8.4 Phase 4: Counter-Model Search

Attempt to find an interpretation where:
- All premises are true
- Conclusion is false

If found: proof is **INVALID**
If impossible: proof is **VALID**

### 8.5 Phase 5: Verdict

```
VERDICT: [VALID / INVALID / NEEDS CLARIFICATION]
CONFIDENCE: [High / Medium / Low]
ISSUES: [list any problems found]
```

---

# Part IV: Reference

## 9. Worked Examples

### Example 1: Valid Syllogism

**Claim:** "All humans are mortal. Socrates is human. Therefore, Socrates is mortal."

**Formal:** ∀x.(Human(x) → Mortal(x)), Human(Socrates) ⊢ Mortal(Socrates)

```rust
struct Human<T>(PhantomData<T>);
struct Mortal<T>(PhantomData<T>);
struct Socrates;

fn socrates_mortal(
    all_humans_mortal: impl Fn(Human<Socrates>) -> Mortal<Socrates>,
    socrates_human: Human<Socrates>,
) -> Mortal<Socrates> {
    all_humans_mortal(socrates_human)
}
```

**Verdict:** VALID — compiles, applies universal instantiation + modus ponens.

### Example 2: Invalid (Affirming the Consequent)

**Claim:** "If it rains, ground is wet. Ground is wet. Therefore, it's raining."

**Formal:** Rain → Wet, Wet ⊢ Rain (INVALID)

```rust
// This CANNOT be implemented without escape hatches!
fn affirming_consequent(
    rain_implies_wet: fn(Rain) -> Wet,
    wet: Wet,
) -> Rain {
    // No way to construct Rain from Wet!
    // Sprinkler could have caused wetness.
}
```

**Verdict:** INVALID — cannot implement, reveals fallacious reasoning.

### Example 3: Constructive Dilemma

**Claim:** "(P → Q) ∧ (R → S) ∧ (P ∨ R) ⊢ (Q ∨ S)"

```rust
fn constructive_dilemma<P, Q, R, S>(
    pq: impl FnOnce(P) -> Q,
    rs: impl FnOnce(R) -> S,
    p_or_r: Or<P, R>,
) -> Or<Q, S> {
    match p_or_r {
        Or::Left(p) => Or::Left(pq(p)),
        Or::Right(r) => Or::Right(rs(r)),
    }
}
```

**Verdict:** VALID — compiles using case analysis.

---

## 10. Limitations

### 10.1 What Rust Cannot Express

| Limitation | Workaround |
|------------|------------|
| Full dependent types | Use type-level encodings or PhantomData |
| Termination checking | Manual review; avoid unbounded recursion |
| Classical logic | Accept as axiom or restructure proof |
| Higher-order unification | Add explicit type annotations |

### 10.2 Escape Hatches That Invalidate Proofs

These constructs allow "proving" anything and must be forbidden:

```rust
// ❌ FORBIDDEN
fn fake_proof<P>() -> P { panic!("cheating") }
fn fake_proof<P>() -> P { todo!() }
fn fake_proof<P>() -> P { unreachable!() }
fn fake_proof<P>() -> P { loop {} }
fn fake_proof<P>() -> P { unsafe { std::mem::zeroed() } }
```

### 10.3 When to Use Other Tools

| Need | Recommended Tool |
|------|-----------------|
| Full dependent types | Coq, Agda, Lean, Idris |
| Classical logic | Isabelle/HOL, Coq with Classical |
| SMT-backed automation | F*, Dafny, Z3 |
| Termination proofs | Coq, Agda with termination checker |

---

## 11. Appendices

### Appendix A: Quick Reference Card

```
╔═══════════════════════════════════════════════════════════════════╗
║              CURRY-HOWARD CORRESPONDENCE QUICK REFERENCE           ║
╠═══════════════════════════════════════════════════════════════════╣
║  CONNECTIVES                                                      ║
║  P → Q        fn(P) -> Q           Implication                    ║
║  P ∧ Q        And<P, Q>            Conjunction                    ║
║  P ∨ Q        Or<P, Q>             Disjunction                    ║
║  ¬P           fn(P) -> Void        Negation                       ║
║  ⊤            ()                   Truth                          ║
║  ⊥            enum Void {}         Falsity                        ║
║                                                                   ║
║  QUANTIFIERS                                                      ║
║  ∀x.P(x)      fn<T>(...) -> P<T>   Universal                      ║
║  ∃x.P(x)      Exists<W, P>         Existential                    ║
║                                                                   ║
║  PROOF = PROGRAM                                                  ║
║  Premises     Function parameters                                 ║
║  Conclusion   Return type                                         ║
║  Valid proof  Compiles without escape hatches                     ║
╚═══════════════════════════════════════════════════════════════════╝
```

### Appendix B: Inference Rules

**Propositional Logic:**
```
Modus Ponens:      P, P → Q  ⊢  Q
Modus Tollens:     ¬Q, P → Q  ⊢  ¬P
Hypothetical Syl:  P → Q, Q → R  ⊢  P → R
Disjunctive Syl:   P ∨ Q, ¬P  ⊢  Q
Conjunction Intro: P, Q  ⊢  P ∧ Q
Conjunction Elim:  P ∧ Q  ⊢  P
Disjunction Intro: P  ⊢  P ∨ Q
Contraposition:    P → Q  ⊢  ¬Q → ¬P
De Morgan:         ¬(P ∨ Q)  ⊢  ¬P ∧ ¬Q
                   ¬P ∧ ¬Q  ⊢  ¬(P ∨ Q)
```

**Predicate Logic:**
```
Universal Instantiation:    ∀x.P(x)  ⊢  P(a)
Universal Generalization:   P(a) [a arbitrary]  ⊢  ∀x.P(x)
Existential Instantiation:  ∃x.P(x)  ⊢  P(c) [c fresh]
Existential Generalization: P(a)  ⊢  ∃x.P(x)
```

### Appendix C: Common Fallacies

| Fallacy | Description |
|---------|-------------|
| Affirming Consequent | P→Q, Q ⊢ P (INVALID) |
| Denying Antecedent | P→Q, ¬P ⊢ ¬Q (INVALID) |
| Equivocation | Same term, different meanings |
| Circular Reasoning | Conclusion assumed in premise |
| False Dichotomy | Only two options when more exist |
| Hasty Generalization | Broad conclusion from limited examples |

### Appendix D: Glossary

| Term | Definition |
|------|------------|
| **Inhabited type** | A type with at least one value |
| **Uninhabited type** | A type with no values (e.g., `Void`) |
| **Witness** | A specific value proving an existential claim |
| **Constructive proof** | A proof that builds the conclusion explicitly |
| **Entailment (⊢)** | "Proves" or "derives" |
| **Tautology** | A formula true under all interpretations |
| **Validity** | Conclusion follows from premises by rules of inference |
| **Soundness** | Validity plus truth of all premises |

### Appendix E: Further Reading

1. **"Proofs and Types"** by Jean-Yves Girard
2. **"Types and Programming Languages"** by Benjamin Pierce
3. **"Software Foundations"** (Coq textbook)
4. **"The Little Typer"** by Friedman & Christiansen

---

## Document Control

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | January 2026 | Initial consolidated release |

---

*"Types are theorems. Programs are proofs. Compilation is verification."*

— NexVigilant LLC
