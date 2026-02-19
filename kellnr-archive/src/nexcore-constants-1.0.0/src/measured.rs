//! T2-C: Measured — A value paired with its confidence/uncertainty.
//!
//! Codex IX (MEASURE): All computed values carry confidence.

use serde::{Deserialize, Serialize};

use crate::Confidence;

/// A value with associated confidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measured<T> {
    /// The measured value.
    pub value: T,
    /// Confidence in the measurement.
    pub confidence: Confidence,
}

impl<T> Measured<T> {
    /// Create a new measured value.
    pub fn new(value: T, confidence: Confidence) -> Self {
        Self { value, confidence }
    }

    /// Create with perfect confidence (1.0).
    pub fn certain(value: T) -> Self {
        Self::new(value, Confidence::PERFECT)
    }

    /// Create with specified confidence.
    pub fn uncertain(value: T, confidence: Confidence) -> Self {
        Self { value, confidence }
    }

    /// Map the value while preserving confidence.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Measured<U> {
        Measured {
            value: f(self.value),
            confidence: self.confidence,
        }
    }

    /// Combine with another measured value using a function,
    /// multiplying confidences (product rule).
    pub fn combine_with<U, V, F: FnOnce(T, U) -> V>(self, other: Measured<U>, f: F) -> Measured<V> {
        Measured {
            value: f(self.value, other.value),
            confidence: self.confidence.combine(other.confidence),
        }
    }
}

impl<T: Copy> Copy for Measured<T> {}

impl<T: PartialEq> PartialEq for Measured<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.confidence == other.confidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certain_has_perfect_confidence() {
        let m = Measured::certain(42);
        assert_eq!(m.value, 42);
        assert!((m.confidence.value() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn map_preserves_confidence() {
        let m = Measured::new(5, Confidence::new(0.8));
        let doubled = m.map(|x| x * 2);
        assert_eq!(doubled.value, 10);
        assert!((doubled.confidence.value() - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn combine_with_multiplies_confidence() {
        let a = Measured::new(2, Confidence::new(0.8));
        let b = Measured::new(3, Confidence::new(0.9));
        let sum = a.combine_with(b, |x, y| x + y);
        assert_eq!(sum.value, 5);
        assert!((sum.confidence.value() - 0.72).abs() < f64::EPSILON);
    }

    #[test]
    fn copy_works_for_copy_types() {
        let m = Measured::new(10, Confidence::new(0.9));
        let m2 = m; // Copy
        assert_eq!(m.value, m2.value);
    }

    #[test]
    fn serde_round_trip() {
        let m = Measured::new(42, Confidence::new(0.85));
        let json = serde_json::to_string(&m).unwrap();
        let back: Measured<i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(m, back);
    }
}
