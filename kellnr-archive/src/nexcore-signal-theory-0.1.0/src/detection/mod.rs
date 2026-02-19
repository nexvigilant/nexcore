// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Detection Primitives
//!
//! Core types for observation spaces, baselines, ratios, and detection outcomes.
//!
//! These are the building blocks that all signal detection methods share.
//!
//! ## Decomposition
//!
//! Every detection method reduces to:
//! 1. An **observation space** (the data)
//! 2. A **baseline** (expected under null)
//! 3. A **measure** (ratio or difference)
//! 4. An **outcome** (detected / not detected / indeterminate)

use alloc::string::String;
use alloc::vec::Vec;

// ═══════════════════════════════════════════════════════════
// OBSERVATION SPACE
// ═══════════════════════════════════════════════════════════

/// An observation space: the universe of data points for detection.
///
/// ## Tier: T2-P (ν + N)
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ObservationSpace {
    /// Total number of observations.
    pub total_count: u64,
    /// Dimensionality (number of variables observed).
    pub dimensions: u32,
    /// Label for the observation space.
    pub label: String,
}

impl ObservationSpace {
    /// Create a new observation space.
    #[must_use]
    pub fn new(total_count: u64, dimensions: u32, label: impl Into<String>) -> Self {
        Self {
            total_count,
            dimensions,
            label: label.into(),
        }
    }

    /// Whether the space is non-empty.
    #[must_use]
    pub fn is_non_empty(&self) -> bool {
        self.total_count > 0
    }

    /// Whether the space is univariate.
    #[must_use]
    pub fn is_univariate(&self) -> bool {
        self.dimensions == 1
    }
}

// ═══════════════════════════════════════════════════════════
// BASELINE
// ═══════════════════════════════════════════════════════════

/// A baseline: the expected value under the null hypothesis.
///
/// ## Tier: T2-P (∅ + N)
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Baseline {
    /// Expected count or rate.
    pub expected: f64,
    /// Variance of the expected value.
    pub variance: f64,
}

impl Baseline {
    /// Create a new baseline.
    ///
    /// Returns `None` if expected is not positive or not finite.
    #[must_use]
    pub fn try_new(expected: f64, variance: f64) -> Option<Self> {
        if expected > 0.0 && expected.is_finite() && variance >= 0.0 && variance.is_finite() {
            Some(Self { expected, variance })
        } else {
            None
        }
    }

    /// Standard deviation of the baseline.
    #[must_use]
    pub fn std_dev(&self) -> f64 {
        self.variance.sqrt()
    }

    /// Coefficient of variation.
    #[must_use]
    pub fn cv(&self) -> f64 {
        if self.expected == 0.0 {
            return 0.0;
        }
        self.std_dev() / self.expected
    }
}

// ═══════════════════════════════════════════════════════════
// MEASURES
// ═══════════════════════════════════════════════════════════

/// A ratio measure: observed / expected.
///
/// The fundamental comparator in disproportionality analysis.
///
/// ## Tier: T2-P (κ + N)
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Ratio(pub f64);

impl Ratio {
    /// Create a ratio from observed and expected.
    ///
    /// Returns `None` if expected <= 0 or result is not finite.
    #[must_use]
    pub fn from_counts(observed: f64, expected: f64) -> Option<Self> {
        if expected > 0.0 && observed.is_finite() && expected.is_finite() {
            let r = observed / expected;
            if r.is_finite() {
                return Some(Self(r));
            }
        }
        None
    }

    /// Whether this ratio indicates disproportionality (> 1.0).
    #[must_use]
    pub fn is_disproportionate(&self) -> bool {
        self.0 > 1.0
    }

    /// Log2 of the ratio (information component).
    #[must_use]
    pub fn log2(&self) -> f64 {
        if self.0 <= 0.0 {
            return f64::NEG_INFINITY;
        }
        self.0.log2()
    }
}

/// A difference measure: observed - expected.
///
/// ## Tier: T2-P (κ + N)
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Difference(pub f64);

impl Difference {
    /// Create a difference from observed and expected.
    #[must_use]
    pub fn from_counts(observed: f64, expected: f64) -> Self {
        Self(observed - expected)
    }

    /// Whether the difference is positive (observed > expected).
    #[must_use]
    pub fn is_positive(&self) -> bool {
        self.0 > 0.0
    }

    /// Absolute value of the difference.
    #[must_use]
    pub fn abs(&self) -> f64 {
        self.0.abs()
    }
}

// ═══════════════════════════════════════════════════════════
// DETECTION INTERVAL
// ═══════════════════════════════════════════════════════════

/// A confidence interval around a detection measure.
///
/// ## Tier: T2-P (∂ + N + κ)
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DetectionInterval {
    /// Lower bound of the interval.
    pub lower: f64,
    /// Upper bound of the interval.
    pub upper: f64,
    /// Confidence level (e.g., 0.95 for 95%).
    pub confidence_level: f64,
}

impl DetectionInterval {
    /// Create a new detection interval.
    ///
    /// Returns `None` if bounds are invalid.
    #[must_use]
    pub fn try_new(lower: f64, upper: f64, confidence_level: f64) -> Option<Self> {
        if lower.is_finite()
            && upper.is_finite()
            && lower <= upper
            && confidence_level > 0.0
            && confidence_level < 1.0
        {
            Some(Self {
                lower,
                upper,
                confidence_level,
            })
        } else {
            None
        }
    }

    /// Width of the interval.
    #[must_use]
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }

    /// Midpoint of the interval.
    #[must_use]
    pub fn midpoint(&self) -> f64 {
        (self.lower + self.upper) / 2.0
    }

    /// Whether the interval excludes a value (typically 1.0 for ratios).
    #[must_use]
    pub fn excludes(&self, value: f64) -> bool {
        value < self.lower || value > self.upper
    }

    /// Whether the lower bound exceeds a threshold.
    #[must_use]
    pub fn lower_exceeds(&self, threshold: f64) -> bool {
        self.lower > threshold
    }
}

// ═══════════════════════════════════════════════════════════
// DETECTION OUTCOME
// ═══════════════════════════════════════════════════════════

/// The outcome of a signal detection evaluation.
///
/// ## Tier: T2-C (∂ + κ + ∃ + ∅)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DetectionOutcome {
    /// Signal detected — threshold exceeded.
    Detected,
    /// Signal not detected — below threshold.
    NotDetected,
    /// Indeterminate — insufficient data or borderline result.
    Indeterminate,
}

impl DetectionOutcome {
    /// Whether the signal was detected.
    #[must_use]
    pub const fn is_detected(&self) -> bool {
        matches!(self, Self::Detected)
    }

    /// Whether the result is definitive (not indeterminate).
    #[must_use]
    pub const fn is_definitive(&self) -> bool {
        !matches!(self, Self::Indeterminate)
    }
}

// ═══════════════════════════════════════════════════════════
// SIGNAL STRENGTH LEVEL
// ═══════════════════════════════════════════════════════════

/// Qualitative signal strength classification.
///
/// ## Tier: T1 (κ)
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
pub enum SignalStrengthLevel {
    /// No signal detected.
    None,
    /// Weak signal — barely above threshold.
    Weak,
    /// Moderate signal — clearly above threshold.
    Moderate,
    /// Strong signal — well above threshold.
    Strong,
    /// Critical signal — overwhelming evidence.
    Critical,
}

impl SignalStrengthLevel {
    /// Classify from a ratio value.
    #[must_use]
    pub fn from_ratio(ratio: f64) -> Self {
        if ratio < 1.0 {
            Self::None
        } else if ratio < 2.0 {
            Self::Weak
        } else if ratio < 4.0 {
            Self::Moderate
        } else if ratio < 8.0 {
            Self::Strong
        } else {
            Self::Critical
        }
    }

    /// All levels in order.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::None,
            Self::Weak,
            Self::Moderate,
            Self::Strong,
            Self::Critical,
        ]
    }

    /// Numeric ordinal (0-4).
    #[must_use]
    pub const fn ordinal(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::Weak => 1,
            Self::Moderate => 2,
            Self::Strong => 3,
            Self::Critical => 4,
        }
    }
}

// ═══════════════════════════════════════════════════════════
// SIGNAL VERIFICATION REPORT
// ═══════════════════════════════════════════════════════════

/// A complete signal verification report combining all detection primitives.
///
/// ## Tier: T2-C (∂ + κ + N + ∃ + ∅)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignalVerificationReport {
    /// The detection outcome.
    pub outcome: DetectionOutcome,
    /// Signal strength classification.
    pub strength: SignalStrengthLevel,
    /// The primary ratio measure.
    pub ratio: Option<Ratio>,
    /// The confidence interval (if computed).
    pub interval: Option<DetectionInterval>,
    /// Free-text notes.
    pub notes: Vec<String>,
}

impl SignalVerificationReport {
    /// Create a minimal report.
    #[must_use]
    pub fn new(outcome: DetectionOutcome, strength: SignalStrengthLevel) -> Self {
        Self {
            outcome,
            strength,
            ratio: None,
            interval: None,
            notes: Vec::new(),
        }
    }

    /// Add a ratio measure.
    #[must_use]
    pub fn with_ratio(mut self, ratio: Ratio) -> Self {
        self.ratio = Some(ratio);
        self
    }

    /// Add a confidence interval.
    #[must_use]
    pub fn with_interval(mut self, interval: DetectionInterval) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Add a note.
    pub fn add_note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_space() {
        let space = ObservationSpace::new(1000, 2, "drug-event");
        assert!(space.is_non_empty());
        assert!(!space.is_univariate());
    }

    #[test]
    fn test_baseline() {
        let b = Baseline::try_new(5.0, 1.0);
        assert!(b.is_some());
        let b = b.unwrap_or_else(|| Baseline {
            expected: 1.0,
            variance: 0.0,
        });
        assert!((b.std_dev() - 1.0).abs() < f64::EPSILON);
        assert!((b.cv() - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_baseline_invalid() {
        assert!(Baseline::try_new(0.0, 1.0).is_none());
        assert!(Baseline::try_new(-1.0, 1.0).is_none());
        assert!(Baseline::try_new(1.0, -1.0).is_none());
    }

    #[test]
    fn test_ratio() {
        let r = Ratio::from_counts(15.0, 5.0);
        assert!(r.is_some());
        let r = r.unwrap_or_else(|| Ratio(1.0));
        assert!((r.0 - 3.0).abs() < f64::EPSILON);
        assert!(r.is_disproportionate());
    }

    #[test]
    fn test_ratio_log2() {
        let r = Ratio(4.0);
        assert!((r.log2() - 2.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_difference() {
        let d = Difference::from_counts(15.0, 5.0);
        assert!((d.0 - 10.0).abs() < f64::EPSILON);
        assert!(d.is_positive());
    }

    #[test]
    fn test_detection_interval() {
        let ci = DetectionInterval::try_new(1.5, 3.5, 0.95);
        assert!(ci.is_some());
        let ci = ci.unwrap_or_else(|| DetectionInterval {
            lower: 0.0,
            upper: 1.0,
            confidence_level: 0.95,
        });
        assert!((ci.width() - 2.0).abs() < f64::EPSILON);
        assert!((ci.midpoint() - 2.5).abs() < f64::EPSILON);
        assert!(ci.excludes(1.0)); // below lower bound
        assert!(ci.lower_exceeds(1.0)); // lower > 1.0
    }

    #[test]
    fn test_detection_interval_invalid() {
        assert!(DetectionInterval::try_new(3.0, 1.0, 0.95).is_none()); // lower > upper
        assert!(DetectionInterval::try_new(1.0, 3.0, 1.0).is_none()); // confidence = 1.0
        assert!(DetectionInterval::try_new(1.0, 3.0, 0.0).is_none()); // confidence = 0.0
    }

    #[test]
    fn test_detection_outcome() {
        assert!(DetectionOutcome::Detected.is_detected());
        assert!(!DetectionOutcome::NotDetected.is_detected());
        assert!(DetectionOutcome::Detected.is_definitive());
        assert!(!DetectionOutcome::Indeterminate.is_definitive());
    }

    #[test]
    fn test_signal_strength_level() {
        assert_eq!(
            SignalStrengthLevel::from_ratio(0.5),
            SignalStrengthLevel::None
        );
        assert_eq!(
            SignalStrengthLevel::from_ratio(1.5),
            SignalStrengthLevel::Weak
        );
        assert_eq!(
            SignalStrengthLevel::from_ratio(3.0),
            SignalStrengthLevel::Moderate
        );
        assert_eq!(
            SignalStrengthLevel::from_ratio(6.0),
            SignalStrengthLevel::Strong
        );
        assert_eq!(
            SignalStrengthLevel::from_ratio(10.0),
            SignalStrengthLevel::Critical
        );
        assert_eq!(SignalStrengthLevel::all().len(), 5);
    }

    #[test]
    fn test_signal_verification_report() {
        let report =
            SignalVerificationReport::new(DetectionOutcome::Detected, SignalStrengthLevel::Strong)
                .with_ratio(Ratio(5.0))
                .with_interval(
                    DetectionInterval::try_new(3.0, 8.0, 0.95).unwrap_or_else(|| {
                        DetectionInterval {
                            lower: 0.0,
                            upper: 1.0,
                            confidence_level: 0.95,
                        }
                    }),
                );
        assert!(report.outcome.is_detected());
        assert!(report.ratio.is_some());
        assert!(report.interval.is_some());
    }
}
