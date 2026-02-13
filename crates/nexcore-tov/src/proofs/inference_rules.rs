#![allow(unreachable_code)]
//! Standard Inference Rules
//!
//! This module provides implementations of standard logical inference rules
//! as Rust functions. Each function's type signature represents the logical
//! rule, and its implementation represents the proof.

use crate::proofs::logic_prelude::*;

// ============================================================================
// PROPOSITIONAL LOGIC RULES
// ============================================================================

/// Conjunction commutes: (P ∧ Q) → (Q ∧ P).
#[inline]
pub fn and_commutative<P, Q>(pq: And<P, Q>) -> And<Q, P> {
    And::intro(pq.right, pq.left)
}

/// Conjunction associates left: ((P ∧ Q) ∧ R) → (P ∧ (Q ∧ R)).
#[inline]
pub fn and_assoc_left<P, Q, R>(pqr: And<And<P, Q>, R>) -> And<P, And<Q, R>> {
    And::intro(pqr.left.left, And::intro(pqr.left.right, pqr.right))
}

/// Conjunction associates right: (P ∧ (Q ∧ R)) → ((P ∧ Q) ∧ R).
#[inline]
pub fn and_assoc_right<P, Q, R>(pqr: And<P, And<Q, R>>) -> And<And<P, Q>, R> {
    And::intro(And::intro(pqr.left, pqr.right.left), pqr.right.right)
}

/// Disjunction commutes: (P ∨ Q) → (Q ∨ P).
#[inline]
pub fn or_commutative<P, Q>(pq: Or<P, Q>) -> Or<Q, P> {
    match pq {
        Or::Left(p) => Or::Right(p),
        Or::Right(q) => Or::Left(q),
    }
}

/// Disjunction associates left: ((P ∨ Q) ∨ R) → (P ∨ (Q ∨ R)).
#[inline]
pub fn or_assoc_left<P, Q, R>(pqr: Or<Or<P, Q>, R>) -> Or<P, Or<Q, R>> {
    match pqr {
        Or::Left(Or::Left(p)) => Or::Left(p),
        Or::Left(Or::Right(q)) => Or::Right(Or::Left(q)),
        Or::Right(r) => Or::Right(Or::Right(r)),
    }
}

/// Disjunction associates right: (P ∨ (Q ∨ R)) → ((P ∨ Q) ∨ R).
#[inline]
pub fn or_assoc_right<P, Q, R>(pqr: Or<P, Or<Q, R>>) -> Or<Or<P, Q>, R> {
    match pqr {
        Or::Left(p) => Or::Left(Or::Left(p)),
        Or::Right(Or::Left(q)) => Or::Left(Or::Right(q)),
        Or::Right(Or::Right(r)) => Or::Right(r),
    }
}

// ============================================================================
// IMPLICATION RULES
// ============================================================================

/// Implication is reflexive: P → P.
#[inline]
pub fn implies_reflexive<P>(p: P) -> P {
    p
}

/// Implication is transitive: (P → Q) → (Q → R) → (P → R).
#[inline]
pub fn implies_transitive<P, Q, R>(pq: impl Fn(P) -> Q, qr: impl Fn(Q) -> R) -> impl Fn(P) -> R {
    move |p| qr(pq(p))
}

/// Currying: ((P ∧ Q) → R), P, Q → R.
///
/// Apply a function expecting a conjunction to separate arguments.
#[inline]
pub fn curry<P, Q, R>(f: impl FnOnce(And<P, Q>) -> R, p: P, q: Q) -> R {
    f(And::intro(p, q))
}

/// Uncurrying: (P → Q → R), (P ∧ Q) → R.
///
/// Apply a curried function to a conjunction.
#[inline]
pub fn uncurry<P, Q, R, F, G>(f: F, pq: And<P, Q>) -> R
where
    F: FnOnce(P) -> G,
    G: FnOnce(Q) -> R,
{
    f(pq.left)(pq.right)
}

// ============================================================================
// ABSORPTION LAWS
// ============================================================================

/// Absorption: P ∧ (P ∨ Q) → P.
#[inline]
pub fn absorb_and<P, Q>(p_and_porq: And<P, Or<P, Q>>) -> P {
    p_and_porq.left
}

/// Absorption: P ∨ (P ∧ Q) → P.
#[inline]
pub fn absorb_or<P: Clone, Q>(p_or_pandq: Or<P, And<P, Q>>) -> P {
    match p_or_pandq {
        Or::Left(p) => p,
        Or::Right(pq) => pq.left,
    }
}

// ============================================================================
// NEGATION RULES
// ============================================================================

/// Law of non-contradiction: ¬(P ∧ ¬P).
///
/// Returns a function that refutes any claim of P ∧ ¬P.
#[inline]
pub fn non_contradiction<P>() -> impl Fn(And<P, Not<P>>) -> Void {
    |p_and_not_p: And<P, Not<P>>| (p_and_not_p.right)(p_and_not_p.left)
}

/// Triple negation reduction: ¬¬¬P, ¬¬P ⊢ ⊥.
///
/// This is valid intuitionistically. We apply ¬¬¬P to ¬¬P directly.
/// Note: The simpler form (¬¬¬P, P ⊢ ⊥) requires closures that capture P,
/// which can't be expressed as function pointers.
#[inline]
pub fn triple_neg_elim<P>(not_not_not_p: Not<Not<Not<P>>>, double_neg_p: Not<Not<P>>) -> Void {
    not_not_not_p(double_neg_p)
}

// ============================================================================
// CONSTRUCTIVE DILEMMA
// ============================================================================

/// Constructive dilemma: (P → Q) ∧ (R → S) ∧ (P ∨ R) → (Q ∨ S).
#[inline]
pub fn constructive_dilemma<P, Q, R, S>(
    pq: impl FnOnce(P) -> Q,
    rs: impl FnOnce(R) -> S,
    p_or_r: Or<P, R>,
) -> Or<Q, S> {
    match p_or_r {
        Or::Left(p) => Or::Left(pq(p)),
        Or::Right(r) => Or::Right(rs(r)),
    }
}

/// Destructive dilemma (left case): (P → Q), ¬Q, P ⊢ ⊥.
#[inline]
pub fn destructive_dilemma_left<P, Q>(pq: impl FnOnce(P) -> Q, not_q: Not<Q>, p: P) -> Void {
    not_q(pq(p))
}

/// Destructive dilemma (right case): (R → S), ¬S, R ⊢ ⊥.
#[inline]
pub fn destructive_dilemma_right<R, S>(rs: impl FnOnce(R) -> S, not_s: Not<S>, r: R) -> Void {
    not_s(rs(r))
}

// ============================================================================
// REDUCTION AD ABSURDUM (Intuitionistic version)
// ============================================================================

/// Intuitionistic reductio: (P → ⊥), P ⊢ ⊥.
///
/// If assuming P leads to contradiction, apply the contradiction.
#[inline]
pub fn reductio_ad_absurdum<P>(p_implies_void: impl FnOnce(P) -> Void, p: P) -> Void {
    p_implies_void(p)
}

/// Proof by contradiction (weak form): ¬P → (P → Q).
///
/// "If P is false, then P implies anything" (vacuous truth).
#[inline]
pub fn vacuous_truth<P, Q>(not_p: Not<P>) -> impl Fn(P) -> Q {
    move |p: P| not_p(p).absurd()
}

// ============================================================================
// EXPORTATION/IMPORTATION
// ============================================================================

/// Exportation: (P ∧ Q → R), P, Q → R.
///
/// Apply a function expecting a conjunction to curried arguments.
#[inline]
pub fn exportation<P, Q, R>(f: impl FnOnce(And<P, Q>) -> R, p: P, q: Q) -> R {
    f(And::intro(p, q))
}

/// Importation: (P → Q → R), (P ∧ Q) → R.
///
/// Apply a curried function to a conjunction.
#[inline]
pub fn importation<P, Q, R, F, G>(f: F, pq: And<P, Q>) -> R
where
    F: FnOnce(P) -> G,
    G: FnOnce(Q) -> R,
{
    f(pq.left)(pq.right)
}

// ============================================================================
// TAUTOLOGY AND CONTRADICTION HANDLING
// ============================================================================

/// From any proposition, derive truth: P → ⊤.
#[inline]
pub fn to_truth<P>(_p: P) -> Truth {
    ()
}

/// From contradiction, derive any proposition: ⊥ → P.
///
/// Alias for ex_falso.
#[inline]
pub fn from_void<P>(void: Void) -> P {
    void.absurd()
}
