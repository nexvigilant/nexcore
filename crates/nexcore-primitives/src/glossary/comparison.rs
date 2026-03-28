//! # Comparison Primitive Types
//!
//! T1/T2-P comparison primitives from Lex Primitiva:
//! Comparison (κ), Boundary (∂), Quantity (N), and Sum (Σ).
//!
//! Four types encode the hidden structure of ordering and thresholding:
//!
//! - [`Ordering`] — κ+Σ: partial order with incomparability (extends `std::cmp::Ordering`)
//! - [`Comparison<T>`] — κ+ς: two values and their computed ordering
//! - [`Threshold<T>`] — ∂+N+κ: gate that defines the frontier
//! - [`Quantity`] — N+π: measured value with physical unit

use serde::{Deserialize, Serialize};

// ─── Ordering ────────────────────────────────────────────────────────────────

/// Partial order result, extending `std::cmp::Ordering` with `Incomparable`.
///
/// Standard Rust [`std::cmp::Ordering`] only models total orders.
/// `Ordering` adds `Incomparable` for partial orders — values that are
/// neither greater than, less than, nor equal to each other (e.g., two
/// non-dominated Pareto points).
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::comparison::Ordering;
///
/// assert_ne!(Ordering::Greater, Ordering::Less);
/// assert_eq!(Ordering::Equal, Ordering::Equal);
/// assert_eq!(Ordering::Incomparable, Ordering::Incomparable);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ordering {
    /// Left value is strictly greater than right.
    Greater,
    /// Left value is strictly less than right.
    Less,
    /// Left and right values are equal.
    Equal,
    /// Values cannot be ranked — neither dominates the other.
    Incomparable,
}

impl From<std::cmp::Ordering> for Ordering {
    #[inline]
    fn from(o: std::cmp::Ordering) -> Self {
        match o {
            std::cmp::Ordering::Greater => Self::Greater,
            std::cmp::Ordering::Less => Self::Less,
            std::cmp::Ordering::Equal => Self::Equal,
        }
    }
}

// ─── Comparison ──────────────────────────────────────────────────────────────

/// Two values and their computed partial ordering (κ).
///
/// `Comparison<T>` captures the act of comparison as a first-class value.
/// It pairs `left` and `right` with the `result` that `PartialOrd` yields,
/// defaulting to `Ordering::Incomparable` when `PartialOrd` returns `None`.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::comparison::{Comparison, Ordering};
///
/// let c = Comparison::compare(1.0_f64, 2.0_f64);
/// assert_eq!(c.result, Ordering::Less);
/// assert!(!c.is_dominated());
///
/// let d = Comparison::compare(3.0_f64, 1.0_f64);
/// assert!(d.is_dominated());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparison<T: PartialOrd> {
    /// The left-hand value in the comparison.
    pub left: T,
    /// The right-hand value in the comparison.
    pub right: T,
    /// The ordering relationship computed from `PartialOrd`.
    pub result: Ordering,
}

impl<T: PartialOrd> Comparison<T> {
    /// Compare `left` and `right`, computing the [`Ordering`] automatically.
    ///
    /// Uses [`PartialOrd::partial_cmp`]; yields [`Ordering::Incomparable`]
    /// when the values cannot be ordered (e.g., NaN).
    #[inline]
    pub fn compare(left: T, right: T) -> Self {
        let result = left
            .partial_cmp(&right)
            .map(Ordering::from)
            .unwrap_or(Ordering::Incomparable);
        Self {
            left,
            right,
            result,
        }
    }

    /// Returns `true` if `left >= right` on this dimension.
    ///
    /// Used in Pareto frontier analysis: a point is *dominated* on a
    /// dimension when its competitor scores at least as well.
    #[inline]
    pub fn is_dominated(&self) -> bool {
        matches!(self.result, Ordering::Greater | Ordering::Equal)
    }
}

// ─── Threshold ───────────────────────────────────────────────────────────────

/// A gate value that defines the frontier (∂+N+κ).
///
/// `Threshold<T>` answers the question "who gets to define the frontier?"
/// It is the hidden primitive inside every binary classification decision:
/// the value against which candidates are judged.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::comparison::Threshold;
///
/// let t = Threshold::new(0.05_f64);
/// assert!(t.below(&0.01));
/// assert!(!t.below(&0.10));
/// assert!(t.above(&0.10));
/// assert!(t.at(&0.05));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threshold<T: PartialOrd> {
    /// The gate value.
    value: T,
}

impl<T: PartialOrd> Threshold<T> {
    /// Construct a threshold at `value`.
    #[inline]
    pub fn new(value: T) -> Self {
        Self { value }
    }

    /// Returns `true` if `candidate` is strictly above the threshold.
    #[inline]
    pub fn above(&self, candidate: &T) -> bool {
        candidate > &self.value
    }

    /// Returns `true` if `candidate` is strictly below the threshold.
    #[inline]
    pub fn below(&self, candidate: &T) -> bool {
        candidate < &self.value
    }

    /// Returns `true` if `candidate` equals the threshold exactly.
    #[inline]
    pub fn at(&self, candidate: &T) -> bool
    where
        T: PartialEq,
    {
        candidate == &self.value
    }

    /// Reference to the gate value.
    #[inline]
    pub fn value(&self) -> &T {
        &self.value
    }
}

// ─── Quantity ─────────────────────────────────────────────────────────────────

/// A measured value with an associated physical unit (N+π).
///
/// `Quantity` pairs a dimensionless `f64` with a `unit` string so that
/// values from different measurement scales cannot be silently mixed.
/// Serialises cleanly for MCP tool payloads.
///
/// # Example
/// ```
/// use nexcore_primitives::glossary::comparison::Quantity;
///
/// let q = Quantity::new(1.5, "mg/dL");
/// assert!(q.is_positive());
/// assert!(!q.is_zero());
///
/// let zero = Quantity::new(0.0, "count");
/// assert!(zero.is_zero());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quantity {
    /// The numeric magnitude.
    pub value: f64,
    /// Physical or conceptual unit (e.g., `"mg/dL"`, `"events/1000py"`).
    pub unit: String,
}

impl Quantity {
    /// Construct a `Quantity` with the given value and unit.
    #[inline]
    pub fn new(value: f64, unit: impl Into<String>) -> Self {
        Self {
            value,
            unit: unit.into(),
        }
    }

    /// Returns `true` if the value is strictly positive.
    #[inline]
    pub fn is_positive(&self) -> bool {
        self.value > 0.0
    }

    /// Returns `true` if the value is exactly zero.
    ///
    /// Uses `== 0.0` — callers working with floating-point accumulation
    /// should apply their own epsilon comparison before calling this.
    #[inline]
    pub fn is_zero(&self) -> bool {
        self.value == 0.0
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // Ordering

    #[test]
    fn ordering_from_std_ordering_maps_all_variants() {
        assert_eq!(
            Ordering::from(std::cmp::Ordering::Greater),
            Ordering::Greater
        );
        assert_eq!(Ordering::from(std::cmp::Ordering::Less), Ordering::Less);
        assert_eq!(Ordering::from(std::cmp::Ordering::Equal), Ordering::Equal);
    }

    #[test]
    fn ordering_incomparable_is_distinct() {
        assert_ne!(Ordering::Incomparable, Ordering::Greater);
        assert_ne!(Ordering::Incomparable, Ordering::Less);
        assert_ne!(Ordering::Incomparable, Ordering::Equal);
    }

    // Comparison

    #[test]
    fn comparison_less_is_not_dominated() {
        let c = Comparison::compare(1u32, 5u32);
        assert_eq!(c.result, Ordering::Less);
        assert!(!c.is_dominated());
    }

    #[test]
    fn comparison_greater_is_dominated() {
        let c = Comparison::compare(10u32, 3u32);
        assert_eq!(c.result, Ordering::Greater);
        assert!(c.is_dominated());
    }

    #[test]
    fn comparison_equal_is_dominated() {
        let c = Comparison::compare(7u32, 7u32);
        assert_eq!(c.result, Ordering::Equal);
        assert!(c.is_dominated());
    }

    #[test]
    fn comparison_nan_yields_incomparable() {
        let c = Comparison::compare(f64::NAN, 1.0_f64);
        assert_eq!(c.result, Ordering::Incomparable);
        assert!(!c.is_dominated());
    }

    // Threshold

    #[test]
    fn threshold_above_and_below_are_exclusive() {
        let t = Threshold::new(0.05_f64);
        let low = 0.01_f64;
        let high = 0.10_f64;
        assert!(t.below(&low));
        assert!(!t.above(&low));
        assert!(t.above(&high));
        assert!(!t.below(&high));
    }

    #[test]
    fn threshold_at_matches_exact_value() {
        let t = Threshold::new(42u32);
        assert!(t.at(&42u32));
        assert!(!t.at(&43u32));
        assert!(!t.above(&42u32));
        assert!(!t.below(&42u32));
    }

    #[test]
    fn threshold_value_accessor_returns_gate() {
        let t = Threshold::new(3.14_f64);
        assert_eq!(*t.value(), 3.14_f64);
    }

    // Quantity

    #[test]
    fn quantity_positive_flag() {
        let q = Quantity::new(1.5, "mg/dL");
        assert!(q.is_positive());
        assert!(!q.is_zero());
    }

    #[test]
    fn quantity_zero_flag() {
        let q = Quantity::new(0.0, "count");
        assert!(q.is_zero());
        assert!(!q.is_positive());
    }

    #[test]
    fn quantity_negative_is_neither_positive_nor_zero() {
        let q = Quantity::new(-1.0, "delta");
        assert!(!q.is_positive());
        assert!(!q.is_zero());
    }

    #[test]
    fn quantity_unit_stored_correctly() {
        let q = Quantity::new(100.0, "events/1000py");
        assert_eq!(q.unit, "events/1000py");
    }
}
