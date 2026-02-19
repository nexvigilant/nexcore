//! T2-P Primitives for Clinical Trial Statistics
//!
//! These are cross-domain statistical primitives wrapped for type safety.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Effect size (Cohen's d, Hedges' g, odds ratio, hazard ratio, etc.)
/// T2-P: Newtype over f64 with domain meaning
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct EffectSize(pub f64);

impl EffectSize {
    /// Create a new effect size
    #[must_use]
    pub const fn new(value: f64) -> Self {
        Self(value)
    }

    /// Get the raw value
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0
    }

    /// Cohen's d interpretation thresholds
    #[must_use]
    pub fn cohen_interpretation(&self) -> EffectSizeInterpretation {
        let abs = self.0.abs();
        if abs < 0.2 {
            EffectSizeInterpretation::Negligible
        } else if abs < 0.5 {
            EffectSizeInterpretation::Small
        } else if abs < 0.8 {
            EffectSizeInterpretation::Medium
        } else {
            EffectSizeInterpretation::Large
        }
    }

    /// Check if clinically meaningful (typical threshold: |d| >= 0.5)
    #[must_use]
    pub fn is_clinically_meaningful(&self, threshold: f64) -> bool {
        self.0.abs() >= threshold
    }
}

impl fmt::Display for EffectSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "d={:.3}", self.0)
    }
}

impl From<f64> for EffectSize {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

/// Cohen's d interpretation categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EffectSizeInterpretation {
    Negligible,
    Small,
    Medium,
    Large,
}

/// P-value (probability of observing results under null hypothesis)
/// T2-P: Newtype with validation
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct PValue(f64);

impl PValue {
    /// Create a new p-value (must be in [0, 1])
    ///
    /// # Errors
    /// Returns error if value is outside [0, 1]
    pub fn new(value: f64) -> Result<Self, PValueError> {
        if !(0.0..=1.0).contains(&value) {
            return Err(PValueError::OutOfRange(value));
        }
        Ok(Self(value))
    }

    /// Create unchecked (for internal use)
    #[must_use]
    pub const fn new_unchecked(value: f64) -> Self {
        Self(value)
    }

    /// Get the raw value
    #[must_use]
    pub const fn value(&self) -> f64 {
        self.0
    }

    /// Check statistical significance at given alpha level
    #[must_use]
    pub fn is_significant(&self, alpha: f64) -> bool {
        self.0 < alpha
    }

    /// Standard significance check (alpha = 0.05)
    #[must_use]
    pub fn is_significant_05(&self) -> bool {
        self.is_significant(0.05)
    }

    /// Stringent significance check (alpha = 0.01)
    #[must_use]
    pub fn is_significant_01(&self) -> bool {
        self.is_significant(0.01)
    }

    /// Get significance interpretation
    #[must_use]
    pub fn significance(&self) -> StatisticalSignificance {
        if self.0 < 0.001 {
            StatisticalSignificance::HighlySignificant
        } else if self.0 < 0.01 {
            StatisticalSignificance::VerySignificant
        } else if self.0 < 0.05 {
            StatisticalSignificance::Significant
        } else if self.0 < 0.10 {
            StatisticalSignificance::Marginal
        } else {
            StatisticalSignificance::NotSignificant
        }
    }
}

impl fmt::Display for PValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 < 0.001 {
            write!(f, "p<0.001")
        } else {
            write!(f, "p={:.3}", self.0)
        }
    }
}

/// P-value validation error
#[derive(Debug, Clone, PartialEq)]
pub enum PValueError {
    OutOfRange(f64),
}

impl fmt::Display for PValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange(v) => write!(f, "P-value {v} out of range [0, 1]"),
        }
    }
}

impl std::error::Error for PValueError {}

/// Statistical significance levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum StatisticalSignificance {
    NotSignificant,
    Marginal,
    Significant,
    VerySignificant,
    HighlySignificant,
}

impl StatisticalSignificance {
    /// Convert to star notation (*, **, ***)
    #[must_use]
    pub const fn stars(&self) -> &'static str {
        match self {
            Self::NotSignificant => "",
            Self::Marginal => "†",
            Self::Significant => "*",
            Self::VerySignificant => "**",
            Self::HighlySignificant => "***",
        }
    }
}

/// Confidence interval
/// T2-P: Lower and upper bounds with confidence level
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Lower bound
    pub lower: f64,
    /// Upper bound
    pub upper: f64,
    /// Confidence level (0.95 = 95% CI)
    pub level: f64,
}

impl ConfidenceInterval {
    /// Create a new confidence interval
    #[must_use]
    pub const fn new(lower: f64, upper: f64, level: f64) -> Self {
        Self {
            lower,
            upper,
            level,
        }
    }

    /// Standard 95% CI
    #[must_use]
    pub const fn ci95(lower: f64, upper: f64) -> Self {
        Self::new(lower, upper, 0.95)
    }

    /// 99% CI
    #[must_use]
    pub const fn ci99(lower: f64, upper: f64) -> Self {
        Self::new(lower, upper, 0.99)
    }

    /// Width of the interval
    #[must_use]
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }

    /// Point estimate (midpoint)
    #[must_use]
    pub fn point_estimate(&self) -> f64 {
        (self.lower + self.upper) / 2.0
    }

    /// Check if CI excludes a value (e.g., null hypothesis value)
    #[must_use]
    pub fn excludes(&self, value: f64) -> bool {
        value < self.lower || value > self.upper
    }

    /// Check if CI excludes zero (common for difference tests)
    #[must_use]
    pub fn excludes_zero(&self) -> bool {
        self.excludes(0.0)
    }

    /// Check if CI excludes one (common for ratio tests like RR, OR)
    #[must_use]
    pub fn excludes_one(&self) -> bool {
        self.excludes(1.0)
    }

    /// FDA effectiveness criterion: lower bound > 1.0 for ratio measures
    #[must_use]
    pub fn demonstrates_benefit(&self) -> bool {
        self.lower > 1.0
    }
}

impl fmt::Display for ConfidenceInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:.0}% CI [{:.3}, {:.3}]",
            self.level * 100.0,
            self.lower,
            self.upper
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_size_interpretation() {
        assert_eq!(
            EffectSize(0.1).cohen_interpretation(),
            EffectSizeInterpretation::Negligible
        );
        assert_eq!(
            EffectSize(0.3).cohen_interpretation(),
            EffectSizeInterpretation::Small
        );
        assert_eq!(
            EffectSize(0.6).cohen_interpretation(),
            EffectSizeInterpretation::Medium
        );
        assert_eq!(
            EffectSize(1.0).cohen_interpretation(),
            EffectSizeInterpretation::Large
        );
    }

    #[test]
    fn test_pvalue_significance() {
        let p = PValue::new(0.03).unwrap();
        assert!(p.is_significant_05());
        assert!(!p.is_significant_01());
        assert_eq!(p.significance(), StatisticalSignificance::Significant);
    }

    #[test]
    fn test_ci_excludes() {
        let ci = ConfidenceInterval::ci95(1.2, 2.5);
        assert!(ci.excludes_one());
        assert!(ci.demonstrates_benefit());
    }
}
