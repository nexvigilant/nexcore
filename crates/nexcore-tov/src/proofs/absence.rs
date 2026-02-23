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
//! ## Absence Score
//!
//! The absence score is `-log₁₀(p_value)`. Higher values indicate stronger
//! evidence of absence. The scale follows Jeffreys (1939):
//!
//! ```text
//! Score < 0.5  → Negligible
//! 0.5 ≤ score < 1.0 → Weak
//! 1.0 ≤ score < 1.5 → Moderate
//! 1.5 ≤ score < 2.0 → Strong
//! score ≥ 2.0  → Decisive
//! ```
//!
//! A p-value of 0.05 maps to score ≈ 1.30 (Moderate). A p-value of 0.001
//! maps to score = 3.0 (Decisive).
//!
//! ## Curry-Howard Connection
//!
//! Absence detection is the computational dual of the `Void` type in our proof
//! framework. Just as `Void` (⊥) has no inhabitants, a phenomenon with decisive
//! absence evidence has no credible observations.

use nexcore_error::Error;
use serde::{Deserialize, Serialize};

// ============================================================================
// ERROR TYPE
// ============================================================================

/// Errors produced by absence detection functions.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::absence::{AbsenceEvidence, AbsenceError, null_hypothesis_test};
///
/// let bad = AbsenceEvidence {
///     observation_window: -1.0,
///     expected_count: 5.0,
///     observed_count: 0,
/// };
/// assert_eq!(null_hypothesis_test(&bad).unwrap_err(), AbsenceError::NonPositiveWindow);
/// ```
#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum AbsenceError {
    /// The observation window must span a positive, finite duration.
    #[error("observation window must be positive and finite")]
    NonPositiveWindow,
    /// The expected count under the null hypothesis must be non-negative and finite.
    #[error("expected count must be non-negative and finite")]
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

/// Ordinal classification of statistical evidence for absence, using the
/// Jeffreys evidence scale expressed in terms of the absence score
/// (`-log₁₀(p_value)`).
///
/// The ordering is meaningful: `Decisive > Strong > Moderate > Weak > Negligible`.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::absence::EvidenceStrength;
///
/// assert!(EvidenceStrength::Decisive > EvidenceStrength::Strong);
/// assert!(EvidenceStrength::Strong  > EvidenceStrength::Moderate);
/// assert!(EvidenceStrength::Moderate > EvidenceStrength::Weak);
/// assert!(EvidenceStrength::Weak    > EvidenceStrength::Negligible);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum EvidenceStrength {
    /// Absence score < 0.5 — data are consistent with the null hypothesis.
    Negligible,
    /// 0.5 ≤ absence score < 1.0 — slight tension with the null hypothesis.
    Weak,
    /// 1.0 ≤ absence score < 1.5 — conventional significance threshold.
    Moderate,
    /// 1.5 ≤ absence score < 2.0 — strong departure from null hypothesis.
    Strong,
    /// Absence score ≥ 2.0 — overwhelming evidence against the null hypothesis.
    Decisive,
}

/// Full result of a Poisson null-hypothesis absence test.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::absence::{AbsenceEvidence, EvidenceStrength, null_hypothesis_test};
///
/// let evidence = AbsenceEvidence {
///     observation_window: 12.0,
///     expected_count: 20.0,
///     observed_count: 0,
/// };
/// let result = null_hypothesis_test(&evidence).unwrap();
/// assert!(result.is_absent);
/// assert!(result.p_value < 0.001);
/// assert_eq!(result.evidence_strength, EvidenceStrength::Decisive);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AbsenceTestResult {
    /// Poisson CDF p-value P(X ≤ observed | λ = expected).
    pub p_value: f64,
    /// Whether absence is considered statistically significant at α = 0.05.
    pub is_absent: bool,
    /// Confidence in absence: `1.0 - p_value`, clamped to `[0.0, 1.0]`.
    pub confidence: f64,
    /// Qualitative classification of the evidence strength via the absence score.
    pub evidence_strength: EvidenceStrength,
}

// ============================================================================
// POISSON CDF — LOG-SPACE IMPLEMENTATION
// ============================================================================

/// Compute `ln(k!)` using an iterative sum.
///
/// For k up to a few thousand this is acceptably fast and exact; Stirling's
/// approximation would introduce error at low k.
fn ln_factorial(k: u64) -> f64 {
    // ln(0!) = ln(1) = 0; the loop starts at i=2 to skip the no-op ln(1)=0.
    let mut acc: f64 = 0.0;
    for i in 2..=k {
        acc += (i as f64).ln();
    }
    acc
}

/// Compute `ln P(X = k)` for `X ~ Poisson(lambda)` in log-space.
///
/// `ln P(X = k) = k · ln(λ) - λ - ln(k!)`
///
/// Handles the degenerate case λ = 0 explicitly:
/// - P(X = 0 | λ = 0) = 1  → ln = 0
/// - P(X = k | λ = 0) = 0  → ln = -∞  (represented as `f64::NEG_INFINITY`)
fn ln_poisson_pmf(lambda: f64, k: u64) -> f64 {
    if lambda == 0.0 {
        return if k == 0 { 0.0 } else { f64::NEG_INFINITY };
    }
    // k*ln(λ) - λ - ln(k!)  — fused-multiply-add for numerical stability
    (k as f64).mul_add(lambda.ln(), -lambda) - ln_factorial(k)
}

/// Compute the Poisson CDF: P(X ≤ k | λ).
///
/// Summation is performed by exponentiating each log-PMF term individually and
/// accumulating. The final sum is clamped to `[0.0, 1.0]` to guard against any
/// floating-point rounding artefacts.
fn poisson_cdf(lambda: f64, k: u64) -> f64 {
    if lambda == 0.0 {
        // Degenerate Poisson: all probability mass sits at 0.
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
// INTERNAL HELPERS
// ============================================================================

/// Classify an absence score into an ordinal [`EvidenceStrength`].
///
/// The score is `-log₁₀(p_value)`. Thresholds follow the Jeffreys (1939)
/// evidence scale.
fn classify_from_score(score: f64) -> EvidenceStrength {
    if score >= 2.0 {
        EvidenceStrength::Decisive
    } else if score >= 1.5 {
        EvidenceStrength::Strong
    } else if score >= 1.0 {
        EvidenceStrength::Moderate
    } else if score >= 0.5 {
        EvidenceStrength::Weak
    } else {
        EvidenceStrength::Negligible
    }
}

/// Validate that a window value is positive and finite.
fn validate_window(window: f64) -> Result<(), AbsenceError> {
    if !window.is_finite() || window <= 0.0 {
        Err(AbsenceError::NonPositiveWindow)
    } else {
        Ok(())
    }
}

/// Validate that an expected count is non-negative and finite.
fn validate_expected(expected: f64) -> Result<(), AbsenceError> {
    if !expected.is_finite() || expected < 0.0 {
        Err(AbsenceError::NegativeExpected)
    } else {
        Ok(())
    }
}

// ============================================================================
// PUBLIC API
// ============================================================================

/// Perform a Poisson null-hypothesis test for absence of a phenomenon.
///
/// Under H₀, the observed count follows `Poisson(expected_count)`. The
/// p-value is `P(X ≤ observed | λ = expected)`. A small p-value means
/// the observed count is much lower than expected — evidence of absence.
///
/// Absence is declared (`is_absent = true`) when `p_value < 0.05`, which
/// corresponds to an absence score > 1.30 on the Jeffreys scale.
///
/// # Errors
///
/// - [`AbsenceError::NonPositiveWindow`] — if `observation_window` is ≤ 0, NaN, or infinite.
/// - [`AbsenceError::NegativeExpected`] — if `expected_count` is < 0, NaN, or infinite.
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
/// // P(X=0 | λ=5) = e^{-5} ≈ 0.0067  →  Strong evidence of absence
/// assert!(result.p_value < 0.01);
/// assert!(result.is_absent);
/// assert_eq!(result.evidence_strength, EvidenceStrength::Strong);
/// ```
#[must_use = "absence test result must be inspected to act on the finding"]
pub fn null_hypothesis_test(evidence: &AbsenceEvidence) -> Result<AbsenceTestResult, AbsenceError> {
    validate_window(evidence.observation_window)?;
    validate_expected(evidence.expected_count)?;

    let p_value = poisson_cdf(evidence.expected_count, evidence.observed_count);
    let is_absent = p_value < 0.05;
    let confidence = (1.0 - p_value).clamp(0.0, 1.0);
    let score = absence_score_from_p(p_value);
    let evidence_strength = classify_from_score(score);

    Ok(AbsenceTestResult {
        p_value,
        is_absent,
        confidence,
        evidence_strength,
    })
}

/// Compute the absence score: `-log₁₀(p_value)` for the given evidence.
///
/// The score is a non-negative real number where higher values indicate
/// stronger evidence that the phenomenon is absent:
///
/// ```text
/// absence_score = -log₁₀(P(X ≤ observed | λ = expected))
/// ```
///
/// Equivalently, a score of 2.0 corresponds to p = 0.01, a score of 3.0
/// corresponds to p = 0.001, and so on.
///
/// When `expected = 0.0`, the p-value is 1.0 (all mass at zero), so the
/// score is 0.0 — no baseline exists against which absence can be measured.
///
/// # Errors
///
/// - [`AbsenceError::NonPositiveWindow`] — if `window` is ≤ 0, NaN, or infinite.
/// - [`AbsenceError::NegativeExpected`] — if `expected` is < 0, NaN, or infinite.
///
/// # Example
///
/// ```
/// use nexcore_tov::proofs::absence::absence_score;
///
/// // Zero observations when 10 were expected → very high score
/// let score = absence_score(10.0, 0, 12.0).unwrap();
/// assert!(score > 2.0, "decisive evidence of absence");
///
/// // Observed equals expected → p ≈ 0.5 → score ≈ 0.3 (negligible)
/// let score = absence_score(5.0, 5, 12.0).unwrap();
/// assert!(score < 0.5, "negligible evidence");
/// ```
#[must_use = "absence score must be inspected to act on the finding"]
pub fn absence_score(expected: f64, observed: u64, window: f64) -> Result<f64, AbsenceError> {
    validate_window(window)?;
    validate_expected(expected)?;

    let p_value = poisson_cdf(expected, observed);
    Ok(absence_score_from_p(p_value))
}

/// Determine whether a phenomenon is meaningfully absent at significance level α.
///
/// Returns `true` when `P(X ≤ observed | λ = expected) < alpha`.
///
/// # Errors
///
/// - [`AbsenceError::NonPositiveWindow`] — if `observation_window` is ≤ 0, NaN, or infinite.
/// - [`AbsenceError::NegativeExpected`] — if `expected_count` is < 0, NaN, or infinite.
/// - [`AbsenceError::InvalidAlpha`] — if `alpha` is not strictly in `(0.0, 1.0)`.
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
/// // Absent at the conventional α = 0.05?
/// assert!(is_meaningfully_absent(&evidence, 0.05).unwrap());
///
/// // e^{-10} ≈ 4.5e-5, so absent even at α = 1e-4
/// assert!(is_meaningfully_absent(&evidence, 1e-4).unwrap());
///
/// // Not absent at an impossibly tight α = 1e-10 (p ≈ 4.5e-5 > 1e-10)
/// assert!(!is_meaningfully_absent(&evidence, 1e-10).unwrap());
/// ```
#[must_use = "absence determination must be inspected to act on the finding"]
pub fn is_meaningfully_absent(
    evidence: &AbsenceEvidence,
    alpha: f64,
) -> Result<bool, AbsenceError> {
    if !alpha.is_finite() || alpha <= 0.0 || alpha >= 1.0 {
        return Err(AbsenceError::InvalidAlpha);
    }
    let result = null_hypothesis_test(evidence)?;
    Ok(result.p_value < alpha)
}

// ============================================================================
// PRIVATE HELPER — SCORE FROM P-VALUE
// ============================================================================

/// Convert a p-value to the `-log₁₀(p)` absence score.
///
/// When `p_value` is effectively zero (≤ f64::MIN_POSITIVE), the log diverges;
/// we cap the score at a large but finite constant (50.0) to remain representable.
fn absence_score_from_p(p_value: f64) -> f64 {
    if p_value <= 0.0 {
        // p is numerically indistinguishable from zero: cap at 50 (corresponds
        // to p ≤ 10^{-50}, which is well beyond any physical measurement).
        return 50.0;
    }
    // p == 1.0 → -log₁₀(1) = 0.0 (no evidence)
    // Guard against p > 1.0 due to floating-point rounding.
    let p_clamped = p_value.min(1.0);
    -(p_clamped.log10())
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Helper -----------------------------------------------------------------

    fn ev(window: f64, expected: f64, observed: u64) -> AbsenceEvidence {
        AbsenceEvidence {
            observation_window: window,
            expected_count: expected,
            observed_count: observed,
        }
    }

    // --- null_hypothesis_test: basic semantics ----------------------------------

    #[test]
    fn test_zero_observed_high_expected_decisive() {
        // P(X=0 | λ=10) = e^{-10} ≈ 4.5e-5  →  score ≈ 4.35  →  Decisive
        let result = null_hypothesis_test(&ev(12.0, 10.0, 0)).unwrap();
        assert!(result.p_value < 0.001);
        assert!(result.is_absent);
        assert_eq!(result.evidence_strength, EvidenceStrength::Decisive);
    }

    #[test]
    fn test_observed_equals_expected_not_absent() {
        // P(X ≤ 5 | λ=5) ≈ 0.616 — well above 0.05, score ≈ 0.21 → Negligible
        let result = null_hypothesis_test(&ev(12.0, 5.0, 5)).unwrap();
        assert!(!result.is_absent);
        assert_eq!(result.evidence_strength, EvidenceStrength::Negligible);
    }

    #[test]
    fn test_observed_greater_than_expected_not_absent() {
        // observed >> expected → p ≈ 1.0, score ≈ 0  →  not absent
        let result = null_hypothesis_test(&ev(12.0, 2.0, 20)).unwrap();
        assert!(!result.is_absent);
        assert!(result.p_value > 0.10);
        assert_eq!(result.evidence_strength, EvidenceStrength::Negligible);
    }

    #[test]
    fn test_zero_expected_zero_observed_not_absent() {
        // λ=0, k=0: P(X ≤ 0 | λ=0) = 1.0 → score = 0.0 → Negligible, not absent.
        let result = null_hypothesis_test(&ev(12.0, 0.0, 0)).unwrap();
        assert!((result.p_value - 1.0).abs() < 1e-10);
        assert!(!result.is_absent);
        assert_eq!(result.evidence_strength, EvidenceStrength::Negligible);
    }

    #[test]
    fn test_very_high_expected_zero_observed_decisive() {
        // P(X=0 | λ=100) = e^{-100} ≈ 3.7e-44 — overwhelmingly decisive
        let result = null_hypothesis_test(&ev(24.0, 100.0, 0)).unwrap();
        assert!(result.p_value < 0.001);
        assert!(result.is_absent);
        assert_eq!(result.evidence_strength, EvidenceStrength::Decisive);
    }

    #[test]
    fn test_confidence_is_complement_of_p_value() {
        let result = null_hypothesis_test(&ev(12.0, 5.0, 0)).unwrap();
        let expected_confidence = (1.0 - result.p_value).clamp(0.0, 1.0);
        assert!((result.confidence - expected_confidence).abs() < 1e-12);
    }

    #[test]
    fn test_small_lambda_zero_observed_negligible() {
        // P(X=0 | λ=0.1) = e^{-0.1} ≈ 0.905 → score ≈ 0.04 → Negligible, not absent
        let result = null_hypothesis_test(&ev(1.0, 0.1, 0)).unwrap();
        assert!(result.p_value > 0.10);
        assert!(!result.is_absent);
        assert_eq!(result.evidence_strength, EvidenceStrength::Negligible);
    }

    #[test]
    fn test_large_lambda_zero_observed_decisive() {
        // P(X=0 | λ=100) is essentially 0 → score is huge → Decisive
        let result = null_hypothesis_test(&ev(24.0, 100.0, 0)).unwrap();
        assert_eq!(result.evidence_strength, EvidenceStrength::Decisive);
        assert!(result.p_value < 1e-10);
    }

    // --- Evidence strength ordering ---------------------------------------------

    #[test]
    fn test_evidence_strength_ordering_is_correct() {
        assert!(EvidenceStrength::Decisive > EvidenceStrength::Strong);
        assert!(EvidenceStrength::Strong > EvidenceStrength::Moderate);
        assert!(EvidenceStrength::Moderate > EvidenceStrength::Weak);
        assert!(EvidenceStrength::Weak > EvidenceStrength::Negligible);
    }

    // --- classify_from_score: boundary transitions ------------------------------

    #[test]
    fn test_score_below_0_5_is_negligible() {
        assert_eq!(classify_from_score(0.0), EvidenceStrength::Negligible);
        assert_eq!(classify_from_score(0.499), EvidenceStrength::Negligible);
    }

    #[test]
    fn test_score_at_0_5_is_weak() {
        assert_eq!(classify_from_score(0.5), EvidenceStrength::Weak);
        assert_eq!(classify_from_score(0.999), EvidenceStrength::Weak);
    }

    #[test]
    fn test_score_at_1_0_is_moderate() {
        assert_eq!(classify_from_score(1.0), EvidenceStrength::Moderate);
        assert_eq!(classify_from_score(1.499), EvidenceStrength::Moderate);
    }

    #[test]
    fn test_score_at_1_5_is_strong() {
        assert_eq!(classify_from_score(1.5), EvidenceStrength::Strong);
        assert_eq!(classify_from_score(1.999), EvidenceStrength::Strong);
    }

    #[test]
    fn test_score_at_2_0_is_decisive() {
        assert_eq!(classify_from_score(2.0), EvidenceStrength::Decisive);
        assert_eq!(classify_from_score(10.0), EvidenceStrength::Decisive);
    }

    // --- absence_score function -------------------------------------------------

    #[test]
    fn test_absence_score_zero_observed_high_expected_large() {
        // P(X=0 | λ=10) = e^{-10} ≈ 4.5e-5 → -log₁₀(4.5e-5) ≈ 4.35
        let score = absence_score(10.0, 0, 12.0).unwrap();
        assert!(
            score > 2.0,
            "zero observed / high expected must be decisive"
        );
    }

    #[test]
    fn test_absence_score_observed_at_expected_negligible() {
        // P(X≤5 | λ=5) ≈ 0.616 → -log₁₀(0.616) ≈ 0.21 → Negligible
        let score = absence_score(5.0, 5, 12.0).unwrap();
        assert!(
            score < 0.5,
            "score for observed≈expected must be negligible"
        );
    }

    #[test]
    fn test_absence_score_zero_expected_returns_zero() {
        // λ=0 → p=1.0 → -log₁₀(1.0) = 0.0
        let score = absence_score(0.0, 0, 12.0).unwrap();
        assert!((score - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_absence_score_increases_with_more_expected() {
        // Holding observed=0, increasing λ should increase the score
        let score_5 = absence_score(5.0, 0, 1.0).unwrap();
        let score_10 = absence_score(10.0, 0, 1.0).unwrap();
        assert!(score_10 > score_5);
    }

    // --- is_meaningfully_absent -------------------------------------------------

    #[test]
    fn test_is_absent_true_when_p_lt_alpha() {
        // P(X=0 | λ=10) ≈ 4.5e-5 < 0.05
        let ev_data = ev(12.0, 10.0, 0);
        assert!(is_meaningfully_absent(&ev_data, 0.05).unwrap());
    }

    #[test]
    fn test_is_absent_false_when_p_gte_alpha() {
        // P(X≤5 | λ=5) ≈ 0.616 ≥ 0.05
        let ev_data = ev(12.0, 5.0, 5);
        assert!(!is_meaningfully_absent(&ev_data, 0.05).unwrap());
    }

    #[test]
    fn test_is_absent_at_tight_alpha() {
        // P(X=0 | λ=10) ≈ 4.5e-5, so absent at α=1e-4 but not at α=1e-6
        let ev_data = ev(12.0, 10.0, 0);
        assert!(is_meaningfully_absent(&ev_data, 1e-4).unwrap());
        assert!(!is_meaningfully_absent(&ev_data, 1e-6).unwrap());
    }

    // --- Error cases: null_hypothesis_test -------------------------------------

    #[test]
    fn test_error_non_positive_window_zero() {
        assert_eq!(
            null_hypothesis_test(&ev(0.0, 5.0, 0)).unwrap_err(),
            AbsenceError::NonPositiveWindow
        );
    }

    #[test]
    fn test_error_non_positive_window_negative() {
        assert_eq!(
            null_hypothesis_test(&ev(-1.0, 5.0, 0)).unwrap_err(),
            AbsenceError::NonPositiveWindow
        );
    }

    #[test]
    fn test_error_nan_window() {
        assert_eq!(
            null_hypothesis_test(&ev(f64::NAN, 5.0, 0)).unwrap_err(),
            AbsenceError::NonPositiveWindow
        );
    }

    #[test]
    fn test_error_infinite_window() {
        assert_eq!(
            null_hypothesis_test(&ev(f64::INFINITY, 5.0, 0)).unwrap_err(),
            AbsenceError::NonPositiveWindow
        );
    }

    #[test]
    fn test_error_negative_expected() {
        assert_eq!(
            null_hypothesis_test(&ev(12.0, -1.0, 0)).unwrap_err(),
            AbsenceError::NegativeExpected
        );
    }

    #[test]
    fn test_error_nan_expected() {
        assert_eq!(
            null_hypothesis_test(&ev(12.0, f64::NAN, 0)).unwrap_err(),
            AbsenceError::NegativeExpected
        );
    }

    #[test]
    fn test_error_infinite_expected() {
        assert_eq!(
            null_hypothesis_test(&ev(12.0, f64::INFINITY, 0)).unwrap_err(),
            AbsenceError::NegativeExpected
        );
    }

    // --- Error cases: is_meaningfully_absent -----------------------------------

    #[test]
    fn test_error_invalid_alpha_zero() {
        assert_eq!(
            is_meaningfully_absent(&ev(12.0, 5.0, 0), 0.0).unwrap_err(),
            AbsenceError::InvalidAlpha
        );
    }

    #[test]
    fn test_error_invalid_alpha_one() {
        assert_eq!(
            is_meaningfully_absent(&ev(12.0, 5.0, 0), 1.0).unwrap_err(),
            AbsenceError::InvalidAlpha
        );
    }

    #[test]
    fn test_error_invalid_alpha_nan() {
        assert_eq!(
            is_meaningfully_absent(&ev(12.0, 5.0, 0), f64::NAN).unwrap_err(),
            AbsenceError::InvalidAlpha
        );
    }

    // --- Error cases: absence_score --------------------------------------------

    #[test]
    fn test_error_absence_score_negative_window() {
        assert_eq!(
            absence_score(5.0, 0, -1.0).unwrap_err(),
            AbsenceError::NonPositiveWindow
        );
    }

    #[test]
    fn test_error_absence_score_nan_window() {
        assert_eq!(
            absence_score(5.0, 0, f64::NAN).unwrap_err(),
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

    // --- absence_score_from_p boundary cases -----------------------------------

    #[test]
    fn test_score_from_p_one_is_zero() {
        // p = 1.0 → -log₁₀(1.0) = 0.0
        let s = absence_score_from_p(1.0);
        assert!((s - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_score_from_p_zero_is_capped() {
        // p = 0.0 → score is capped at 50.0
        let s = absence_score_from_p(0.0);
        assert!((s - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_score_from_p_0_001_is_3() {
        // p = 0.001 → -log₁₀(0.001) = 3.0
        let s = absence_score_from_p(0.001);
        assert!((s - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_score_from_p_0_01_is_2() {
        // p = 0.01 → -log₁₀(0.01) = 2.0
        let s = absence_score_from_p(0.01);
        assert!((s - 2.0).abs() < 1e-10);
    }
}
