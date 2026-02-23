//! # Absence Detection (Null Hypothesis Testing)
//!
//! **T1 Grounding**: ∅(Void) dominant — absence is itself a structured claim.
//! κ(Comparison) drives hypothesis testing (observed vs expected).
//! N(Quantity) governs event counts and observation windows.
//!
//! **PV Transfer**: "No signal detected" is itself a signal — the absence of
//! expected ADRs after drug approval is meaningful pharmacovigilance evidence.
//! When a drug has been on the market for N patient-years with zero reports of
//! an expected adverse event, that constitutes statistical evidence that the
//! event does not occur at the anticipated rate.
//!
//! ## Statistical Basis
//!
//! Under the null hypothesis H₀, rare-event counts follow a Poisson distribution.
//! Given expected count λ = `expected_count` and observed count k = `observed_count`:
//!
//! ```text
//! P(X ≤ k | λ) = Σ_{i=0}^{k} e^{-λ} · λ^i / i!
//! ```
//!
//! A small p-value means the observation is unlikely under H₀ — if we observed
//! far fewer events than expected, that is evidence the phenomenon is absent.
//!
//! ## Curry-Howard Connection
//!
//! Absence detection is the computational dual of the `Void` type in our proof
//! framework. Just as `Void` (⊥) has no inhabitants, a phenomenon with decisive
//! absence evidence has no credible observations.

use serde::{Deserialize, Serialize};
use nexcore_error::Error;

// ============================================================================
// ERROR TYPE
// ============================================================================

/// Errors produced by absence detection functions.
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum AbsenceError {
    /// The observation window must span a positive duration.
    #[error("observation window must be positive")]
    NonPositiveWindow,
    /// The expected count under the null hypothesis must be non-negative.
    #[error("expected count must be non-negative")]
    NegativeExpected,
    /// The significance level alpha must lie strictly inside (0, 1).
    #[error("alpha must be in (0, 1)")]
    InvalidAlpha,
}

// ============================================================================
// CORE DATA TYPES
// ============================================================================

/// Evidence collected about the potential absence of a phenomenon.
///
/// All three fields are required to run a null-hypothesis test. The
/// `observation_window` anchors N(Quantity) in time; `expected_count` and
/// `observed_count` provide the κ(Comparison) substrate for inference.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::absence::AbsenceEvidence;
///
/// // Drug on market for 24 months; expected 10 hepatotoxicity reports; none received.
/// let evidence = AbsenceEvidence {
///     observation_window: 24.0,
///     expected_count: 10.0,
///     observed_count: 0,
/// };
/// assert_eq!(evidence.observed_count, 0);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsenceEvidence {
    /// Duration of the observation period (any consistent time unit).
    pub observation_window: f64,
    /// Expected event count under the null hypothesis (λ in Poisson model).
    pub expected_count: f64,
    /// Actual observed event count during the observation period.
    pub observed_count: u64,
}

/// Ordinal classification of statistical evidence for absence.
///
/// Thresholds mirror the conventional p-value ladder used in hypothesis testing.
/// The ordering is meaningful: `Decisive > Strong > Moderate > Weak > Negligible`.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::absence::EvidenceStrength;
///
/// assert!(EvidenceStrength::Decisive > EvidenceStrength::Strong);
/// assert!(EvidenceStrength::Moderate > EvidenceStrength::Weak);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceStrength {
    /// p ≥ 0.10 — data are consistent with the null hypothesis.
    Negligible,
    /// 0.05 ≤ p < 0.10 — slight tension with the null hypothesis.
    Weak,
    /// 0.01 ≤ p < 0.05 — conventional significance threshold.
    Moderate,
    /// 0.001 ≤ p < 0.01 — strong departure from null hypothesis.
    Strong,
    /// p < 0.001 — overwhelming evidence against the null hypothesis.
    Decisive,
}

/// Full result of a Poisson null-hypothesis absence test.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::absence::{AbsenceEvidence, null_hypothesis_test};
///
/// let evidence = AbsenceEvidence {
///     observation_window: 12.0,
///     expected_count: 20.0,
///     observed_count: 0,
/// };
/// let result = null_hypothesis_test(&evidence).unwrap();
/// assert!(result.is_absent);
/// assert!(result.p_value < 0.001);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsenceTestResult {
    /// Poisson CDF p-value P(X ≤ observed | λ = expected).
    pub p_value: f64,
    /// Whether absence is considered statistically significant at α = 0.05.
    pub is_absent: bool,
    /// Confidence in absence: `1.0 - p_value`.
    pub confidence: f64,
    /// Qualitative classification of the evidence strength.
    pub evidence_strength: EvidenceStrength,
}

// ============================================================================
// POISSON CDF — LOG-SPACE IMPLEMENTATION
// ============================================================================

/// Compute `ln(k!)` using an iterative sum for exact small values.
///
/// For k up to a few thousand this is acceptably fast and exact; the
/// Stirling approximation would introduce error at low k.
fn ln_factorial(k: u64) -> f64 {
    // ln(0!) = ln(1) = 0; sum starts from 2
    let mut acc: f64 = 0.0;
    for i in 2..=k {
        acc += (i as f64).ln();
    }
    acc
}

/// Compute ln P(X = k) for X ~ Poisson(lambda) in log-space.
///
/// `ln P(X = k) = -λ + k · ln(λ) - ln(k!)`
///
/// Handles the degenerate case λ = 0 explicitly:
/// - P(X = 0 | λ = 0) = 1  → ln = 0
/// - P(X = k | λ = 0) = 0  → ln = -∞  (represented as `f64::NEG_INFINITY`)
fn ln_poisson_pmf(lambda: f64, k: u64) -> f64 {
    if lambda == 0.0 {
        return if k == 0 { 0.0 } else { f64::NEG_INFINITY };
    }
    (k as f64).mul_add(lambda.ln(), -lambda) - ln_factorial(k)
}

/// Compute the Poisson CDF: P(X ≤ k | λ).
///
/// Summation is performed in log-space and then exponentiated term-by-term
/// to avoid overflow for large λ or k.
///
/// # Strategy
///
/// We accumulate probabilities as linear values after exponentiating each
/// log-PMF term. The final sum is clamped to `[0.0, 1.0]` to guard against
/// any floating-point rounding artefacts.
fn poisson_cdf(lambda: f64, k: u64) -> f64 {
    if lambda == 0.0 {
        // Degenerate: all mass at 0.
        return 1.0;
    }

    let mut cumulative: f64 = 0.0;
    for i in 0..=k {
        let ln_prob = ln_poisson_pmf(lambda, i);
        if ln_prob > f64::NEG_INFINITY {
            cumulative += ln_prob.exp();
        }
    }

    cumulative.clamp(0.0, 1.0)
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Classify a p-value into an ordinal evidence strength.
fn classify_evidence(p_value: f64) -> EvidenceStrength {
    if p_value < 0.001 {
        EvidenceStrength::Decisive
    } else if p_value < 0.01 {
        EvidenceStrength::Strong
    } else if p_value < 0.05 {
        EvidenceStrength::Moderate
    } else if p_value < 0.10 {
        EvidenceStrength::Weak
    } else {
        EvidenceStrength::Negligible
    }
}

/// Perform a Poisson null-hypothesis test for absence of a phenomenon.
///
/// Under H₀, the observed count follows `Poisson(expected_count)`. The
/// p-value is `P(X ≤ observed | λ = expected)`. A small p-value means
/// the observed count is much lower than expected — evidence of absence.
///
/// # Errors
///
/// Returns [`AbsenceError::NonPositiveWindow`] if `observation_window ≤ 0`.
/// Returns [`AbsenceError::NegativeExpected`] if `expected_count < 0`.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::absence::{AbsenceEvidence, EvidenceStrength, null_hypothesis_test};
///
/// let evidence = AbsenceEvidence {
///     observation_window: 12.0,
///     expected_count: 5.0,
///     observed_count: 0,
/// };
/// let result = null_hypothesis_test(&evidence).unwrap();
/// // p-value = P(X <= 0 | lambda=5) = e^{-5} ≈ 0.0067
/// assert!(result.p_value < 0.01);
/// assert!(result.is_absent);
/// assert_eq!(result.evidence_strength, EvidenceStrength::Strong);
/// ```
pub fn null_hypothesis_test(evidence: &AbsenceEvidence) -> Result<AbsenceTestResult, AbsenceError> {
    if evidence.observation_window <= 0.0 {
        return Err(AbsenceError::NonPositiveWindow);
    }
    if evidence.expected_count < 0.0 {
        return Err(AbsenceError::NegativeExpected);
    }

    let p_value = poisson_cdf(evidence.expected_count, evidence.observed_count);
    let is_absent = p_value < 0.05;
    let confidence = 1.0 - p_value;
    let evidence_strength = classify_evidence(p_value);

    Ok(AbsenceTestResult {
        p_value,
        is_absent,
        confidence,
        evidence_strength,
    })
}

/// Compute a continuous absence score in `[0.0, 1.0]`.
///
/// The score measures how far the observed count is below the expected count:
///
/// ```text
/// score = clamp(1.0 - observed / expected, 0.0, 1.0)
/// ```
///
/// - `score = 1.0` → zero observations (maximally absent).
/// - `score = 0.0` → observed ≥ expected (not absent).
/// - `score = 0.5` → observed is half of expected.
///
/// When `expected = 0.0` the ratio is undefined; the function returns `0.0`
/// because there is no baseline against which absence can be measured.
///
/// # Errors
///
/// Returns [`AbsenceError::NonPositiveWindow`] if `window ≤ 0`.
/// Returns [`AbsenceError::NegativeExpected`] if `expected < 0`.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::absence::absence_score;
///
/// let score = absence_score(10.0, 0, 12.0).unwrap();
/// assert!((score - 1.0).abs() < 1e-10, "zero observed = fully absent");
///
/// let score = absence_score(10.0, 5, 12.0).unwrap();
/// assert!((score - 0.5).abs() < 1e-10, "half observed");
///
/// let score = absence_score(10.0, 15, 12.0).unwrap();
/// assert!((score - 0.0).abs() < 1e-10, "over-observed clamped to 0");
/// ```
pub fn absence_score(expected: f64, observed: u64, window: f64) -> Result<f64, AbsenceError> {
    if window <= 0.0 {
        return Err(AbsenceError::NonPositiveWindow);
    }
    if expected < 0.0 {
        return Err(AbsenceError::NegativeExpected);
    }

    if expected == 0.0 {
        // No expected events → no meaningful baseline for absence.
        return Ok(0.0);
    }

    let ratio = observed as f64 / expected;
    let score = (1.0 - ratio).clamp(0.0, 1.0);
    Ok(score)
}

/// Determine whether a phenomenon is meaningfully absent at significance level α.
///
/// Returns `true` when the Poisson p-value from [`null_hypothesis_test`] is
/// strictly less than `alpha`.
///
/// # Errors
///
/// Returns [`AbsenceError::NonPositiveWindow`] if `observation_window ≤ 0`.
/// Returns [`AbsenceError::NegativeExpected`] if `expected_count < 0`.
/// Returns [`AbsenceError::InvalidAlpha`] if `alpha` is not in `(0.0, 1.0)`.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::absence::{AbsenceEvidence, is_meaningfully_absent};
///
/// let evidence = AbsenceEvidence {
///     observation_window: 24.0,
///     expected_count: 10.0,
///     observed_count: 0,
/// };
///
/// // Absent at α = 0.05?
/// assert!(is_meaningfully_absent(&evidence, 0.05).unwrap());
///
/// // Absent at an impossibly tight α = 1e-10? (e^{-10} ≈ 4.5e-5 > 1e-10)
/// assert!(!is_meaningfully_absent(&evidence, 1e-10).unwrap());
/// ```
pub fn is_meaningfully_absent(
    evidence: &AbsenceEvidence,
    alpha: f64,
) -> Result<bool, AbsenceError> {
    if alpha <= 0.0 || alpha >= 1.0 {
        return Err(AbsenceError::InvalidAlpha);
    }
    let result = null_hypothesis_test(evidence)?;
    Ok(result.p_value < alpha)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Helper ------------------------------------------------------------------

    fn evidence(window: f64, expected: f64, observed: u64) -> AbsenceEvidence {
        AbsenceEvidence {
            observation_window: window,
            expected_count: expected,
            observed_count: observed,
        }
    }

    // --- null_hypothesis_test: basic semantics ------------------------------------

    #[test]
    fn test_zero_observed_high_expected_strong_absence() {
        // P(X=0 | λ=10) = e^{-10} ≈ 4.5e-5  →  Decisive
        let result = null_hypothesis_test(&evidence(12.0, 10.0, 0)).unwrap();
        assert!(result.p_value < 0.001);
        assert!(result.is_absent);
        assert_eq!(result.evidence_strength, EvidenceStrength::Decisive);
    }

    #[test]
    fn test_observed_equals_expected_not_absent() {
        // When observed == expected, p-value ≈ 0.5 (median of Poisson) — not absent.
        let result = null_hypothesis_test(&evidence(12.0, 5.0, 5)).unwrap();
        // P(X ≤ 5 | λ=5) ≈ 0.616 — well above 0.05
        assert!(!result.is_absent);
        assert_eq!(result.evidence_strength, EvidenceStrength::Negligible);
    }

    #[test]
    fn test_observed_greater_than_expected_not_absent() {
        // observed >> expected → p-value is high (near 1), definitely not absent
        let result = null_hypothesis_test(&evidence(12.0, 2.0, 20)).unwrap();
        assert!(!result.is_absent);
        assert!(result.p_value > 0.10);
        assert_eq!(result.evidence_strength, EvidenceStrength::Negligible);
    }

    #[test]
    fn test_zero_expected_zero_observed_no_absence() {
        // λ=0, k=0: P(X ≤ 0 | λ=0) = 1.0 — the null holds perfectly, no absence claim.
        let result = null_hypothesis_test(&evidence(12.0, 0.0, 0)).unwrap();
        assert!((result.p_value - 1.0).abs() < 1e-10);
        assert!(!result.is_absent);
        assert_eq!(result.evidence_strength, EvidenceStrength::Negligible);
    }

    #[test]
    fn test_very_high_expected_zero_observed_decisive() {
        // P(X=0 | λ=100) = e^{-100} ≈ 3.7e-44 — overwhelmingly decisive
        let result = null_hypothesis_test(&evidence(24.0, 100.0, 0)).unwrap();
        assert!(result.p_value < 0.001);
        assert!(result.is_absent);
        assert_eq!(result.evidence_strength, EvidenceStrength::Decisive);
    }

    #[test]
    fn test_confidence_is_complement_of_p_value() {
        let result = null_hypothesis_test(&evidence(12.0, 5.0, 0)).unwrap();
        let expected_confidence = 1.0 - result.p_value;
        assert!((result.confidence - expected_confidence).abs() < 1e-12);
    }

    // --- Evidence strength boundary tests ----------------------------------------

    #[test]
    fn test_evidence_strength_negligible_when_p_gte_010() {
        // λ=1, k=2: P(X ≤ 2 | λ=1) = e^{-1}(1 + 1 + 0.5) ≈ 0.92 → Negligible
        let result = null_hypothesis_test(&evidence(1.0, 1.0, 2)).unwrap();
        assert!(result.p_value >= 0.10);
        assert_eq!(result.evidence_strength, EvidenceStrength::Negligible);
    }

    #[test]
    fn test_evidence_strength_ordering_is_correct() {
        assert!(EvidenceStrength::Decisive > EvidenceStrength::Strong);
        assert!(EvidenceStrength::Strong > EvidenceStrength::Moderate);
        assert!(EvidenceStrength::Moderate > EvidenceStrength::Weak);
        assert!(EvidenceStrength::Weak > EvidenceStrength::Negligible);
    }

    #[test]
    fn test_classify_evidence_moderate_boundary() {
        // p = 0.04 → Moderate
        let strength = super::classify_evidence(0.04);
        assert_eq!(strength, EvidenceStrength::Moderate);
    }

    #[test]
    fn test_classify_evidence_weak_boundary() {
        // p = 0.07 → Weak
        let strength = super::classify_evidence(0.07);
        assert_eq!(strength, EvidenceStrength::Weak);
    }

    #[test]
    fn test_classify_evidence_strong_boundary() {
        // p = 0.005 → Strong
        let strength = super::classify_evidence(0.005);
        assert_eq!(strength, EvidenceStrength::Strong);
    }

    #[test]
    fn test_classify_evidence_decisive_boundary() {
        // p = 0.0005 → Decisive
        let strength = super::classify_evidence(0.0005);
        assert_eq!(strength, EvidenceStrength::Decisive);
    }

    // --- absence_score -----------------------------------------------------------

    #[test]
    fn test_absence_score_zero_observed_full_score() {
        let score = absence_score(10.0, 0, 12.0).unwrap();
        assert!((score - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_absence_score_half_observed() {
        let score = absence_score(10.0, 5, 12.0).unwrap();
        assert!((score - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_absence_score_observed_equals_expected_zero_score() {
        let score = absence_score(10.0, 10, 12.0).unwrap();
        assert!((score - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_absence_score_over_observed_clamped_to_zero() {
        // 15 observed / 10 expected → 1.0 - 1.5 = -0.5, clamped to 0.0
        let score = absence_score(10.0, 15, 12.0).unwrap();
        assert!((score - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_absence_score_zero_expected_returns_zero() {
        // No expected baseline → undefined ratio, returns 0.0
        let score = absence_score(0.0, 0, 12.0).unwrap();
        assert!((score - 0.0).abs() < 1e-10);
    }

    // --- is_meaningfully_absent --------------------------------------------------

    #[test]
    fn test_is_meaningfully_absent_true_when_p_lt_alpha() {
        // P(X=0 | λ=10) ≈ 4.5e-5 < 0.05
        let ev = evidence(12.0, 10.0, 0);
        assert!(is_meaningfully_absent(&ev, 0.05).unwrap());
    }

    #[test]
    fn test_is_meaningfully_absent_false_when_p_gte_alpha() {
        // P(X ≤ 5 | λ=5) ≈ 0.616 ≥ 0.05
        let ev = evidence(12.0, 5.0, 5);
        assert!(!is_meaningfully_absent(&ev, 0.05).unwrap());
    }

    // --- Error cases -------------------------------------------------------------

    #[test]
    fn test_error_non_positive_window_zero() {
        let ev = evidence(0.0, 5.0, 0);
        assert_eq!(
            null_hypothesis_test(&ev).unwrap_err(),
            AbsenceError::NonPositiveWindow
        );
    }

    #[test]
    fn test_error_non_positive_window_negative() {
        let ev = evidence(-1.0, 5.0, 0);
        assert_eq!(
            null_hypothesis_test(&ev).unwrap_err(),
            AbsenceError::NonPositiveWindow
        );
    }

    #[test]
    fn test_error_negative_expected() {
        let ev = evidence(12.0, -1.0, 0);
        assert_eq!(
            null_hypothesis_test(&ev).unwrap_err(),
            AbsenceError::NegativeExpected
        );
    }

    #[test]
    fn test_error_invalid_alpha_zero() {
        let ev = evidence(12.0, 5.0, 0);
        assert_eq!(
            is_meaningfully_absent(&ev, 0.0).unwrap_err(),
            AbsenceError::InvalidAlpha
        );
    }

    #[test]
    fn test_error_invalid_alpha_one() {
        let ev = evidence(12.0, 5.0, 0);
        assert_eq!(
            is_meaningfully_absent(&ev, 1.0).unwrap_err(),
            AbsenceError::InvalidAlpha
        );
    }

    #[test]
    fn test_error_absence_score_negative_window() {
        assert_eq!(
            absence_score(5.0, 0, -1.0).unwrap_err(),
            AbsenceError::NonPositiveWindow
        );
    }

    #[test]
    fn test_error_absence_score_negative_expected() {
        assert_eq!(
            absence_score(-1.0, 0, 12.0).unwrap_err(),
            AbsenceError::NegativeExpected
        );
    }

    // --- Small and large lambda edge cases ---------------------------------------

    #[test]
    fn test_small_lambda_zero_observed() {
        // λ=0.1, k=0: P(X=0 | λ=0.1) = e^{-0.1} ≈ 0.905 → not absent
        let result = null_hypothesis_test(&evidence(1.0, 0.1, 0)).unwrap();
        assert!(result.p_value > 0.10);
        assert!(!result.is_absent);
        assert_eq!(result.evidence_strength, EvidenceStrength::Negligible);
    }

    #[test]
    fn test_large_lambda_zero_observed_decisive() {
        // λ=100, k=0: P(X=0 | λ=100) = e^{-100} — effectively 0
        let result = null_hypothesis_test(&evidence(24.0, 100.0, 0)).unwrap();
        assert_eq!(result.evidence_strength, EvidenceStrength::Decisive);
        assert!(result.p_value < 1e-10);
    }

    #[test]
    fn test_observation_window_validated_in_absence_score() {
        assert_eq!(
            absence_score(5.0, 2, 0.0).unwrap_err(),
            AbsenceError::NonPositiveWindow
        );
    }
}
