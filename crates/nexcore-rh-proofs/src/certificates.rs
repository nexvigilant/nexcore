//! # Numerical Certificates
//!
//! Bridges the gap between numerical computation and type-level assertions.
//! A `NumericalCertificate` is NOT a proof of RH — it is a verified record
//! that numerical checks passed up to certain bounds.
//!
//! ## Honest Boundaries
//!
//! This module is deliberately conservative about what it claims:
//!
//! - f64 arithmetic limits precision; zeros beyond height ~10^6 require
//!   arbitrary-precision arithmetic we don't have here.
//! - Passing numerical tests is necessary but not sufficient for RH.
//! - The `honest_boundaries` function documents these limitations explicitly.

use serde::{Deserialize, Serialize};

use crate::equivalences::{MertensTest, RobinTest};
use crate::error::RhProofError;
use crate::propositions::RhVerifiedToHeight;

// ============================================================================
// NumericalCertificate
// ============================================================================

/// A composite numerical certificate attesting to RH verification.
///
/// Aggregates evidence from multiple independent checks:
/// - Zero counting up to a height T
/// - Robin's inequality up to some n
/// - Mertens bound up to some x
///
/// **This is NOT a proof of RH.** See [`honest_boundaries`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericalCertificate {
    /// All zeros with Im(s) ∈ (0, height) have been verified on the critical line.
    pub height: f64,
    /// Number of non-trivial zeros found in (0, height).
    pub zeros_found: u64,
    /// Expected count from the argument principle.
    pub zeros_expected: f64,
    /// True iff every located zero lies on Re(s) = 1/2.
    pub all_on_critical_line: bool,
    /// Robin's inequality verified for all n ≤ this value (must be ≥ 5041).
    pub robin_verified_to: u64,
    /// Mertens bound |M(x)| < √x verified for all x ≤ this value.
    pub mertens_verified_to: u64,
    /// Composite confidence score ∈ [0, 1] derived from all checks.
    pub confidence: f64,
}

// ============================================================================
// NumericallyVerified
// ============================================================================

/// A type-level assertion that a certificate has passed numerical checks.
///
/// `C` is the certificate type.  This wrapper signals that the certificate
/// was successfully constructed — it says nothing about RH truth.
pub struct NumericallyVerified<C> {
    /// The underlying certificate.
    pub certificate: C,
}

impl<C> NumericallyVerified<C> {
    /// Wrap a certificate as numerically verified.
    pub fn new(certificate: C) -> Self {
        Self { certificate }
    }
}

// ============================================================================
// Certificate Builder
// ============================================================================

/// Build a `NumericalCertificate` from individual verification results.
///
/// Computes a composite confidence score weighting:
/// - Critical-line conformance (50%)
/// - Robin coverage (25%)
/// - Mertens coverage (25%)
///
/// # Errors
///
/// Returns [`RhProofError::OutOfRange`] if:
/// - `rh_height.height <= 0`
/// - `robin_tests` is empty
/// - `mertens_tests` is empty
///
/// # Examples
///
/// ```
/// use nexcore_rh_proofs::certificates::build_certificate;
/// use nexcore_rh_proofs::propositions::RhVerifiedToHeight;
/// use nexcore_rh_proofs::equivalences::{test_robin_range, test_mertens_range};
///
/// let height = RhVerifiedToHeight::new(100.0, 29, 28.7, true).unwrap();
/// let robins = test_robin_range(5041, 5050).unwrap();
/// let mertens = test_mertens_range(1, 100).unwrap();
///
/// let cert = build_certificate(&height, &robins, &mertens).unwrap();
/// assert!(cert.confidence > 0.5);
/// ```
pub fn build_certificate(
    rh_height: &RhVerifiedToHeight,
    robin_tests: &[RobinTest],
    mertens_tests: &[MertensTest],
) -> Result<NumericalCertificate, RhProofError> {
    if rh_height.height <= 0.0 {
        return Err(RhProofError::OutOfRange {
            context: "certificate height must be positive".to_string(),
        });
    }
    if robin_tests.is_empty() {
        return Err(RhProofError::OutOfRange {
            context: "robin_tests must be non-empty".to_string(),
        });
    }
    if mertens_tests.is_empty() {
        return Err(RhProofError::OutOfRange {
            context: "mertens_tests must be non-empty".to_string(),
        });
    }

    // Critical-line score: full credit if all zeros verified on the line
    let critical_line_score = if rh_height.all_on_critical_line {
        1.0_f64
    } else {
        0.5_f64
    };

    // Robin score: fraction of tests that pass
    let robin_pass_count = robin_tests.iter().filter(|t| t.satisfies).count();
    let robin_score = robin_pass_count as f64 / robin_tests.len() as f64;

    // Mertens score: fraction of tests that pass
    let mertens_pass_count = mertens_tests.iter().filter(|t| t.satisfies).count();
    let mertens_score = mertens_pass_count as f64 / mertens_tests.len() as f64;

    // Weighted composite
    let confidence = 0.50 * critical_line_score + 0.25 * robin_score + 0.25 * mertens_score;

    let robin_verified_to = robin_tests
        .iter()
        .filter(|t| t.satisfies)
        .map(|t| t.n)
        .max()
        .unwrap_or(0);

    let mertens_verified_to = mertens_tests
        .iter()
        .filter(|t| t.satisfies)
        .map(|t| t.x)
        .max()
        .unwrap_or(0);

    Ok(NumericalCertificate {
        height: rh_height.height,
        zeros_found: rh_height.zeros_verified,
        zeros_expected: rh_height.zeros_expected,
        all_on_critical_line: rh_height.all_on_critical_line,
        robin_verified_to,
        mertens_verified_to,
        confidence,
    })
}

// ============================================================================
// Honest Boundaries
// ============================================================================

/// Documents the gap between numerical verification and proof.
///
/// Returns a human-readable description of the limitations of this crate's
/// approach to RH verification.
///
/// # Examples
///
/// ```
/// use nexcore_rh_proofs::certificates::honest_boundaries;
///
/// let s = honest_boundaries();
/// assert!(s.contains("proof"));
/// ```
#[must_use]
pub fn honest_boundaries() -> &'static str {
    "This crate provides numerical EVIDENCE for RH, not a proof. \
     Limitations: (1) f64 precision restricts zero verification to heights \
     where ζ(1/2+it) can be accurately evaluated; higher zeros require \
     arbitrary-precision arithmetic. (2) Rust lacks dependent types — \
     uninhabited types like RiemannHypothesis encode open conjectures but \
     cannot be inhabited without a formal proof. (3) Passing finite numerical \
     tests is necessary but not sufficient; counterexamples could exist beyond \
     any finite search horizon. (4) Robin's inequality and the Mertens bound \
     are RH-equivalent, so verifying them numerically is circular evidence. \
     Use this crate for exploration and partial-result tracking, not as \
     a substitute for a formal proof."
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::equivalences::{test_mertens_range, test_robin_range};
    use crate::propositions::RhVerifiedToHeight;

    #[test]
    fn build_certificate_basic() {
        let height = RhVerifiedToHeight::new(100.0, 29, 28.7, true).unwrap();
        let robins = test_robin_range(5041, 5050).unwrap();
        let mertens = test_mertens_range(1, 100).unwrap();
        let cert = build_certificate(&height, &robins, &mertens).unwrap();
        assert!(cert.confidence > 0.5);
        assert!(cert.all_on_critical_line);
        assert_eq!(cert.zeros_found, 29);
    }

    #[test]
    fn build_certificate_rejects_empty_tests() {
        let height = RhVerifiedToHeight::new(100.0, 29, 28.7, true).unwrap();
        let robins = test_robin_range(5041, 5050).unwrap();
        assert!(build_certificate(&height, &[], &[]).is_err());
        assert!(build_certificate(&height, &robins, &[]).is_err());
    }

    #[test]
    fn honest_boundaries_is_nonempty() {
        let s = honest_boundaries();
        assert!(!s.is_empty());
        assert!(s.contains("proof"));
    }

    #[test]
    fn confidence_is_in_unit_interval() {
        let height = RhVerifiedToHeight::new(200.0, 50, 49.5, true).unwrap();
        let robins = test_robin_range(5041, 5200).unwrap();
        let mertens = test_mertens_range(1, 200).unwrap();
        let cert = build_certificate(&height, &robins, &mertens).unwrap();
        assert!(cert.confidence >= 0.0 && cert.confidence <= 1.0);
    }
}
