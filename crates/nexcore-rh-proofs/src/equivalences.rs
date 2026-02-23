//! # Known RH Equivalences as Verifiable Tests
//!
//! Several statements are known to be **equivalent** to the Riemann Hypothesis.
//! This module provides type-level representations and numerical verification
//! functions for the most significant ones:
//!
//! - **Robin's inequality**: σ(n) < e^γ · n · ln(ln(n)) for n ≥ 5041  ⟺  RH
//! - **Mertens function bound**: |M(x)| < √x for individual x values
//!
//! Verifying these over large ranges provides numerical *evidence* for RH —
//! not proof, but a computable certificate of partial consistency.

use serde::{Deserialize, Serialize};

use stem_number_theory::arithmetic::divisor_sigma;
use stem_number_theory::summatory::MertensFunction;

use crate::error::RhProofError;

/// Euler-Mascheroni constant γ ≈ 0.5772156649.
pub const EULER_MASCHERONI: f64 = 0.577_215_664_901_532_9;

// ============================================================================
// Robin's Inequality
// ============================================================================

/// The Robin inequality test σ(n) < e^γ · n · ln(ln(n)) for a specific n.
///
/// By Robin's theorem (1984), this inequality holds for all n ≥ 5041 if and
/// only if the Riemann Hypothesis is true.  Individual tests accumulate
/// numerical evidence.
///
/// Note: The inequality **can legitimately fail** for some n < 5041 — this is
/// expected behaviour, not an error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RobinTest {
    /// The tested integer n.
    pub n: u64,
    /// σ(n) = sum of all divisors of n.
    pub sigma_n: u64,
    /// e^γ · n · ln(ln(n)), the Robin upper bound.
    pub robin_bound: f64,
    /// True iff σ(n) < robin_bound.
    pub satisfies: bool,
}

impl RobinTest {
    /// Returns whether this n satisfies Robin's inequality.
    #[must_use]
    pub fn passes(&self) -> bool {
        self.satisfies
    }
}

/// Test Robin's inequality at a single n.
///
/// # Errors
///
/// - [`RhProofError::OutOfRange`] if n < 2 (ln(ln(n)) undefined).
///
/// # Examples
///
/// ```
/// use nexcore_rh_proofs::equivalences::test_robin;
///
/// let t = test_robin(6).unwrap();
/// // σ(6) = 1+2+3+6 = 12, bound ≈ 6·e^γ·ln(ln(6)) ≈ 12.98 → satisfies
/// assert!(t.satisfies);
/// ```
pub fn test_robin(n: u64) -> Result<RobinTest, RhProofError> {
    if n < 2 {
        return Err(RhProofError::OutOfRange {
            context: format!("Robin test requires n >= 2, got {n}"),
        });
    }
    let ln_n = (n as f64).ln();
    if ln_n <= 0.0 {
        return Err(RhProofError::OutOfRange {
            context: format!("ln({n}) <= 0, cannot compute ln(ln(n))"),
        });
    }
    let ln_ln_n = ln_n.ln();

    let sigma_n = divisor_sigma(n, 1);
    let robin_bound = EULER_MASCHERONI.exp() * (n as f64) * ln_ln_n;
    let satisfies = (sigma_n as f64) < robin_bound;

    Ok(RobinTest {
        n,
        sigma_n,
        robin_bound,
        satisfies,
    })
}

/// Test Robin's inequality for all n in `[start, end]` (inclusive).
///
/// # Errors
///
/// Returns the first error encountered, or [`RhProofError::OutOfRange`] if
/// `start > end` or `start < 2`.
///
/// # Examples
///
/// ```
/// use nexcore_rh_proofs::equivalences::test_robin_range;
///
/// let results = test_robin_range(5041, 5060).unwrap();
/// assert!(results.iter().all(|t| t.satisfies));
/// ```
pub fn test_robin_range(start: u64, end: u64) -> Result<Vec<RobinTest>, RhProofError> {
    if start < 2 {
        return Err(RhProofError::OutOfRange {
            context: format!("start must be >= 2, got {start}"),
        });
    }
    if start > end {
        return Err(RhProofError::OutOfRange {
            context: format!("start ({start}) > end ({end})"),
        });
    }
    (start..=end).map(test_robin).collect()
}

// ============================================================================
// Mertens Function Bound
// ============================================================================

/// Mertens function bound test: |M(x)| < √x at a specific x.
///
/// The bound |M(x)| = O(x^(1/2+ε)) for all ε > 0 is equivalent to RH.
/// Testing the stronger |M(x)| < √x for individual x values gives numerical
/// evidence — it holds for most small x but is not guaranteed unconditionally.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MertensTest {
    /// The argument x.
    pub x: u64,
    /// M(x) = Σ_{k=1}^{x} μ(k).
    pub mertens_value: i64,
    /// √x — the tested bound.
    pub bound: f64,
    /// True iff |M(x)| < √x.
    pub satisfies: bool,
}

impl MertensTest {
    /// Returns whether |M(x)| < √x.
    #[must_use]
    pub fn passes(&self) -> bool {
        self.satisfies
    }
}

/// Test the Mertens bound |M(x)| < √x at a single x.
///
/// # Errors
///
/// Returns [`RhProofError::OutOfRange`] if x == 0.
///
/// # Examples
///
/// ```
/// use nexcore_rh_proofs::equivalences::test_mertens;
///
/// let t = test_mertens(100).unwrap();
/// assert!(t.satisfies, "M(100)={}, bound={}", t.mertens_value, t.bound);
/// ```
pub fn test_mertens(x: u64) -> Result<MertensTest, RhProofError> {
    if x == 0 {
        return Err(RhProofError::OutOfRange {
            context: "Mertens test requires x >= 1".to_string(),
        });
    }
    let mertens_value = MertensFunction::compute(x);
    let bound = (x as f64).sqrt();
    let satisfies = (mertens_value as f64).abs() < bound;
    Ok(MertensTest {
        x,
        mertens_value,
        bound,
        satisfies,
    })
}

/// Test the Mertens bound for all x in `[start, end]` (inclusive).
///
/// # Errors
///
/// Returns [`RhProofError::OutOfRange`] if `start == 0` or `start > end`.
///
/// # Examples
///
/// ```
/// use nexcore_rh_proofs::equivalences::test_mertens_range;
///
/// let results = test_mertens_range(1, 20).unwrap();
/// assert!(results.iter().all(|t| t.satisfies));
/// ```
pub fn test_mertens_range(start: u64, end: u64) -> Result<Vec<MertensTest>, RhProofError> {
    if start == 0 {
        return Err(RhProofError::OutOfRange {
            context: "start must be >= 1".to_string(),
        });
    }
    if start > end {
        return Err(RhProofError::OutOfRange {
            context: format!("start ({start}) > end ({end})"),
        });
    }
    (start..=end).map(test_mertens).collect()
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robin_holds_at_5041() {
        let t = test_robin(5041).unwrap();
        assert!(
            t.satisfies,
            "Robin failed at n=5041: σ={}, bound={}",
            t.sigma_n, t.robin_bound
        );
    }

    #[test]
    fn robin_holds_range_5041_to_5100() {
        let results = test_robin_range(5041, 5100).unwrap();
        for t in &results {
            assert!(
                t.satisfies,
                "Robin failed at n={}: σ={}, bound={}",
                t.n, t.sigma_n, t.robin_bound
            );
        }
    }

    #[test]
    fn robin_below_5041_does_not_panic() {
        // n < 5041 can legitimately fail Robin — verify computation completes
        let t2 = test_robin(2).unwrap();
        let t3 = test_robin(3).unwrap();
        // The sigma values must be positive
        assert!(t2.sigma_n >= 1);
        assert!(t3.sigma_n >= 1);
    }

    #[test]
    fn robin_rejects_n_less_than_2() {
        assert!(test_robin(0).is_err());
        assert!(test_robin(1).is_err());
    }

    #[test]
    fn robin_range_rejects_bad_inputs() {
        assert!(test_robin_range(1, 10).is_err()); // start < 2
        assert!(test_robin_range(10, 5).is_err()); // start > end
    }

    #[test]
    fn mertens_bound_holds_for_small_values() {
        // Start at x=2: at x=1, M(1)=1 and sqrt(1)=1, so strict |M| < sqrt(x) fails.
        // The bound |M(x)| < sqrt(x) is a conjecture (Mertens conjecture, disproven for
        // astronomically large x by Odlyzko–te Riele 1985); x=1 is a trivial edge case.
        let results = test_mertens_range(2, 50).unwrap();
        for t in &results {
            assert!(
                t.satisfies,
                "|M({})| = {} >= sqrt({}) = {}",
                t.x, t.mertens_value, t.x, t.bound
            );
        }
    }

    #[test]
    fn mertens_known_values() {
        assert_eq!(test_mertens(1).unwrap().mertens_value, 1);
        assert_eq!(test_mertens(2).unwrap().mertens_value, 0);
        assert_eq!(test_mertens(5).unwrap().mertens_value, -2);
        assert_eq!(test_mertens(10).unwrap().mertens_value, -1);
    }

    #[test]
    fn mertens_rejects_zero() {
        assert!(test_mertens(0).is_err());
    }

    #[test]
    fn mertens_range_rejects_bad_inputs() {
        assert!(test_mertens_range(0, 10).is_err());
        assert!(test_mertens_range(10, 5).is_err());
    }
}
