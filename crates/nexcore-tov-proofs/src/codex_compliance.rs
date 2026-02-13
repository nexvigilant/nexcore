//! Primitive Codex Compliance Module
//!
//! This module provides types and traits that bring the logic_proofs crate
//! into full compliance with the Fourteen Primitives of the Lex Primitiva.
//!
//! ## Compliance Additions
//!
//! | Commandment | Addition | Purpose |
//! |-------------|----------|---------|
//! | I (QUANTIFY) | `From` impls | Quality → Quantity conversion |
//! | V (COMPARE) | `CompareAbsent` marker | Document non-Ord types |
//! | VII (TYPE) | `IsPrimitive` trait | Primitive marker in bounds |
//! | IX (MEASURE) | `Confidence<C>` wrapper | All values carry confidence |
//! | XI (CORRECT) | `Versioned<T, V>` wrapper | State correction mechanism |

use crate::logic_prelude::*;
use std::marker::PhantomData;

// ============================================================================
// COMMANDMENT I: QUANTIFY - Quality → Quantity via From
// ============================================================================

/// Re-export canonical Tier from nexcore-constants (single source of truth).
pub use nexcore_constants::Tier;

/// Trait for types that have a Codex tier classification.
///
/// COMMANDMENT II: Every type receives tier classification.
pub trait HasTier {
    /// The tier of this type in the Primitive Codex hierarchy.
    const TIER: Tier;

    /// Human-readable tier name.
    fn tier_name() -> &'static str {
        match Self::TIER {
            Tier::T1Universal => "T1-Universal",
            Tier::T2Primitive => "T2-Primitive",
            Tier::T2Composite => "T2-Composite",
            Tier::T3DomainSpecific => "T3-DomainSpecific",
        }
    }
}

// ============================================================================
// COMMANDMENT VII: TYPE - T1 Marker Types
// ============================================================================

/// Marker types for the 14 T1 Universal Primitives.
pub mod t1 {
    use super::*;

    /// $\sigma$ (Sequence): Ordered succession.
    pub struct Sequence;
    /// $\mu$ (Mapping): Transformation A $\Rightarrow$ B.
    pub struct Mapping;
    /// $\varsigma$ (State): Encapsulated context.
    pub struct State;
    /// $\rho$ (Recursion): Self-reference via indirection.
    pub struct Recursion;
    /// $\emptyset$ (Void): Meaningful absence.
    pub struct Void;
    /// $\partial$ (Boundary): Delimiters or limits.
    pub struct Boundary;
    /// $f$ (Frequency): Rate of occurrence.
    pub struct Frequency;
    /// $\exists$ (Existence): Instantiation of being.
    pub struct Existence;
    /// $\pi$ (Persistence): Continuity through duration.
    pub struct Persistence;
    /// $\rightarrow$ (Causality): Cause and consequence.
    pub struct Causality;
    /// $\kappa$ (Comparison): Predicate matching.
    pub struct Comparison;
    /// $N$ (Quantity): Numerical magnitude.
    pub struct Quantity;
    /// $\lambda$ (Location): Positional context.
    pub struct Location;
    /// $\propto$ (Irreversibility): One-way transition.
    pub struct Irreversibility;

    impl HasTier for Sequence {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for Mapping {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for State {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for Recursion {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for Void {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for Boundary {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for Frequency {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for Existence {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for Persistence {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for Causality {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for Comparison {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for Quantity {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for Location {
        const TIER: Tier = Tier::T1Universal;
    }
    impl HasTier for Irreversibility {
        const TIER: Tier = Tier::T1Universal;
    }
}

// Map standard Rust types to T1 Tiers
impl HasTier for () {
    const TIER: Tier = Tier::T1Universal;
} // Grounds to Void
impl HasTier for bool {
    const TIER: Tier = Tier::T1Universal;
} // Grounds to Comparison
impl HasTier for u32 {
    const TIER: Tier = Tier::T1Universal;
} // Grounds to Quantity
impl HasTier for f64 {
    const TIER: Tier = Tier::T1Universal;
} // Grounds to Quantity
impl HasTier for usize {
    const TIER: Tier = Tier::T1Universal;
} // Grounds to Quantity

// T2-P: Cross-domain primitives
impl<P, Q> HasTier for And<P, Q> {
    const TIER: Tier = Tier::T2Primitive;
}
impl<P, Q> HasTier for Or<P, Q> {
    const TIER: Tier = Tier::T2Primitive;
}
impl<W, P> HasTier for crate::logic_prelude::Exists<W, P> {
    const TIER: Tier = Tier::T2Primitive;
}
impl<P> HasTier for Proof<P> {
    const TIER: Tier = Tier::T2Primitive;
}

// From<Tier> for u8 and TryFrom<u8> for Tier are provided by nexcore-constants.

// ============================================================================
// COMMANDMENT V: COMPARE - Document Ord absence
// ============================================================================

/// Marker trait for types where ordering is undefined (compare = ⊥).
pub trait CompareAbsent {
    /// Reason why ordering is undefined for this type.
    const REASON: &'static str;
}

impl<P, Q> CompareAbsent for And<P, Q> {
    const REASON: &'static str = "Logical conjunction has no meaningful total order";
}

impl<P, Q> CompareAbsent for Or<P, Q> {
    const REASON: &'static str = "Logical disjunction has no meaningful total order";
}

impl CompareAbsent for crate::logic_prelude::Void {
    const REASON: &'static str = "Uninhabited type cannot be compared (no values exist)";
}

impl<W, P> CompareAbsent for crate::logic_prelude::Exists<W, P> {
    const REASON: &'static str = "Existential proofs have no canonical ordering";
}

impl<P> CompareAbsent for Proof<P> {
    const REASON: &'static str = "Proof markers carry no orderable data";
}

// ============================================================================
// COMMANDMENT VII: TYPE - Primitive marker trait for bounds
// ============================================================================

/// Marker trait for T1 universal primitives.
pub trait IsPrimitive: HasTier {
    /// Verify this type is T1.
    fn verify_primitive() -> bool {
        Self::TIER == Tier::T1Universal
    }
}

impl IsPrimitive for t1::Sequence {}
impl IsPrimitive for t1::Mapping {}
impl IsPrimitive for t1::State {}
impl IsPrimitive for t1::Recursion {}
impl IsPrimitive for t1::Void {}
impl IsPrimitive for t1::Boundary {}
impl IsPrimitive for t1::Frequency {}
impl IsPrimitive for t1::Existence {}
impl IsPrimitive for t1::Persistence {}
impl IsPrimitive for t1::Causality {}
impl IsPrimitive for t1::Comparison {}
impl IsPrimitive for t1::Quantity {}
impl IsPrimitive for t1::Location {}
impl IsPrimitive for t1::Irreversibility {}

// Standard types are primitives
impl IsPrimitive for () {}
impl IsPrimitive for bool {}
impl IsPrimitive for u32 {}
impl IsPrimitive for f64 {}
impl IsPrimitive for usize {}

/// Marker trait for types that compose from primitives.
///
/// COMMANDMENT III: Every type traces to T1 primitives.
pub trait GroundsTo<Primitive: IsPrimitive> {}

// Proof markers ground to specific T1 symbols
impl<P, Q> GroundsTo<t1::Comparison> for And<P, Q> {}
impl<P, Q> GroundsTo<t1::Comparison> for Or<P, Q> {}
impl<W, P> GroundsTo<t1::Existence> for crate::logic_prelude::Exists<W, P> {}
impl<P> GroundsTo<t1::State> for Proof<P> {}

// ============================================================================
// COMMANDMENT IX: MEASURE - All values carry confidence
// ============================================================================

/// A value with explicit confidence annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Confident<T, const CONFIDENCE: u8> {
    value: T,
}

impl<T, const C: u8> Confident<T, C> {
    /// Create a confident value. Only compiles if C ∈ [0, 100].
    pub const fn new(value: T) -> Self {
        assert!(C <= 100, "Confidence must be in range [0, 100]");
        Self { value }
    }

    /// Get the confidence level.
    pub const fn confidence(&self) -> u8 {
        C
    }

    /// Get the inner value.
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Get a reference to the inner value.
    pub fn as_inner(&self) -> &T {
        &self.value
    }
}

/// Type aliases for common confidence levels.
pub type Certain<T> = Confident<T, 100>;
pub type HighConfidence<T> = Confident<T, 90>;
pub type MediumConfidence<T> = Confident<T, 70>;
pub type LowConfidence<T> = Confident<T, 50>;
pub type Uncertain<T> = Confident<T, 30>;

/// A proof with explicit confidence annotation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfidentProof<P, const CONFIDENCE: u8>(PhantomData<P>);

impl<P, const C: u8> ConfidentProof<P, C> {
    /// Create a confident proof marker.
    pub const fn qed() -> Self {
        assert!(C <= 100, "Confidence must be in range [0, 100]");
        Self(PhantomData)
    }

    /// Get the confidence level.
    pub const fn confidence(&self) -> u8 {
        C
    }
}

impl<P, const C: u8> Default for ConfidentProof<P, C> {
    fn default() -> Self {
        Self::qed()
    }
}

/// Type aliases for proof confidence levels.
pub type ProvenCertain<P> = ConfidentProof<P, 100>;
pub type ProvenHighConf<P> = ConfidentProof<P, 90>;
pub type ProvenMedConf<P> = ConfidentProof<P, 70>;

// ============================================================================
// COMMANDMENT XI: CORRECT - All state is correctable
// ============================================================================

/// A versioned value that supports correction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Versioned<T, const VERSION: u32> {
    value: T,
}

impl<T, const V: u32> Versioned<T, V> {
    /// Create a versioned value.
    pub const fn new(value: T) -> Self {
        Self { value }
    }

    /// Get the version number.
    pub const fn version(&self) -> u32 {
        V
    }

    /// Get the inner value.
    pub fn into_inner(self) -> T {
        self.value
    }

    /// Get a reference to the inner value.
    pub fn as_inner(&self) -> &T {
        &self.value
    }

    /// Upgrade to next version (consumes self).
    pub fn upgrade<const NEW_V: u32>(self, transform: impl FnOnce(T) -> T) -> Versioned<T, NEW_V> {
        Versioned::new(transform(self.value))
    }
}

/// Initial version for new proofs.
pub type V1<T> = Versioned<T, 1>;

/// Corrected version (first correction).
pub type V2<T> = Versioned<T, 2>;

/// Second correction.
pub type V3<T> = Versioned<T, 3>;

// ============================================================================
// COMMANDMENT X: ACKNOWLEDGE - Three limits documented
// ============================================================================

/// The Three Unfixable Limits of the Primitive Codex.
pub mod limits {
    /// **Grounding Axiom** - T1 primitives are declared by fiat, not derived.
    pub const GROUNDING_AXIOM: &str = "T1 primitives (σ, μ, ς, ρ, ∅, ∂, f, ∃, π, →, κ, N, λ, ∝) are declared by fiat, not derived from deeper truths";

    /// **Incompleteness** - The primitive set is not provably complete.
    pub const INCOMPLETENESS: &str = "Cannot prove the 14-primitive T-Set is complete";

    /// **Self-Reference** - The system cannot fully validate itself.
    pub const SELF_REFERENCE: &str =
        "The Primitive Codex cannot prove its own consistency (Gödel's second incompleteness)";
}

// ============================================================================
// COMMANDMENT VI: MATCH - Exhaustive matching helper
// ============================================================================

/// Helper for exhaustive matching on Or<P, Q>.
#[inline]
pub fn exhaustive_match<P, Q, R>(
    disjunction: Or<P, Q>,
    case_left: impl FnOnce(P) -> R,
    case_right: impl FnOnce(Q) -> R,
) -> R {
    match disjunction {
        Or::Left(p) => case_left(p),
        Or::Right(q) => case_right(q),
    }
}

/// Helper for exhaustive matching on Void.
#[inline]
pub fn exhaustive_void<R>(void: crate::logic_prelude::Void) -> R {
    match void {}
}

// ============================================================================
// VIGILANCE T3 TYPES - TIER CLASSIFICATION (Commandment II)
// ============================================================================

pub mod vigilance_tiers {
    use super::*;
    use crate::proofs::vigilance::*;

    // Axiom types - T3 (domain-specific)
    impl<V, E, Phi, S> HasTier for Axiom1<V, E, Phi, S> {
        const TIER: Tier = Tier::T3DomainSpecific;
    }

    impl<S, L, const N: usize> HasTier for Axiom2<S, L, N> {
        const TIER: Tier = Tier::T3DomainSpecific;
    }

    impl<S, U, Theta, G> HasTier for Axiom3<S, U, Theta, G> {
        const TIER: Tier = Tier::T3DomainSpecific;
    }

    impl<S, G, M> HasTier for Axiom4<S, G, M> {
        const TIER: Tier = Tier::T3DomainSpecific;
    }

    impl<const H: usize> HasTier for Axiom5<H> {
        const TIER: Tier = Tier::T3DomainSpecific;
    }

    // System types - T3
    impl<S, U, Theta, HSpec> HasTier for VigilanceSystem<S, U, Theta, HSpec> {
        const TIER: Tier = Tier::T3DomainSpecific;
    }

    impl<V, S, U, Theta, E, Phi, L, G, M, const N: usize, const H: usize> HasTier
        for TheoryOfVigilance<V, S, U, Theta, E, Phi, L, G, M, N, H>
    {
        const TIER: Tier = Tier::T3DomainSpecific;
    }

    // Harm types - T2-C (cross-domain composite)
    impl HasTier for HarmEvent {
        const TIER: Tier = Tier::T2Composite;
    }

    impl HasTier for HarmType {
        const TIER: Tier = Tier::T2Composite;
    }

    impl HasTier for CharacterizedHarmEvent {
        const TIER: Tier = Tier::T2Composite;
    }

    // Propagation dynamics - T2-P (cross-domain primitive)
    impl HasTier for Markovian {
        const TIER: Tier = Tier::T2Primitive;
    }

    impl HasTier for NonMarkovian {
        const TIER: Tier = Tier::T2Primitive;
    }

    // Marker types - T2-P
    impl<X> HasTier for Finite<X> {
        const TIER: Tier = Tier::T2Primitive;
    }

    impl<F> HasTier for Surjective<F> {
        const TIER: Tier = Tier::T2Primitive;
    }

    impl<F> HasTier for Measurable<F> {
        const TIER: Tier = Tier::T2Primitive;
    }

    impl<X> HasTier for NonEmpty<X> {
        const TIER: Tier = Tier::T2Primitive;
    }

    impl HasTier for Safe {
        const TIER: Tier = Tier::T2Primitive;
    }

    impl HasTier for Harmful {
        const TIER: Tier = Tier::T2Primitive;
    }

    // Conservation laws - T3
    impl HasTier for LawType {
        const TIER: Tier = Tier::T3DomainSpecific;
    }

    impl<const ID: usize> HasTier for ConservationLaw<ID> {
        const TIER: Tier = Tier::T3DomainSpecific;
    }

    // Harm characteristics - T2-C
    impl HasTier for Multiplicity {
        const TIER: Tier = Tier::T2Composite;
    }

    impl HasTier for Temporal {
        const TIER: Tier = Tier::T2Composite;
    }

    impl HasTier for Determinism {
        const TIER: Tier = Tier::T2Composite;
    }

    impl HasTier for HarmCharacteristics {
        const TIER: Tier = Tier::T2Composite;
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_quantification() {
        assert_eq!(u8::from(Tier::T1Universal), 1);
        assert_eq!(u8::from(Tier::T2Primitive), 2);
        assert_eq!(u8::from(Tier::T3DomainSpecific), 4);
    }

    #[test]
    fn tier_round_trip() {
        assert_eq!(Tier::try_from(1), Ok(Tier::T1Universal));
        assert_eq!(Tier::try_from(4), Ok(Tier::T3DomainSpecific));
        assert!(Tier::try_from(5).is_err());
    }

    #[test]
    fn is_primitive_verification() {
        assert!(<t1::Sequence>::verify_primitive());
        assert!(<t1::Void>::verify_primitive());
        assert!(<u32>::verify_primitive());
    }

    #[test]
    fn grounding_verification() {
        // Verify type-level grounding enforcement
        fn assert_grounds<T, P: IsPrimitive>(_val: T)
        where
            T: GroundsTo<P>,
        {
        }

        let and_proof = And {
            left: (),
            right: (),
        };
        assert_grounds::<And<(), ()>, t1::Comparison>(and_proof);
    }
}
