//! # stem-math: Mathematics Primitives as Rust Traits
//!
//! Implements cross-domain T2-P primitives derived from mathematics.
//!
//! ## The MATHS Composite (T2-C)
//!
//! ```text
//! M - MEMBERSHIP  : Element ∈ Set                   (T1: MAPPING μ)
//! A - ASSOCIATE   : (a·b)·c = a·(b·c)               (T1: RECURSION ρ)
//! T - TRANSIT     : a→b ∧ b→c ⟹ a→c                 (T1: SEQUENCE σ)
//! H - HOMEOMORPH  : Structure-preserving map        (T1: MAPPING μ)
//! S - SYMMETRIC   : a~b ⟹ b~a                       (T1: COMPARISON κ)
//! ```
//!
//! Plus: BOUND, PROVE, COMMUTE, IDENTIFY
//!
//! ## Cross-Domain Transfer
//!
//! | Math | PV Signals | Economics | Software |
//! |------|------------|-----------|----------|
//! | Membership | Case in cohort | Asset in portfolio | Type membership |
//! | Transitivity | Causal chain | Supply chain | Inheritance |
//! | Bounds | Confidence interval | Price limits | Range types |
//!
//! ## Three Unfixable Limits
//!
//! 1. **Heisenberg**: Measuring precision affects accuracy
//! 2. **Gödel**: Mathematics cannot prove its own consistency
//! 3. **Shannon**: Proof compression has irreducible loss

#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![forbid(unsafe_code)]

pub mod grounding;
pub mod spatial;
pub mod spatial_grounding;

use serde::{Deserialize, Serialize};
use stem_core::Confidence;

// ============================================================================
// Core Types (T2-P)
// ============================================================================

/// A bounded value with upper and lower limits (T2-P)
///
/// Grounded in T1 Boundary (∂) and State (ς): constrained range
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Bounded<T> {
    /// The value
    pub value: T,
    /// Lower bound (if any)
    pub lower: Option<T>,
    /// Upper bound (if any)
    pub upper: Option<T>,
}

impl<T: PartialOrd + Copy> Bounded<T> {
    /// Create unbounded value
    #[must_use]
    pub fn unbounded(value: T) -> Self {
        Self {
            value,
            lower: None,
            upper: None,
        }
    }

    /// Create bounded value
    #[must_use]
    pub fn new(value: T, lower: Option<T>, upper: Option<T>) -> Self {
        Self {
            value,
            lower,
            upper,
        }
    }

    /// Check if value is within bounds
    #[must_use]
    pub fn in_bounds(&self) -> bool {
        let above_lower = self.lower.is_none_or(|l| self.value >= l);
        let below_upper = self.upper.is_none_or(|u| self.value <= u);
        above_lower && below_upper
    }

    /// Clamp value to bounds
    #[must_use]
    pub fn clamp(&self) -> T {
        let mut v = self.value;
        if let Some(l) = self.lower {
            if v < l {
                v = l;
            }
        }
        if let Some(u) = self.upper {
            if v > u {
                v = u;
            }
        }
        v
    }
}

/// Proof result with premises and conclusion (T2-P)
///
/// Grounded in T1 Sequence (σ): premises → conclusion
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof<P, C> {
    /// Premises (assumptions)
    pub premises: Vec<P>,
    /// Conclusion (derived)
    pub conclusion: C,
    /// Whether proof is valid
    pub valid: bool,
}

impl<P, C> Proof<P, C> {
    /// Create a valid proof
    pub fn valid(premises: Vec<P>, conclusion: C) -> Self {
        Self {
            premises,
            conclusion,
            valid: true,
        }
    }

    /// Create an invalid proof (premises don't support conclusion)
    pub fn invalid(premises: Vec<P>, conclusion: C) -> Self {
        Self {
            premises,
            conclusion,
            valid: false,
        }
    }
}

/// Relation between two elements (T2-P)
///
/// Grounded in T1 Mapping (μ): (a, b) → related
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Relation {
    /// a < b
    LessThan,
    /// a = b
    Equal,
    /// a > b
    GreaterThan,
    /// a and b are incomparable
    Incomparable,
}

impl Relation {
    /// Invert the relation (swap a and b)
    #[must_use]
    pub fn invert(&self) -> Self {
        match self {
            Relation::LessThan => Relation::GreaterThan,
            Relation::GreaterThan => Relation::LessThan,
            Relation::Equal => Relation::Equal,
            Relation::Incomparable => Relation::Incomparable,
        }
    }

    /// Check if relation is symmetric
    #[must_use]
    pub fn is_symmetric(&self) -> bool {
        matches!(self, Relation::Equal | Relation::Incomparable)
    }
}

/// Identity element marker (T2-P)
///
/// Grounded in T1 State (ς): neutral element
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Identity<T>(pub T);

impl<T> Identity<T> {
    /// Get the identity value
    pub fn value(&self) -> &T {
        &self.0
    }

    /// Consume and return inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

// ============================================================================
// MATHS Traits (T2-P)
// ============================================================================

/// T2-P: Set membership test
///
/// Grounded in T1 Comparison (κ): element → boolean
///
/// # Cross-Domain Transfer
/// - PV: Case in study cohort
/// - Economics: Asset in portfolio
/// - Software: Type membership, set contains
pub trait Membership {
    /// Element type
    type Element;

    /// Check if element is a member
    fn contains(&self, elem: &Self::Element) -> bool;

    /// Count of members (cardinality)
    fn cardinality(&self) -> usize;

    /// Check if empty
    fn is_empty(&self) -> bool {
        self.cardinality() == 0
    }
}

/// T2-P: Associative operation (grouping invariance)
///
/// Grounded in T1 Recursion (ρ): nested grouping equivalent
///
/// # Cross-Domain Transfer
/// - PV: Nested case grouping
/// - Economics: Compound operations
/// - Software: Fold/reduce operations
pub trait Associate {
    /// Operand type
    type Operand;

    /// Binary operation
    fn op(&self, a: &Self::Operand, b: &Self::Operand) -> Self::Operand;

    /// Check associativity: (a·b)·c = a·(b·c)
    fn is_associative(&self, a: &Self::Operand, b: &Self::Operand, c: &Self::Operand) -> bool
    where
        Self::Operand: PartialEq,
    {
        let left = self.op(&self.op(a, b), c);
        let right = self.op(a, &self.op(b, c));
        left == right
    }
}

/// T2-P: Transitive relation (chain implication)
///
/// Grounded in T1 Sequence (σ): a→b ∧ b→c ⟹ a→c
///
/// # Cross-Domain Transfer
/// - PV: Causal chain validation
/// - Economics: Supply chain tracing
/// - Software: Inheritance hierarchies
pub trait Transit {
    /// Element type in the relation
    type Element;

    /// Check if a relates to b
    fn relates(&self, a: &Self::Element, b: &Self::Element) -> bool;

    /// Check transitivity: if a→b and b→c, then a→c
    fn is_transitive(&self, a: &Self::Element, b: &Self::Element, c: &Self::Element) -> bool {
        if self.relates(a, b) && self.relates(b, c) {
            self.relates(a, c)
        } else {
            true // vacuously true if precondition not met
        }
    }
}

/// T2-P: Structure-preserving map (homomorphism)
///
/// Grounded in T1 Mapping (μ): preserve structure across transformation
///
/// # Cross-Domain Transfer
/// - PV: Standardization that preserves signal
/// - Economics: Currency conversion preserving value
/// - Software: Serialization/deserialization
pub trait Homeomorph {
    /// Source type
    type Source;
    /// Target type
    type Target;

    /// Transform source to target
    fn transform(&self, source: &Self::Source) -> Self::Target;

    /// Check if transformation preserves structure
    fn preserves_structure(&self, s1: &Self::Source, s2: &Self::Source) -> bool;
}

/// T2-P: Symmetric relation (bidirectional equivalence)
///
/// Grounded in T1 Comparison (κ): a~b ⟹ b~a
///
/// # Cross-Domain Transfer
/// - PV: Bidirectional drug interactions
/// - Economics: Mutual trade agreements
/// - Software: Equality, bidirectional references
pub trait Symmetric {
    /// Element type
    type Element;

    /// Check if a relates to b
    fn related(&self, a: &Self::Element, b: &Self::Element) -> bool;

    /// Check symmetry: a~b ⟹ b~a
    fn is_symmetric(&self, a: &Self::Element, b: &Self::Element) -> bool {
        self.related(a, b) == self.related(b, a)
    }
}

/// T2-P: Upper and lower bounds
///
/// Grounded in T1 Boundary (∂): limit constraints
///
/// # Cross-Domain Transfer
/// - PV: Confidence intervals
/// - Economics: Price ceilings/floors
/// - Software: Range types, assertions
pub trait Bound {
    /// Value type
    type Value;

    /// Get upper bound (supremum)
    fn upper_bound(&self) -> Option<Self::Value>;

    /// Get lower bound (infimum)
    fn lower_bound(&self) -> Option<Self::Value>;

    /// Check if value is within bounds
    fn within(&self, value: &Self::Value) -> bool
    where
        Self::Value: PartialOrd,
    {
        let above_lower = self.lower_bound().is_none_or(|l| *value >= l);
        let below_upper = self.upper_bound().is_none_or(|u| *value <= u);
        above_lower && below_upper
    }
}

/// T2-P: Logical implication (proof)
///
/// Grounded in T1 Sequence (σ): premises → conclusion
///
/// # Cross-Domain Transfer
/// - PV: Causality assessment
/// - Economics: Due diligence validation
/// - Software: Type checking, assertions
pub trait Prove {
    /// Premise type
    type Premise;
    /// Conclusion type
    type Conclusion;

    /// Check if premises imply conclusion
    fn implies(&self, premises: &[Self::Premise]) -> Option<Self::Conclusion>;

    /// Validate a proof
    fn validate(&self, proof: &Proof<Self::Premise, Self::Conclusion>) -> bool
    where
        Self::Conclusion: PartialEq,
    {
        self.implies(&proof.premises)
            .is_some_and(|c| c == proof.conclusion)
    }
}

/// T2-P: Commutative operation (order invariance)
///
/// Grounded in T1 Comparison (κ): a·b = b·a
///
/// # Cross-Domain Transfer
/// - PV: Order-independent combination
/// - Economics: Arbitrage-free pricing
/// - Software: Parallel-safe operations
pub trait Commute {
    /// Operand type
    type Operand;

    /// Binary operation
    fn op(&self, a: &Self::Operand, b: &Self::Operand) -> Self::Operand;

    /// Check commutativity: a·b = b·a
    fn is_commutative(&self, a: &Self::Operand, b: &Self::Operand) -> bool
    where
        Self::Operand: PartialEq,
    {
        self.op(a, b) == self.op(b, a)
    }
}

/// T2-P: Identity element (neutral operation)
///
/// Grounded in T1 State (ς): no-op element
///
/// # Cross-Domain Transfer
/// - PV: Baseline (no effect)
/// - Economics: Zero position
/// - Software: Default values, Option::None
pub trait Identify {
    /// Element type
    type Element;

    /// Get the identity element
    fn identity(&self) -> Self::Element;

    /// Check if element is identity: a·e = e·a = a
    fn is_identity(&self, elem: &Self::Element) -> bool
    where
        Self::Element: PartialEq;
}

// ============================================================================
// Mathematics Composite Trait (T2-C)
// ============================================================================

/// T2-C: The complete mathematics methodology as composite trait
///
/// Combines T2-P primitives for algebraic structures.
///
/// # Gödel Acknowledgment
///
/// A mathematical system proving its own consistency is impossible.
pub trait Mathematics:
    Membership + Associate + Transit + Symmetric + Bound + Prove + Commute + Identify
{
    /// Check if structure forms a group
    fn is_group(&self) -> bool
    where
        Self: Sized;
}

// ============================================================================
// Measured Math Types
// ============================================================================

/// A mathematical value with confidence (Codex IX: MEASURE)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeasuredBound<T> {
    /// The bounded value
    pub value: Bounded<T>,
    /// Confidence in the bounds
    pub confidence: Confidence,
}

impl<T> MeasuredBound<T> {
    /// Create new measured bound
    pub fn new(value: Bounded<T>, confidence: Confidence) -> Self {
        Self { value, confidence }
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Errors in mathematical operations
#[derive(Debug, thiserror::Error)]
pub enum MathError {
    /// Value out of bounds
    #[error("value out of bounds")]
    OutOfBounds,

    /// Proof invalid
    #[error("proof invalid: premises do not support conclusion")]
    ProofInvalid,

    /// Property not satisfied
    #[error("property {0} not satisfied")]
    PropertyNotSatisfied(String),

    /// Operation undefined
    #[error("operation undefined for given inputs")]
    OperationUndefined,
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bounded_in_bounds() {
        let b = Bounded::new(5, Some(0), Some(10));
        assert!(b.in_bounds());

        let out = Bounded::new(15, Some(0), Some(10));
        assert!(!out.in_bounds());
    }

    #[test]
    fn bounded_clamp() {
        let b = Bounded::new(15, Some(0), Some(10));
        assert_eq!(b.clamp(), 10);

        let low = Bounded::new(-5, Some(0), Some(10));
        assert_eq!(low.clamp(), 0);
    }

    #[test]
    fn bounded_unbounded() {
        let b: Bounded<i32> = Bounded::unbounded(100);
        assert!(b.in_bounds()); // Always in bounds when unbounded
    }

    #[test]
    fn relation_invert() {
        assert_eq!(Relation::LessThan.invert(), Relation::GreaterThan);
        assert_eq!(Relation::Equal.invert(), Relation::Equal);
    }

    #[test]
    fn relation_symmetric() {
        assert!(Relation::Equal.is_symmetric());
        assert!(!Relation::LessThan.is_symmetric());
    }

    #[test]
    fn proof_valid() {
        let p: Proof<&str, &str> = Proof::valid(vec!["a", "b"], "c");
        assert!(p.valid);
    }

    #[test]
    fn identity_value() {
        let id = Identity(0i32);
        assert_eq!(*id.value(), 0);
        assert_eq!(id.into_inner(), 0);
    }

    #[test]
    fn measured_bound_confidence() {
        let mb = MeasuredBound::new(
            Bounded::new(5.0, Some(0.0), Some(10.0)),
            Confidence::new(0.95),
        );
        assert!(mb.value.in_bounds());
        assert!((mb.confidence.value() - 0.95).abs() < f64::EPSILON);
    }
}
