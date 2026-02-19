#![allow(unreachable_code, unreachable_pub)]
//! Propositional Logic Proofs
//!
//! This module contains proofs of standard propositional logic theorems,
//! demonstrating the Curry-Howard correspondence in action.
//!
//! Each function's type signature is the theorem statement, and the
//! implementation is the proof. Compilation verifies validity.

use crate::proofs::logic_prelude::*;

// ============================================================================
// CONJUNCTION THEOREMS
// ============================================================================

/// THEOREM: (P ∧ Q) → P
///
/// "From a conjunction, we can extract the left conjunct."
///
/// PROOF: Direct - extract the left component of the pair.
pub fn and_elimination_left<P, Q>(pq: And<P, Q>) -> P {
    pq.left
}

/// THEOREM: (P ∧ Q) → Q
///
/// "From a conjunction, we can extract the right conjunct."
///
/// PROOF: Direct - extract the right component of the pair.
pub fn and_elimination_right<P, Q>(pq: And<P, Q>) -> Q {
    pq.right
}

/// THEOREM: P → Q → (P ∧ Q)
///
/// "Given P and Q separately, we can form their conjunction."
///
/// PROOF: Direct - construct the pair.
pub fn and_introduction<P, Q>(p: P, q: Q) -> And<P, Q> {
    And { left: p, right: q }
}

/// THEOREM: (P ∧ Q) → (Q ∧ P)
///
/// "Conjunction is commutative."
///
/// PROOF: Direct - swap the components.
pub fn and_commutativity<P, Q>(pq: And<P, Q>) -> And<Q, P> {
    And {
        left: pq.right,
        right: pq.left,
    }
}

/// THEOREM: ((P ∧ Q) ∧ R) → (P ∧ (Q ∧ R))
///
/// "Conjunction is associative (left to right)."
///
/// PROOF: Direct - restructure the nested pairs.
pub fn and_associativity_left<P, Q, R>(pqr: And<And<P, Q>, R>) -> And<P, And<Q, R>> {
    And {
        left: pqr.left.left,
        right: And {
            left: pqr.left.right,
            right: pqr.right,
        },
    }
}

/// THEOREM: (P ∧ (Q ∧ R)) → ((P ∧ Q) ∧ R)
///
/// "Conjunction is associative (right to left)."
///
/// PROOF: Direct - restructure the nested pairs.
pub fn and_associativity_right<P, Q, R>(pqr: And<P, And<Q, R>>) -> And<And<P, Q>, R> {
    And {
        left: And {
            left: pqr.left,
            right: pqr.right.left,
        },
        right: pqr.right.right,
    }
}

// ============================================================================
// DISJUNCTION THEOREMS
// ============================================================================

/// THEOREM: P → (P ∨ Q)
///
/// "From P, we can conclude P or Q (choosing left)."
///
/// PROOF: Direct - inject into left alternative.
pub fn or_introduction_left<P, Q>(p: P) -> Or<P, Q> {
    Or::Left(p)
}

/// THEOREM: Q → (P ∨ Q)
///
/// "From Q, we can conclude P or Q (choosing right)."
///
/// PROOF: Direct - inject into right alternative.
pub fn or_introduction_right<P, Q>(q: Q) -> Or<P, Q> {
    Or::Right(q)
}

/// THEOREM: (P ∨ Q) → (P → R) → (Q → R) → R
///
/// "Disjunction elimination via case analysis."
///
/// PROOF: Pattern match on the disjunction and apply relevant function.
pub fn or_elimination<P, Q, R>(
    p_or_q: Or<P, Q>,
    p_to_r: impl FnOnce(P) -> R,
    q_to_r: impl FnOnce(Q) -> R,
) -> R {
    match p_or_q {
        Or::Left(p) => p_to_r(p),
        Or::Right(q) => q_to_r(q),
    }
}

/// THEOREM: (P ∨ Q) → (Q ∨ P)
///
/// "Disjunction is commutative."
///
/// PROOF: Case analysis - swap the injection.
pub fn or_commutativity<P, Q>(pq: Or<P, Q>) -> Or<Q, P> {
    match pq {
        Or::Left(p) => Or::Right(p),
        Or::Right(q) => Or::Left(q),
    }
}

// ============================================================================
// IMPLICATION THEOREMS
// ============================================================================

/// THEOREM: P → P
///
/// "Identity: P implies P (reflexivity of implication)."
///
/// PROOF: Direct - return the input unchanged.
pub fn implication_reflexive<P>(p: P) -> P {
    p
}

/// THEOREM: (P → Q) → (Q → R) → (P → R)
///
/// "Hypothetical syllogism (transitivity of implication)."
///
/// PROOF: Direct - compose the functions.
pub fn hypothetical_syllogism<P, Q, R>(
    p_to_q: impl Fn(P) -> Q,
    q_to_r: impl Fn(Q) -> R,
) -> impl Fn(P) -> R {
    move |p: P| q_to_r(p_to_q(p))
}

/// THEOREM: P → (Q → P)
///
/// "Weakening: Given P, Q implies P regardless of Q."
///
/// PROOF: Direct - ignore Q and return P.
pub fn weakening<P: Clone, Q>(p: P) -> impl Fn(Q) -> P {
    move |_q: Q| p.clone()
}

// ============================================================================
// NEGATION THEOREMS
// ============================================================================

/// THEOREM: P, ¬P ⊢ ⊥
///
/// "Double negation introduction (applied form)."
///
/// PROOF: Given P and a function ¬P = (P → ⊥), apply the function to P.
pub fn double_negation_introduction<P>(p: P, not_p: Not<P>) -> Void {
    not_p(p)
}

/// THEOREM: ¬(P ∧ ¬P)
///
/// "Law of non-contradiction: P and not-P cannot both hold."
///
/// PROOF: Given (P ∧ ¬P), apply ¬P to P to get ⊥.
pub fn non_contradiction<P>() -> impl Fn(And<P, Not<P>>) -> Void {
    |p_and_not_p: And<P, Not<P>>| {
        let p = p_and_not_p.left;
        let not_p = p_and_not_p.right;
        not_p(p)
    }
}

/// THEOREM: (P → Q), ¬Q, P ⊢ ⊥
///
/// "Contraposition (applied form): If P implies Q, and not-Q, then not-P."
///
/// PROOF: Given P → Q, ¬Q, and P, derive Q then apply ¬Q.
pub fn contraposition<P, Q>(p_to_q: impl FnOnce(P) -> Q, not_q: Not<Q>, p: P) -> Void {
    not_q(p_to_q(p))
}

/// THEOREM: (P ∨ Q) → ¬P → Q
///
/// "Disjunctive syllogism: If P or Q, and not P, then Q."
///
/// PROOF: Case analysis - the P case contradicts ¬P, so we must have Q.
pub fn disjunctive_syllogism<P, Q>(p_or_q: Or<P, Q>, not_p: Not<P>) -> Q {
    match p_or_q {
        Or::Left(p) => {
            let void: Void = not_p(p);
            void.absurd()
        }
        Or::Right(q) => q,
    }
}

// ============================================================================
// MODUS PONENS AND MODUS TOLLENS
// ============================================================================

/// THEOREM: P → (P → Q) → Q
///
/// "Modus ponens: From P and P → Q, derive Q."
///
/// PROOF: Apply the function to the argument.
pub fn modus_ponens<P, Q>(p: P, p_to_q: impl FnOnce(P) -> Q) -> Q {
    p_to_q(p)
}

/// THEOREM: (P → Q), ¬Q, P ⊢ ⊥
///
/// "Modus tollens (applied form): From P → Q, ¬Q, and P, derive contradiction."
///
/// PROOF: Same as contraposition.
pub fn modus_tollens<P, Q>(p_to_q: impl FnOnce(P) -> Q, not_q: Not<Q>, p: P) -> Void {
    not_q(p_to_q(p))
}

// ============================================================================
// DISTRIBUTIVITY THEOREMS
// ============================================================================

/// THEOREM: (P ∧ (Q ∨ R)) → ((P ∧ Q) ∨ (P ∧ R))
///
/// "Conjunction distributes over disjunction."
///
/// PROOF: Case analysis on Q ∨ R.
pub fn and_distributes_over_or<P: Clone, Q, R>(
    p_and_qr: And<P, Or<Q, R>>,
) -> Or<And<P, Q>, And<P, R>> {
    let p = p_and_qr.left;
    match p_and_qr.right {
        Or::Left(q) => Or::Left(And { left: p, right: q }),
        Or::Right(r) => Or::Right(And { left: p, right: r }),
    }
}

/// THEOREM: (P ∨ (Q ∧ R)) → ((P ∨ Q) ∧ (P ∨ R))
///
/// "Disjunction distributes over conjunction."
///
/// PROOF: Case analysis on the disjunction.
pub fn or_distributes_over_and<P: Clone, Q, R>(
    p_or_qr: Or<P, And<Q, R>>,
) -> And<Or<P, Q>, Or<P, R>> {
    match p_or_qr {
        Or::Left(p) => And {
            left: Or::Left(p.clone()),
            right: Or::Left(p),
        },
        Or::Right(qr) => And {
            left: Or::Right(qr.left),
            right: Or::Right(qr.right),
        },
    }
}

// ============================================================================
// ABSORPTION THEOREMS
// ============================================================================

/// THEOREM: (P ∧ (P ∨ Q)) → P
///
/// "Absorption law for conjunction."
///
/// PROOF: Direct - just extract P from the conjunction.
pub fn absorption_and<P, Q>(p_and_por: And<P, Or<P, Q>>) -> P {
    p_and_por.left
}

/// THEOREM: (P ∨ (P ∧ Q)) → P
///
/// "Absorption law for disjunction."
///
/// PROOF: Case analysis - both cases yield P.
pub fn absorption_or<P: Clone, Q>(p_or_pand: Or<P, And<P, Q>>) -> P {
    match p_or_pand {
        Or::Left(p) => p,
        Or::Right(pq) => pq.left,
    }
}

// ============================================================================
// DE MORGAN'S LAWS (Intuitionistically valid directions)
// ============================================================================

/// THEOREM: ¬(P ∨ Q), P ⊢ ⊥
///
/// "De Morgan (left): From ¬(P ∨ Q) and P, derive contradiction."
///
/// PROOF: From ¬(P ∨ Q) and P, construct P ∨ Q and apply negation.
pub fn de_morgan_not_or_left<P, Q>(not_p_or_q: Not<Or<P, Q>>, p: P) -> Void {
    not_p_or_q(Or::Left(p))
}

/// THEOREM: ¬(P ∨ Q), Q ⊢ ⊥
///
/// "De Morgan (right): From ¬(P ∨ Q) and Q, derive contradiction."
///
/// PROOF: From ¬(P ∨ Q) and Q, construct P ∨ Q and apply negation.
pub fn de_morgan_not_or_right<P, Q>(not_p_or_q: Not<Or<P, Q>>, q: Q) -> Void {
    not_p_or_q(Or::Right(q))
}

/// THEOREM: (¬P ∧ ¬Q), (P ∨ Q) ⊢ ⊥
///
/// "De Morgan converse: From conjunction of negations and disjunction, derive contradiction."
///
/// PROOF: From P ∨ Q, do case analysis using ¬P or ¬Q.
pub fn de_morgan_not_or_converse<P, Q>(
    not_p_and_not_q: And<Not<P>, Not<Q>>,
    p_or_q: Or<P, Q>,
) -> Void {
    match p_or_q {
        Or::Left(p) => (not_p_and_not_q.left)(p),
        Or::Right(q) => (not_p_and_not_q.right)(q),
    }
}

// NOTE: ¬(P ∧ Q) → (¬P ∨ ¬Q) is NOT intuitionistically provable!
// This would require deciding which of P or Q fails, which requires LEM.

// ============================================================================
// CURRYING AND UNCURRYING
// ============================================================================

/// THEOREM: ((P ∧ Q) → R) → (P → (Q → R))
///
/// "Currying: A function of a pair can be curried."
///
/// PROOF: Take P first, then Q, then construct the pair and apply.
pub fn curry<P: Clone + 'static, Q: 'static, R: 'static>(
    f: impl Fn(And<P, Q>) -> R + Clone + 'static,
) -> impl Fn(P) -> Box<dyn Fn(Q) -> R> {
    move |p: P| {
        let f_clone = f.clone();
        let p_clone = p.clone();
        Box::new(move |q: Q| {
            f_clone(And {
                left: p_clone.clone(),
                right: q,
            })
        })
    }
}

/// THEOREM: (P → (Q → R)), (P ∧ Q) ⊢ R
///
/// "Uncurrying: A curried function applied to a pair."
///
/// PROOF: Extract P and Q from the pair and apply both.
pub fn uncurry<P, Q, R, F, G>(f: F, pq: And<P, Q>) -> R
where
    F: FnOnce(P) -> G,
    G: FnOnce(Q) -> R,
{
    f(pq.left)(pq.right)
}

// ============================================================================
// PROOFS ABOUT TRUTH AND FALSITY
// ============================================================================

/// THEOREM: ⊤
///
/// "Truth is always provable."
///
/// PROOF: Return the unit value.
pub fn truth_is_provable() -> Truth {
    ()
}

/// THEOREM: ⊥ → P
///
/// "Ex falso quodlibet: From falsity, anything follows."
///
/// PROOF: Match on the empty Void type - no cases to handle.
pub fn ex_falso_quodlibet<P>(void: Void) -> P {
    match void {}
}

/// THEOREM: P → (⊥ → P)
///
/// "Given P, falsity still implies P (vacuously)."
///
/// PROOF: Ignore the Void and use ex falso.
pub fn falsity_implies_anything<P>(_p: P) -> impl Fn(Void) -> P {
    |void: Void| void.absurd()
}

// ============================================================================
// COMPILATION TESTS
// ============================================================================

#[cfg(test)]
mod compilation_tests {
    use super::*;

    /// This test exists solely to verify that proofs compile.
    /// If this module compiles, all theorems are verified.
    #[test]
    fn all_proofs_typecheck() {
        // Instantiate proof functions with unit types to verify they compile
        let _ = and_elimination_left(And {
            left: (),
            right: (),
        });
        let _ = and_elimination_right(And {
            left: (),
            right: (),
        });
        let _ = and_introduction((), ());
        let _ = and_commutativity(And {
            left: (),
            right: (),
        });
        let _ = or_introduction_left::<(), ()>(());
        let _ = or_introduction_right::<(), ()>(());
        let _ = implication_reflexive(());
        let _ = modus_ponens((), |x| x);

        // This test passing means all proofs are valid
        assert!(true, "All proofs type-check successfully");
    }
}
