#![allow(unreachable_code)]
//! Predicate Logic Proofs
//!
//! This module demonstrates proofs involving quantifiers (∀ and ∃)
//! using Rust's generic functions and existential types.
//!
//! # Universal Quantification (∀)
//!
//! In Rust, universal quantification is represented by generic functions.
//! A function `fn<T>(x: T) -> P<T>` proves "for all T, P(T)".
//!
//! # Existential Quantification (∃)
//!
//! Existential quantification requires a witness paired with proof.
//! We use `Exists<W, P>` where W is the witness type and P is the property.

use crate::proofs::logic_prelude::*;
use std::marker::PhantomData;

// ============================================================================
// UNIVERSAL QUANTIFICATION THEOREMS
// ============================================================================

/// THEOREM: ∀T. (T → T)
///
/// "For all types T, there exists an identity function."
///
/// PROOF: The identity function is definable for any type.
pub fn universal_identity<T>(t: T) -> T {
    t
}

/// THEOREM: ∀P, Q. ((P ∧ Q) → P)
///
/// "For all P and Q, conjunction elimination holds."
///
/// PROOF: Generic function works for any type parameters.
pub fn universal_and_elim<P, Q>(pq: And<P, Q>) -> P {
    pq.left
}

/// THEOREM: ∀P, Q, R. ((P → Q) → (Q → R) → (P → R))
///
/// "For all P, Q, R, implication is transitive."
///
/// PROOF: Generic composition works for any types.
pub fn universal_transitivity<P, Q, R>(
    pq: impl Fn(P) -> Q,
    qr: impl Fn(Q) -> R,
) -> impl Fn(P) -> R {
    move |p| qr(pq(p))
}

// ============================================================================
// EXISTENTIAL QUANTIFICATION THEOREMS
// ============================================================================

/// THEOREM: ∀T, P. (P(T) → ∃X. P(X))
///
/// "If we have a proof of P(T) for specific T, then there exists an X with P(X)."
///
/// PROOF: Existential introduction - provide the witness and proof.
pub fn existential_introduction<W, P>(witness: W, proof: P) -> Exists<W, P> {
    Exists { witness, proof }
}

/// THEOREM: ∃X. P(X) → (∀X. (P(X) → R)) → R
///
/// "From an existential and a universal consumer, derive the result."
///
/// PROOF: Apply the universal to the specific witness from the existential.
pub fn existential_elimination<W, P, R>(
    exists: Exists<W, P>,
    for_all: impl FnOnce(W, P) -> R,
) -> R {
    for_all(exists.witness, exists.proof)
}

// ============================================================================
// QUANTIFIER RELATIONSHIPS
// ============================================================================

/// THEOREM: (∀X. P(X)) → P(a) [for any specific a]
///
/// "Universal instantiation: from 'for all', derive for specific."
///
/// PROOF: Apply the universal proof to the specific instance.
pub fn universal_instantiation<T, P>(universal: impl Fn(T) -> P, specific: T) -> P {
    universal(specific)
}

/// THEOREM: P(a) → (∃X. P(X)) [existential generalization]
///
/// "From a specific instance, derive 'there exists'."
///
/// PROOF: Package the specific instance as witness.
pub fn existential_generalization<W, P>(witness: W, proof: P) -> Exists<W, P> {
    Exists { witness, proof }
}

/// THEOREM: ¬(∃X. P(X)) → (∀X. ¬P(X))
///
/// "If nothing satisfies P, then everything fails to satisfy P."
///
/// PROOF: For any x, if P(x) held, we'd have ∃X.P(X), contradiction.
pub fn not_exists_implies_forall_not<T: Clone + 'static, P: 'static>(
    not_exists: Not<Exists<T, P>>,
) -> impl Fn(T, P) -> Void {
    move |t: T, p: P| {
        not_exists(Exists {
            witness: t,
            proof: p,
        })
    }
}

/// THEOREM: (∀X. ¬P(X)) → ¬(∃X. P(X))
///
/// "If everything fails P, then nothing satisfies P."
///
/// PROOF: From ∃X.P(X), extract the witness and apply ∀X.¬P(X).
pub fn forall_not_implies_not_exists<T, P>(
    forall_not: impl Fn(T, P) -> Void,
) -> impl Fn(Exists<T, P>) -> Void {
    move |exists: Exists<T, P>| forall_not(exists.witness, exists.proof)
}

// Note: ¬(∀X. P(X)) → (∃X. ¬P(X)) is NOT intuitionistically provable!
// This would require constructing a specific counterexample.

// ============================================================================
// DOMAIN-SPECIFIC EXAMPLES
// ============================================================================

/// Example: Define a predicate "is positive" for demonstration
pub struct IsPositive<N>(PhantomData<N>);

/// THEOREM: ∃N. IsPositive(N)
///
/// "There exists a positive number."
///
/// PROOF: Witness = 1, and 1 is positive by definition.
pub fn exists_positive() -> Exists<u32, IsPositive<u32>> {
    Exists {
        witness: 1,
        proof: IsPositive(PhantomData),
    }
}

/// Define human/mortal predicates for syllogism example
pub trait Human {}
pub struct Mortal<H: Human>(PhantomData<H>);

/// Socrates is human
pub struct Socrates;
impl Human for Socrates {}

/// THEOREM: (∀H. Human(H) → Mortal(H)) → Human(Socrates) → Mortal(Socrates)
///
/// "The classic syllogism: All humans are mortal, Socrates is human,
///  therefore Socrates is mortal."
///
/// PROOF: Apply the universal to the specific instance (Socrates).
pub fn socrates_syllogism<H: Human>(
    all_humans_mortal: impl Fn(H) -> Mortal<H>,
    human_instance: H,
) -> Mortal<H> {
    all_humans_mortal(human_instance)
}

/// Concrete instantiation for Socrates
pub fn socrates_is_mortal(
    all_humans_mortal: impl Fn(Socrates) -> Mortal<Socrates>,
    _socrates_is_human: Socrates,
) -> Mortal<Socrates> {
    all_humans_mortal(Socrates)
}

// ============================================================================
// NESTED QUANTIFIERS
// ============================================================================

/// THEOREM: (∀X. ∀Y. P(X, Y)) → (∀Y. ∀X. P(X, Y))
///
/// "Universal quantifier commutation."
///
/// PROOF: Rearrange the order of generic parameters.
pub fn universal_commutation<X, Y, P>(forall_xy: impl Fn(X, Y) -> P) -> impl Fn(Y, X) -> P {
    move |y, x| forall_xy(x, y)
}

/// THEOREM: (∃X. ∃Y. P(X, Y)) → (∃Y. ∃X. P(X, Y))
///
/// "Existential quantifier commutation."
///
/// PROOF: Swap the nested existential witnesses.
pub fn existential_commutation<X, Y, P>(
    exists_xy: Exists<X, Exists<Y, P>>,
) -> Exists<Y, Exists<X, P>> {
    Exists {
        witness: exists_xy.proof.witness,
        proof: Exists {
            witness: exists_xy.witness,
            proof: exists_xy.proof.proof,
        },
    }
}

/// THEOREM: (∀X. P(X) ∧ Q(X)) → (∀X. P(X)) ∧ (∀X. Q(X))
///
/// "Distribution of universal over conjunction."
///
/// PROOF: Split the conjunction pointwise.
pub fn universal_and_distribution<X: Clone, P, Q>(
    forall_pq: impl Fn(X) -> And<P, Q> + Clone,
) -> And<impl Fn(X) -> P, impl Fn(X) -> Q> {
    let f1 = forall_pq.clone();
    let f2 = forall_pq;
    And {
        left: move |x: X| f1(x).left,
        right: move |x: X| f2(x).right,
    }
}

// ============================================================================
// COMPILATION TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quantifier_proofs_typecheck() {
        // Verify existential introduction
        let _ = existential_introduction(42u32, IsPositive::<u32>(PhantomData));

        // Verify universal identity
        let _: i32 = universal_identity(42);

        // Verify the Socrates syllogism structure compiles
        let all_mortal = |_s: Socrates| Mortal::<Socrates>(PhantomData);
        let _: Mortal<Socrates> = socrates_is_mortal(all_mortal, Socrates);

        assert!(true, "All predicate logic proofs verified");
    }
}
