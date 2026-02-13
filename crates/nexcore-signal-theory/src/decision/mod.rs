// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Signal Detection Theory (SDT) Decision Module
//!
//! The classical 2x2 decision matrix and derived metrics.
//!
//! ## The Four Outcomes
//!
//! ```text
//!                    Signal Present    Signal Absent
//! Decision: Yes      HIT               FALSE ALARM
//! Decision: No       MISS              CORRECT REJECTION
//! ```
//!
//! ## Derived Metrics
//!
//! | Metric | Formula | Meaning |
//! |--------|---------|---------|
//! | Sensitivity | hits / (hits + misses) | True positive rate |
//! | Specificity | CR / (CR + FA) | True negative rate |
//! | PPV | hits / (hits + FA) | Positive predictive value |
//! | NPV | CR / (CR + misses) | Negative predictive value |
//! | d' | z(sensitivity) - z(specificity) | Discriminability |

use alloc::vec::Vec;

// ═══════════════════════════════════════════════════════════
// DECISION OUTCOME
// ═══════════════════════════════════════════════════════════

/// The four possible outcomes in signal detection theory.
///
/// ## Tier: T1 (κ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DecisionOutcome {
    /// Signal present AND correctly detected.
    Hit,
    /// Signal present BUT not detected.
    Miss,
    /// Signal absent BUT incorrectly detected.
    FalseAlarm,
    /// Signal absent AND correctly rejected.
    CorrectRejection,
}

impl DecisionOutcome {
    /// Whether this is a correct decision.
    #[must_use]
    pub const fn is_correct(&self) -> bool {
        matches!(self, Self::Hit | Self::CorrectRejection)
    }

    /// Whether signal was actually present.
    #[must_use]
    pub const fn signal_present(&self) -> bool {
        matches!(self, Self::Hit | Self::Miss)
    }

    /// Whether the decision was "yes" (detected).
    #[must_use]
    pub const fn decision_positive(&self) -> bool {
        matches!(self, Self::Hit | Self::FalseAlarm)
    }

    /// All outcomes.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [
            Self::Hit,
            Self::Miss,
            Self::FalseAlarm,
            Self::CorrectRejection,
        ]
    }
}

// ═══════════════════════════════════════════════════════════
// DECISION MATRIX (2x2)
// ═══════════════════════════════════════════════════════════

/// The 2x2 decision matrix from Signal Detection Theory.
///
/// ## Tier: T2-C (κ + N + ∂ + Σ)
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DecisionMatrix {
    /// True positives (signal present, detected).
    pub hits: u64,
    /// False negatives (signal present, not detected).
    pub misses: u64,
    /// False positives (signal absent, detected).
    pub false_alarms: u64,
    /// True negatives (signal absent, not detected).
    pub correct_rejections: u64,
}

impl DecisionMatrix {
    /// Create a new decision matrix.
    #[must_use]
    pub const fn new(hits: u64, misses: u64, false_alarms: u64, correct_rejections: u64) -> Self {
        Self {
            hits,
            misses,
            false_alarms,
            correct_rejections,
        }
    }

    /// Total number of observations.
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.hits + self.misses + self.false_alarms + self.correct_rejections
    }

    /// Number of signal-present cases.
    #[must_use]
    pub const fn signal_present(&self) -> u64 {
        self.hits + self.misses
    }

    /// Number of signal-absent cases.
    #[must_use]
    pub const fn signal_absent(&self) -> u64 {
        self.false_alarms + self.correct_rejections
    }

    /// Number of positive decisions.
    #[must_use]
    pub const fn positive_decisions(&self) -> u64 {
        self.hits + self.false_alarms
    }

    /// Number of negative decisions.
    #[must_use]
    pub const fn negative_decisions(&self) -> u64 {
        self.misses + self.correct_rejections
    }

    /// Sensitivity (true positive rate): hits / (hits + misses).
    #[must_use]
    pub fn sensitivity(&self) -> f64 {
        let denom = self.signal_present();
        if denom == 0 {
            return 0.0;
        }
        self.hits as f64 / denom as f64
    }

    /// Specificity (true negative rate): CR / (CR + FA).
    #[must_use]
    pub fn specificity(&self) -> f64 {
        let denom = self.signal_absent();
        if denom == 0 {
            return 0.0;
        }
        self.correct_rejections as f64 / denom as f64
    }

    /// Positive predictive value: hits / (hits + FA).
    #[must_use]
    pub fn ppv(&self) -> f64 {
        let denom = self.positive_decisions();
        if denom == 0 {
            return 0.0;
        }
        self.hits as f64 / denom as f64
    }

    /// Negative predictive value: CR / (CR + misses).
    #[must_use]
    pub fn npv(&self) -> f64 {
        let denom = self.negative_decisions();
        if denom == 0 {
            return 0.0;
        }
        self.correct_rejections as f64 / denom as f64
    }

    /// Accuracy: (hits + CR) / total.
    #[must_use]
    pub fn accuracy(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 0.0;
        }
        (self.hits + self.correct_rejections) as f64 / total as f64
    }

    /// False positive rate: FA / (FA + CR).
    #[must_use]
    pub fn false_positive_rate(&self) -> f64 {
        1.0 - self.specificity()
    }

    /// False negative rate: misses / (hits + misses).
    #[must_use]
    pub fn false_negative_rate(&self) -> f64 {
        1.0 - self.sensitivity()
    }

    /// Prevalence: signal_present / total.
    #[must_use]
    pub fn prevalence(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 0.0;
        }
        self.signal_present() as f64 / total as f64
    }

    /// F1 score: 2 * (PPV * Sensitivity) / (PPV + Sensitivity).
    #[must_use]
    pub fn f1_score(&self) -> f64 {
        let ppv = self.ppv();
        let sens = self.sensitivity();
        let denom = ppv + sens;
        if denom == 0.0 {
            return 0.0;
        }
        2.0 * ppv * sens / denom
    }

    /// Matthews Correlation Coefficient.
    #[must_use]
    pub fn mcc(&self) -> f64 {
        let tp = self.hits as f64;
        let tn = self.correct_rejections as f64;
        let fp = self.false_alarms as f64;
        let f_n = self.misses as f64;

        let numerator = tp * tn - fp * f_n;
        let denominator = ((tp + fp) * (tp + f_n) * (tn + fp) * (tn + f_n)).sqrt();

        if denominator == 0.0 {
            return 0.0;
        }
        numerator / denominator
    }
}

// ═══════════════════════════════════════════════════════════
// ROC CURVE
// ═══════════════════════════════════════════════════════════

/// A single point on an ROC curve.
///
/// ## Tier: T2-P (κ + N)
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RocPoint {
    /// False positive rate (x-axis).
    pub fpr: f64,
    /// True positive rate / sensitivity (y-axis).
    pub tpr: f64,
    /// The threshold that produced this point.
    pub threshold: f64,
}

impl RocPoint {
    /// Create a new ROC point.
    #[must_use]
    pub fn new(fpr: f64, tpr: f64, threshold: f64) -> Self {
        Self {
            fpr,
            tpr,
            threshold,
        }
    }

    /// Distance from the ideal point (0, 1).
    #[must_use]
    pub fn distance_to_ideal(&self) -> f64 {
        (self.fpr * self.fpr + (1.0 - self.tpr) * (1.0 - self.tpr)).sqrt()
    }

    /// Youden's J statistic (sensitivity + specificity - 1).
    #[must_use]
    pub fn youden_j(&self) -> f64 {
        self.tpr - self.fpr
    }
}

/// An ROC curve composed of multiple points.
///
/// ## Tier: T2-C (κ + N + σ + ∂)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RocCurve {
    /// Points on the curve, sorted by threshold (ascending FPR).
    pub points: Vec<RocPoint>,
}

impl RocCurve {
    /// Create a new ROC curve from points.
    #[must_use]
    pub fn new(mut points: Vec<RocPoint>) -> Self {
        points.sort_by(|a, b| {
            a.fpr
                .partial_cmp(&b.fpr)
                .unwrap_or(core::cmp::Ordering::Equal)
        });
        Self { points }
    }

    /// Area Under the Curve (trapezoidal rule).
    #[must_use]
    pub fn auc(&self) -> f64 {
        if self.points.len() < 2 {
            return 0.0;
        }
        let mut area = 0.0;
        for i in 1..self.points.len() {
            let dx = self.points[i].fpr - self.points[i - 1].fpr;
            let avg_y = (self.points[i].tpr + self.points[i - 1].tpr) / 2.0;
            area += dx * avg_y;
        }
        area
    }

    /// Find the optimal threshold (maximizing Youden's J).
    #[must_use]
    pub fn optimal_threshold(&self) -> Option<f64> {
        self.points
            .iter()
            .max_by(|a, b| {
                a.youden_j()
                    .partial_cmp(&b.youden_j())
                    .unwrap_or(core::cmp::Ordering::Equal)
            })
            .map(|p| p.threshold)
    }

    /// Number of points on the curve.
    #[must_use]
    pub fn point_count(&self) -> usize {
        self.points.len()
    }
}

// ═══════════════════════════════════════════════════════════
// D-PRIME (DISCRIMINABILITY)
// ═══════════════════════════════════════════════════════════

/// D-prime (d'): a measure of signal discriminability.
///
/// d' = z(hit_rate) - z(false_alarm_rate)
/// where z is the inverse normal CDF.
///
/// ## Tier: T2-P (κ + N)
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DPrime(pub f64);

impl DPrime {
    /// Compute d' from hit rate and false alarm rate.
    ///
    /// Uses a rational approximation to the inverse normal CDF.
    #[must_use]
    pub fn from_rates(hit_rate: f64, false_alarm_rate: f64) -> Self {
        let z_hit = probit_approx(hit_rate);
        let z_fa = probit_approx(false_alarm_rate);
        Self(z_hit - z_fa)
    }

    /// Compute from a decision matrix.
    #[must_use]
    pub fn from_matrix(matrix: &DecisionMatrix) -> Self {
        Self::from_rates(matrix.sensitivity(), matrix.false_positive_rate())
    }

    /// Whether the detector can discriminate at all (d' > 0).
    #[must_use]
    pub fn is_discriminable(&self) -> bool {
        self.0 > 0.0
    }

    /// Qualitative discrimination level.
    #[must_use]
    pub fn level(&self) -> &'static str {
        if self.0 < 0.5 {
            "poor"
        } else if self.0 < 1.0 {
            "fair"
        } else if self.0 < 2.0 {
            "good"
        } else if self.0 < 3.0 {
            "excellent"
        } else {
            "near-perfect"
        }
    }
}

// ═══════════════════════════════════════════════════════════
// RESPONSE BIAS
// ═══════════════════════════════════════════════════════════

/// Response bias (c): the tendency to say "yes" or "no".
///
/// c = -0.5 * (z(hit_rate) + z(false_alarm_rate))
/// - Negative c → liberal (say "yes" more often)
/// - Positive c → conservative (say "no" more often)
/// - Zero → no bias
///
/// ## Tier: T2-P (κ + ∂)
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ResponseBias(pub f64);

impl ResponseBias {
    /// Compute response bias from hit rate and false alarm rate.
    #[must_use]
    pub fn from_rates(hit_rate: f64, false_alarm_rate: f64) -> Self {
        let z_hit = probit_approx(hit_rate);
        let z_fa = probit_approx(false_alarm_rate);
        Self(-0.5 * (z_hit + z_fa))
    }

    /// Compute from a decision matrix.
    #[must_use]
    pub fn from_matrix(matrix: &DecisionMatrix) -> Self {
        Self::from_rates(matrix.sensitivity(), matrix.false_positive_rate())
    }

    /// Whether the bias is liberal (tends to say "yes").
    #[must_use]
    pub fn is_liberal(&self) -> bool {
        self.0 < 0.0
    }

    /// Whether the bias is conservative (tends to say "no").
    #[must_use]
    pub fn is_conservative(&self) -> bool {
        self.0 > 0.0
    }

    /// Qualitative bias description.
    #[must_use]
    pub fn description(&self) -> &'static str {
        if self.0 < -0.5 {
            "strongly liberal"
        } else if self.0 < -0.1 {
            "liberal"
        } else if self.0 <= 0.1 {
            "neutral"
        } else if self.0 <= 0.5 {
            "conservative"
        } else {
            "strongly conservative"
        }
    }
}

// ═══════════════════════════════════════════════════════════
// PROBIT APPROXIMATION
// ═══════════════════════════════════════════════════════════

/// Approximate inverse normal CDF (probit function).
///
/// Uses rational approximation (Abramowitz and Stegun 26.2.23).
/// Accurate to ~4.5e-4 for 0.0 < p < 1.0.
fn probit_approx(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 {
        // Clamp to avoid infinity
        if p <= 0.0 {
            return -3.5;
        }
        return 3.5;
    }

    let sign;
    let q;
    if p < 0.5 {
        sign = -1.0;
        q = p;
    } else {
        sign = 1.0;
        q = 1.0 - p;
    };

    let t = (-2.0 * q.ln()).sqrt();

    // Rational approximation constants
    let c0 = 2.515517;
    let c1 = 0.802853;
    let c2 = 0.010328;
    let d1 = 1.432788;
    let d2 = 0.189269;
    let d3 = 0.001308;

    let numerator = c0 + c1 * t + c2 * t * t;
    let denominator = 1.0 + d1 * t + d2 * t * t + d3 * t * t * t;

    sign * (t - numerator / denominator)
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_outcome() {
        assert!(DecisionOutcome::Hit.is_correct());
        assert!(!DecisionOutcome::Miss.is_correct());
        assert!(DecisionOutcome::FalseAlarm.decision_positive());
        assert!(!DecisionOutcome::CorrectRejection.decision_positive());
        assert_eq!(DecisionOutcome::all().len(), 4);
    }

    #[test]
    fn test_decision_matrix_metrics() {
        let m = DecisionMatrix::new(80, 20, 10, 90);
        assert_eq!(m.total(), 200);
        assert_eq!(m.signal_present(), 100);
        assert_eq!(m.signal_absent(), 100);

        // Sensitivity = 80 / 100 = 0.8
        assert!((m.sensitivity() - 0.8).abs() < f64::EPSILON);
        // Specificity = 90 / 100 = 0.9
        assert!((m.specificity() - 0.9).abs() < f64::EPSILON);
        // PPV = 80 / 90
        assert!((m.ppv() - 80.0 / 90.0).abs() < 1e-10);
        // NPV = 90 / 110
        assert!((m.npv() - 90.0 / 110.0).abs() < 1e-10);
        // Accuracy = 170 / 200 = 0.85
        assert!((m.accuracy() - 0.85).abs() < f64::EPSILON);
        // Prevalence = 100/200 = 0.5
        assert!((m.prevalence() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_decision_matrix_empty() {
        let m = DecisionMatrix::new(0, 0, 0, 0);
        assert!((m.sensitivity() - 0.0).abs() < f64::EPSILON);
        assert!((m.accuracy() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_decision_matrix_f1() {
        let m = DecisionMatrix::new(80, 20, 10, 90);
        let f1 = m.f1_score();
        // F1 = 2 * PPV * Sens / (PPV + Sens)
        let ppv = 80.0 / 90.0;
        let sens = 0.8;
        let expected = 2.0 * ppv * sens / (ppv + sens);
        assert!((f1 - expected).abs() < 1e-10);
    }

    #[test]
    fn test_decision_matrix_mcc() {
        let m = DecisionMatrix::new(80, 20, 10, 90);
        let mcc = m.mcc();
        // MCC should be positive for this good classifier
        assert!(mcc > 0.5);
    }

    #[test]
    fn test_roc_point() {
        let p = RocPoint::new(0.1, 0.8, 2.0);
        assert!(p.distance_to_ideal() < 0.3);
        assert!((p.youden_j() - 0.7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_roc_curve_auc() {
        let curve = RocCurve::new(alloc::vec![
            RocPoint::new(0.0, 0.0, 10.0),
            RocPoint::new(0.1, 0.8, 3.0),
            RocPoint::new(0.5, 0.9, 2.0),
            RocPoint::new(1.0, 1.0, 0.0),
        ]);

        let auc = curve.auc();
        // AUC should be > 0.5 (better than chance)
        assert!(auc > 0.5);
        // AUC should be < 1.0 (not perfect)
        assert!(auc < 1.0);
    }

    #[test]
    fn test_roc_optimal_threshold() {
        let curve = RocCurve::new(alloc::vec![
            RocPoint::new(0.0, 0.0, 10.0),
            RocPoint::new(0.1, 0.8, 3.0),
            RocPoint::new(0.5, 0.9, 2.0),
            RocPoint::new(1.0, 1.0, 0.0),
        ]);

        let optimal = curve.optimal_threshold();
        assert!(optimal.is_some());
        // The point with max Youden's J is (0.1, 0.8) → threshold 3.0
        let opt = optimal.unwrap_or(0.0);
        assert!((opt - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_dprime() {
        let m = DecisionMatrix::new(80, 20, 10, 90);
        let dprime = DPrime::from_matrix(&m);
        assert!(dprime.is_discriminable());
        assert!(dprime.0 > 1.0); // Good discriminability
    }

    #[test]
    fn test_dprime_levels() {
        assert_eq!(DPrime(0.3).level(), "poor");
        assert_eq!(DPrime(0.7).level(), "fair");
        assert_eq!(DPrime(1.5).level(), "good");
        assert_eq!(DPrime(2.5).level(), "excellent");
        assert_eq!(DPrime(3.5).level(), "near-perfect");
    }

    #[test]
    fn test_response_bias() {
        // Liberal detector (high hit rate, high false alarm rate)
        let liberal = ResponseBias::from_rates(0.9, 0.5);
        assert!(liberal.is_liberal());

        // Conservative detector (low hit rate, low false alarm rate)
        let conservative = ResponseBias::from_rates(0.5, 0.1);
        assert!(conservative.is_conservative());
    }

    #[test]
    fn test_response_bias_description() {
        assert_eq!(ResponseBias(-1.0).description(), "strongly liberal");
        assert_eq!(ResponseBias(-0.3).description(), "liberal");
        assert_eq!(ResponseBias(0.0).description(), "neutral");
        assert_eq!(ResponseBias(0.3).description(), "conservative");
        assert_eq!(ResponseBias(1.0).description(), "strongly conservative");
    }

    #[test]
    fn test_probit_approx_midpoint() {
        // probit(0.5) should be approximately 0
        let z = probit_approx(0.5);
        assert!(z.abs() < 0.01);
    }

    #[test]
    fn test_probit_approx_bounds() {
        // probit(0.975) should be approximately 1.96
        let z = probit_approx(0.975);
        assert!((z - 1.96).abs() < 0.05);
    }
}
