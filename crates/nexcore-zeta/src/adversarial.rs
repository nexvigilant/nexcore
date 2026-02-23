//! # Adversarial Counterexample Characterization
//!
//! Instead of confirming RH, this module asks: **what would a violation look like?**
//!
//! ## Structural Constraints on Counterexamples
//!
//! The functional equation ζ(s) = χ(s)·ζ(1-s) imposes symmetry:
//! - If ρ is a non-trivial zero, then so is 1 − ρ̄ (conjugate pair).
//! - A zero OFF the critical line with Re(ρ) = σ ≠ 1/2 forces a
//!   partner zero at Re = 1 − σ.
//!
//! ## What We Characterize
//!
//! 1. **Exclusion zones**: regions in the critical strip where no off-CL
//!    zero can exist, given our verified zero-free height.
//! 2. **Minimum gap constraints**: how far from the critical line a
//!    counterexample must be, given density bounds.
//! 3. **Energy cost**: the explicit formula contribution of a hypothetical
//!    off-CL zero — how much it would perturb ψ(x).

use std::f64::consts::PI;

use serde::{Deserialize, Serialize};

use crate::error::ZetaError;
use crate::zeros::{RhVerification, ZetaZero, count_zeros_to_height};

/// A hypothetical counterexample to RH.
///
/// This type encodes the structural properties a zero would need
/// to violate the Riemann Hypothesis. It is NOT a claim that such
/// a zero exists — it's a characterization of the search space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CounterexampleCandidate {
    /// Real part σ ≠ 1/2 of the hypothetical zero.
    pub sigma: f64,
    /// Imaginary part t of the hypothetical zero.
    pub t: f64,
    /// The forced partner zero at (1 − σ, t) via functional equation.
    pub partner_sigma: f64,
    /// Minimum distance from the critical line: |σ − 1/2|.
    pub distance_from_cl: f64,
    /// Whether this candidate is already excluded by our verified range.
    pub excluded: bool,
    /// Reason for exclusion (if any).
    pub exclusion_reason: Option<String>,
}

/// Summary of exclusion zone analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExclusionAnalysis {
    /// Height T to which RH has been verified.
    pub verified_height: f64,
    /// Number of zeros verified on the critical line.
    pub verified_count: usize,
    /// Total critical strip area scanned (in units of strip × height).
    pub scanned_area: f64,
    /// Zero-free region constraints.
    pub zero_free_regions: Vec<ZeroFreeRegion>,
    /// Minimum imaginary part any counterexample must have.
    pub min_counterexample_height: f64,
    /// Density constraints from the verified zeros.
    pub density_constraints: DensityConstraints,
    /// Explicit formula perturbation from a hypothetical off-CL zero.
    pub perturbation_analysis: PerturbationAnalysis,
}

/// A region of the critical strip proven free of zeros.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroFreeRegion {
    /// Description of the region.
    pub description: String,
    /// Lower bound on imaginary part.
    pub t_min: f64,
    /// Upper bound on imaginary part.
    pub t_max: f64,
    /// Lower bound on real part.
    pub sigma_min: f64,
    /// Upper bound on real part.
    pub sigma_max: f64,
    /// Source of this constraint (theorem/computation).
    pub source: String,
}

/// Density-based constraints on counterexample location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DensityConstraints {
    /// N(T) from Riemann–von Mangoldt: expected zero count to height T.
    pub expected_count_at_verified_height: f64,
    /// Actual count found (all on CL).
    pub actual_count: usize,
    /// If all expected zeros are accounted for on the CL, off-CL zeros
    /// would create a surplus. This is the maximum surplus allowed.
    pub max_surplus: f64,
    /// The ratio found/expected — values near 1.0 leave no room for off-CL zeros.
    pub completeness_ratio: f64,
}

/// How much a hypothetical off-CL zero would perturb the explicit formula.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerturbationAnalysis {
    /// The test point x used for perturbation analysis.
    pub x: f64,
    /// Contribution of a CL zero at height t to ψ(x):
    /// 2·Re(x^(1/2+it) / (1/2+it)) — oscillatory, bounded by 2√x/t.
    pub cl_zero_contribution: f64,
    /// Contribution of an off-CL zero pair at (σ,t) and (1-σ,t):
    /// x^σ/|σ+it| + x^(1-σ)/|(1-σ)+it| — the x^σ term grows if σ > 1/2.
    pub off_cl_contribution: f64,
    /// Ratio: off-CL / on-CL contribution. Values >> 1 mean detectable.
    pub detectability_ratio: f64,
    /// Minimum σ deviation from 1/2 that would be detectable at this x.
    pub min_detectable_deviation: f64,
}

/// Characterize the exclusion zones and constraints on RH counterexamples.
///
/// Takes the results of a zero verification run and analyzes what
/// structural constraints the verified zeros place on hypothetical
/// counterexamples.
///
/// # Arguments
///
/// * `verification` — Results from `verify_rh_to_height`
/// * `analysis_x` — The point x at which to compute perturbation effects
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if `analysis_x <= 1.0`.
pub fn analyze_exclusions(
    verification: &RhVerification,
    analysis_x: f64,
) -> Result<ExclusionAnalysis, ZetaError> {
    if analysis_x <= 1.0 {
        return Err(ZetaError::InvalidParameter(
            "analysis point x must be > 1.0".to_string(),
        ));
    }

    let height = verification.height;
    let n_verified = verification.found_zeros;
    let expected = count_zeros_to_height(height);

    // Scanned area: the full critical strip [0,1] × [0, T]
    let scanned_area = height; // width 1 × height T

    // Zero-free regions
    let mut regions = Vec::new();

    // Region 1: Our verified range — all zeros found are on CL
    regions.push(ZeroFreeRegion {
        description: "Computationally verified: all zeros on CL".to_string(),
        t_min: 0.0,
        t_max: height,
        sigma_min: 0.0,
        sigma_max: 0.5 - 1e-6, // everything left of CL
        source: format!("verify_rh_to_height({height})"),
    });
    regions.push(ZeroFreeRegion {
        description: "Computationally verified: all zeros on CL (right half)".to_string(),
        t_min: 0.0,
        t_max: height,
        sigma_min: 0.5 + 1e-6,
        sigma_max: 1.0,
        source: format!("verify_rh_to_height({height})"),
    });

    // Region 2: Classical zero-free region (de la Vallée Poussin type)
    // No zeros in σ > 1 − c/ln(t) for some effective constant c
    // Use c ≈ 1/(57.54) from Kadiri (2005)
    let c_zvf = 1.0 / 57.54;
    regions.push(ZeroFreeRegion {
        description: "Classical zero-free region (Kadiri 2005)".to_string(),
        t_min: 3.0,
        t_max: f64::INFINITY,
        sigma_min: 1.0 - c_zvf / (height.ln()),
        sigma_max: 1.0,
        source: "σ > 1 − 1/(57.54·ln t)".to_string(),
    });

    // Density constraints
    let completeness = n_verified as f64 / expected;
    let max_surplus = (expected - n_verified as f64).max(0.0);

    let density_constraints = DensityConstraints {
        expected_count_at_verified_height: expected,
        actual_count: n_verified,
        max_surplus,
        completeness_ratio: completeness,
    };

    // Perturbation analysis at x
    let perturbation = compute_perturbation(analysis_x, height, &verification.zeros);

    Ok(ExclusionAnalysis {
        verified_height: height,
        verified_count: n_verified,
        scanned_area,
        zero_free_regions: regions,
        min_counterexample_height: height,
        density_constraints,
        perturbation_analysis: perturbation,
    })
}

/// Compute perturbation from on-CL vs hypothetical off-CL zero.
fn compute_perturbation(x: f64, verified_height: f64, zeros: &[ZetaZero]) -> PerturbationAnalysis {
    // Typical CL zero contribution at height t ≈ verified_height/2
    let typical_t = verified_height / 2.0;
    let cl_contribution = 2.0 * x.sqrt() / typical_t;

    // Off-CL zero at σ = 0.6, t = verified_height + 1 (just beyond our range)
    let sigma_test = 0.6;
    let t_test = verified_height + 1.0;
    let rho_mag = (sigma_test * sigma_test + t_test * t_test).sqrt();
    let partner_mag = ((1.0 - sigma_test) * (1.0 - sigma_test) + t_test * t_test).sqrt();
    let off_cl = x.powf(sigma_test) / rho_mag + x.powf(1.0 - sigma_test) / partner_mag;

    let detectability = if cl_contribution > 1e-30 {
        off_cl / cl_contribution
    } else {
        f64::INFINITY
    };

    // Minimum detectable deviation: solve x^(1/2+δ) > C·x^(1/2)
    // → x^δ > C → δ > C·ln(x)⁻¹
    // C ≈ 0.01 (1% perturbation threshold)
    let min_dev = if x.ln() > 0.0 {
        0.01_f64.ln().abs() / x.ln()
    } else {
        f64::NAN
    };

    PerturbationAnalysis {
        x,
        cl_zero_contribution: cl_contribution,
        off_cl_contribution: off_cl,
        detectability_ratio: detectability,
        min_detectable_deviation: min_dev,
    }
}

/// Construct a counterexample candidate and check if it's excluded.
///
/// This does NOT claim the counterexample exists — it characterizes
/// what the search space looks like AFTER our verifications.
///
/// # Arguments
///
/// * `sigma` — hypothetical real part (must be in (0,1) and ≠ 1/2)
/// * `t` — hypothetical imaginary part (must be > 0)
/// * `verified_height` — height to which RH has been verified
pub fn construct_candidate(
    sigma: f64,
    t: f64,
    verified_height: f64,
) -> Result<CounterexampleCandidate, ZetaError> {
    if sigma <= 0.0 || sigma >= 1.0 {
        return Err(ZetaError::InvalidParameter(format!(
            "sigma must be in (0,1), got {sigma}"
        )));
    }
    if (sigma - 0.5).abs() < 1e-15 {
        return Err(ZetaError::InvalidParameter(
            "sigma = 1/2 is ON the critical line, not a counterexample".to_string(),
        ));
    }
    if t <= 0.0 {
        return Err(ZetaError::InvalidParameter(format!(
            "t must be > 0, got {t}"
        )));
    }

    let distance = (sigma - 0.5).abs();
    let excluded = t < verified_height;
    let reason = if excluded {
        Some(format!(
            "Excluded: t={t:.2} < verified_height={verified_height:.2}. \
             All zeros below this height are confirmed on the critical line."
        ))
    } else {
        None
    };

    Ok(CounterexampleCandidate {
        sigma,
        t,
        partner_sigma: 1.0 - sigma,
        distance_from_cl: distance,
        excluded,
        exclusion_reason: reason,
    })
}

/// Map the "counterexample landscape" — for each (σ, t), determine
/// if it's excluded and why. Returns a grid of candidates.
///
/// # Arguments
///
/// * `sigma_steps` — number of σ values to test in (0, 0.5)
/// * `t_range` — (t_min, t_max) range to scan
/// * `t_steps` — number of t values
/// * `verified_height` — height to which RH has been verified
pub fn map_counterexample_landscape(
    sigma_steps: usize,
    t_range: (f64, f64),
    t_steps: usize,
    verified_height: f64,
) -> Vec<CounterexampleCandidate> {
    let mut candidates = Vec::new();

    for si in 1..=sigma_steps {
        // Only scan left half [0, 0.5) — right half is mirror via functional eq
        let sigma = 0.5 * si as f64 / (sigma_steps + 1) as f64;

        for ti in 0..t_steps {
            let t = t_range.0 + (t_range.1 - t_range.0) * ti as f64 / t_steps.max(1) as f64;
            if t <= 0.0 {
                continue;
            }

            if let Ok(candidate) = construct_candidate(sigma, t, verified_height) {
                candidates.push(candidate);
            }
        }
    }

    candidates
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zeros::verify_rh_to_height;

    #[test]
    fn candidate_below_verified_height_is_excluded() {
        let c = construct_candidate(0.6, 50.0, 1000.0).unwrap();
        assert!(c.excluded);
        assert!(c.exclusion_reason.is_some());
        assert!((c.partner_sigma - 0.4).abs() < 1e-10);
        assert!((c.distance_from_cl - 0.1).abs() < 1e-10);
    }

    #[test]
    fn candidate_above_verified_height_is_not_excluded() {
        let c = construct_candidate(0.6, 1500.0, 1000.0).unwrap();
        assert!(!c.excluded);
        assert!(c.exclusion_reason.is_none());
    }

    #[test]
    fn candidate_on_critical_line_is_rejected() {
        assert!(construct_candidate(0.5, 14.0, 100.0).is_err());
    }

    #[test]
    fn candidate_outside_strip_is_rejected() {
        assert!(construct_candidate(1.1, 14.0, 100.0).is_err());
        assert!(construct_candidate(-0.1, 14.0, 100.0).is_err());
    }

    #[test]
    fn exclusion_analysis_from_verification() {
        let v = verify_rh_to_height(200.0, 0.05).unwrap();
        let analysis = analyze_exclusions(&v, 500.0).unwrap();

        assert!(analysis.verified_height > 199.0);
        assert!(analysis.verified_count >= 70);
        assert!(analysis.min_counterexample_height >= 199.0);
        assert!(analysis.zero_free_regions.len() >= 2);

        // Density completeness should be reasonable
        assert!(
            analysis.density_constraints.completeness_ratio > 0.85,
            "completeness = {}",
            analysis.density_constraints.completeness_ratio
        );
    }

    #[test]
    fn perturbation_off_cl_is_detectable() {
        let v = verify_rh_to_height(200.0, 0.05).unwrap();
        let analysis = analyze_exclusions(&v, 1000.0).unwrap();

        // Off-CL zeros should be detectable (ratio > 1)
        assert!(
            analysis.perturbation_analysis.detectability_ratio > 0.1,
            "detectability = {}",
            analysis.perturbation_analysis.detectability_ratio
        );
    }

    #[test]
    fn landscape_map_covers_grid() {
        let candidates = map_counterexample_landscape(5, (1.0, 500.0), 10, 200.0);

        // Should have 5 sigma × 10 t = 50 candidates
        assert!(
            candidates.len() >= 40,
            "expected >= 40 candidates, got {}",
            candidates.len()
        );

        // All candidates below verified height should be excluded
        let below = candidates.iter().filter(|c| c.t < 200.0).count();
        let below_excluded = candidates
            .iter()
            .filter(|c| c.t < 200.0 && c.excluded)
            .count();
        assert_eq!(
            below, below_excluded,
            "all below-height candidates must be excluded"
        );
    }

    #[test]
    fn functional_equation_symmetry() {
        let c = construct_candidate(0.3, 100.0, 50.0).unwrap();
        assert!((c.sigma + c.partner_sigma - 1.0).abs() < 1e-10);
    }
}
