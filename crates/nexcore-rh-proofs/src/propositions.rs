//! # Core RH Propositions as Rust Types
//!
//! In the Curry-Howard correspondence, propositions are types and proofs are
//! values. An **unproven** proposition is an **uninhabited** type — no value
//! of that type can be constructed.
//!
//! This module encodes the Riemann Hypothesis and related statements as Rust
//! types, giving us a type-level framework for reasoning about partial results
//! and known consequences.

use serde::{Deserialize, Serialize};

use crate::error::RhProofError;

// ============================================================================
// Uninhabited Proposition Types (Conjectures)
// ============================================================================

/// The Riemann Hypothesis as a type.
///
/// **Proposition:** Every non-trivial zero of ζ(s) has real part 1/2.
///
/// This type is **uninhabited** — no value of this type can be constructed in
/// Rust's type system without dependent types. It encodes the open conjecture.
/// A proof would be a function `fn prove_rh() -> RiemannHypothesis`, which
/// cannot exist until the conjecture is resolved.
///
/// We interact with RH through verified **partial results** (`RhVerifiedToHeight`,
/// `ZeroOnCriticalLine`) rather than inhabiting this type directly.
pub enum RiemannHypothesis {}

/// Proposition: RH implies sharpened PNT error bounds.
///
/// If RH is true then |π(x) − Li(x)| ≤ C·√x·ln(x) for some explicit C.
/// This type is uninhabited until RH is proven.
pub enum RhImpliesSharpPnt {}

/// Proposition: RH is equivalent to Robin's inequality for n ≥ 5041.
///
/// σ(n) < e^γ · n · ln(ln(n)) for all n ≥ 5041  ⟺  RH.
pub enum RobinsInequality {}

/// Proposition: RH is equivalent to |M(x)| = O(x^(1/2+ε)) for all ε > 0.
///
/// Where M(x) = Σ_{k=1}^{x} μ(k) is the Mertens function.
pub enum MertensBound {}

// ============================================================================
// Inhabited Witness Types (Verified Partial Results)
// ============================================================================

/// A verified zero of ζ(s) lying on the critical line Re(s) = 1/2.
///
/// Construction requires passing a precision threshold: the residual
/// |ζ(1/2 + it)| must be less than 0.01.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ZeroOnCriticalLine {
    /// Ordinal index of this zero (1-indexed, ordered by imaginary part).
    pub ordinal: u64,
    /// Imaginary part t of the zero s = 1/2 + it.
    pub t: f64,
    /// Verified real part — always 0.5 for zeros on the critical line.
    pub verified_re: f64,
    /// |ζ(1/2 + it)| — measures how close we are to an actual zero.
    pub residual: f64,
}

impl ZeroOnCriticalLine {
    /// Construct a verified critical-line zero witness.
    ///
    /// The `residual` must be < 0.01 to be accepted as a zero.
    ///
    /// # Errors
    ///
    /// Returns [`RhProofError::InsufficientPrecision`] if `residual >= 0.01`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_rh_proofs::propositions::ZeroOnCriticalLine;
    ///
    /// let z = ZeroOnCriticalLine::new(1, 14.134725, 1e-6).unwrap();
    /// assert!((z.t - 14.134725).abs() < 1e-6);
    /// assert!((z.verified_re - 0.5).abs() < f64::EPSILON);
    /// ```
    pub fn new(ordinal: u64, t: f64, residual: f64) -> Result<Self, RhProofError> {
        const THRESHOLD: f64 = 0.01;
        if residual.abs() >= THRESHOLD {
            return Err(RhProofError::InsufficientPrecision {
                residual,
                threshold: THRESHOLD,
            });
        }
        Ok(Self {
            ordinal,
            t,
            verified_re: 0.5,
            residual,
        })
    }

    /// Returns true if this witness represents a genuine zero (residual < ε).
    #[must_use]
    pub fn is_genuine_zero(&self, epsilon: f64) -> bool {
        self.residual.abs() < epsilon
    }
}

// ============================================================================
// RH Verified to Height
// ============================================================================

/// Asserts: all non-trivial zeros with 0 < Im(s) < `height` have Re(s) = 1/2.
///
/// This is a **partial result** — a finite certificate toward RH.  The record
/// as of 2024 stands at around T = 3×10^12 zeros verified by Platt & Trudgian.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RhVerifiedToHeight {
    /// Upper bound T: all zeros with Im(s) ∈ (0, T) are on the critical line.
    pub height: f64,
    /// Number of non-trivial zeros found in (0, T).
    pub zeros_verified: u64,
    /// Expected count from Gram's law / argument principle (approximate).
    pub zeros_expected: f64,
    /// True iff every located zero has Re(s) = 1/2.
    pub all_on_critical_line: bool,
}

impl RhVerifiedToHeight {
    /// Construct a verification certificate.
    ///
    /// # Errors
    ///
    /// Returns [`RhProofError::OutOfRange`] if `height <= 0.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_rh_proofs::propositions::RhVerifiedToHeight;
    ///
    /// let cert = RhVerifiedToHeight::new(100.0, 29, 28.7, true).unwrap();
    /// assert!(cert.all_on_critical_line);
    /// ```
    pub fn new(
        height: f64,
        zeros_verified: u64,
        zeros_expected: f64,
        all_on_critical_line: bool,
    ) -> Result<Self, RhProofError> {
        if height <= 0.0 {
            return Err(RhProofError::OutOfRange {
                context: format!("height must be positive, got {height}"),
            });
        }
        Ok(Self {
            height,
            zeros_verified,
            zeros_expected,
            all_on_critical_line,
        })
    }

    /// Returns the discrepancy between found and expected zeros.
    ///
    /// A value near zero is expected; large deviations indicate numerical issues.
    #[must_use]
    pub fn count_discrepancy(&self) -> f64 {
        (self.zeros_verified as f64 - self.zeros_expected).abs()
    }
}

// ============================================================================
// Prime Number Theorem Witness
// ============================================================================

/// A constructive witness for the Prime Number Theorem at a specific x.
///
/// PNT states: π(x) ~ x/ln(x) as x → ∞.  Unlike RH, PNT is **proven**.
/// We can construct genuine inhabitants of this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimeNumberTheorem {
    /// The argument x.
    pub x: f64,
    /// Actual prime counting function π(x).
    pub pi_x: u64,
    /// Classical PNT estimate x / ln(x).
    pub estimate: f64,
    /// Relative error |π(x) − x/ln(x)| / π(x).
    pub relative_error: f64,
}

impl PrimeNumberTheorem {
    /// Construct a PNT witness for a given x.
    ///
    /// # Errors
    ///
    /// Returns [`RhProofError::OutOfRange`] if `x < 2.0` (ln(x) undefined at x ≤ 1).
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_rh_proofs::propositions::PrimeNumberTheorem;
    ///
    /// let w = PrimeNumberTheorem::new(100.0, 25).unwrap();
    /// assert!(w.relative_error < 0.15);
    /// ```
    pub fn new(x: f64, pi_x: u64) -> Result<Self, RhProofError> {
        if x < 2.0 {
            return Err(RhProofError::OutOfRange {
                context: format!("PNT requires x >= 2, got {x}"),
            });
        }
        let estimate = x / x.ln();
        let relative_error = if pi_x == 0 {
            f64::INFINITY
        } else {
            (pi_x as f64 - estimate).abs() / pi_x as f64
        };
        Ok(Self {
            x,
            pi_x,
            estimate,
            relative_error,
        })
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_on_critical_line_rejects_high_residual() {
        let result = ZeroOnCriticalLine::new(1, 14.134, 0.05);
        assert!(result.is_err());
    }

    #[test]
    fn zero_on_critical_line_accepts_low_residual() {
        let z = ZeroOnCriticalLine::new(1, 14.134_725, 1e-6).unwrap();
        assert!((z.verified_re - 0.5).abs() < f64::EPSILON);
        assert_eq!(z.ordinal, 1);
    }

    #[test]
    fn rh_verified_to_height_rejects_non_positive() {
        assert!(RhVerifiedToHeight::new(0.0, 0, 0.0, true).is_err());
        assert!(RhVerifiedToHeight::new(-1.0, 0, 0.0, true).is_err());
    }

    #[test]
    fn rh_verified_to_height_count_discrepancy() {
        let v = RhVerifiedToHeight::new(100.0, 29, 28.7, true).unwrap();
        assert!(v.count_discrepancy() < 1.0);
    }

    #[test]
    fn pnt_witness_relative_error_at_1000() {
        // π(1000) = 168, x/ln(x) ≈ 144.8 — relative error < 14%
        let w = PrimeNumberTheorem::new(1000.0, 168).unwrap();
        assert!(
            w.relative_error < 0.15,
            "relative_error={}",
            w.relative_error
        );
    }

    #[test]
    fn pnt_witness_rejects_small_x() {
        assert!(PrimeNumberTheorem::new(1.0, 0).is_err());
    }
}
