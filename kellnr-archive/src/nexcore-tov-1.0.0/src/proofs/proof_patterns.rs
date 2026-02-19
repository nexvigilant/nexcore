#![allow(unreachable_code)]
//! Proof Patterns Library
//!
//! This module provides common proof patterns as templates that can be
//! instantiated for specific theorems. Each pattern encapsulates a proof
//! strategy commonly used in mathematical reasoning.

use crate::proofs::logic_prelude::*;

// ============================================================================
// DIRECT PROOF PATTERNS
// ============================================================================

/// Pattern: Chain of implications.
///
/// Given P → Q, Q → R, R → S, derive P → S.
#[inline]
pub fn chain2<P, Q, R>(pq: impl Fn(P) -> Q, qr: impl Fn(Q) -> R) -> impl Fn(P) -> R {
    move |p| qr(pq(p))
}

#[inline]
pub fn chain3<P, Q, R, S>(
    pq: impl Fn(P) -> Q,
    qr: impl Fn(Q) -> R,
    rs: impl Fn(R) -> S,
) -> impl Fn(P) -> S {
    move |p| rs(qr(pq(p)))
}

#[inline]
pub fn chain4<P, Q, R, S, T>(
    pq: impl Fn(P) -> Q,
    qr: impl Fn(Q) -> R,
    rs: impl Fn(R) -> S,
    st: impl Fn(S) -> T,
) -> impl Fn(P) -> T {
    move |p| st(rs(qr(pq(p))))
}

// ============================================================================
// PROOF BY CASES
// ============================================================================

/// Pattern: Exhaustive case analysis.
///
/// If P ∨ Q holds, and we can derive R from either case, then R holds.
#[inline]
pub fn by_cases<P, Q, R>(
    disjunction: Or<P, Q>,
    case_left: impl FnOnce(P) -> R,
    case_right: impl FnOnce(Q) -> R,
) -> R {
    disjunction.elim(case_left, case_right)
}

/// Pattern: Three-way case analysis.
#[inline]
pub fn by_cases3<P, Q, R, S>(
    disjunction: Or<P, Or<Q, R>>,
    case_p: impl FnOnce(P) -> S,
    case_q: impl FnOnce(Q) -> S,
    case_r: impl FnOnce(R) -> S,
) -> S {
    match disjunction {
        Or::Left(p) => case_p(p),
        Or::Right(Or::Left(q)) => case_q(q),
        Or::Right(Or::Right(r)) => case_r(r),
    }
}

// ============================================================================
// PROOF BY CONTRAPOSITION
// ============================================================================

// NOTE: Classical contraposition (¬Q → ¬P) ⊢ (P → Q) cannot be implemented
// in intuitionistic logic. The reverse direction (P → Q) ⊢ (¬Q → ¬P) IS valid
// and is implemented in logic_prelude::contraposition.
//
// To use classical logic, you would need to add the Law of Excluded Middle
// or Double Negation Elimination as axioms.

// ============================================================================
// PROOF BY CONTRADICTION (Intuitionistic version)
// ============================================================================

/// Pattern: Derive ⊥ by assuming P and applying its refutation.
///
/// This is the intuitionistically valid form of proof by contradiction.
/// Given a way to derive contradiction from P, and P itself, derive ⊥.
#[inline]
pub fn prove_negation<P>(derive_contradiction: impl FnOnce(P) -> Void, p: P) -> Void {
    derive_contradiction(p)
}

/// Pattern: From contradiction, conclude anything.
///
/// If we have both P and ¬P, we can derive any Q.
#[inline]
pub fn from_contradiction<P, Q>(p: P, not_p: Not<P>) -> Q {
    not_p(p).absurd()
}

// ============================================================================
// UNIVERSAL QUANTIFICATION PATTERNS
// ============================================================================

/// Pattern: Universal introduction.
///
/// To prove ∀x. P(x), define a generic function that works for any x.
///
/// This is implicit in Rust's generics - any function `fn<T>(x: T) -> P<T>`
/// is a proof of ∀T. P(T).
#[inline]
pub fn universal_intro<T, P>(proof_for_arbitrary: impl Fn(T) -> P) -> impl Fn(T) -> P {
    proof_for_arbitrary
}

/// Pattern: Universal elimination (instantiation).
///
/// From ∀x. P(x), derive P(a) for any specific a.
#[inline]
pub fn universal_elim<T, P>(universal_proof: impl Fn(T) -> P, specific_instance: T) -> P {
    universal_proof(specific_instance)
}

// ============================================================================
// EXISTENTIAL QUANTIFICATION PATTERNS
// ============================================================================

/// Pattern: Existential introduction.
///
/// Provide a witness and proof that the property holds for that witness.
#[inline]
pub fn existential_intro<W, P>(witness: W, proof: P) -> Exists<W, P> {
    Exists::intro(witness, proof)
}

/// Pattern: Existential elimination.
///
/// From ∃x. P(x) and a proof that P(x) → Q (for arbitrary x), derive Q.
#[inline]
pub fn existential_elim<W, P, Q>(exists: Exists<W, P>, use_witness: impl FnOnce(W, P) -> Q) -> Q {
    exists.elim(use_witness)
}

// ============================================================================
// BICONDITIONAL PROOF PATTERNS
// ============================================================================

/// Pattern: Prove P ↔ Q by proving both directions.
#[inline]
pub fn prove_iff<P: 'static, Q: 'static>(
    forward: impl Fn(P) -> Q + 'static,
    backward: impl Fn(Q) -> P + 'static,
) -> Iff<P, Q> {
    Iff::new(forward, backward)
}

/// Pattern: Use biconditional to substitute.
#[inline]
pub fn substitute_iff<P: 'static, Q: 'static, R>(
    iff: Iff<P, Q>,
    context_on_p: impl FnOnce(P) -> R,
    q: Q,
) -> R {
    context_on_p(iff.backward(q))
}

// ============================================================================
// WEAKENING AND STRENGTHENING
// ============================================================================

/// Pattern: Weaken a conjunction - P ∧ Q → P.
#[inline]
pub fn weaken_and_left<P, Q>(pq: And<P, Q>) -> P {
    pq.left
}

/// Pattern: Weaken a conjunction - P ∧ Q → Q.
#[inline]
pub fn weaken_and_right<P, Q>(pq: And<P, Q>) -> Q {
    pq.right
}

/// Pattern: Strengthen to disjunction - P → P ∨ Q.
#[inline]
pub fn strengthen_to_or_left<P, Q>(p: P) -> Or<P, Q> {
    Or::Left(p)
}

/// Pattern: Strengthen to disjunction - Q → P ∨ Q.
#[inline]
pub fn strengthen_to_or_right<P, Q>(q: Q) -> Or<P, Q> {
    Or::Right(q)
}

// ============================================================================
// LEMMA APPLICATION PATTERN
// ============================================================================

/// Pattern: Use a lemma in a proof.
///
/// Given a lemma (proven elsewhere) and a way to use it, derive the conclusion.
#[inline]
pub fn use_lemma<Lemma, Goal>(lemma: Lemma, application: impl FnOnce(Lemma) -> Goal) -> Goal {
    application(lemma)
}

/// Pattern: Apply two lemmas.
#[inline]
pub fn use_lemmas2<L1, L2, Goal>(
    lemma1: L1,
    lemma2: L2,
    application: impl FnOnce(L1, L2) -> Goal,
) -> Goal {
    application(lemma1, lemma2)
}

// ============================================================================
// CONDITIONAL INTRODUCTION PATTERN
// ============================================================================

/// Pattern: Prove P → Q by assuming P and deriving Q.
///
/// This is the standard way to prove implications: assume the antecedent
/// and show the consequent follows.
#[inline]
pub fn conditional_intro<P, Q>(derive_q_from_p: impl Fn(P) -> Q) -> impl Fn(P) -> Q {
    derive_q_from_p
}

// ============================================================================
// COMBINATION PATTERNS
// ============================================================================

/// Pattern: Split and conquer - prove P ∧ Q by proving each separately.
#[inline]
pub fn split_goal<P, Q>(prove_p: impl FnOnce() -> P, prove_q: impl FnOnce() -> Q) -> And<P, Q> {
    And::intro(prove_p(), prove_q())
}

/// Pattern: Transform conjunction into curried application.
///
/// Apply (P ∧ Q → R) as a curried function: given f, p, q, return R.
#[inline]
pub fn and_to_impl<P, Q, R>(f: impl FnOnce(And<P, Q>) -> R, p: P, q: Q) -> R {
    f(And::intro(p, q))
}

/// Pattern: Transform curried function into conjunction application.
///
/// Apply (P → Q → R) to (P ∧ Q): given f and pq, return R.
#[inline]
pub fn impl_to_and<P, Q, R, F, G>(f: F, pq: And<P, Q>) -> R
where
    F: FnOnce(P) -> G,
    G: FnOnce(Q) -> R,
{
    f(pq.left)(pq.right)
}

// ============================================================================
// PROOF DOCUMENTATION MACRO (for demonstration)
// ============================================================================

/// A marker trait for documented proofs.
///
/// Implementing this trait indicates that a function serves as a formal proof,
/// with the type signature representing the theorem statement.
pub trait ProofOf<Theorem> {
    /// Describes the proof strategy used.
    const STRATEGY: &'static str;

    /// Describes the theorem in natural language.
    const THEOREM_STATEMENT: &'static str;
}

// Example usage (not functional, just documentation):
//
// impl ProofOf<fn(And<P, Q>) -> P> for AndElimLeft {
//     const STRATEGY: &'static str = "Direct: Extract left component";
//     const THEOREM_STATEMENT: &'static str = "Conjunction elimination (left): (P ∧ Q) → P";
// }
