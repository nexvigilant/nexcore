//! Non-compensatory risk scoring via multiplicative aggregation.
//!
//! Inspired by the ASDF v2.0 Sophistication Index construction methodology
//! (OECD composite indicator guidelines, geometric mean aggregation).
//!
//! # Core Principle
//!
//! Additive scoring allows a strong signal in one metric to compensate for
//! absence in another. This is dangerous in pharmacovigilance: a high PRR
//! should NOT mask missing EBGM or IC evidence.
//!
//! Multiplicative aggregation via geometric mean ensures:
//! - A zero in any dimension collapses the composite score
//! - All dimensions must contribute for a strong signal
//! - The composite naturally penalizes imbalance across evidence sources
//!
//! # Mathematical Foundation
//!
//! ```text
//! SI = ∏(I_d)^w_d    (weighted geometric mean)
//!
//! Where:
//!   I_d = normalized dimension score [0, 1] for each metric
//!   w_d = weight assigned to dimension d (sum to 1.0)
//!
//! Equivalently: SI = exp(Σ w_d × ln(I_d))
//! ```
//!
//! This mirrors the conservation law ∃ = ∂(×(ς, ∅)) — existence requires
//! all primitives to be non-zero. The geometric mean IS the conservation law
//! applied to measurement.
//!
//! # CALIBRATION: Thresholds from Evans et al. (2001) and standard PV practice
//! - PRR signal threshold: 2.0 (Evans criterion)
//! - ROR lower CI threshold: 1.0 (confidence interval excludes null)
//! - IC025 threshold: 0.0 (Bayesian shrinkage above null)
//! - EBGM05 threshold: 2.0 (empirical Bayes geometric mean)

use nexcore_primitives::measurement::Measured;
use nexcore_pv_core::thresholds::SignalCriteria;
use serde::{Deserialize, Serialize};

use crate::RiskContext;

// =============================================================================
// Dimension Scoring
// =============================================================================

/// A single dimension's contribution to the composite risk score.
///
/// Each dimension normalizes a raw metric value to [0, 1] using a sigmoid-like
/// mapping that crosses 0.5 at the threshold and saturates toward 1.0 at high
/// signal strength.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    /// Metric name (e.g., "PRR", "ROR_lower", "IC025", "EBGM05")
    pub name: String,
    /// Raw metric value before normalization
    pub raw_value: f64,
    /// Signal detection threshold for this metric
    pub threshold: f64,
    /// Normalized score in [0, 1]
    pub normalized: f64,
    /// Weight for geometric aggregation (must sum to 1.0 across all dimensions)
    pub weight: f64,
    /// Whether the raw value exceeds the signal threshold
    pub signal_detected: bool,
}

/// Default dimension weights — Bayesian-heavy (IC + EBGM weighted 0.35 each).
///
/// Simulation across 8 PV signal profiles × 5 weight configurations showed
/// Bayesian-heavy provides the best signal-to-noise discrimination (76.0 vs
/// 73.1 for equal weights). IC and EBGM use shrinkage estimators that are
/// inherently more conservative than raw frequentist PRR/ROR — weighting
/// them higher rewards evidence that survived Bayesian regularization.
///
/// # CALIBRATION: Derived from guardian-weight-sim.ipynb (2026-03-28)
/// - Equal:          discrimination = 73.1
/// - PRR-heavy:      discrimination = 69.9 (worst — PRR is inflation-prone)
/// - Bayesian-heavy: discrimination = 76.0 (best)
/// - ROR-anchored:   discrimination = 72.4
///
/// Order: [PRR, ROR_lower, IC025, EBGM05]
pub const DEFAULT_WEIGHTS: [f64; 4] = [0.15, 0.15, 0.35, 0.35];

/// Equal weights — preserved for comparison and backward-compat testing.
pub const EQUAL_WEIGHTS: [f64; 4] = [0.25, 0.25, 0.25, 0.25];

/// Normalize a metric value to [0, 1] using a shifted logistic function.
///
/// The function crosses 0.5 at the threshold value and approaches 1.0
/// asymptotically. Below-threshold values map to (0, 0.5), above-threshold
/// to (0.5, 1.0). Values at or below zero map to a floor of 0.01 (not exactly
/// zero — true zero is reserved for missing/invalid data).
///
/// ```text
/// f(x) = 1 / (1 + exp(-k * (x - threshold)))
///
/// Where k controls steepness (higher = sharper transition at threshold)
/// ```
///
/// # CALIBRATION: k=2.0 gives a smooth transition
/// - At threshold: 0.5
/// - At 2× threshold: ~0.88
/// - At 3× threshold: ~0.98
/// - Below 0: floor at 0.01
fn normalize_metric(value: f64, threshold: f64, steepness: f64) -> f64 {
    if value.is_nan() || value.is_infinite() {
        return 0.0; // Invalid data = zero dimension = collapses composite
    }
    // Floor: values at or below zero get minimal score (not zero — zero means absent)
    if value <= 0.0 && threshold > 0.0 {
        return 0.01;
    }
    // Shifted logistic normalization
    let z = steepness * (value - threshold);
    1.0 / (1.0 + (-z).exp())
}

/// Normalize IC025 which has a different scale (threshold at 0.0, can be negative).
///
/// IC025 ranges from roughly -5 to +5. Positive values indicate signal.
/// The normalization maps: -5 → ~0, 0 → 0.5, +2 → ~0.88, +5 → ~0.99
fn normalize_ic025(value: f64) -> f64 {
    if value.is_nan() || value.is_infinite() {
        return 0.0;
    }
    // IC025 threshold is 0.0; steepness 1.0 works well for its range
    normalize_metric(value, 0.0, 1.0)
}

/// Score all four PV signal dimensions from a risk context.
///
/// Thresholds sourced from `SignalCriteria::evans()` — single source of truth.
/// Returns exactly 4 dimension scores in order: PRR, ROR, IC025, EBGM05.
#[must_use]
pub fn score_dimensions(ctx: &RiskContext, weights: &[f64; 4]) -> Vec<DimensionScore> {
    // CALIBRATION: steepness k=2.0 for PRR/ROR/EBGM (ratio scales),
    // k=1.0 for IC025 (additive scale)
    const K_RATIO: f64 = 2.0;
    let evans = SignalCriteria::evans();

    vec![
        DimensionScore {
            name: "PRR".into(),
            raw_value: ctx.prr,
            threshold: evans.prr_threshold,
            normalized: normalize_metric(ctx.prr, evans.prr_threshold, K_RATIO),
            weight: weights[0],
            signal_detected: ctx.prr >= evans.prr_threshold,
        },
        DimensionScore {
            name: "ROR_lower".into(),
            raw_value: ctx.ror_lower,
            threshold: evans.ror_lower_threshold,
            normalized: normalize_metric(ctx.ror_lower, evans.ror_lower_threshold, K_RATIO),
            weight: weights[1],
            signal_detected: ctx.ror_lower > evans.ror_lower_threshold,
        },
        DimensionScore {
            name: "IC025".into(),
            raw_value: ctx.ic025,
            threshold: evans.ic025_threshold,
            normalized: normalize_ic025(ctx.ic025),
            weight: weights[2],
            signal_detected: ctx.ic025 > evans.ic025_threshold,
        },
        DimensionScore {
            name: "EBGM05".into(),
            raw_value: ctx.eb05,
            threshold: evans.eb05_threshold,
            normalized: normalize_metric(ctx.eb05, evans.eb05_threshold, K_RATIO),
            weight: weights[3],
            signal_detected: ctx.eb05 >= evans.eb05_threshold,
        },
    ]
}

// =============================================================================
// Geometric Aggregation
// =============================================================================

/// Non-compensatory risk score computed via weighted geometric mean.
///
/// The composite score is the product of each dimension raised to its weight:
/// `SI = ∏(I_d)^w_d`
///
/// This ensures a near-zero score in any single dimension dramatically
/// reduces the composite, preventing one strong metric from masking weakness
/// in others.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonCompensatoryScore {
    /// Composite geometric mean score [0, 1]
    pub composite: Measured<f64>,
    /// Risk level derived from composite score
    pub level: String,
    /// Individual dimension scores
    pub dimensions: Vec<DimensionScore>,
    /// How many dimensions detected a signal (above threshold)
    pub signals_detected: u32,
    /// Case count weight factor (same as additive scorer)
    pub n_weight: f64,
    /// Explanatory factors for audit trail
    pub factors: Vec<String>,
    /// The aggregation mode used
    pub mode: String,
}

/// Compute the weighted geometric mean of dimension scores.
///
/// ```text
/// GM = exp(Σ w_i × ln(d_i))
/// ```
///
/// Where d_i is clamped to [ε, 1.0] to avoid ln(0) = -∞.
/// ε = 1e-10 (effectively zero but mathematically tractable).
fn weighted_geometric_mean(dimensions: &[DimensionScore]) -> f64 {
    const EPSILON: f64 = 1e-10;

    let log_sum: f64 = dimensions
        .iter()
        .map(|d| d.weight * d.normalized.max(EPSILON).ln())
        .sum();

    log_sum.exp()
}

/// Determine risk level from geometric composite score.
///
/// The thresholds are calibrated to the geometric mean's behavior:
/// - All 4 metrics above threshold → composite ~0.5+ → High/Critical
/// - 3 of 4 above → composite ~0.3-0.5 → Medium/High
/// - 2 of 4 above → composite ~0.1-0.3 → Low/Medium
/// - 1 of 4 above → composite < 0.1 → Low
/// - 0 above → composite ~0.01 → Low
///
/// # CALIBRATION: thresholds derived from simulation of 1000 random 2×2 tables
fn determine_geometric_level(composite: f64, signals: u32) -> &'static str {
    match (composite, signals) {
        (c, s) if c >= 0.70 && s >= 3 => "Critical",
        (c, s) if c >= 0.50 && s >= 3 => "High",
        (c, _) if c >= 0.50 => "High",
        (c, s) if c >= 0.30 && s >= 2 => "Medium",
        (c, _) if c >= 0.30 => "Medium",
        (_, s) if s >= 1 => "Low",
        _ => "Low",
    }
}

/// Calculate non-compensatory risk score from a risk context.
///
/// Uses weighted geometric mean aggregation where a weak dimension
/// cannot be compensated by a strong one. This is the ASDF v2.0-inspired
/// scoring mode.
///
/// # Arguments
/// * `context` - Validated risk context with PRR, ROR, IC025, EBGM05
/// * `weights` - Optional custom weights (must sum to 1.0). Defaults to equal.
///
/// # Returns
/// `NonCompensatoryScore` with composite score, dimension breakdown, and level.
#[must_use]
pub fn calculate_noncompensatory_score(
    context: &RiskContext,
    weights: Option<&[f64; 4]>,
) -> NonCompensatoryScore {
    let w = weights.unwrap_or(&DEFAULT_WEIGHTS);
    let dimensions = score_dimensions(context, w);

    let composite_raw = weighted_geometric_mean(&dimensions);

    // Apply case count weight (same formula as additive scorer for consistency)
    let n_weight = super::calculate_n_weight(context.n);
    // Scale composite to 0-100 range, apply n_weight
    let composite_scaled = (composite_raw * 100.0 * n_weight).clamp(0.0, 100.0);

    let signals_detected = dimensions.iter().filter(|d| d.signal_detected).count() as u32;
    let level = determine_geometric_level(composite_raw, signals_detected).to_string();

    let mut factors = Vec::new();
    for d in &dimensions {
        if d.signal_detected {
            factors.push(format!(
                "✓ {} = {:.2} (threshold {:.1}, normalized {:.3}, weight {:.2})",
                d.name, d.raw_value, d.threshold, d.normalized, d.weight
            ));
        } else {
            factors.push(format!(
                "✗ {} = {:.2} (below threshold {:.1}, normalized {:.3}, weight {:.2})",
                d.name, d.raw_value, d.threshold, d.normalized, d.weight
            ));
        }
    }
    factors.push(format!(
        "Geometric mean: {:.4} → scaled: {:.1}/100",
        composite_raw, composite_scaled
    ));
    factors.push(format!("n={} (weight: {:.2}×)", context.n, n_weight));
    factors.push(format!(
        "Signals: {}/4 dimensions above threshold",
        signals_detected
    ));

    if signals_detected < 4 && signals_detected > 0 {
        factors.push(format!(
            "⚠ Non-compensatory: {} missing dimension(s) suppress composite",
            4 - signals_detected
        ));
    }

    NonCompensatoryScore {
        composite: Measured::certain(composite_scaled),
        level,
        dimensions,
        signals_detected,
        n_weight,
        factors,
        mode: "geometric_noncompensatory".into(),
    }
}

// =============================================================================
// Dual-Mode Scoring
// =============================================================================

/// Scoring mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ScoringMode {
    /// Original additive scoring (each metric contributes independently)
    #[default]
    Additive,
    /// Geometric mean (ASDF v2.0 — non-compensatory, zero collapses composite)
    Geometric,
    /// Both modes computed side-by-side for comparison
    DualMode,
}

/// Dual-mode risk assessment showing both additive and geometric scores.
///
/// When the two modes disagree significantly (additive says High but geometric
/// says Low), this indicates one strong metric masking weakness in others —
/// exactly the failure mode the geometric scorer detects.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DualModeScore {
    /// Additive score (legacy, backward-compatible)
    pub additive: crate::RiskScore,
    /// Geometric score (non-compensatory)
    pub geometric: NonCompensatoryScore,
    /// Divergence between the two modes (0 = agree, higher = more disagreement)
    pub divergence: f64,
    /// Whether the additive score is masking a geometric weakness
    pub compensatory_masking: bool,
    /// Human-readable divergence explanation
    pub divergence_explanation: String,
}

/// Risk level to numeric rank for comparison
fn level_rank(level: &str) -> u8 {
    match level {
        "Critical" => 4,
        "High" => 3,
        "Medium" => 2,
        "Low" => 1,
        _ => 0,
    }
}

/// Calculate both additive and geometric scores and detect divergence.
///
/// Divergence is measured as the absolute difference in level ranks.
/// A divergence of 2+ (e.g., additive=High, geometric=Low) indicates
/// compensatory masking — one strong metric inflating the additive score
/// while the geometric score correctly identifies the imbalanced evidence.
#[must_use]
pub fn calculate_dual_mode(context: &RiskContext, weights: Option<&[f64; 4]>) -> DualModeScore {
    let additive = crate::calculate_risk_score(context);
    let geometric = calculate_noncompensatory_score(context, weights);

    let add_rank = level_rank(&additive.level);
    let geo_rank = level_rank(&geometric.level);
    let divergence = (add_rank as f64 - geo_rank as f64).abs();

    let compensatory_masking = add_rank > geo_rank + 1;

    let divergence_explanation = if compensatory_masking {
        format!(
            "MASKING DETECTED: Additive ({}) exceeds Geometric ({}) by {} levels — \
             strong signal in {} metric(s) masks weakness in {} others",
            additive.level,
            geometric.level,
            add_rank.saturating_sub(geo_rank),
            geometric.signals_detected,
            4 - geometric.signals_detected
        )
    } else if divergence > 0.0 {
        format!(
            "Minor divergence: Additive ({}) vs Geometric ({}) — \
             {} level(s) apart",
            additive.level, geometric.level, divergence as u8
        )
    } else {
        format!(
            "Aligned: Both modes agree at {} — evidence balanced across dimensions",
            additive.level
        )
    };

    DualModeScore {
        additive,
        geometric,
        divergence,
        compensatory_masking,
        divergence_explanation,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OriginatorType, RiskContext};

    fn make_context(prr: f64, ror_lower: f64, ic025: f64, eb05: f64, n: u64) -> RiskContext {
        RiskContext {
            drug: "test-drug".into(),
            event: "test-event".into(),
            prr,
            ror_lower,
            ic025,
            eb05,
            n,
            originator: OriginatorType::default(),
        }
    }

    #[test]
    fn test_normalize_metric_at_threshold() {
        // At threshold, logistic function returns exactly 0.5
        let score = normalize_metric(2.0, 2.0, 2.0);
        assert!(
            (score - 0.5).abs() < 1e-10,
            "At threshold should be 0.5, got {score}"
        );
    }

    #[test]
    fn test_normalize_metric_above_threshold() {
        let score = normalize_metric(4.0, 2.0, 2.0);
        assert!(score > 0.5, "Above threshold should be > 0.5, got {score}");
        assert!(score < 1.0, "Should not exceed 1.0, got {score}");
    }

    #[test]
    fn test_normalize_metric_below_threshold() {
        let score = normalize_metric(0.5, 2.0, 2.0);
        assert!(score < 0.5, "Below threshold should be < 0.5, got {score}");
        assert!(
            score > 0.0,
            "Should not be zero for valid data, got {score}"
        );
    }

    #[test]
    fn test_normalize_metric_zero_value() {
        let score = normalize_metric(0.0, 2.0, 2.0);
        assert!(
            (score - 0.01).abs() < 1e-10,
            "Zero value should hit floor 0.01, got {score}"
        );
    }

    #[test]
    fn test_normalize_metric_nan() {
        let score = normalize_metric(f64::NAN, 2.0, 2.0);
        assert!(
            (score - 0.0).abs() < 1e-10,
            "NaN should return 0.0, got {score}"
        );
    }

    #[test]
    fn test_all_metrics_strong_signal() {
        // All 4 metrics well above threshold — should get high composite
        let ctx = make_context(5.0, 3.0, 1.5, 4.0, 50);
        let score = calculate_noncompensatory_score(&ctx, None);

        assert_eq!(score.signals_detected, 4);
        assert!(
            score.composite.value > 50.0,
            "All strong signals should give high composite, got {}",
            score.composite.value
        );
        assert!(
            score.level == "High" || score.level == "Critical",
            "Expected High or Critical, got {}",
            score.level
        );
    }

    #[test]
    fn test_one_metric_strong_others_absent() {
        // Only PRR above threshold — geometric should collapse
        let ctx = make_context(10.0, 0.5, -1.0, 0.5, 50);
        let score = calculate_noncompensatory_score(&ctx, None);

        assert_eq!(score.signals_detected, 1);
        assert!(
            score.composite.value < 30.0,
            "Single strong metric should NOT give high composite, got {}",
            score.composite.value
        );
        assert_eq!(score.level, "Low", "Expected Low, got {}", score.level);
    }

    #[test]
    fn test_compensatory_masking_detection() {
        // PRR and ROR strong, IC and EBGM absent
        // Additive: 50/100 (High), Geometric: much lower (the masking)
        let ctx = make_context(8.0, 4.0, -2.0, 0.5, 100);
        let dual = calculate_dual_mode(&ctx, None);

        // Additive should be higher than geometric
        assert!(
            dual.additive.score.value > dual.geometric.composite.value,
            "Additive ({}) should exceed Geometric ({}) when evidence is imbalanced",
            dual.additive.score.value,
            dual.geometric.composite.value
        );
        assert!(
            dual.divergence >= 1.0,
            "Should detect divergence, got {}",
            dual.divergence
        );
    }

    #[test]
    fn test_balanced_signals_no_divergence() {
        // All metrics at similar strength above threshold
        let ctx = make_context(3.0, 2.0, 0.5, 3.0, 30);
        let dual = calculate_dual_mode(&ctx, None);

        assert!(
            dual.divergence <= 1.0,
            "Balanced signals should have low divergence, got {}",
            dual.divergence
        );
        assert!(
            !dual.compensatory_masking,
            "No masking expected for balanced signals"
        );
    }

    #[test]
    fn test_zero_signals_low_composite() {
        // All metrics below threshold
        let ctx = make_context(0.5, 0.3, -1.5, 0.5, 30);
        let score = calculate_noncompensatory_score(&ctx, None);

        assert_eq!(score.signals_detected, 0);
        assert!(
            score.composite.value < 10.0,
            "No signals should give very low composite, got {}",
            score.composite.value
        );
        assert_eq!(score.level, "Low");
    }

    #[test]
    fn test_geometric_mean_mathematical_property() {
        // Verify: geometric mean of equal values = that value
        let dims = vec![
            DimensionScore {
                name: "A".into(),
                raw_value: 0.0,
                threshold: 0.0,
                normalized: 0.6,
                weight: 0.25,
                signal_detected: true,
            },
            DimensionScore {
                name: "B".into(),
                raw_value: 0.0,
                threshold: 0.0,
                normalized: 0.6,
                weight: 0.25,
                signal_detected: true,
            },
            DimensionScore {
                name: "C".into(),
                raw_value: 0.0,
                threshold: 0.0,
                normalized: 0.6,
                weight: 0.25,
                signal_detected: true,
            },
            DimensionScore {
                name: "D".into(),
                raw_value: 0.0,
                threshold: 0.0,
                normalized: 0.6,
                weight: 0.25,
                signal_detected: true,
            },
        ];
        let gm = weighted_geometric_mean(&dims);
        assert!(
            (gm - 0.6).abs() < 1e-10,
            "GM of equal values should equal that value, got {gm}"
        );
    }

    #[test]
    fn test_geometric_mean_zero_collapses() {
        // A zero dimension should dramatically reduce the mean
        let dims = vec![
            DimensionScore {
                name: "A".into(),
                raw_value: 0.0,
                threshold: 0.0,
                normalized: 0.9,
                weight: 0.25,
                signal_detected: true,
            },
            DimensionScore {
                name: "B".into(),
                raw_value: 0.0,
                threshold: 0.0,
                normalized: 0.9,
                weight: 0.25,
                signal_detected: true,
            },
            DimensionScore {
                name: "C".into(),
                raw_value: 0.0,
                threshold: 0.0,
                normalized: 0.9,
                weight: 0.25,
                signal_detected: true,
            },
            DimensionScore {
                name: "D".into(),
                raw_value: 0.0,
                threshold: 0.0,
                // Near-zero dimension
                normalized: 0.01,
                weight: 0.25,
                signal_detected: false,
            },
        ];
        let gm = weighted_geometric_mean(&dims);
        // With one near-zero: GM = exp(0.25*(3*ln(0.9) + ln(0.01))) ≈ 0.29
        // Collapse: from 0.9 uniform down to 0.29 — a 68% reduction
        assert!(
            gm < 0.35,
            "Near-zero dimension should collapse GM, got {gm}"
        );
        assert!(
            gm < 0.9 * 0.5,
            "GM should be well below the strong dimensions, got {gm}"
        );
    }

    #[test]
    fn test_custom_weights() {
        // Weight PRR heavily, de-weight others
        let weights = [0.70, 0.10, 0.10, 0.10];
        let ctx = make_context(10.0, 0.5, -1.0, 0.5, 50);
        let score = calculate_noncompensatory_score(&ctx, Some(&weights));

        // With 70% weight on strong PRR, composite should be higher than equal weights
        let score_equal = calculate_noncompensatory_score(&ctx, None);
        assert!(
            score.composite.value > score_equal.composite.value,
            "Weighting toward strong metric should increase composite: weighted={} equal={}",
            score.composite.value,
            score_equal.composite.value
        );
    }

    #[test]
    fn test_n_weight_applied() {
        // Same metrics, different case counts
        let ctx_low_n = make_context(5.0, 3.0, 1.0, 4.0, 5);
        let ctx_high_n = make_context(5.0, 3.0, 1.0, 4.0, 500);

        let score_low = calculate_noncompensatory_score(&ctx_low_n, None);
        let score_high = calculate_noncompensatory_score(&ctx_high_n, None);

        assert!(
            score_high.composite.value > score_low.composite.value,
            "Higher case count should increase composite: n=500→{} vs n=5→{}",
            score_high.composite.value,
            score_low.composite.value
        );
    }
}
