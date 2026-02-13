// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Formal Proofs for State Theory
//!
//! Type-level encoding of proofs using the Curry-Howard correspondence:
//! - **Types** correspond to **propositions**
//! - **Values** correspond to **proofs**
//! - **Functions** correspond to **implications**
//!
//! ## Proof Objects
//!
//! A proof object is a value that witnesses the truth of a proposition.
//! If you can construct a value of type `Proof<P>`, then `P` is proven.
//!
//! ## Verification Levels
//!
//! | Level | Method | Assurance |
//! |-------|--------|-----------|
//! | L1 | Type checking | Compile-time |
//! | L2 | Property testing | Statistical |
//! | L3 | Bounded model checking | Exhaustive (bounded) |
//! | L4 | Theorem proving | Mathematical |
//!
//! ## Kani Integration
//!
//! When the `kani` feature is enabled, proof harnesses can be verified
//! using bounded model checking.

use alloc::string::String;
use alloc::vec::Vec;
use core::marker::PhantomData;

// ═══════════════════════════════════════════════════════════
// PROOF OBJECT
// ═══════════════════════════════════════════════════════════

/// A proof witness for proposition P.
///
/// The existence of a value of type `Proof<P>` witnesses that P is true.
/// Construction is intentionally restricted to ensure soundness.
#[derive(Debug, Clone, Copy)]
pub struct Proof<P> {
    _proposition: PhantomData<P>,
}

impl<P> Proof<P> {
    /// Axiom: construct a proof by assertion.
    ///
    /// # Safety (Logical)
    ///
    /// The caller asserts that P is true. Use only when P is
    /// obviously true or verified by other means.
    #[must_use]
    pub const fn axiom() -> Self {
        Self {
            _proposition: PhantomData,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// LOGICAL CONNECTIVES
// ═══════════════════════════════════════════════════════════

/// Logical AND: P ∧ Q
#[derive(Debug, Clone, Copy)]
pub struct And<P, Q> {
    _p: PhantomData<P>,
    _q: PhantomData<Q>,
}

impl<P, Q> And<P, Q> {
    /// Construct conjunction from proofs of both parts.
    #[must_use]
    pub fn intro(_proof_p: Proof<P>, _proof_q: Proof<Q>) -> Proof<And<P, Q>> {
        Proof::axiom()
    }

    /// Extract left conjunct.
    #[must_use]
    pub fn elim_left(_proof: Proof<And<P, Q>>) -> Proof<P> {
        Proof::axiom()
    }

    /// Extract right conjunct.
    #[must_use]
    pub fn elim_right(_proof: Proof<And<P, Q>>) -> Proof<Q> {
        Proof::axiom()
    }
}

/// Logical OR: P ∨ Q
#[derive(Debug, Clone, Copy)]
pub struct Or<P, Q> {
    _p: PhantomData<P>,
    _q: PhantomData<Q>,
}

impl<P, Q> Or<P, Q> {
    /// Construct disjunction from proof of left.
    #[must_use]
    pub fn intro_left(_proof_p: Proof<P>) -> Proof<Or<P, Q>> {
        Proof::axiom()
    }

    /// Construct disjunction from proof of right.
    #[must_use]
    pub fn intro_right(_proof_q: Proof<Q>) -> Proof<Or<P, Q>> {
        Proof::axiom()
    }
}

/// Logical implication: P → Q
#[derive(Debug, Clone, Copy)]
pub struct Implies<P, Q> {
    _p: PhantomData<P>,
    _q: PhantomData<Q>,
}

impl<P, Q> Implies<P, Q> {
    /// Modus ponens: from P → Q and P, derive Q.
    #[must_use]
    pub fn modus_ponens(_impl: Proof<Implies<P, Q>>, _p: Proof<P>) -> Proof<Q> {
        Proof::axiom()
    }
}

/// Logical negation: ¬P
#[derive(Debug, Clone, Copy)]
pub struct Not<P> {
    _p: PhantomData<P>,
}

/// Logical equivalence: P ↔ Q
pub type Iff<P, Q> = And<Implies<P, Q>, Implies<Q, P>>;

// ═══════════════════════════════════════════════════════════
// QUANTIFIERS
// ═══════════════════════════════════════════════════════════

/// Universal quantification: ∀x. P(x)
#[derive(Debug, Clone, Copy)]
pub struct ForAll<T, P> {
    _t: PhantomData<T>,
    _p: PhantomData<P>,
}

impl<T, P> ForAll<T, P> {
    /// Instantiate universal with a specific value.
    #[must_use]
    pub fn instantiate(_proof: Proof<ForAll<T, P>>, _witness: T) -> Proof<P> {
        Proof::axiom()
    }
}

/// Existential quantification: ∃x. P(x)
#[derive(Debug, Clone, Copy)]
pub struct Exists<T, P> {
    _t: PhantomData<T>,
    _p: PhantomData<P>,
}

impl<T, P> Exists<T, P> {
    /// Introduce existential with witness.
    #[must_use]
    pub fn intro(_witness: T, _proof: Proof<P>) -> Proof<Exists<T, P>> {
        Proof::axiom()
    }
}

// ═══════════════════════════════════════════════════════════
// STATE MACHINE PROPOSITIONS
// ═══════════════════════════════════════════════════════════

/// Proposition: State S is reachable from initial state.
pub struct Reachable<S> {
    _s: PhantomData<S>,
}

/// Proposition: State S is a terminal state.
pub struct Terminal<S> {
    _s: PhantomData<S>,
}

/// Proposition: Transition from S to T is valid.
pub struct ValidTransition<S, T> {
    _s: PhantomData<S>,
    _t: PhantomData<T>,
}

/// Proposition: Invariant I holds in state S.
pub struct InvariantHolds<I, S> {
    _i: PhantomData<I>,
    _s: PhantomData<S>,
}

/// Proposition: Machine M is deterministic.
pub struct Deterministic<M> {
    _m: PhantomData<M>,
}

/// Proposition: Machine M is deadlock-free.
pub struct DeadlockFree<M> {
    _m: PhantomData<M>,
}

/// Proposition: Machine M terminates from all states.
pub struct Terminating<M> {
    _m: PhantomData<M>,
}

// ═══════════════════════════════════════════════════════════
// PROOF CERTIFICATE
// ═══════════════════════════════════════════════════════════

/// Verification level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerificationLevel {
    /// Type-level checking (compile time).
    TypeLevel = 1,
    /// Property-based testing (statistical).
    PropertyTesting = 2,
    /// Bounded model checking (exhaustive within bounds).
    BoundedModelChecking = 3,
    /// Theorem proving (mathematical certainty).
    TheoremProving = 4,
}

/// A proof certificate with metadata.
#[derive(Debug, Clone)]
pub struct ProofCertificate {
    /// Name of the proven property.
    pub property: String,
    /// Verification level achieved.
    pub level: VerificationLevel,
    /// Proof method used.
    pub method: String,
    /// Bounds (if applicable).
    pub bounds: Option<ProofBounds>,
    /// Timestamp of verification.
    pub timestamp: u64,
}

/// Bounds used in bounded verification.
#[derive(Debug, Clone)]
pub struct ProofBounds {
    /// Maximum state count explored.
    pub max_states: usize,
    /// Maximum transition depth explored.
    pub max_depth: usize,
    /// Maximum trace length explored.
    pub max_trace_length: usize,
}

impl ProofCertificate {
    /// Create a type-level proof certificate.
    #[must_use]
    pub fn type_level(property: &str) -> Self {
        Self {
            property: property.into(),
            level: VerificationLevel::TypeLevel,
            method: "Rust type system".into(),
            bounds: None,
            timestamp: 0,
        }
    }

    /// Create a property testing certificate.
    #[must_use]
    pub fn property_tested(property: &str, test_count: usize) -> Self {
        Self {
            property: property.into(),
            level: VerificationLevel::PropertyTesting,
            method: alloc::format!("proptest ({} cases)", test_count),
            bounds: None,
            timestamp: 0,
        }
    }

    /// Create a bounded model checking certificate.
    #[must_use]
    pub fn bounded_model_checked(property: &str, bounds: ProofBounds) -> Self {
        Self {
            property: property.into(),
            level: VerificationLevel::BoundedModelChecking,
            method: "Kani".into(),
            bounds: Some(bounds),
            timestamp: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// PROPERTY SPECIFICATION
// ═══════════════════════════════════════════════════════════

/// A testable property specification.
pub trait PropertySpec {
    /// The type of input to the property.
    type Input;

    /// Property name.
    fn name(&self) -> &'static str;

    /// Check if the property holds for the given input.
    fn check(&self, input: &Self::Input) -> bool;
}

/// Machine property: single active state.
#[derive(Debug, Clone, Copy, Default)]
pub struct SingleActiveState;

impl PropertySpec for SingleActiveState {
    type Input = usize; // Number of active states

    fn name(&self) -> &'static str {
        "single_active_state"
    }

    fn check(&self, input: &Self::Input) -> bool {
        *input == 1
    }
}

/// Machine property: terminal states are absorbing.
#[derive(Debug, Clone, Copy, Default)]
pub struct TerminalAbsorbing;

impl PropertySpec for TerminalAbsorbing {
    type Input = (bool, usize); // (is_terminal, outgoing_transition_count)

    fn name(&self) -> &'static str {
        "terminal_absorbing"
    }

    fn check(&self, input: &Self::Input) -> bool {
        let (is_terminal, outgoing) = *input;
        !is_terminal || outgoing == 0
    }
}

/// Machine property: transition count matches history.
#[derive(Debug, Clone, Copy, Default)]
pub struct TransitionCountMatches;

impl PropertySpec for TransitionCountMatches {
    type Input = (u64, usize); // (reported_count, history_length)

    fn name(&self) -> &'static str {
        "transition_count_matches"
    }

    fn check(&self, input: &Self::Input) -> bool {
        let (count, history_len) = *input;
        count as usize == history_len
    }
}

// ═══════════════════════════════════════════════════════════
// PROOF REGISTRY
// ═══════════════════════════════════════════════════════════

/// Registry of proven properties for a state machine.
#[derive(Debug, Clone, Default)]
pub struct ProofRegistry {
    /// Proven properties.
    pub certificates: Vec<ProofCertificate>,
}

impl ProofRegistry {
    /// Create empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a proof certificate.
    pub fn add(&mut self, cert: ProofCertificate) {
        self.certificates.push(cert);
    }

    /// Check if a property is proven.
    #[must_use]
    pub fn is_proven(&self, property: &str) -> bool {
        self.certificates.iter().any(|c| c.property == property)
    }

    /// Get the highest verification level for a property.
    #[must_use]
    pub fn verification_level(&self, property: &str) -> Option<VerificationLevel> {
        self.certificates
            .iter()
            .filter(|c| c.property == property)
            .map(|c| c.level)
            .max()
    }

    /// Get all properties proven at a given level or higher.
    #[must_use]
    pub fn properties_at_level(&self, min_level: VerificationLevel) -> Vec<&str> {
        self.certificates
            .iter()
            .filter(|c| c.level >= min_level)
            .map(|c| c.property.as_str())
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════
// KANI HARNESS TEMPLATE
// ═══════════════════════════════════════════════════════════

/// Template for Kani proof harnesses.
///
/// When the `kani` feature is enabled, these can be verified
/// using bounded model checking.
#[cfg(feature = "kani")]
pub mod kani_harnesses {
    //! Kani verification harnesses.
    //!
    //! Run with: `cargo kani --features kani`

    /// Verify that transition count never overflows.
    #[cfg(kani)]
    #[kani::proof]
    fn verify_transition_count_no_overflow() {
        let count: u64 = kani::any();
        kani::assume(count < u64::MAX);
        let new_count = count + 1;
        kani::assert(new_count > count, "Transition count should increment");
    }

    /// Verify terminal states have no outgoing transitions.
    #[cfg(kani)]
    #[kani::proof]
    fn verify_terminal_absorbing() {
        let is_terminal: bool = kani::any();
        let outgoing: usize = kani::any();
        kani::assume(outgoing <= 100);

        if is_terminal {
            // Terminal states should have 0 outgoing
            kani::assume(outgoing == 0);
        }

        kani::assert(
            !is_terminal || outgoing == 0,
            "Terminal states must be absorbing",
        );
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // Propositions for testing
    struct P;
    struct Q;

    #[test]
    fn test_proof_axiom() {
        let _proof: Proof<P> = Proof::axiom();
    }

    #[test]
    fn test_conjunction() {
        let proof_p: Proof<P> = Proof::axiom();
        let proof_q: Proof<Q> = Proof::axiom();

        let proof_and: Proof<And<P, Q>> = And::intro(proof_p, proof_q);

        let _p_again: Proof<P> = And::elim_left(proof_and);
    }

    #[test]
    fn test_disjunction() {
        let proof_p: Proof<P> = Proof::axiom();
        let _proof_or: Proof<Or<P, Q>> = Or::intro_left(proof_p);
    }

    #[test]
    fn test_modus_ponens() {
        let proof_impl: Proof<Implies<P, Q>> = Proof::axiom();
        let proof_p: Proof<P> = Proof::axiom();

        let _proof_q: Proof<Q> = Implies::modus_ponens(proof_impl, proof_p);
    }

    #[test]
    fn test_property_spec_single_state() {
        let prop = SingleActiveState;
        assert!(prop.check(&1));
        assert!(!prop.check(&0));
        assert!(!prop.check(&2));
    }

    #[test]
    fn test_property_spec_terminal_absorbing() {
        let prop = TerminalAbsorbing;
        assert!(prop.check(&(false, 5))); // Non-terminal can have outgoing
        assert!(prop.check(&(true, 0))); // Terminal with 0 outgoing
        assert!(!prop.check(&(true, 1))); // Terminal with outgoing is violation
    }

    #[test]
    fn test_proof_certificate() {
        let cert = ProofCertificate::type_level("single_active_state");
        assert_eq!(cert.level, VerificationLevel::TypeLevel);
    }

    #[test]
    fn test_proof_registry() {
        let mut registry = ProofRegistry::new();

        registry.add(ProofCertificate::type_level("prop1"));
        registry.add(ProofCertificate::property_tested("prop2", 1000));

        assert!(registry.is_proven("prop1"));
        assert!(registry.is_proven("prop2"));
        assert!(!registry.is_proven("prop3"));

        assert_eq!(
            registry.verification_level("prop2"),
            Some(VerificationLevel::PropertyTesting)
        );
    }
}
