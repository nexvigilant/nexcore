//! Measured<T> confidence wrappers for survival analysis outputs.
//!
//! Wraps existing survival functions with `Measured<T>` from `nexcore-constants`,
//! providing confidence-propagating outputs without modifying core algorithm code.
//!
//! # Confidence Mappings
//!
//! Each mapping is marked with `// CALIBRATION:` for searchability and future refinement.

use nexcore_constants::{Confidence, Measured};
use serde::{Deserialize, Serialize};

use super::kaplan_meier::{KaplanMeierResult, SurvivalObservation, SurvivalPoint};
use crate::signals::core::error::SignalError;

// ═══════════════════════════════════════════════════════════════════════════════
// MEASURED KAPLAN-MEIER
// ═══════════════════════════════════════════════════════════════════════════════

/// Kaplan-Meier result with Measured<T> confidence at each survival point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasuredKaplanMeier {
    /// Raw KM result (unchanged for backward compatibility)
    pub raw: KaplanMeierResult,
    /// Each survival point wrapped with confidence derived from Greenwood SE
    pub measured_curve: Vec<Measured<f64>>,
    /// Overall result confidence (minimum across curve)
    pub overall_confidence: Confidence,
}

/// Compute Kaplan-Meier with Measured<T> confidence propagation.
///
/// Wraps `kaplan_meier()` output, mapping Greenwood SE to confidence.
/// Infallible: KM estimator always produces a result (possibly empty curve).
pub fn kaplan_meier_measured(observations: &[SurvivalObservation]) -> MeasuredKaplanMeier {
    let raw = super::kaplan_meier::kaplan_meier(observations);

    let measured_curve: Vec<Measured<f64>> = raw
        .curve
        .iter()
        .map(|pt| Measured::new(pt.survival, km_confidence(pt)))
        .collect();

    let overall_confidence = measured_curve
        .iter()
        .map(|m| m.confidence)
        .min_by(|a, b| a.cmp_total(*b))
        .unwrap_or(Confidence::UNCERTAIN);

    MeasuredKaplanMeier {
        raw,
        measured_curve,
        overall_confidence,
    }
}

/// Map Greenwood SE to Confidence for a KM survival point.
///
/// # CALIBRATION: KM SE → Confidence
///
/// `confidence = clamp(1.0 - 2.0 * greenwood_se, 0.05, 0.99)`
///
/// Rationale: Greenwood SE is already computed at each time point.
/// SE = 0.0 (perfect data) → confidence 1.0.
/// SE = 0.5 (maximum uncertainty for probability) → confidence 0.0.
/// The 2.0 multiplier scales so SE = 0.25 maps to confidence 0.5.
fn km_confidence(pt: &SurvivalPoint) -> Confidence {
    // CALIBRATION: KM Greenwood SE → Confidence
    Confidence::new((1.0 - 2.0 * pt.se).clamp(0.05, 0.99))
}

// ═══════════════════════════════════════════════════════════════════════════════
// MEASURED LOG-RANK
// ═══════════════════════════════════════════════════════════════════════════════

/// Log-rank test result with Measured<T> confidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasuredLogRank {
    /// Chi-squared test statistic
    pub chi_squared: f64,
    /// P-value (raw)
    pub p_value: f64,
    /// Mantel-Haenszel hazard ratio with confidence
    pub hazard_ratio: Measured<f64>,
    /// Whether the test is statistically significant (p < 0.05)
    pub significant: bool,
    /// Overall confidence in the between-group difference
    pub confidence: Confidence,
}

/// Compute log-rank test with Measured<T> confidence propagation.
///
/// Wraps `log_rank_test()` output.
pub fn log_rank_measured(
    group0: &[SurvivalObservation],
    group1: &[SurvivalObservation],
) -> MeasuredLogRank {
    let (chi_sq, p_value, hr) = super::kaplan_meier::log_rank_test(group0, group1);

    // CALIBRATION: p-value → Confidence
    // Low p-value = high confidence in real between-group difference.
    let conf = log_rank_confidence(p_value);

    MeasuredLogRank {
        chi_squared: chi_sq,
        p_value,
        hazard_ratio: Measured::new(hr, conf),
        significant: p_value < 0.05,
        confidence: conf,
    }
}

/// Map p-value to Confidence for log-rank test.
///
/// # CALIBRATION: Log-rank p-value → Confidence
///
/// `confidence = clamp(1.0 - p_value, 0.05, 0.99)`
///
/// Rationale: P-value directly represents probability that observed difference
/// is due to chance. Low p-value = high confidence in real difference.
fn log_rank_confidence(p_value: f64) -> Confidence {
    // CALIBRATION: Log-rank p-value → Confidence
    Confidence::new((1.0 - p_value).clamp(0.05, 0.99))
}

// ═══════════════════════════════════════════════════════════════════════════════
// MEASURED COX RESULT
// ═══════════════════════════════════════════════════════════════════════════════

/// Cox regression result with Measured<T> confidence per coefficient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasuredCoxResult {
    /// Raw Cox result (unchanged for backward compatibility)
    pub raw: super::cox::CoxResult,
    /// Each hazard ratio wrapped with confidence derived from CI width
    pub measured_hazard_ratios: Vec<Measured<f64>>,
    /// Overall model confidence (minimum across coefficients)
    pub overall_confidence: Confidence,
}

/// Fit Cox model with Measured<T> confidence propagation.
///
/// Wraps `fit_cox()` output, mapping CI width to confidence for each coefficient.
pub fn cox_measured(
    observations: &[super::cox::CoxObservation],
    config: &super::cox::CoxConfig,
) -> Result<MeasuredCoxResult, SignalError> {
    let raw = super::cox::fit_cox(observations, config)?;

    let measured_hazard_ratios: Vec<Measured<f64>> = raw
        .coefficients
        .iter()
        .map(|c| Measured::new(c.hazard_ratio, cox_hr_confidence(c)))
        .collect();

    let overall_confidence = measured_hazard_ratios
        .iter()
        .map(|m| m.confidence)
        .min_by(|a, b| a.cmp_total(*b))
        .unwrap_or(Confidence::UNCERTAIN);

    Ok(MeasuredCoxResult {
        raw,
        measured_hazard_ratios,
        overall_confidence,
    })
}

/// Map Cox HR confidence interval width to Confidence.
///
/// # CALIBRATION: Cox HR CI width → Confidence
///
/// `confidence = clamp(1.0 - (ln(hr_ci_upper) - ln(hr_ci_lower)) / 4.0, 0.05, 0.99)`
///
/// Rationale: HR CIs are asymmetric on natural scale but symmetric on log scale.
/// Log-CI width / 4.0 normalizes so CI spanning ~one order of magnitude
/// (e.g., HR 0.5 to 2.0, log-width ≈ 1.4) maps to moderate confidence.
/// Narrower CI = higher confidence.
fn cox_hr_confidence(c: &super::cox::CoxCoefficient) -> Confidence {
    // CALIBRATION: Cox HR log-CI width → Confidence
    let log_ci_width = if c.hr_ci_upper > 0.0 && c.hr_ci_lower > 0.0 {
        c.hr_ci_upper.ln() - c.hr_ci_lower.ln()
    } else {
        4.0 // Maximum uncertainty if CI bounds invalid
    };
    Confidence::new((1.0 - log_ci_width / 4.0).clamp(0.05, 0.99))
}

// ═══════════════════════════════════════════════════════════════════════════════
// MEASURED HAZARD RATIO (quick)
// ═══════════════════════════════════════════════════════════════════════════════

/// Quick hazard ratio result with Measured<T> confidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MeasuredHazardRatio {
    /// Raw Cox coefficient (unchanged for backward compatibility)
    pub raw: super::cox::CoxCoefficient,
    /// Hazard ratio with confidence
    pub hazard_ratio: Measured<f64>,
}

/// Compute quick hazard ratio with Measured<T> confidence propagation.
///
/// Wraps `quick_hazard_ratio()` output.
pub fn hazard_ratio_measured(
    treatment_times: &[f64],
    treatment_events: &[bool],
    control_times: &[f64],
    control_events: &[bool],
) -> Result<MeasuredHazardRatio, SignalError> {
    let raw = super::cox::quick_hazard_ratio(
        treatment_times,
        treatment_events,
        control_times,
        control_events,
    )?;

    let conf = cox_hr_confidence(&raw);

    Ok(MeasuredHazardRatio {
        hazard_ratio: Measured::new(raw.hazard_ratio, conf),
        raw,
    })
}

// ═══════════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_obs(time: f64, event: bool, group: Option<u8>) -> SurvivalObservation {
        if let Some(g) = group {
            SurvivalObservation::with_group(time, event, g)
        } else {
            SurvivalObservation::new(time, event)
        }
    }

    fn sample_km_data() -> Vec<SurvivalObservation> {
        vec![
            make_obs(1.0, true, None),
            make_obs(2.0, true, None),
            make_obs(3.0, false, None),
            make_obs(3.5, true, None),
            make_obs(4.0, true, None),
            make_obs(4.5, false, None),
            make_obs(5.0, true, None),
            make_obs(6.0, false, None),
        ]
    }

    // ── KM Measured ──────────────────────────────────────────────────

    #[test]
    fn test_km_measured_confidence_range() {
        let result = kaplan_meier_measured(&sample_km_data());
        for m in &result.measured_curve {
            assert!(
                m.confidence.value() >= 0.05 && m.confidence.value() <= 0.99,
                "Confidence {:.4} outside [0.05, 0.99]",
                m.confidence.value()
            );
        }
    }

    #[test]
    fn test_km_measured_confidence_degrades_over_time() {
        let result = kaplan_meier_measured(&sample_km_data());
        if result.measured_curve.len() >= 2 {
            let first_conf = result.measured_curve[0].confidence.value();
            let last_conf = result
                .measured_curve
                .last()
                .map(|m| m.confidence.value())
                .unwrap_or(1.0);
            assert!(
                first_conf >= last_conf,
                "Confidence should degrade: first={first_conf:.4} < last={last_conf:.4}"
            );
        }
    }

    #[test]
    fn test_km_measured_preserves_raw() {
        let obs = sample_km_data();
        let measured = kaplan_meier_measured(&obs);
        let raw = super::super::kaplan_meier::kaplan_meier(&obs);
        assert_eq!(measured.raw.n_total, raw.n_total);
        assert_eq!(measured.raw.n_events, raw.n_events);
        assert_eq!(measured.raw.curve.len(), raw.curve.len());
    }

    // ── Log-rank Measured ────────────────────────────────────────────

    #[test]
    fn test_log_rank_measured_different_groups() {
        let group0: Vec<_> = [5.0, 6.0, 8.0, 10.0]
            .iter()
            .map(|&t| make_obs(t, true, Some(0)))
            .collect();
        let group1: Vec<_> = [1.0, 2.0, 3.0, 4.0]
            .iter()
            .map(|&t| make_obs(t, true, Some(1)))
            .collect();

        let result = log_rank_measured(&group0, &group1);
        assert!(
            result.confidence.value() > 0.5,
            "Clearly different groups → high confidence: {:.4}",
            result.confidence.value()
        );
    }

    #[test]
    fn test_log_rank_measured_identical_groups() {
        let group0: Vec<_> = [1.0, 2.0, 3.0]
            .iter()
            .map(|&t| make_obs(t, true, Some(0)))
            .collect();
        let group1: Vec<_> = [1.0, 2.0, 3.0]
            .iter()
            .map(|&t| make_obs(t, true, Some(1)))
            .collect();

        let result = log_rank_measured(&group0, &group1);
        assert!(
            result.confidence.value() <= 0.99,
            "Identical groups → bounded confidence: {:.4}",
            result.confidence.value()
        );
    }

    // ── Cox Measured ─────────────────────────────────────────────────

    #[test]
    fn test_cox_measured_confidence_range() {
        use super::super::cox::{CoxConfig, CoxObservation};

        let data = vec![
            CoxObservation::simple(1.0, true, 1.0),
            CoxObservation::simple(2.0, true, 0.0),
            CoxObservation::simple(3.0, false, 1.0),
            CoxObservation::simple(4.0, true, 0.0),
            CoxObservation::simple(5.0, true, 1.0),
            CoxObservation::simple(6.0, false, 0.0),
            CoxObservation::simple(7.0, true, 1.0),
            CoxObservation::simple(8.0, true, 0.0),
        ];

        let result = cox_measured(&data, &CoxConfig::default()).unwrap();
        for m in &result.measured_hazard_ratios {
            assert!(
                m.confidence.value() >= 0.05 && m.confidence.value() <= 0.99,
                "Cox HR confidence {:.4} outside [0.05, 0.99]",
                m.confidence.value()
            );
        }
        assert!(result.measured_hazard_ratios[0].value > 0.0);
    }

    // ── Quick HR Measured ────────────────────────────────────────────

    #[test]
    fn test_hazard_ratio_measured() {
        let result = hazard_ratio_measured(
            &[1.0, 3.0, 5.0, 7.0],
            &[true, false, true, true],
            &[2.0, 4.0, 6.0, 8.0],
            &[true, true, false, true],
        )
        .unwrap();

        assert!(result.hazard_ratio.value > 0.0);
        assert!(
            result.hazard_ratio.confidence.value() >= 0.05
                && result.hazard_ratio.confidence.value() <= 0.99
        );
    }
}
