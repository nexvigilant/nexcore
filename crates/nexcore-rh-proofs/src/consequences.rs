//! # Constructive Proofs of RH Consequences
//!
//! Things we **can** prove constructively — results that follow from PNT
//! (proven) or can be numerically verified at specific x values.
//!
//! ## What's Here
//!
//! - **PNT witnesses**: verify π(x) ~ x/ln(x) at concrete x
//! - **Chebyshev bounds**: unconditional bounds on π(x) (no RH needed)
//! - **Sharp PNT under RH**: numerically spot-check the sharpened error bound

use serde::{Deserialize, Serialize};

use stem_number_theory::primes::{prime_counting, sieve_of_eratosthenes};

use crate::error::RhProofError;
use crate::propositions::PrimeNumberTheorem;

// ============================================================================
// Logarithmic Integral
// ============================================================================

/// Compute the logarithmic integral Li(x) = ∫₂ˣ dt/ln(t) numerically.
///
/// Uses the trapezoidal rule with 1000 steps over [2+ε, x].
/// Returns 0.0 for x ≤ 2.
fn li(x: f64) -> f64 {
    if x <= 2.0 {
        return 0.0;
    }
    const STEPS: usize = 1000;
    let a = 2.000_001_f64; // avoid ln(2) ≠ 0 singularity
    let h = (x - a) / STEPS as f64;
    let mut sum = 0.0_f64;
    for i in 0..=STEPS {
        let t = a + i as f64 * h;
        let weight = if i == 0 || i == STEPS { 0.5 } else { 1.0 };
        sum += weight / t.ln();
    }
    sum * h
}

// ============================================================================
// PNT Witness
// ============================================================================

/// Verify the Prime Number Theorem at a specific x using exact prime counting.
///
/// Constructs a [`PrimeNumberTheorem`] by computing π(x) via sieve and
/// comparing to the classical estimate x/ln(x).
///
/// # Errors
///
/// Returns [`RhProofError::OutOfRange`] if x < 2.
///
/// # Examples
///
/// ```
/// use nexcore_rh_proofs::consequences::verify_pnt;
///
/// let w = verify_pnt(1000).unwrap();
/// assert_eq!(w.pi_x, 168);
/// assert!(w.relative_error < 0.15);
/// ```
pub fn verify_pnt(x: u64) -> Result<PrimeNumberTheorem, RhProofError> {
    if x < 2 {
        return Err(RhProofError::OutOfRange {
            context: format!("PNT requires x >= 2, got {x}"),
        });
    }
    let pi_x = prime_counting(x);
    PrimeNumberTheorem::new(x as f64, pi_x)
}

// ============================================================================
// Chebyshev Bounds
// ============================================================================

/// Witness for Chebyshev's unconditional bounds on π(x).
///
/// For x ≥ 25: 0.92 · x/ln(x) < π(x) < 1.25 · x/ln(x).
///
/// The upper coefficient 1.25 = 5/4 is Rosser's (1941) proven bound for all x ≥ 2.
/// The lower coefficient 0.92 holds for all x ≥ 25.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChebyshevBoundsWitness {
    /// The argument x.
    pub x: u64,
    /// Actual π(x).
    pub pi_x: u64,
    /// Lower bound: 0.92 · x/ln(x).
    pub lower: f64,
    /// Upper bound: 1.25 · x/ln(x) (Rosser 1941).
    pub upper: f64,
    /// True iff lower < π(x) < upper.
    pub satisfies: bool,
}

/// Verify Chebyshev's unconditional bounds at a specific x.
///
/// These bounds are proven unconditionally — no RH required.
///
/// # Errors
///
/// Returns [`RhProofError::OutOfRange`] if x < 25.
///
/// # Examples
///
/// ```
/// use nexcore_rh_proofs::consequences::verify_chebyshev_bounds;
///
/// let w = verify_chebyshev_bounds(1000).unwrap();
/// assert!(w.satisfies);
/// assert_eq!(w.pi_x, 168);
/// ```
pub fn verify_chebyshev_bounds(x: u64) -> Result<ChebyshevBoundsWitness, RhProofError> {
    if x < 25 {
        return Err(RhProofError::OutOfRange {
            context: format!("Chebyshev bounds require x >= 25, got {x}"),
        });
    }
    let xf = x as f64;
    let estimate = xf / xf.ln();
    let lower = 0.92 * estimate;
    let upper = 1.25 * estimate; // Rosser (1941): π(x) < 5x/(4 ln x) for x ≥ 2
    let pi_x = prime_counting(x);
    let pif = pi_x as f64;
    let satisfies = lower < pif && pif < upper;
    Ok(ChebyshevBoundsWitness {
        x,
        pi_x,
        lower,
        upper,
        satisfies,
    })
}

// ============================================================================
// Sharp PNT Bound (conditional on RH)
// ============================================================================

/// Witness for the sharpened PNT error bound predicted by RH.
///
/// Under RH, |π(x) − Li(x)| ≤ (1/(8π)) · √x · ln(x) for x ≥ 2657.
/// We cannot prove this without RH, but we can **verify** it numerically
/// at specific x values as a spot check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharpPntWitness {
    /// The argument x.
    pub x: u64,
    /// Actual π(x).
    pub pi_x: u64,
    /// Li(x) = ∫₂ˣ dt/ln(t) (numerical approximation).
    pub li_x: f64,
    /// |π(x) − Li(x)|.
    pub error: f64,
    /// (1/(8π)) · √x · ln(x) — the RH-predicted bound.
    pub rh_bound: f64,
    /// True iff error < rh_bound.
    pub satisfies: bool,
}

/// Verify the sharp PNT error bound predicted by RH at a specific x.
///
/// Uses numerical Li(x) computed by trapezoidal rule (~1000 steps).
///
/// # Errors
///
/// Returns [`RhProofError::OutOfRange`] if x < 2657.
///
/// # Examples
///
/// ```
/// use nexcore_rh_proofs::consequences::verify_sharp_pnt_bound;
///
/// let w = verify_sharp_pnt_bound(10_000).unwrap();
/// assert!(w.satisfies, "error={}, rh_bound={}", w.error, w.rh_bound);
/// ```
pub fn verify_sharp_pnt_bound(x: u64) -> Result<SharpPntWitness, RhProofError> {
    if x < 2657 {
        return Err(RhProofError::OutOfRange {
            context: format!("sharp PNT bound requires x >= 2657, got {x}"),
        });
    }
    let xf = x as f64;
    let pi_x = prime_counting(x);
    let li_x = li(xf);
    let error = (pi_x as f64 - li_x).abs();
    let rh_bound = xf.sqrt() * xf.ln() / (8.0 * std::f64::consts::PI);
    let satisfies = error < rh_bound;
    Ok(SharpPntWitness {
        x,
        pi_x,
        li_x,
        error,
        rh_bound,
        satisfies,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // π(10000) = 1229, x/ln(x) ≈ 1086 → relative error < 12%
    #[test]
    fn pnt_at_10000() {
        let w = verify_pnt(10_000).unwrap();
        assert_eq!(w.pi_x, 1229);
        assert!(
            w.relative_error < 0.12,
            "relative_error={}",
            w.relative_error
        );
    }

    #[test]
    fn pnt_at_1000() {
        let w = verify_pnt(1000).unwrap();
        assert_eq!(w.pi_x, 168);
    }

    #[test]
    fn pnt_rejects_small_x() {
        assert!(verify_pnt(1).is_err());
        assert!(verify_pnt(0).is_err());
    }

    #[test]
    fn chebyshev_holds_at_canonical_values() {
        for &x in &[25_u64, 100, 1000, 10_000] {
            let w = verify_chebyshev_bounds(x).unwrap();
            assert!(
                w.satisfies,
                "Chebyshev failed at x={x}: pi={}, lower={}, upper={}",
                w.pi_x, w.lower, w.upper
            );
        }
    }

    #[test]
    fn chebyshev_rejects_small_x() {
        assert!(verify_chebyshev_bounds(24).is_err());
        assert!(verify_chebyshev_bounds(1).is_err());
    }

    #[test]
    fn sharp_pnt_holds_at_10000() {
        let w = verify_sharp_pnt_bound(10_000).unwrap();
        assert!(w.satisfies, "error={}, rh_bound={}", w.error, w.rh_bound);
    }

    #[test]
    fn sharp_pnt_rejects_small_x() {
        assert!(verify_sharp_pnt_bound(2656).is_err());
    }

    #[test]
    fn li_is_close_to_pi_at_10000() {
        let w = verify_sharp_pnt_bound(10_000).unwrap();
        // Li(10000) should be close to 1229; difference < 5%
        let relative = (w.li_x - w.pi_x as f64).abs() / w.pi_x as f64;
        assert!(relative < 0.05, "Li/pi relative gap={relative}");
    }

    #[test]
    fn prime_counting_at_known_values() {
        // π(10) = 4, π(100) = 25, π(1000) = 168
        let primes_10 = sieve_of_eratosthenes(10);
        assert_eq!(primes_10.len(), 4);
    }
}
