//! CRO performance scoring — composite metrics and tier classification.
//!
//! After each completed order, CRO performance is evaluated across multiple
//! dimensions. The composite score determines the provider's tier, which
//! affects search ranking and marketplace visibility.

use serde::{Deserialize, Serialize};

/// Performance metrics for a CRO provider.
///
/// All score fields (except `delivery_time_days` and `quoted_vs_actual_days`)
/// are normalized to the range `[0.0, 1.0]` where 1.0 is best.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CroPerformanceMetrics {
    /// Actual delivery time in business days.
    pub delivery_time_days: f64,
    /// Ratio of actual delivery days to quoted days (1.0 = on time, <1.0 = early, >1.0 = late).
    pub quoted_vs_actual_days: f64,
    /// Data quality score (0.0 worst, 1.0 best).
    pub data_quality_score: f64,
    /// QC pass rate across orders (0.0 = all fail, 1.0 = all pass).
    pub qc_pass_rate: f64,
    /// Cost efficiency: how competitive pricing is vs. market average
    /// (0.0 = most expensive, 1.0 = most competitive).
    pub cost_efficiency: f64,
    /// Communication responsiveness and clarity (0.0 worst, 1.0 best).
    pub communication_score: f64,
}

/// Calculate a composite performance score from CRO metrics.
///
/// Weighted average:
/// - Delivery timeliness: 25% (derived from `quoted_vs_actual_days`)
/// - Data quality: 30%
/// - QC pass rate: 20%
/// - Cost efficiency: 15%
/// - Communication: 10%
///
/// The delivery component converts `quoted_vs_actual_days` into a 0-1 score:
/// - ratio <= 1.0 (on time or early) → 1.0
/// - ratio > 1.0 → linearly decays, reaching 0.0 at ratio 2.0 (100% late)
///
/// Returns a value clamped to `[0.0, 1.0]`.
#[must_use]
pub fn calculate_composite_score(metrics: &CroPerformanceMetrics) -> f64 {
    // Convert delivery ratio to a 0-1 score.
    let delivery_score = if metrics.quoted_vs_actual_days <= 1.0 {
        1.0
    } else {
        // Linear decay from 1.0 at ratio=1.0 to 0.0 at ratio=2.0
        (2.0 - metrics.quoted_vs_actual_days).clamp(0.0, 1.0)
    };

    let weighted = delivery_score * 0.25
        + metrics.data_quality_score.clamp(0.0, 1.0) * 0.30
        + metrics.qc_pass_rate.clamp(0.0, 1.0) * 0.20
        + metrics.cost_efficiency.clamp(0.0, 1.0) * 0.15
        + metrics.communication_score.clamp(0.0, 1.0) * 0.10;

    weighted.clamp(0.0, 1.0)
}

/// Classify a composite score into a named performance tier.
///
/// | Score Range     | Tier              |
/// |-----------------|-------------------|
/// | < 0.5           | needs_improvement |
/// | 0.5 – 0.7       | satisfactory      |
/// | 0.7 – 0.85      | good              |
/// | 0.85 – 0.95     | excellent         |
/// | >= 0.95          | premier           |
#[must_use]
pub fn performance_tier(score: f64) -> &'static str {
    if score < 0.5 {
        "needs_improvement"
    } else if score < 0.7 {
        "satisfactory"
    } else if score < 0.85 {
        "good"
    } else if score < 0.95 {
        "excellent"
    } else {
        "premier"
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn perfect_metrics() -> CroPerformanceMetrics {
        CroPerformanceMetrics {
            delivery_time_days: 10.0,
            quoted_vs_actual_days: 0.8, // early
            data_quality_score: 1.0,
            qc_pass_rate: 1.0,
            cost_efficiency: 1.0,
            communication_score: 1.0,
        }
    }

    #[test]
    fn perfect_score_is_1_0() {
        let score = calculate_composite_score(&perfect_metrics());
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn perfect_score_is_premier() {
        let score = calculate_composite_score(&perfect_metrics());
        assert_eq!(performance_tier(score), "premier");
    }

    #[test]
    fn all_zeros_is_zero() {
        let metrics = CroPerformanceMetrics {
            delivery_time_days: 100.0,
            quoted_vs_actual_days: 3.0, // very late → 0.0
            data_quality_score: 0.0,
            qc_pass_rate: 0.0,
            cost_efficiency: 0.0,
            communication_score: 0.0,
        };
        let score = calculate_composite_score(&metrics);
        assert!(score.abs() < f64::EPSILON);
        assert_eq!(performance_tier(score), "needs_improvement");
    }

    #[test]
    fn on_time_delivery_scores_1() {
        let mut metrics = perfect_metrics();
        metrics.quoted_vs_actual_days = 1.0; // exactly on time
        let score = calculate_composite_score(&metrics);
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn late_delivery_reduces_score() {
        let mut metrics = perfect_metrics();
        metrics.quoted_vs_actual_days = 1.5; // 50% late → delivery score 0.5
        let score = calculate_composite_score(&metrics);
        // delivery: 0.5 * 0.25 = 0.125
        // data quality: 1.0 * 0.30 = 0.30
        // qc pass: 1.0 * 0.20 = 0.20
        // cost: 1.0 * 0.15 = 0.15
        // comm: 1.0 * 0.10 = 0.10
        // total = 0.875
        let expected = 0.875;
        assert!(
            (score - expected).abs() < 1e-10,
            "expected {expected}, got {score}"
        );
        assert_eq!(performance_tier(score), "excellent");
    }

    #[test]
    fn very_late_delivery_floors_at_zero() {
        let mut metrics = perfect_metrics();
        metrics.quoted_vs_actual_days = 2.5; // 150% late → delivery score 0.0
        let score = calculate_composite_score(&metrics);
        // delivery: 0.0 * 0.25 = 0.0
        // rest: 0.30 + 0.20 + 0.15 + 0.10 = 0.75
        let expected = 0.75;
        assert!(
            (score - expected).abs() < 1e-10,
            "expected {expected}, got {score}"
        );
        assert_eq!(performance_tier(score), "good");
    }

    #[test]
    fn mixed_metrics_specific_calculation() {
        let metrics = CroPerformanceMetrics {
            delivery_time_days: 14.0,
            quoted_vs_actual_days: 1.2, // 20% late → delivery score 0.8
            data_quality_score: 0.9,
            qc_pass_rate: 0.85,
            cost_efficiency: 0.7,
            communication_score: 0.6,
        };
        let score = calculate_composite_score(&metrics);
        // delivery: 0.8 * 0.25 = 0.200
        // data quality: 0.9 * 0.30 = 0.270
        // qc pass: 0.85 * 0.20 = 0.170
        // cost: 0.7 * 0.15 = 0.105
        // comm: 0.6 * 0.10 = 0.060
        // total = 0.805
        let expected = 0.805;
        assert!(
            (score - expected).abs() < 1e-10,
            "expected {expected}, got {score}"
        );
        assert_eq!(performance_tier(score), "good");
    }

    #[test]
    fn tier_boundaries() {
        assert_eq!(performance_tier(0.0), "needs_improvement");
        assert_eq!(performance_tier(0.49), "needs_improvement");
        assert_eq!(performance_tier(0.5), "satisfactory");
        assert_eq!(performance_tier(0.69), "satisfactory");
        assert_eq!(performance_tier(0.7), "good");
        assert_eq!(performance_tier(0.84), "good");
        assert_eq!(performance_tier(0.85), "excellent");
        assert_eq!(performance_tier(0.94), "excellent");
        assert_eq!(performance_tier(0.95), "premier");
        assert_eq!(performance_tier(1.0), "premier");
    }

    #[test]
    fn metrics_serialization_roundtrip() {
        let metrics = perfect_metrics();
        let json = serde_json::to_string(&metrics).unwrap();
        let back: CroPerformanceMetrics = serde_json::from_str(&json).unwrap();
        assert!((back.data_quality_score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn score_clamped_even_with_out_of_range_inputs() {
        let metrics = CroPerformanceMetrics {
            delivery_time_days: 5.0,
            quoted_vs_actual_days: 0.5, // early → 1.0
            data_quality_score: 1.5,    // over 1.0 → clamped to 1.0
            qc_pass_rate: -0.1,         // negative → clamped to 0.0
            cost_efficiency: 1.0,
            communication_score: 1.0,
        };
        let score = calculate_composite_score(&metrics);
        // delivery: 1.0 * 0.25 = 0.25
        // data quality: 1.0 * 0.30 = 0.30 (clamped)
        // qc pass: 0.0 * 0.20 = 0.00 (clamped)
        // cost: 1.0 * 0.15 = 0.15
        // comm: 1.0 * 0.10 = 0.10
        // total = 0.80
        let expected = 0.80;
        assert!(
            (score - expected).abs() < 1e-10,
            "expected {expected}, got {score}"
        );
    }
}
