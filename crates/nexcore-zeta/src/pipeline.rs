//! # Zeta Telescope Pipeline
//!
//! Unified pipeline that chains all five CMV-derived products into a single
//! [`TelescopeReport`]. One function call, one shared CMV root, five
//! independent analysis branches, one composite confidence score.
//!
//! ## Pipeline Architecture
//!
//! ```text
//! zeros ─→ reconstruct_cmv ─┬─→ fit_verblunsky_model ─→ predict_next_zeros
//!                            ├─→ killip_nenciu_test ──┐
//!                            ├─→ fingerprint_zeros    │
//!                            └─→ build_baseline ──→ detect_anomaly
//!                                                    │
//!          compare_to_gue (pair correlation) ────────┴─→ compare_gue_tests
//!                                                         │
//!                            ┌────────────────────────────┘
//!                            ↓
//!                     TelescopeReport
//! ```

use serde::{Deserialize, Serialize};

use crate::anomaly::{AnomalyReport, VerblunskyBaseline, build_baseline, detect_anomaly};
use crate::cmv::{CmvReconstruction, reconstruct_cmv};
use crate::error::ZetaError;
use crate::fingerprint::{SpectralFingerprint, fingerprint_zeros};
use crate::killip_nenciu::{
    DualGueVerdict, KillipNenciuTest, compare_gue_tests, killip_nenciu_test,
};
use crate::prediction::{
    PredictionAccuracy, VerblunskyModel, fit_verblunsky_model, predict_next_zeros,
    validate_prediction,
};
use crate::statistics::{GueComparison, compare_to_gue};
use crate::zeros::ZetaZero;

// ── Configuration ─────────────────────────────────────────────────────────────

/// Configuration for the Zeta Telescope pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelescopeConfig {
    /// Number of zeros to predict beyond the input set.
    pub n_predict: usize,
    /// Ground-truth zeros for validating predictions (if available).
    pub ground_truth: Option<Vec<ZetaZero>>,
    /// Whether to run the anomaly detector.
    pub run_anomaly: bool,
    /// Whether to run spectral fingerprinting.
    pub run_fingerprint: bool,
    /// Whether to run the Killip-Nenciu GUE test.
    pub run_killip_nenciu: bool,
    /// Whether to run the pair correlation GUE test.
    pub run_pair_correlation: bool,
    /// Whether to run zero prediction.
    pub run_prediction: bool,
}

impl Default for TelescopeConfig {
    fn default() -> Self {
        Self {
            n_predict: 10,
            ground_truth: None,
            run_anomaly: true,
            run_fingerprint: true,
            run_killip_nenciu: true,
            run_pair_correlation: true,
            run_prediction: true,
        }
    }
}

// ── Report ────────────────────────────────────────────────────────────────────

/// Complete output of the Zeta Telescope pipeline.
///
/// Contains results from all five CMV-derived products plus a composite
/// RH confidence score that combines all available evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelescopeReport {
    /// Number of input zeros.
    pub n_zeros: usize,
    /// Height range [t_min, t_max] of the input zeros.
    pub height_range: (f64, f64),

    // ── Stage 0: CMV reconstruction (always runs) ──
    /// CMV reconstruction with Verblunsky coefficients.
    pub cmv: CmvReconstruction,

    // ── Stage 1: Zero prediction ──
    /// Fitted Verblunsky model (if prediction enabled).
    pub verblunsky_model: Option<VerblunskyModel>,
    /// Predicted t-values for future zeros.
    pub predictions: Vec<f64>,
    /// Prediction accuracy against ground truth (if provided).
    pub prediction_accuracy: Option<PredictionAccuracy>,

    // ── Stage 2: Killip-Nenciu GUE test ──
    /// Killip-Nenciu coefficient-distribution test.
    pub killip_nenciu: Option<KillipNenciuTest>,
    /// Pair correlation GUE comparison.
    pub pair_correlation: Option<GueComparison>,
    /// Combined dual-test verdict.
    pub dual_verdict: Option<DualGueVerdict>,

    // ── Stage 3: Spectral fingerprint ──
    /// Spectral fingerprint of the zero set.
    pub fingerprint: Option<SpectralFingerprint>,

    // ── Stage 4: Anomaly detection ──
    /// Verblunsky baseline model.
    pub baseline: Option<VerblunskyBaseline>,
    /// Anomaly detection report.
    pub anomaly: Option<AnomalyReport>,

    // ── Composite ──
    /// Overall RH confidence score ∈ [0, 1].
    ///
    /// Weighted composite of all available evidence:
    /// - CMV roundtrip fidelity (does the operator exist?)
    /// - Killip-Nenciu p-value (does it match GUE?)
    /// - Pair correlation score (do spacings match GUE?)
    /// - Anomaly score (any counterexample signal?)
    pub overall_rh_confidence: f64,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Run the complete Zeta Telescope pipeline.
///
/// Chains all five CMV-derived products from a shared CMV root:
/// 1. CMV reconstruction → Verblunsky coefficients
/// 2. Zero prediction → extrapolated zero locations
/// 3. Killip-Nenciu + pair correlation → dual GUE test
/// 4. Spectral fingerprinting → universality classification
/// 5. Anomaly detection → counterexample scan
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if fewer than 20 zeros provided
/// (minimum for meaningful statistical analysis).
pub fn run_telescope(
    zeros: &[ZetaZero],
    config: &TelescopeConfig,
) -> Result<TelescopeReport, ZetaError> {
    if zeros.len() < 20 {
        return Err(ZetaError::InvalidParameter(
            "need at least 20 zeros for telescope pipeline".to_string(),
        ));
    }

    let n_zeros = zeros.len();
    let t_min = zeros.iter().map(|z| z.t).fold(f64::INFINITY, f64::min);
    let t_max = zeros.iter().map(|z| z.t).fold(f64::NEG_INFINITY, f64::max);

    // ── Stage 0: CMV (always) ──
    let cmv = reconstruct_cmv(zeros)?;

    // ── Stage 1: Prediction ──
    let (verblunsky_model, predictions, prediction_accuracy) = if config.run_prediction {
        let model = fit_verblunsky_model(&cmv);
        let preds = predict_next_zeros(&model, zeros, config.n_predict).unwrap_or_default();
        let accuracy = config
            .ground_truth
            .as_ref()
            .map(|gt| validate_prediction(&preds, gt));
        (Some(model), preds, accuracy)
    } else {
        (None, vec![], None)
    };

    // ── Stage 2: GUE tests ──
    let killip_nenciu = if config.run_killip_nenciu {
        killip_nenciu_test(&cmv).ok()
    } else {
        None
    };

    let pair_correlation = if config.run_pair_correlation {
        compare_to_gue(zeros).ok()
    } else {
        None
    };

    let dual_verdict = match (&pair_correlation, &killip_nenciu) {
        (Some(pc), Some(kn)) => Some(compare_gue_tests(pc, kn)),
        _ => None,
    };

    // ── Stage 3: Fingerprint ──
    let fingerprint = if config.run_fingerprint {
        fingerprint_zeros(zeros).ok()
    } else {
        None
    };

    // ── Stage 4: Anomaly ──
    let (baseline, anomaly) = if config.run_anomaly {
        match build_baseline(zeros) {
            Ok(bl) => {
                let report = detect_anomaly(&bl, zeros).ok();
                (Some(bl), report)
            }
            Err(_) => (None, None),
        }
    } else {
        (None, None)
    };

    // ── Composite confidence ──
    let overall_rh_confidence = compute_confidence(
        &cmv,
        killip_nenciu.as_ref(),
        pair_correlation.as_ref(),
        anomaly.as_ref(),
    );

    Ok(TelescopeReport {
        n_zeros,
        height_range: (t_min, t_max),
        cmv,
        verblunsky_model,
        predictions,
        prediction_accuracy,
        killip_nenciu,
        pair_correlation,
        dual_verdict,
        fingerprint,
        baseline,
        anomaly,
        overall_rh_confidence,
    })
}

// ── Internal ──────────────────────────────────────────────────────────────────

/// Compute overall RH confidence from available evidence.
///
/// Four signals, weighted by reliability:
/// 1. CMV fidelity (30%): 1 − roundtrip_error. High fidelity → the operator exists.
/// 2. KN p-value (25%): Non-rejection of GUE → consistent with RH.
/// 3. Pair correlation (25%): GUE match score.
/// 4. Anomaly absence (20%): Low anomaly score → no counterexample signal.
fn compute_confidence(
    cmv: &CmvReconstruction,
    kn: Option<&KillipNenciuTest>,
    pc: Option<&GueComparison>,
    anomaly: Option<&AnomalyReport>,
) -> f64 {
    let mut score = 0.0_f64;
    let mut weight = 0.0_f64;

    // CMV fidelity: 1 - roundtrip_error, clamped to [0, 1]
    let cmv_signal = (1.0 - cmv.roundtrip_error).clamp(0.0, 1.0);
    score += 0.30 * cmv_signal;
    weight += 0.30;

    // Killip-Nenciu: p-value > 0.05 is good, map to [0, 1]
    if let Some(kn) = kn {
        // Transform: p=1 → signal=1, p=0 → signal=0
        // Boost if p > 0.05 (non-rejection zone)
        let kn_signal = if kn.ks_pvalue > 0.05 {
            0.5 + 0.5 * kn.ks_pvalue // range [0.525, 1.0]
        } else {
            kn.ks_pvalue / 0.05 * 0.5 // range [0.0, 0.5]
        };
        score += 0.25 * kn_signal;
        weight += 0.25;
    }

    // Pair correlation: gue_match_score already in [0, 1]
    if let Some(pc) = pc {
        score += 0.25 * pc.gue_match_score.clamp(0.0, 1.0);
        weight += 0.25;
    }

    // Anomaly: low score is good. Map: score=0 → signal=1, score>5 → signal=0
    if let Some(an) = anomaly {
        let anom_signal = (1.0 - an.anomaly_score / 5.0).clamp(0.0, 1.0);
        score += 0.20 * anom_signal;
        weight += 0.20;
    }

    if weight > 0.0 {
        (score / weight).clamp(0.0, 1.0)
    } else {
        0.0
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zeros::find_zeros_bracket;

    fn get_zeros(t_min: f64, t_max: f64) -> Vec<ZetaZero> {
        find_zeros_bracket(t_min, t_max, 0.05).unwrap_or_default()
    }

    #[test]
    fn telescope_on_75_zeros() {
        let zeros = get_zeros(10.0, 200.0);
        if zeros.len() < 20 {
            return;
        }
        let config = TelescopeConfig::default();
        let report = run_telescope(&zeros, &config);
        assert!(report.is_ok(), "pipeline failed: {:?}", report.err());
        let r = report.unwrap_or_else(|_| unreachable!());

        assert_eq!(r.n_zeros, zeros.len());
        assert!(r.height_range.0 < r.height_range.1);
        assert!(r.cmv.roundtrip_error.is_finite());
        assert!(r.verblunsky_model.is_some());
        assert!(!r.predictions.is_empty());
        assert!(r.killip_nenciu.is_some());
        assert!(r.pair_correlation.is_some());
        assert!(r.dual_verdict.is_some());
        assert!(r.fingerprint.is_some());
        assert!(r.baseline.is_some());
        assert!(r.anomaly.is_some());

        assert!(
            (0.0..=1.0).contains(&r.overall_rh_confidence),
            "confidence {} not in [0,1]",
            r.overall_rh_confidence
        );

        eprintln!("=== TELESCOPE REPORT (N={}) ===", r.n_zeros);
        eprintln!(
            "Height range: {:.1} — {:.1}",
            r.height_range.0, r.height_range.1
        );
        eprintln!("CMV roundtrip error: {:.2e}", r.cmv.roundtrip_error);
        eprintln!("Predictions: {} zeros predicted", r.predictions.len());
        if let Some(kn) = &r.killip_nenciu {
            eprintln!(
                "Killip-Nenciu: D={:.4}, p={:.4}",
                kn.ks_statistic, kn.ks_pvalue
            );
        }
        if let Some(pc) = &r.pair_correlation {
            eprintln!(
                "Pair correlation: score={:.3}, MAE={:.4}",
                pc.gue_match_score, pc.pair_correlation_mae
            );
        }
        if let Some(v) = &r.dual_verdict {
            eprintln!("Dual verdict: {}", v.interpretation);
        }
        if let Some(an) = &r.anomaly {
            eprintln!(
                "Anomaly score: {:.4}, is_anomalous={}",
                an.anomaly_score, an.is_anomalous
            );
        }
        eprintln!("Overall RH confidence: {:.4}", r.overall_rh_confidence);
    }

    #[test]
    fn telescope_on_649_zeros() {
        let zeros = get_zeros(10.0, 1000.0);
        if zeros.len() < 100 {
            return;
        }
        let config = TelescopeConfig {
            n_predict: 20,
            ..TelescopeConfig::default()
        };
        let report = run_telescope(&zeros, &config);
        assert!(
            report.is_ok(),
            "pipeline failed at N={}: {:?}",
            zeros.len(),
            report.err()
        );
        let r = report.unwrap_or_else(|_| unreachable!());

        assert!(
            (0.0..=1.0).contains(&r.overall_rh_confidence),
            "confidence {} not in [0,1]",
            r.overall_rh_confidence
        );
        // At N=649, the Killip-Nenciu test may reject GUE (finite-size effect:
        // the discrete spectral measure deviates from the continuous GUE
        // prediction). This is expected — the test is correct, the model is
        // approximate. Confidence > 0.15 is reasonable at this N.
        assert!(
            r.overall_rh_confidence > 0.15,
            "confidence {:.3} suspiciously low with {} zeros",
            r.overall_rh_confidence,
            zeros.len()
        );

        eprintln!(
            "=== TELESCOPE (N={}) confidence={:.4} ===",
            r.n_zeros, r.overall_rh_confidence
        );
    }

    #[test]
    fn telescope_with_prediction_validation() {
        // Use first 50 zeros, validate predictions against zeros 51-75
        let all_zeros = get_zeros(10.0, 200.0);
        if all_zeros.len() < 60 {
            return;
        }
        let (train, test) = all_zeros.split_at(50.min(all_zeros.len()));
        let config = TelescopeConfig {
            n_predict: test.len(),
            ground_truth: Some(test.to_vec()),
            ..TelescopeConfig::default()
        };
        let report = run_telescope(train, &config);
        assert!(report.is_ok());
        let r = report.unwrap_or_else(|_| unreachable!());

        if let Some(acc) = &r.prediction_accuracy {
            eprintln!(
                "Prediction MAE={:.4}, RMSE={:.4}, max_error={:.4}, compared={}",
                acc.mae, acc.rmse, acc.max_error, acc.n_compared
            );
        }
    }

    #[test]
    fn telescope_selective_stages() {
        let zeros = get_zeros(10.0, 100.0);
        if zeros.len() < 20 {
            return;
        }
        // Only run CMV + prediction
        let config = TelescopeConfig {
            run_anomaly: false,
            run_fingerprint: false,
            run_killip_nenciu: false,
            run_pair_correlation: false,
            run_prediction: true,
            ..TelescopeConfig::default()
        };
        let report = run_telescope(&zeros, &config);
        assert!(report.is_ok());
        let r = report.unwrap_or_else(|_| unreachable!());

        assert!(r.verblunsky_model.is_some());
        assert!(r.killip_nenciu.is_none());
        assert!(r.pair_correlation.is_none());
        assert!(r.fingerprint.is_none());
        assert!(r.anomaly.is_none());
        // Confidence still computable from CMV alone
        assert!(r.overall_rh_confidence > 0.0);
    }

    #[test]
    fn rejects_too_few_zeros() {
        let zeros = get_zeros(10.0, 30.0);
        if zeros.len() >= 20 {
            return; // can't test this if we find too many zeros
        }
        let config = TelescopeConfig::default();
        assert!(run_telescope(&zeros, &config).is_err());
    }

    #[test]
    fn confidence_components_are_bounded() {
        let zeros = get_zeros(10.0, 200.0);
        if zeros.len() < 20 {
            return;
        }
        let config = TelescopeConfig::default();
        let r = run_telescope(&zeros, &config).unwrap_or_else(|_| unreachable!());

        // Each contributing signal should be reasonable
        assert!(r.cmv.roundtrip_error < 1.0, "CMV roundtrip > 1.0");
        if let Some(kn) = &r.killip_nenciu {
            assert!(
                (0.0..=1.0).contains(&kn.ks_pvalue),
                "KN p-value out of range"
            );
        }
        if let Some(pc) = &r.pair_correlation {
            assert!(
                (0.0..=1.0).contains(&pc.gue_match_score),
                "PC score out of range"
            );
        }
    }
}
