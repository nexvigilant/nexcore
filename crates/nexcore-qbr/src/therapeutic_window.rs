//! Form 4: Geometric QBR — Therapeutic Window as Computed Area.
//!
//! ```text
//! QTW = ∫[dose_min → dose_max] (Hill_efficacy(d) - Hill_toxicity(d)) dd
//! ```
//!
//! The therapeutic window is the area between two sigmoid (Hill) curves:
//! the efficacy dose-response curve and the toxicity dose-response curve.
//!
//! This is a definite integral, computable from Hill equation parameters.
//! The result is a single `Measured<f64>` representing how much
//! pharmacological room a drug has between helping and harming.
//!
//! Negative regions (toxicity > efficacy) contribute negatively.
//! QTW can be negative for drugs where toxicity onset precedes efficacy.
//!
//! Tier: T3-D (Domain Composite)
//! Grounding: ∂(Boundary) + N(Quantity) + Σ(Sum) + →(Causality)

use crate::error::QbrError;
use crate::integration::simpson_integrate;
use crate::types::{HillCurveParams, IntegrationBounds};
use nexcore_constants::{Confidence, Measured};
use nexcore_primitives::chemistry::cooperativity::hill_response;

/// Compute the Quantitative Therapeutic Window (QTW).
///
/// Integrates the difference between efficacy and toxicity Hill curves
/// over the specified dose range.
///
/// # Arguments
/// * `efficacy` — Hill curve parameters for the efficacy dose-response
/// * `toxicity` — Hill curve parameters for the toxicity dose-response
/// * `bounds` — Dose range and integration precision
///
/// # Returns
/// `Measured<f64>` where:
/// - Positive value = efficacy exceeds toxicity over dose range (favorable)
/// - Near-zero = curves overlap significantly (narrow window)
/// - Negative = toxicity dominates (unfavorable)
pub fn compute_therapeutic_window(
    efficacy: &HillCurveParams,
    toxicity: &HillCurveParams,
    bounds: &IntegrationBounds,
) -> Result<Measured<f64>, QbrError> {
    // Validate Hill parameters
    validate_hill_params(efficacy, "efficacy")?;
    validate_hill_params(toxicity, "toxicity")?;

    if bounds.dose_min < 0.0 {
        return Err(QbrError::InvalidHillParams(
            "dose_min must be non-negative".to_string(),
        ));
    }
    if bounds.dose_max <= bounds.dose_min {
        return Err(QbrError::Integration(
            "dose_max must be greater than dose_min".to_string(),
        ));
    }

    // Ensure even intervals
    let n = if bounds.intervals % 2 != 0 {
        bounds.intervals + 1
    } else {
        bounds.intervals
    };

    // The integrand: Hill_efficacy(d) - Hill_toxicity(d)
    let eff_k = efficacy.k_half;
    let eff_n = efficacy.n_hill;
    let tox_k = toxicity.k_half;
    let tox_n = toxicity.n_hill;

    let integrand = |dose: f64| -> f64 {
        hill_response(dose, eff_k, eff_n) - hill_response(dose, tox_k, tox_n)
    };

    let area = simpson_integrate(integrand, bounds.dose_min, bounds.dose_max, n)?;

    // Normalize by dose range to get an average window width (0 to 1 scale)
    let dose_range = bounds.dose_max - bounds.dose_min;
    let normalized = area / dose_range;

    // Confidence derived from how well-characterized the Hill parameters are.
    // Higher Hill coefficients (steeper curves) = more defined transitions = higher confidence.
    // Use geometric mean of cooperativity factors.
    let eff_steepness = (efficacy.n_hill / 4.0).clamp(0.1, 1.0);
    let tox_steepness = (toxicity.n_hill / 4.0).clamp(0.1, 1.0);
    let confidence_raw = (eff_steepness * tox_steepness).sqrt().clamp(0.2, 0.95);

    Ok(Measured::new(normalized, Confidence::new(confidence_raw)))
}

/// Validate Hill curve parameters.
fn validate_hill_params(params: &HillCurveParams, label: &str) -> Result<(), QbrError> {
    if params.k_half <= 0.0 {
        return Err(QbrError::InvalidHillParams(format!(
            "{label}: k_half must be positive (got {})",
            params.k_half
        )));
    }
    if params.n_hill <= 0.0 {
        return Err(QbrError::InvalidHillParams(format!(
            "{label}: n_hill must be positive (got {})",
            params.n_hill
        )));
    }
    if !params.k_half.is_finite() || !params.n_hill.is_finite() {
        return Err(QbrError::InvalidHillParams(format!(
            "{label}: parameters must be finite"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_bounds() -> IntegrationBounds {
        IntegrationBounds {
            dose_min: 0.1, // avoid zero (Hill returns 0 at dose=0)
            dose_max: 100.0,
            intervals: 1000,
        }
    }

    #[test]
    fn test_equal_curves_zero_window() {
        // Two identical Hill curves → area between them ≈ 0
        let params = HillCurveParams {
            k_half: 10.0,
            n_hill: 2.0,
        };
        let result = compute_therapeutic_window(&params, &params, &default_bounds());
        assert!(result.is_ok());
        let qtw = result.unwrap_or_else(|_| Measured::new(999.0, Confidence::NONE));
        assert!(
            qtw.value.abs() < 1e-10,
            "Equal curves should yield QTW ≈ 0, got {}",
            qtw.value
        );
    }

    #[test]
    fn test_wide_therapeutic_window() {
        // Efficacy at low dose (EC50=10), toxicity at high dose (TC50=100)
        // Efficacy kicks in well before toxicity → positive window
        let efficacy = HillCurveParams {
            k_half: 10.0,
            n_hill: 2.0,
        };
        let toxicity = HillCurveParams {
            k_half: 100.0,
            n_hill: 2.0,
        };
        let result = compute_therapeutic_window(&efficacy, &toxicity, &default_bounds());
        assert!(result.is_ok());
        let qtw = result.unwrap_or_else(|_| Measured::new(0.0, Confidence::NONE));
        assert!(
            qtw.value > 0.0,
            "Wide window should be positive, got {}",
            qtw.value
        );
    }

    #[test]
    fn test_narrow_therapeutic_window() {
        // Toxicity at lower dose than efficacy → negative window
        let efficacy = HillCurveParams {
            k_half: 100.0,
            n_hill: 2.0,
        };
        let toxicity = HillCurveParams {
            k_half: 10.0,
            n_hill: 2.0,
        };
        let result = compute_therapeutic_window(&efficacy, &toxicity, &default_bounds());
        assert!(result.is_ok());
        let qtw = result.unwrap_or_else(|_| Measured::new(0.0, Confidence::NONE));
        assert!(
            qtw.value < 0.0,
            "Toxicity-first should be negative, got {}",
            qtw.value
        );
    }

    #[test]
    fn test_steep_curves_higher_confidence() {
        // Steeper curves (higher n_hill) = better characterized = higher confidence
        let low_n = HillCurveParams {
            k_half: 10.0,
            n_hill: 1.0,
        };
        let high_n = HillCurveParams {
            k_half: 10.0,
            n_hill: 4.0,
        };
        let tox = HillCurveParams {
            k_half: 50.0,
            n_hill: 2.0,
        };
        let bounds = default_bounds();

        let r_low = compute_therapeutic_window(&low_n, &tox, &bounds);
        let r_high = compute_therapeutic_window(&high_n, &tox, &bounds);
        assert!(r_low.is_ok());
        assert!(r_high.is_ok());

        let c_low = r_low.map(|m| m.confidence.value()).unwrap_or(0.0);
        let c_high = r_high.map(|m| m.confidence.value()).unwrap_or(0.0);
        assert!(
            c_high >= c_low,
            "Steeper curves should have >= confidence: {} vs {}",
            c_high,
            c_low
        );
    }

    #[test]
    fn test_invalid_k_half() {
        let bad = HillCurveParams {
            k_half: -5.0,
            n_hill: 2.0,
        };
        let good = HillCurveParams {
            k_half: 10.0,
            n_hill: 2.0,
        };
        let result = compute_therapeutic_window(&bad, &good, &default_bounds());
        assert!(matches!(result, Err(QbrError::InvalidHillParams(_))));
    }

    #[test]
    fn test_invalid_n_hill() {
        let good = HillCurveParams {
            k_half: 10.0,
            n_hill: 2.0,
        };
        let bad = HillCurveParams {
            k_half: 10.0,
            n_hill: 0.0,
        };
        let result = compute_therapeutic_window(&good, &bad, &default_bounds());
        assert!(matches!(result, Err(QbrError::InvalidHillParams(_))));
    }
}
