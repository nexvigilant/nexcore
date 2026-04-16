//! Shared anomaly-assessment primitives.
//!
//! Ports the anomaly logic that Python's `sensing/base.py::Sensor._assess_anomaly`
//! duplicates across every sensor. Pulling it into a single assessor keeps the
//! sensor implementations small and testable.

use serde::{Deserialize, Serialize};

/// Whether a reading is anomalous, how severe, and how confident the assessment.
///
/// Maps to the `(is_anomalous, severity, confidence)` tuple returned by
/// Python's `Sensor._assess_anomaly`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct AnomalyAssessment {
    /// Whether this reading is classified as anomalous.
    pub is_anomalous: bool,
    /// Severity of the anomaly, `0.0..=1.0`.
    pub severity: f64,
    /// Confidence in the assessment, `0.0..=1.0`.
    pub confidence: f64,
}

impl AnomalyAssessment {
    /// A "normal" assessment — not anomalous, zero severity, zero confidence.
    pub const NORMAL: Self = Self {
        is_anomalous: false,
        severity: 0.0,
        confidence: 0.0,
    };

    /// Create an anomalous assessment, clamping severity and confidence.
    pub fn anomalous(severity: f64, confidence: f64) -> Self {
        Self {
            is_anomalous: true,
            severity: severity.clamp(0.0, 1.0),
            confidence: confidence.clamp(0.0, 1.0),
        }
    }
}

/// Threshold-based anomaly assessor — the default implementation in Python's
/// `Sensor._assess_anomaly`.
///
/// Assessment rules (checked in order):
/// 1. `value >= critical_threshold` → anomalous, severity 1.0, confidence 0.95
/// 2. `value >= warning_threshold` → anomalous, severity interpolated between
///    warning and critical (0.5 if no critical), confidence 0.80
/// 3. `value > 5× baseline` → anomalous, severity 0.7, confidence 0.60
/// 4. `value > 2× baseline` → anomalous, severity 0.3, confidence 0.50
/// 5. Otherwise normal.
#[derive(Clone, Copy, Debug)]
pub struct AnomalyAssessor {
    /// Healthy baseline value. Used for ratio-based detection when thresholds
    /// are unavailable.
    pub baseline: f64,
    /// First-tier threshold; readings at or above trigger a warning.
    pub warning_threshold: Option<f64>,
    /// Second-tier threshold; readings at or above trigger critical alert.
    pub critical_threshold: Option<f64>,
}

impl AnomalyAssessor {
    /// Create a new assessor. All thresholds and baselines are optional.
    pub fn new(
        baseline: f64,
        warning_threshold: Option<f64>,
        critical_threshold: Option<f64>,
    ) -> Self {
        Self {
            baseline,
            warning_threshold,
            critical_threshold,
        }
    }

    /// Convenience constructor for the common case of both thresholds set.
    pub fn with_thresholds(baseline: f64, warning: f64, critical: f64) -> Self {
        Self {
            baseline,
            warning_threshold: Some(warning),
            critical_threshold: Some(critical),
        }
    }

    /// Assess a value against the configured thresholds.
    pub fn assess(&self, value: f64) -> AnomalyAssessment {
        // Rule 1: critical threshold
        if let Some(critical) = self.critical_threshold {
            if value >= critical {
                return AnomalyAssessment::anomalous(1.0, 0.95);
            }
        }

        // Rule 2: warning threshold
        if let Some(warning) = self.warning_threshold {
            if value >= warning {
                let severity = if let Some(critical) = self.critical_threshold {
                    let span = critical - warning;
                    if span > 0.0 {
                        ((value - warning) / span).min(1.0)
                    } else {
                        0.5
                    }
                } else {
                    0.5
                };
                return AnomalyAssessment::anomalous(severity, 0.80);
            }
        }

        // Rules 3/4: ratio against baseline
        if self.baseline > 0.0 {
            let ratio = value / self.baseline;
            if ratio > 5.0 {
                return AnomalyAssessment::anomalous(0.7, 0.60);
            }
            if ratio > 2.0 {
                return AnomalyAssessment::anomalous(0.3, 0.50);
            }
        }

        AnomalyAssessment::NORMAL
    }
}

/// Direction of change over a recent window of readings.
///
/// Maps to the `"increasing" | "stable" | "decreasing"` string returned by
/// Python's `Sensor.get_trend`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendDirection {
    /// Values trending up over the recent window.
    Increasing,
    /// Values unchanged (within 10% of first-half average) over the window.
    Stable,
    /// Values trending down over the window.
    Decreasing,
}

impl TrendDirection {
    /// Determine the trend from a slice of recent values using the split-mean
    /// algorithm from Python's `Sensor.get_trend`.
    ///
    /// The slice is split in half; the difference between the two halves'
    /// averages is compared against a 10%-of-first-half threshold (minimum 0.01).
    pub fn from_values(values: &[f64]) -> Self {
        if values.len() < 2 {
            return Self::Stable;
        }
        let mid = values.len() / 2;
        let (first, second) = values.split_at(mid);
        let avg = |slice: &[f64]| -> f64 {
            if slice.is_empty() {
                0.0
            } else {
                slice.iter().sum::<f64>() / slice.len() as f64
            }
        };
        let avg_first = avg(first);
        let avg_second = avg(second);

        let threshold = if avg_first.abs() > f64::EPSILON {
            avg_first.abs() * 0.1
        } else {
            0.01
        };

        if avg_second > avg_first + threshold {
            Self::Increasing
        } else if avg_second < avg_first - threshold {
            Self::Decreasing
        } else {
            Self::Stable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_is_not_anomalous() {
        assert!(!AnomalyAssessment::NORMAL.is_anomalous);
        assert_eq!(AnomalyAssessment::NORMAL.severity, 0.0);
    }

    #[test]
    fn anomalous_clamps_to_unit_interval() {
        let a = AnomalyAssessment::anomalous(5.0, -2.0);
        assert_eq!(a.severity, 1.0);
        assert_eq!(a.confidence, 0.0);
    }

    #[test]
    fn assessor_fires_on_critical() {
        let a = AnomalyAssessor::with_thresholds(1.0, 5.0, 10.0);
        let r = a.assess(12.0);
        assert!(r.is_anomalous);
        assert_eq!(r.severity, 1.0);
    }

    #[test]
    fn assessor_interpolates_warning_to_critical() {
        let a = AnomalyAssessor::with_thresholds(1.0, 5.0, 10.0);
        let r = a.assess(7.5);
        assert!(r.is_anomalous);
        assert!(
            (r.severity - 0.5).abs() < f64::EPSILON,
            "severity should be 0.5 halfway, got {}",
            r.severity
        );
    }

    #[test]
    fn assessor_uses_warning_only_when_no_critical() {
        let a = AnomalyAssessor::new(1.0, Some(5.0), None);
        let r = a.assess(6.0);
        assert!(r.is_anomalous);
        assert_eq!(r.severity, 0.5);
    }

    #[test]
    fn assessor_falls_back_to_baseline_ratio() {
        let a = AnomalyAssessor::new(1.0, None, None);
        let r = a.assess(6.0); // 6× baseline
        assert!(r.is_anomalous);
        assert_eq!(r.severity, 0.7);
    }

    #[test]
    fn assessor_normal_below_all_thresholds() {
        let a = AnomalyAssessor::with_thresholds(1.0, 5.0, 10.0);
        let r = a.assess(0.5);
        assert!(!r.is_anomalous);
    }

    #[test]
    fn trend_stable_with_one_sample() {
        assert_eq!(TrendDirection::from_values(&[1.0]), TrendDirection::Stable);
    }

    #[test]
    fn trend_increasing() {
        let values = [1.0, 1.0, 1.0, 2.0, 3.0, 4.0];
        assert_eq!(
            TrendDirection::from_values(&values),
            TrendDirection::Increasing
        );
    }

    #[test]
    fn trend_decreasing() {
        let values = [10.0, 9.0, 8.0, 3.0, 2.0, 1.0];
        assert_eq!(
            TrendDirection::from_values(&values),
            TrendDirection::Decreasing
        );
    }

    #[test]
    fn trend_stable_within_tolerance() {
        let values = [10.0, 10.1, 10.0, 10.05, 10.02, 10.01];
        assert_eq!(TrendDirection::from_values(&values), TrendDirection::Stable);
    }
}
