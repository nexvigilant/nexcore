#![allow(unreachable_code)]
//! Logic Prelude - Core Types for Curry-Howard Proofs
//!
//! This module provides the foundational type definitions that implement
//! the Curry-Howard correspondence, allowing logical propositions to be
//! represented as Rust types and proofs as programs.
//!
//! # The Correspondence
//!
//! | Logic | Rust |
//! |-------|------|
//! | ⊤ (true) | `()` |
//! | ⊥ (false) | `Void` |
//! | P ∧ Q | `And<P, Q>` |
//! | P ∨ Q | `Or<P, Q>` |
//! | P → Q | `fn(P) -> Q` |
//! | ¬P | `fn(P) -> Void` |

#![allow(dead_code)]

use std::marker::PhantomData;

// ============================================================================
// FALSITY (⊥) - The empty/never type
// ============================================================================

/// Represents logical falsity (⊥).
///
/// `Void` has no constructors, therefore no inhabitants exist.
/// A function returning `Void` can never actually return.
///
/// # Properties
/// - Uninhabited: No value of type `Void` can ever be constructed
/// - Ex falso quodlibet: From `Void`, any proposition follows
///
/// # Example
/// ```compile_fail
/// use nexcore_tov_proofs::prelude::Void;
///
/// fn impossible() -> Void {
///     // This function can never be implemented!
///     // Any attempt would require constructing a Void value.
/// }
/// ```
/// # Ord Absence
///
/// `Void` does not implement `Ord` because it is uninhabited—no values
/// exist to compare. This is semantically correct per Codex Commandment V.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Void {}

impl From<std::convert::Infallible> for Void {
    /// Convert from `Infallible` (the standard library's uninhabited type).
    ///
    /// This provides semantic equivalence: both types represent impossibility.
    fn from(x: std::convert::Infallible) -> Self {
        match x {}
    }
}

impl Void {
    /// Ex falso quodlibet: from falsity, anything follows.
    ///
    /// Since we can never actually have a `Void` value, this function
    /// can claim to return any type - it will never be executed.
    ///
    /// In logic: ⊥ → P (for any P)
    #[inline]
    pub fn absurd<T>(self) -> T {
        match self {}
    }
}

// ============================================================================
// TRUTH (⊤) - The unit type
// ============================================================================

/// Type alias for logical truth.
///
/// The unit type `()` has exactly one inhabitant: `()`.
/// This corresponds to a proposition that is trivially provable.
pub type Truth = ();

/// Construct the trivial proof of truth.
///
/// In logic: ⊤ is always provable.
#[inline]
pub const fn trivial() -> Truth {
    ()
}

// ============================================================================
// CONJUNCTION (∧) - Product types
// ============================================================================

/// Represents logical conjunction (P ∧ Q).
///
/// A proof of `And<P, Q>` requires both a proof of `P` and a proof of `Q`.
///
/// # Logical Properties
/// - Introduction: P, Q ⊢ P ∧ Q
/// - Left Elimination: P ∧ Q ⊢ P
/// - Right Elimination: P ∧ Q ⊢ Q
/// - Commutativity: P ∧ Q ↔ Q ∧ P
/// - Associativity: (P ∧ Q) ∧ R ↔ P ∧ (Q ∧ R)
///
/// # Ord Absence (Codex V)
///
/// `And<P, Q>` does not implement `Ord` because logical conjunctions have no
/// meaningful total ordering—propositions are not comparable by magnitude.
///
/// # Example
/// ```
/// use nexcore_tov_proofs::prelude::And;
///
/// // Prove (A ∧ B) → A
/// fn and_elim<A, B>(proof: And<A, B>) -> A {
///     proof.left
/// }
///
/// let proof = And { left: 1, right: "hello" };
/// assert_eq!(and_elim(proof), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct And<P, Q> {
    pub left: P,
    pub right: Q,
}

impl<P, Q> And<P, Q> {
    /// Conjunction introduction: from P and Q, derive P ∧ Q.
    ///
    /// In logic: P, Q ⊢ P ∧ Q
    #[inline]
    pub fn intro(p: P, q: Q) -> Self {
        And { left: p, right: q }
    }

    /// Left elimination: from P ∧ Q, derive P.
    ///
    /// In logic: P ∧ Q ⊢ P
    #[inline]
    pub fn elim_left(self) -> P {
        self.left
    }

    /// Right elimination: from P ∧ Q, derive Q.
    ///
    /// In logic: P ∧ Q ⊢ Q
    #[inline]
    pub fn elim_right(self) -> Q {
        self.right
    }

    /// Commutativity: P ∧ Q → Q ∧ P.
    #[inline]
    pub fn commute(self) -> And<Q, P> {
        And {
            left: self.right,
            right: self.left,
        }
    }

    /// Map over both components.
    #[inline]
    pub fn bimap<P2, Q2>(self, f: impl FnOnce(P) -> P2, g: impl FnOnce(Q) -> Q2) -> And<P2, Q2> {
        And {
            left: f(self.left),
            right: g(self.right),
        }
    }
}

/// Convert a tuple to And (conjunction).
#[inline]
pub fn and_from_tuple<P, Q>(tuple: (P, Q)) -> And<P, Q> {
    And {
        left: tuple.0,
        right: tuple.1,
    }
}

/// Convert And (conjunction) to a tuple.
#[inline]
pub fn and_to_tuple<P, Q>(and: And<P, Q>) -> (P, Q) {
    (and.left, and.right)
}

// ============================================================================
// DISJUNCTION (∨) - Sum types
// ============================================================================

/// Represents logical disjunction (P ∨ Q).
///
/// A proof of `Or<P, Q>` requires either a proof of `P` OR a proof of `Q`,
/// together with a tag indicating which alternative is proven.
///
/// # Logical Properties
/// - Left Introduction: P ⊢ P ∨ Q
/// - Right Introduction: Q ⊢ P ∨ Q
/// - Elimination: P ∨ Q, P → R, Q → R ⊢ R
/// - Commutativity: P ∨ Q ↔ Q ∨ P
///
/// # Ord Absence (Codex V)
///
/// `Or<P, Q>` does not implement `Ord` because disjunctions have no canonical
/// ordering—`Left(p)` vs `Right(q)` represents alternative proofs, not magnitudes.
///
/// # Example
/// ```
/// use nexcore_tov_proofs::prelude::Or;
///
/// // Prove P → (P ∨ Q)
/// fn or_intro_left<P, Q>(p: P) -> Or<P, Q> {
///     Or::Left(p)
/// }
///
/// let proof: Or<i32, &str> = or_intro_left(42);
/// assert!(matches!(proof, Or::Left(42)));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Or<P, Q> {
    /// Proof of the left disjunct.
    Left(P),
    /// Proof of the right disjunct.
    Right(Q),
}

impl<P, Q> Or<P, Q> {
    /// Left introduction: from P, derive P ∨ Q.
    ///
    /// In logic: P ⊢ P ∨ Q
    #[inline]
    pub fn intro_left(p: P) -> Self {
        Or::Left(p)
    }

    /// Right introduction: from Q, derive P ∨ Q.
    ///
    /// In logic: Q ⊢ P ∨ Q
    #[inline]
    pub fn intro_right(q: Q) -> Self {
        Or::Right(q)
    }

    /// Disjunction elimination (case analysis).
    ///
    /// If P → R and Q → R, then P ∨ Q → R.
    ///
    /// In logic: P ∨ Q, P → R, Q → R ⊢ R
    #[inline]
    pub fn elim<R>(self, left_case: impl FnOnce(P) -> R, right_case: impl FnOnce(Q) -> R) -> R {
        match self {
            Or::Left(p) => left_case(p),
            Or::Right(q) => right_case(q),
        }
    }

    /// Commutativity: P ∨ Q → Q ∨ P.
    #[inline]
    pub fn commute(self) -> Or<Q, P> {
        match self {
            Or::Left(p) => Or::Right(p),
            Or::Right(q) => Or::Left(q),
        }
    }

    /// Map over both alternatives.
    #[inline]
    pub fn bimap<P2, Q2>(self, f: impl FnOnce(P) -> P2, g: impl FnOnce(Q) -> Q2) -> Or<P2, Q2> {
        match self {
            Or::Left(p) => Or::Left(f(p)),
            Or::Right(q) => Or::Right(g(q)),
        }
    }
}

// ============================================================================
// NEGATION (¬) - Function to Void
// ============================================================================

/// Type alias for logical negation (function pointer form).
///
/// ¬P is defined as P → ⊥ (if P, then contradiction).
/// Use this type for parameters; for return types that capture variables,
/// use `impl FnOnce(P) -> Void` instead.
pub type Not<P> = fn(P) -> Void;

/// Double negation introduction: P → ¬¬P.
///
/// This is always valid, even in intuitionistic logic.
/// Given a proof of P, we can refute any refutation of P.
///
/// In logic: P ⊢ ¬¬P
#[inline]
pub fn double_neg_intro<P>(p: P) -> impl FnOnce(Not<P>) -> Void {
    move |not_p: Not<P>| not_p(p)
}

/// Contradiction introduction: P, ¬P → ⊥.
///
/// In logic: P, ¬P ⊢ ⊥
#[inline]
pub fn contradiction<P>(p: P, not_p: Not<P>) -> Void {
    not_p(p)
}

/// Ex falso quodlibet (standalone function): ⊥ → P.
///
/// From a contradiction, anything follows.
#[inline]
pub fn ex_falso<P>(void: Void) -> P {
    void.absurd()
}

// ============================================================================
// BICONDITIONAL (↔) - Pair of implications
// ============================================================================

/// Represents logical biconditional (P ↔ Q).
///
/// Equivalent to (P → Q) ∧ (Q → P).
pub struct Iff<P, Q> {
    forward: Box<dyn Fn(P) -> Q>,
    backward: Box<dyn Fn(Q) -> P>,
}

impl<P: 'static, Q: 'static> Iff<P, Q> {
    /// Construct a biconditional from both implications.
    pub fn new(forward: impl Fn(P) -> Q + 'static, backward: impl Fn(Q) -> P + 'static) -> Self {
        Iff {
            forward: Box::new(forward),
            backward: Box::new(backward),
        }
    }

    /// Apply the forward implication: P → Q.
    #[inline]
    pub fn forward(&self, p: P) -> Q {
        (self.forward)(p)
    }

    /// Apply the backward implication: Q → P.
    #[inline]
    pub fn backward(&self, q: Q) -> P {
        (self.backward)(q)
    }
}

// ============================================================================
// EXISTENTIAL QUANTIFICATION (∃)
// ============================================================================

/// Represents existential quantification (∃x. P(x)).
///
/// A proof requires providing a witness `x` and a proof that `P(x)` holds.
///
/// # Example
/// ```
/// use nexcore_tov_proofs::prelude::Exists;
///
/// // Prove: ∃n. (n is even)
/// // Witness: 4, Proof: () (trivial proof marker)
/// let proof: Exists<u32, ()> = Exists::intro(4, ());
/// assert_eq!(proof.witness, 4);
/// ```
///
/// # Ord Absence (Codex V)
///
/// `Exists<W, P>` does not implement `Ord` because existential proofs have no
/// canonical ordering—different witnesses proving the same property are equivalent.
#[derive(Debug, Clone)]
pub struct Exists<Witness, Property> {
    /// The witness that satisfies the property.
    pub witness: Witness,
    /// The proof that the property holds for the witness.
    pub proof: Property,
}

impl<W, P> Exists<W, P> {
    /// Existential introduction: provide a witness and proof.
    ///
    /// In logic: P(a) ⊢ ∃x. P(x)
    #[inline]
    pub fn intro(witness: W, proof: P) -> Self {
        Exists { witness, proof }
    }

    /// Existential elimination: use the witness in a context.
    ///
    /// In logic: ∃x. P(x), (∀x. P(x) → R) ⊢ R
    #[inline]
    pub fn elim<R>(self, consumer: impl FnOnce(W, P) -> R) -> R {
        consumer(self.witness, self.proof)
    }

    /// Map over the witness.
    #[inline]
    pub fn map_witness<W2>(self, f: impl FnOnce(W) -> W2) -> Exists<W2, P> {
        Exists {
            witness: f(self.witness),
            proof: self.proof,
        }
    }
}

// ============================================================================
// PROOF MARKERS - For propositions without computational content
// ============================================================================

/// A zero-cost proof marker for type-level propositions.
///
/// Use when the proposition has no computational content but you need
/// to carry proof evidence through the type system.
///
/// # Ord Absence (Codex V)
///
/// `Proof<P>` does not implement `Ord` because proof markers carry no
/// orderable data—they exist purely for type-level evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Proof<P>(PhantomData<P>);

impl<P> Proof<P> {
    /// Create a proof marker.
    ///
    /// Only call this when you have actually established the truth of P
    /// through other means (construction, axiom, etc.).
    #[inline]
    pub const fn qed() -> Self {
        Proof(PhantomData)
    }
}

impl<P> Default for Proof<P> {
    fn default() -> Self {
        Proof::qed()
    }
}

// ============================================================================
// STANDARD INFERENCE RULES
// ============================================================================

/// Modus ponens: P, P → Q ⊢ Q.
///
/// Given a proof of P and a proof that P implies Q, derive Q.
#[inline]
pub fn modus_ponens<P, Q>(premise: P, implication: impl FnOnce(P) -> Q) -> Q {
    implication(premise)
}

/// Hypothetical syllogism: (P → Q), (Q → R) ⊢ (P → R).
///
/// Chain two implications together.
#[inline]
pub fn hypothetical_syllogism<P, Q, R>(
    pq: impl Fn(P) -> Q,
    qr: impl Fn(Q) -> R,
) -> impl Fn(P) -> R {
    move |p| qr(pq(p))
}

/// Modus tollens: ¬Q, P → Q ⊢ ¬P.
///
/// If Q is false and P implies Q, then P must be false.
#[inline]
pub fn modus_tollens<P, Q>(
    not_q: Not<Q>,
    p_implies_q: impl FnOnce(P) -> Q,
) -> impl FnOnce(P) -> Void {
    move |p: P| not_q(p_implies_q(p))
}

/// Disjunctive syllogism: P ∨ Q, ¬P ⊢ Q.
///
/// If P or Q holds and P is false, then Q must be true.
#[inline]
pub fn disjunctive_syllogism<P, Q>(p_or_q: Or<P, Q>, not_p: Not<P>) -> Q {
    match p_or_q {
        Or::Left(p) => not_p(p).absurd(),
        Or::Right(q) => q,
    }
}

/// Contraposition: (P → Q), ¬Q ⊢ ¬P.
///
/// Given an implication and the negation of its consequent,
/// derive the negation of its antecedent.
#[inline]
pub fn contraposition<P, Q>(
    p_implies_q: impl FnOnce(P) -> Q,
    not_q: Not<Q>,
) -> impl FnOnce(P) -> Void {
    move |p: P| not_q(p_implies_q(p))
}

// ============================================================================
// DE MORGAN'S LAWS (Intuitionistically valid directions only)
// ============================================================================

/// De Morgan: ¬(P ∨ Q), P ⊢ ⊥ and ¬(P ∨ Q), Q ⊢ ⊥.
///
/// This direction is intuitionistically valid.
/// Returns functions that derive contradiction from P or Q given ¬(P ∨ Q).
#[inline]
pub fn de_morgan_nor_left<P, Q>(not_p_or_q: Not<Or<P, Q>>, p: P) -> Void {
    not_p_or_q(Or::Left(p))
}

#[inline]
pub fn de_morgan_nor_right<P, Q>(not_p_or_q: Not<Or<P, Q>>, q: Q) -> Void {
    not_p_or_q(Or::Right(q))
}

/// De Morgan: (¬P ∧ ¬Q), (P ∨ Q) ⊢ ⊥.
///
/// This direction is intuitionistically valid.
#[inline]
pub fn de_morgan_nor_converse<P, Q>(
    not_p_and_not_q: And<Not<P>, Not<Q>>,
    p_or_q: Or<P, Q>,
) -> Void {
    match p_or_q {
        Or::Left(p) => (not_p_and_not_q.left)(p),
        Or::Right(q) => (not_p_and_not_q.right)(q),
    }
}

// Note: ¬(P ∧ Q) → (¬P ∨ ¬Q) is NOT intuitionistically valid!

// ============================================================================
// DISTRIBUTIVITY
// ============================================================================

/// Distribute conjunction over disjunction: P ∧ (Q ∨ R) → (P ∧ Q) ∨ (P ∧ R).
#[inline]
pub fn distribute_and_over_or<P: Clone, Q, R>(
    p_and_qr: And<P, Or<Q, R>>,
) -> Or<And<P, Q>, And<P, R>> {
    let p = p_and_qr.left;
    match p_and_qr.right {
        Or::Left(q) => Or::Left(And::intro(p, q)),
        Or::Right(r) => Or::Right(And::intro(p, r)),
    }
}

/// Distribute disjunction over conjunction: P ∨ (Q ∧ R) → (P ∨ Q) ∧ (P ∨ R).
#[inline]
pub fn distribute_or_over_and<P: Clone, Q, R>(
    p_or_qr: Or<P, And<Q, R>>,
) -> And<Or<P, Q>, Or<P, R>> {
    match p_or_qr {
        Or::Left(p) => And::intro(Or::Left(p.clone()), Or::Left(p)),
        Or::Right(qr) => And::intro(Or::Right(qr.left), Or::Right(qr.right)),
    }
}

// ============================================================================
// ADDITIONAL COMBINATORS
// ============================================================================

/// Identity: P → P.
#[inline]
pub fn identity<P>(p: P) -> P {
    p
}

/// Constant: P → Q → P.
#[inline]
pub fn constant<P: Clone, Q>(p: P) -> impl Fn(Q) -> P {
    move |_q| p.clone()
}

/// Flip: Given a curried function P → Q → R and both arguments, apply in reverse order.
#[inline]
pub fn flip<P, Q, R>(f: impl FnOnce(P) -> Box<dyn FnOnce(Q) -> R>, q: Q, p: P) -> R {
    f(p)(q)
}

/// Composition: (Q → R) → (P → Q) → (P → R).
#[inline]
pub fn compose<P, Q, R>(qr: impl Fn(Q) -> R, pq: impl Fn(P) -> Q) -> impl Fn(P) -> R {
    move |p| qr(pq(p))
}
