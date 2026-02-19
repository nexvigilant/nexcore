//! Confidence: a bounded [0.0, 1.0] measure of epistemic certainty.
//!
//! Grounds to: N(Quantity) + ∂(Boundary)

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::GroundedError;

/// A confidence value in [0.0, 1.0].
///
/// Cannot be constructed with an out-of-range value.
/// Propagation follows multiplicative composition: combining
/// two independent confidence values yields their product.
#[derive(Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Confidence(f64);

impl Confidence {
    /// Create a new Confidence, returning error if out of [0.0, 1.0].
    pub fn new(value: f64) -> Result<Self, GroundedError> {
        if value.is_nan() || !(0.0..=1.0).contains(&value) {
            return Err(GroundedError::ConfidenceOutOfRange(value));
        }
        Ok(Self(value))
    }

    /// Full confidence (1.0).
    pub const CERTAIN: Self = Self(1.0);

    /// Zero confidence (0.0).
    pub const NONE: Self = Self(0.0);

    /// Common thresholds.
    pub const HIGH: Self = Self(0.95);
    pub const MEDIUM: Self = Self(0.80);
    pub const LOW: Self = Self(0.50);

    /// Raw value.
    pub fn value(self) -> f64 {
        self.0
    }

    /// Multiplicative composition: P(A and B) = P(A) * P(B) for independent events.
    pub fn compose(self, other: Self) -> Self {
        Self(self.0 * other.0)
    }

    /// Which band does this confidence fall into?
    pub fn band(self) -> ConfidenceBand {
        if self.0 >= 0.95 {
            ConfidenceBand::High
        } else if self.0 >= 0.80 {
            ConfidenceBand::Medium
        } else if self.0 >= 0.50 {
            ConfidenceBand::Low
        } else {
            ConfidenceBand::Negligible
        }
    }
}

impl fmt::Debug for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Confidence({:.4})", self.0)
    }
}

impl fmt::Display for Confidence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}%", self.0 * 100.0)
    }
}

/// Discrete confidence bands for exhaustive matching.
///
/// Grounds to: Σ(Sum) — one of four exclusive states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfidenceBand {
    /// >= 0.95 — safe to act on directly
    High,
    /// >= 0.80 — act with fallback
    Medium,
    /// >= 0.50 — requires additional evidence
    Low,
    /// < 0.50 — insufficient basis for action
    Negligible,
}

impl fmt::Display for ConfidenceBand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::High => write!(f, "HIGH (≥95%)"),
            Self::Medium => write!(f, "MEDIUM (≥80%)"),
            Self::Low => write!(f, "LOW (≥50%)"),
            Self::Negligible => write!(f, "NEGLIGIBLE (<50%)"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_bounds() {
        assert!(Confidence::new(0.5).is_ok());
        assert!(Confidence::new(0.0).is_ok());
        assert!(Confidence::new(1.0).is_ok());
        assert!(Confidence::new(-0.1).is_err());
        assert!(Confidence::new(1.1).is_err());
        assert!(Confidence::new(f64::NAN).is_err());
    }

    #[test]
    fn confidence_composition() {
        let a = Confidence::new(0.9).unwrap();
        let b = Confidence::new(0.8).unwrap();
        let composed = a.compose(b);
        assert!((composed.value() - 0.72).abs() < 1e-10);
    }

    #[test]
    fn confidence_bands() {
        assert_eq!(Confidence::new(0.96).unwrap().band(), ConfidenceBand::High);
        assert_eq!(
            Confidence::new(0.85).unwrap().band(),
            ConfidenceBand::Medium
        );
        assert_eq!(Confidence::new(0.60).unwrap().band(), ConfidenceBand::Low);
        assert_eq!(
            Confidence::new(0.30).unwrap().band(),
            ConfidenceBand::Negligible
        );
    }
}
