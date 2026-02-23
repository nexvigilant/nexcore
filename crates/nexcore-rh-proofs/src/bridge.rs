//! # Bridge: `nexcore-zeta` → `nexcore-rh-proofs`
//!
//! Wires [`nexcore_zeta`] computation output into type-level proof certificates.
//!
//! The main entry point is [`build_certificate_from_zeta`], which drives a full
//! RH verification pass and assembles a [`NumericalCertificate`].
//!
//! ## CHIRALITY Notes
//!
//! Several fields look similar but are semantically distinct — mapping is
//! explicit and documented here:
//!
//! | Source (`nexcore_zeta`) | Target (`nexcore_rh_proofs`) | Notes |
//! |-------------------------|------------------------------|-------|
//! | `RhVerification::found_zeros: usize` | `RhVerifiedToHeight::zeros_verified: u64` | type widening |
//! | `RhVerification::expected_zeros: f64` | `RhVerifiedToHeight::zeros_expected: f64` | field rename |
//! | `ZetaZero::z_value: f64` | `ZeroOnCriticalLine::residual: f64` | `z_value.abs()` IS the residual |
//! | `ZetaZero::on_critical_line: bool` | (omitted) | always `true` by construction in `ZeroOnCriticalLine::new` |

use nexcore_zeta::zeros::verify_rh_to_height;
use nexcore_zeta::{RhVerification, ZetaZero};

use crate::certificates::{NumericalCertificate, build_certificate};
use crate::equivalences::{test_mertens_range, test_robin_range};
use crate::error::RhProofError;
use crate::propositions::{RhVerifiedToHeight, ZeroOnCriticalLine};

// ============================================================================
// Type Conversions
// ============================================================================

/// Convert a [`nexcore_zeta::RhVerification`] into a [`RhVerifiedToHeight`].
///
/// Field mapping (CHIRALITY — similar names, non-identical types/semantics):
/// - `found_zeros: usize` → `zeros_verified: u64` (widening cast)
/// - `expected_zeros: f64` → `zeros_expected: f64` (field rename only)
///
/// # Errors
///
/// Returns [`RhProofError::OutOfRange`] if `verification.height <= 0.0`.
pub fn convert_rh_verification(
    verification: &RhVerification,
) -> Result<RhVerifiedToHeight, RhProofError> {
    RhVerifiedToHeight::new(
        verification.height,
        verification.found_zeros as u64,
        verification.expected_zeros,
        verification.all_on_critical_line,
    )
}

/// Convert a slice of [`ZetaZero`] into verified [`ZeroOnCriticalLine`] witnesses.
///
/// CHIRALITY mapping:
/// - `z_value: f64` → `residual: f64` via `z_value.abs()` — Z(t) at the
///   zero position is the residual measuring how close we are to an exact zero.
/// - `on_critical_line: bool` — omitted; [`ZeroOnCriticalLine`] asserts this
///   by construction.
///
/// Zeros where `z_value.abs() >= 0.01` fail the precision threshold in
/// [`ZeroOnCriticalLine::new`] and are silently filtered from the output.
///
/// # Examples
///
/// ```
/// use nexcore_rh_proofs::bridge::convert_zeros;
/// use nexcore_zeta::ZetaZero;
///
/// let zeros = vec![
///     ZetaZero { ordinal: 1, t: 14.134725, z_value: 1e-6, on_critical_line: true },
///     ZetaZero { ordinal: 2, t: 21.022040, z_value: 0.05, on_critical_line: true },
/// ];
/// let verified = convert_zeros(&zeros);
/// // Second zero is filtered: residual 0.05 >= 0.01
/// assert_eq!(verified.len(), 1);
/// assert_eq!(verified[0].ordinal, 1);
/// ```
pub fn convert_zeros(zeta_zeros: &[ZetaZero]) -> Vec<ZeroOnCriticalLine> {
    zeta_zeros
        .iter()
        .filter_map(|z| ZeroOnCriticalLine::new(z.ordinal, z.t, z.z_value.abs()).ok())
        .collect()
}

// ============================================================================
// Main Entry Point
// ============================================================================

/// Build a [`NumericalCertificate`] from a fresh zeta computation.
///
/// Runs the full pipeline:
/// 1. [`verify_rh_to_height`] scans zeros up to `height` with step `step`
/// 2. Converts [`RhVerification`] → [`RhVerifiedToHeight`] (CHIRALITY checked)
/// 3. Runs Robin's inequality for n ∈ [5041, 6000]
/// 4. Runs Mertens bound for x ∈ [2, 500]
/// 5. Assembles and returns a [`NumericalCertificate`] — target confidence > 0.8
///
/// # Errors
///
/// - [`RhProofError::OutOfRange`] if `height <= 0.0`
/// - [`RhProofError::NumberTheory`] if the underlying zeta computation fails
///
/// # Examples
///
/// ```
/// use nexcore_rh_proofs::bridge::build_certificate_from_zeta;
///
/// let cert = build_certificate_from_zeta(40.0, 0.1).unwrap();
/// assert!(cert.confidence > 0.8);
/// ```
pub fn build_certificate_from_zeta(
    height: f64,
    step: f64,
) -> Result<NumericalCertificate, RhProofError> {
    if height <= 0.0 {
        return Err(RhProofError::OutOfRange {
            context: format!("height must be positive, got {height}"),
        });
    }

    // Phase 1: zeta computation
    let verification =
        verify_rh_to_height(height, step).map_err(|e| RhProofError::NumberTheory(e.to_string()))?;

    // Phase 2: type conversion (CHIRALITY checked — found_zeros:usize → u64)
    let rh_height = convert_rh_verification(&verification)?;

    // Phase 3: RH equivalences
    let robin_tests = test_robin_range(5041, 6000)?;
    let mertens_tests = test_mertens_range(2, 500)?;

    // Phase 4: assemble certificate
    build_certificate(&rh_height, &robin_tests, &mertens_tests)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_zeta::ZetaZero;

    #[test]
    fn bridge_produces_certificate() {
        let cert = build_certificate_from_zeta(40.0, 0.1);
        assert!(cert.is_ok(), "bridge failed: {:?}", cert.err());
        let c = cert.ok();
        assert!(c.is_some());
        if let Some(c) = c {
            assert!(c.zeros_found > 0, "expected zeros, got 0");
            assert!(c.all_on_critical_line);
        }
    }

    #[test]
    fn bridge_confidence_above_threshold() {
        let cert = build_certificate_from_zeta(40.0, 0.1);
        assert!(cert.is_ok(), "bridge failed: {:?}", cert.err());
        let c = cert.ok();
        assert!(c.is_some());
        if let Some(c) = c {
            assert!(
                c.confidence > 0.8,
                "confidence {:.3} did not exceed 0.8",
                c.confidence
            );
        }
    }

    #[test]
    fn bridge_converts_zeros() {
        let inputs = vec![
            ZetaZero {
                ordinal: 1,
                t: 14.134_725,
                z_value: 1e-6, // residual 1e-6 < 0.01 — passes
                on_critical_line: true,
            },
            ZetaZero {
                ordinal: 2,
                t: 21.022_040,
                z_value: 0.05, // residual 0.05 >= 0.01 — filtered
                on_critical_line: true,
            },
        ];
        let verified = convert_zeros(&inputs);
        assert_eq!(
            verified.len(),
            1,
            "expected 1 verified zero, got {}",
            verified.len()
        );
        assert_eq!(verified[0].ordinal, 1);
        assert!((verified[0].t - 14.134_725).abs() < 1e-6);
        assert!((verified[0].residual - 1e-6).abs() < 1e-9);
    }

    #[test]
    fn bridge_rejects_bad_height() {
        let result = build_certificate_from_zeta(-1.0, 0.1);
        assert!(result.is_err(), "expected error for negative height");

        let result_zero = build_certificate_from_zeta(0.0, 0.1);
        assert!(result_zero.is_err(), "expected error for zero height");
    }
}
