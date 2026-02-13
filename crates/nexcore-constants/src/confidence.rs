//! T2-P: Confidence — Bayesian probability of correctness [0.0, 1.0].
//!
//! Merged superset of stem-core and nexcore-vigilance definitions.
//! Includes `cmp_total()` (stem-core) and by-ref `combine()` (both).

use serde::{Deserialize, Serialize};
use std::fmt;

/// Confidence score in range [0.0, 1.0].
///
/// # Codex V Exception
///
/// `f64` lacks `Ord` due to NaN. `PartialOrd` provided.
/// Use [`Confidence::cmp_total()`] for total ordering when required.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Confidence(f64);

impl Confidence {
    /// Perfect confidence (1.0).
    pub const PERFECT: Self = Self(1.0);
    /// Zero confidence (0.0).
    pub const NONE: Self = Self(0.0);
    /// High confidence (0.9).
    pub const HIGH: Self = Self(0.9);
    /// Uncertain / maximum entropy (0.5).
    pub const UNCERTAIN: Self = Self(0.5);
    /// Low confidence (0.3).
    pub const LOW: Self = Self(0.3);

    /// Create new confidence score, clamping to [0.0, 1.0].
    #[must_use]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 1.0))
    }

    /// Get the raw confidence value.
    #[must_use]
    pub const fn value(self) -> f64 {
        self.0
    }

    /// Combine two confidence scores (product rule for independent probabilities).
    ///
    /// Accepts both by-value and by-ref (type is `Copy`).
    #[must_use]
    pub fn combine(self, other: Self) -> Self {
        Self::new(self.0 * other.0)
    }

    /// Total ordering comparison (Codex V compliance).
    ///
    /// Treats NaN as less than all other values for deterministic ordering.
    #[must_use]
    pub fn cmp_total(self, other: Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }

    /// Full confidence (1.0). Alias for [`PERFECT`](Self::PERFECT).
    pub const CERTAIN: Self = Self(1.0);

    /// Common 95% confidence level.
    #[must_use]
    pub fn ninety_five() -> Self {
        Self(0.95)
    }

    /// Common 99% confidence level.
    #[must_use]
    pub fn ninety_nine() -> Self {
        Self(0.99)
    }

    /// Whether confidence is certain (1.0).
    #[must_use]
    pub fn is_certain(self) -> bool {
        (self.0 - 1.0).abs() < f64::EPSILON
    }
}

impl Default for Confidence {
    fn default() -> Self {
        Self(0.5) // Maximum entropy default
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}%", self.0 * 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_to_range() {
        assert!((Confidence::new(1.5).value() - 1.0).abs() < f64::EPSILON);
        assert!((Confidence::new(-0.5).value() - 0.0).abs() < f64::EPSILON);
        assert!((Confidence::new(0.7).value() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn combines_multiplicatively() {
        let a = Confidence::new(0.8);
        let b = Confidence::new(0.9);
        let combined = a.combine(b);
        assert!((combined.value() - 0.72).abs() < f64::EPSILON);
    }

    #[test]
    fn cmp_total_provides_ordering() {
        let a = Confidence::new(0.3);
        let b = Confidence::new(0.7);
        assert_eq!(a.cmp_total(b), std::cmp::Ordering::Less);
        assert_eq!(b.cmp_total(a), std::cmp::Ordering::Greater);
        assert_eq!(a.cmp_total(a), std::cmp::Ordering::Equal);
    }

    #[test]
    fn default_is_maximum_entropy() {
        assert!((Confidence::default().value() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn named_constants() {
        assert!((Confidence::PERFECT.value() - 1.0).abs() < f64::EPSILON);
        assert!((Confidence::NONE.value() - 0.0).abs() < f64::EPSILON);
        assert!((Confidence::HIGH.value() - 0.9).abs() < f64::EPSILON);
        assert!((Confidence::UNCERTAIN.value() - 0.5).abs() < f64::EPSILON);
        assert!((Confidence::LOW.value() - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn display_format() {
        assert_eq!(format!("{}", Confidence::PERFECT), "100.00%");
        assert_eq!(format!("{}", Confidence::NONE), "0.00%");
    }

    #[test]
    fn serde_round_trip() {
        let c = Confidence::new(0.85);
        let json = serde_json::to_string(&c).unwrap();
        let back: Confidence = serde_json::from_str(&json).unwrap();
        assert!((c.value() - back.value()).abs() < f64::EPSILON);
    }
}
