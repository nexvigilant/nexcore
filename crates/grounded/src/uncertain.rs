//! Uncertain<T>: A value paired with its confidence.
//!
//! Grounds to: ×(Product) of T and Confidence.
//! Forces explicit handling of epistemic uncertainty at the type level.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::confidence::{Confidence, ConfidenceBand};

/// A value of type T with an associated confidence measure.
///
/// This is the core GROUNDED type. It cannot be silently unwrapped —
/// you must explicitly handle the confidence level before accessing the value.
///
/// # Examples
/// ```
/// use grounded::{Uncertain, Confidence};
///
/// let prediction = Uncertain::new(42.0_f64, Confidence::new(0.87).unwrap());
/// match prediction.band() {
///     grounded::ConfidenceBand::High => println!("act: {}", prediction.value()),
///     grounded::ConfidenceBand::Medium => println!("act with fallback: {}", prediction.value()),
///     grounded::ConfidenceBand::Low => println!("gather more evidence"),
///     grounded::ConfidenceBand::Negligible => println!("cannot act"),
/// }
/// ```
#[derive(Clone, Serialize, Deserialize)]
pub struct Uncertain<T> {
    value: T,
    confidence: Confidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    provenance: Option<String>,
}

impl<T> Uncertain<T> {
    /// Create a new uncertain value.
    pub fn new(value: T, confidence: Confidence) -> Self {
        Self {
            value,
            confidence,
            provenance: None,
        }
    }

    /// Create with provenance tracking.
    pub fn with_provenance(
        value: T,
        confidence: Confidence,
        provenance: impl Into<String>,
    ) -> Self {
        Self {
            value,
            confidence,
            provenance: Some(provenance.into()),
        }
    }

    /// Create a certain value (confidence = 1.0).
    pub fn certain(value: T) -> Self {
        Self {
            value,
            confidence: Confidence::CERTAIN,
            provenance: None,
        }
    }

    /// Access the inner value (explicit acknowledgment of uncertainty).
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Consume and return the inner value.
    pub fn into_value(self) -> T {
        self.value
    }

    /// Get the confidence level.
    pub fn confidence(&self) -> Confidence {
        self.confidence
    }

    /// Get the confidence band for pattern matching.
    pub fn band(&self) -> ConfidenceBand {
        self.confidence.band()
    }

    /// Get provenance if set.
    pub fn provenance(&self) -> Option<&str> {
        self.provenance.as_deref()
    }

    /// Transform the value while preserving confidence.
    /// Grounds to: μ(Mapping) — transforms A→B while preserving metadata.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Uncertain<U> {
        Uncertain {
            value: f(self.value),
            confidence: self.confidence,
            provenance: self.provenance,
        }
    }

    /// Transform with a function that may reduce confidence.
    /// The resulting confidence is the product of the original and the function's confidence.
    pub fn and_then<U, F>(self, f: F, transform_confidence: Confidence) -> Uncertain<U>
    where
        F: FnOnce(T) -> U,
    {
        Uncertain {
            value: f(self.value),
            confidence: self.confidence.compose(transform_confidence),
            provenance: self.provenance,
        }
    }

    /// Combine two uncertain values. Confidence is the product (independent assumption).
    /// Grounds to: ×(Product) — conjunctive combination.
    pub fn combine<U, V, F>(self, other: Uncertain<U>, f: F) -> Uncertain<V>
    where
        F: FnOnce(T, U) -> V,
    {
        Uncertain {
            value: f(self.value, other.value),
            confidence: self.confidence.compose(other.confidence),
            provenance: None,
        }
    }

    /// Only proceed if confidence meets threshold.
    /// Grounds to: ∂(Boundary) — gate on confidence.
    pub fn require(self, min_confidence: Confidence) -> Result<T, Uncertain<T>> {
        if self.confidence >= min_confidence {
            Ok(self.value)
        } else {
            Err(self)
        }
    }

    /// Provide a fallback value if confidence is below threshold.
    pub fn unwrap_or(self, min_confidence: Confidence, fallback: T) -> T {
        if self.confidence >= min_confidence {
            self.value
        } else {
            fallback
        }
    }

    /// Provide a computed fallback if confidence is below threshold.
    pub fn unwrap_or_else<F: FnOnce(Confidence) -> T>(self, min_confidence: Confidence, f: F) -> T {
        if self.confidence >= min_confidence {
            self.value
        } else {
            f(self.confidence)
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Uncertain<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Uncertain({:?} @ {})", self.value, self.confidence)
    }
}

impl<T: fmt::Display> fmt::Display for Uncertain<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.value, self.confidence)
    }
}

impl<T: PartialEq> PartialEq for Uncertain<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.confidence == other.confidence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GroundedError;

    fn c(v: f64) -> Confidence {
        Confidence::new(v).unwrap_or_else(|_: GroundedError| Confidence::NONE)
    }

    #[test]
    fn uncertain_basic() {
        let u = Uncertain::new(42, c(0.9));
        assert_eq!(*u.value(), 42);
        assert_eq!(u.confidence(), c(0.9));
        assert_eq!(u.band(), ConfidenceBand::Medium);
    }

    #[test]
    fn uncertain_certain() {
        let u = Uncertain::certain(42);
        assert_eq!(u.confidence(), Confidence::CERTAIN);
        assert_eq!(u.band(), ConfidenceBand::High);
    }

    #[test]
    fn uncertain_map() {
        let u = Uncertain::new(10, c(0.8));
        let doubled = u.map(|x| x * 2);
        assert_eq!(*doubled.value(), 20);
        assert_eq!(doubled.confidence(), c(0.8));
    }

    #[test]
    fn uncertain_combine() {
        let a = Uncertain::new(10, c(0.9));
        let b = Uncertain::new(20, c(0.8));
        let sum = a.combine(b, |x, y| x + y);
        assert_eq!(*sum.value(), 30);
        // 0.9 * 0.8 = 0.72
        assert!((sum.confidence().value() - 0.72).abs() < 1e-10);
    }

    #[test]
    fn uncertain_require() {
        let high = Uncertain::new(42, c(0.95));
        let low = Uncertain::new(42, c(0.5));

        assert!(high.require(c(0.9)).is_ok());
        assert!(low.require(c(0.9)).is_err());
    }

    #[test]
    fn uncertain_fallback() {
        let low = Uncertain::new(42, c(0.3));
        let result = low.unwrap_or(c(0.5), 0);
        assert_eq!(result, 0);
    }

    #[test]
    fn uncertain_display() {
        let u = Uncertain::new(42, c(0.87));
        let s = format!("{u}");
        assert!(s.contains("42"));
        assert!(s.contains("87.0%"));
    }

    #[test]
    fn uncertain_provenance() {
        let u = Uncertain::with_provenance(42, c(0.9), "model-v3 inference");
        assert_eq!(u.provenance(), Some("model-v3 inference"));
    }
}
