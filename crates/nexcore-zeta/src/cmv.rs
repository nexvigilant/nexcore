//! # CMV Matrix Reconstruction
//!
//! Provides an alternative to Jacobi (tridiagonal) reconstruction via
//! CMV matrices — five-diagonal unitary matrices parameterized by
//! Verblunsky coefficients.
//!
//! CMV matrices are the unitary analogue of Jacobi matrices. They are
//! better conditioned for the zeta zero eigenvalue problem because:
//! - They operate on the unit circle rather than the real line
//! - Verblunsky coefficients lie strictly in the unit disk (|αₖ| < 1)
//! - The Szegő/Levinson recursion is numerically stable
//!
//! ## Algorithm
//!
//! 1. Sort eigenvalues t₁ < … < tₙ
//! 2. Map to unit circle: θₖ = 2π·(tₖ − tₘᵢₙ)/(tₘₐₓ − tₘᵢₙ)
//! 3. Compute moments: cₘ = (1/N) Σₖ e^{−imθₖ}
//! 4. Levinson-Durbin recursion → Verblunsky coefficients α₀, …, αₙ₋₂
//! 5. Reconstruct moments from coefficients and measure roundtrip fidelity
//! 6. Analyze coefficient structure

use serde::{Deserialize, Serialize};

use crate::error::ZetaError;
use crate::zeros::ZetaZero;

// ── Private complex arithmetic ────────────────────────────────────────────────

/// Minimal complex number used only within this module.
#[derive(Clone, Copy, Debug, Default)]
struct C64(f64, f64);

impl C64 {
    #[inline]
    fn conj(self) -> Self {
        Self(self.0, -self.1)
    }
    #[inline]
    fn abs_sq(self) -> f64 {
        self.0 * self.0 + self.1 * self.1
    }
    #[inline]
    fn abs(self) -> f64 {
        self.abs_sq().sqrt()
    }
    #[inline]
    fn arg(self) -> f64 {
        self.1.atan2(self.0)
    }
    #[inline]
    fn from_polar(r: f64, theta: f64) -> Self {
        Self(r * theta.cos(), r * theta.sin())
    }
}

impl std::ops::Add for C64 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl std::ops::Sub for C64 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl std::ops::Mul for C64 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self(
            self.0 * rhs.0 - self.1 * rhs.1,
            self.0 * rhs.1 + self.1 * rhs.0,
        )
    }
}

impl std::ops::Neg for C64 {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self {
        Self(-self.0, -self.1)
    }
}

impl std::ops::AddAssign for C64 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

// ── Public types ──────────────────────────────────────────────────────────────

/// A CMV matrix reconstruction from zeta zero eigenvalues.
///
/// CMV matrices are five-diagonal unitary matrices parameterized by
/// Verblunsky coefficients α₀, …, αₙ₋₂ where |αₖ| < 1. This struct
/// captures the coefficient sequence and its structural properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmvReconstruction {
    /// Magnitudes of Verblunsky coefficients: |αₖ| for k = 0, …, N-2.
    pub verblunsky_magnitudes: Vec<f64>,
    /// Phase angles of Verblunsky coefficients: arg(αₖ) in radians.
    pub verblunsky_phases: Vec<f64>,
    /// Input eigenvalues sorted in ascending order.
    pub eigenvalues: Vec<f64>,
    /// Relative L2 moment roundtrip error: ‖ĉ − c‖₂ / ‖c‖₂.
    ///
    /// Measures information preservation: how well the Verblunsky
    /// coefficients encode the original spectral measure.
    pub roundtrip_error: f64,
    /// Structural analysis of the Verblunsky coefficient sequence.
    pub structure: CmvStructure,
}

/// Structural analysis of the CMV Verblunsky coefficient sequence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmvStructure {
    /// Mean |αₖ| over all N−1 coefficients.
    pub mean_coefficient_magnitude: f64,
    /// Power-law decay exponent β where |αₖ| ~ A·k^(−β).
    ///
    /// Positive β indicates decaying coefficients (well-conditioned).
    pub coefficient_decay_rate: f64,
    /// Coefficient of variation of |αₖ|: std(|αₖ|) / mean(|αₖ|).
    ///
    /// Low value → uniform magnitude sequence (regular operator).
    pub coefficient_regularity: f64,
    /// Standard deviation of consecutive phase differences arg(αₖ₊₁) − arg(αₖ).
    ///
    /// Low value → smooth phase progression.
    pub phase_regularity: f64,
    /// Maximum |αₖ| over all coefficients — must be strictly < 1.
    pub max_coefficient: f64,
    /// Number of Verblunsky coefficients (= N − 1 for N input zeros).
    pub n: usize,
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Reconstruct a CMV matrix from zeta zero eigenvalues via Verblunsky coefficients.
///
/// Maps eigenvalues to the unit circle, computes the Verblunsky coefficients
/// via the Levinson-Durbin recursion on the resulting Hermitian Toeplitz system,
/// and analyzes the coefficient structure.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if fewer than 3 zeros are provided.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::cmv::reconstruct_cmv;
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 50.0, 0.1).unwrap();
/// let cmv = reconstruct_cmv(&zeros).unwrap();
/// assert!(cmv.verblunsky_magnitudes.iter().all(|&m| m < 1.0));
/// ```
pub fn reconstruct_cmv(zeros: &[ZetaZero]) -> Result<CmvReconstruction, ZetaError> {
    if zeros.len() < 3 {
        return Err(ZetaError::InvalidParameter(
            "need at least 3 zeros for CMV reconstruction".to_string(),
        ));
    }

    let n = zeros.len();

    // Step 1: sort eigenvalues
    let eigenvalues: Vec<f64> = {
        let mut ev: Vec<f64> = zeros.iter().map(|z| z.t).collect();
        ev.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        ev
    };

    // Step 2: map to unit-circle angles θₖ ∈ [0, 2π)
    let ev_min = eigenvalues[0];
    let ev_max = eigenvalues[n - 1];
    let span = ev_max - ev_min;
    let angles: Vec<f64> = if span < 1e-30 {
        // Degenerate: distribute uniformly
        (0..n)
            .map(|k| 2.0 * std::f64::consts::PI * k as f64 / n as f64)
            .collect()
    } else {
        eigenvalues
            .iter()
            .map(|&t| 2.0 * std::f64::consts::PI * (t - ev_min) / span)
            .collect()
    };

    // Step 3: compute moments cₘ = (1/N) Σₖ exp(−imθₖ) for m = 0..N
    let moments = compute_moments(&angles);

    // Step 4: Levinson-Durbin → Verblunsky coefficients α₀, …, αₙ₋₂
    let verblunsky = levinson_durbin(&moments);

    // Step 5: reconstruct moments from Verblunsky coefficients and measure error
    let reconstructed = reconstruct_moments_from_verblunsky(&verblunsky, n);
    let roundtrip_error = moment_roundtrip_error(&moments, &reconstructed);

    // Extract real outputs
    let verblunsky_magnitudes: Vec<f64> = verblunsky.iter().map(|a| a.abs()).collect();
    let verblunsky_phases: Vec<f64> = verblunsky.iter().map(|a| a.arg()).collect();

    let structure = analyze_cmv_structure(&verblunsky_magnitudes, &verblunsky_phases);

    Ok(CmvReconstruction {
        verblunsky_magnitudes,
        verblunsky_phases,
        eigenvalues,
        roundtrip_error,
        structure,
    })
}

// ── Internal helpers ──────────────────────────────────────────────────────────

/// Compute moments cₘ = (1/N) Σₖ exp(−imθₖ) for m = 0, …, N−1.
fn compute_moments(angles: &[f64]) -> Vec<C64> {
    let n = angles.len();
    let w = 1.0 / n as f64;
    (0..n)
        .map(|m| {
            let mut c = C64(0.0, 0.0);
            for &theta in angles {
                c += C64::from_polar(w, -(m as f64) * theta);
            }
            c
        })
        .collect()
}

/// Levinson-Durbin recursion on the Hermitian Toeplitz matrix of moments.
///
/// Returns N−1 Verblunsky (reflection) coefficients α₀, …, αₙ₋₂.
/// Each |αₖ| < 1 is guaranteed by the structure of discrete measures on
/// the unit circle; numerical breakdown is handled by early termination
/// with zero padding.
fn levinson_durbin(moments: &[C64]) -> Vec<C64> {
    let n = moments.len();
    if n < 2 {
        return vec![];
    }

    // AR filter a[1..=k] (1-indexed) at current order k
    let mut a: Vec<C64> = vec![C64(0.0, 0.0); n];
    let mut verblunsky: Vec<C64> = Vec::with_capacity(n - 1);

    // Prediction error power (real, positive)
    let mut p = moments[0].0; // c[0].re = 1.0

    for k in 0..n - 1 {
        // Prediction error: e = c[k+1] + Σ_{j=1}^{k} a[j]·c[k+1−j]
        let mut e = moments[k + 1];
        for j in 1..=k {
            e += a[j] * moments[k + 1 - j];
        }

        // Reflection coefficient α_k = −e / p
        let alpha = if p.abs() < 1e-30 {
            C64(0.0, 0.0)
        } else {
            C64(-e.0 / p, -e.1 / p)
        };

        // Levinson update: a_new[j] = a[j] + α_k · conj(a[k+1−j]) for j=1..k
        if k > 0 {
            let a_old: Vec<C64> = a[1..=k].to_vec();
            for j in 1..=k {
                // a[k+1−j] is a_old[k−j] (0-indexed: a_old[m] = a[m+1])
                let conj_prev = a_old[k - j].conj();
                a[j] = a_old[j - 1] + alpha * conj_prev;
            }
        }
        // Guard: if |α| ≥ 1 the recursion has broken down numerically.
        // Discard this coefficient and pad the remaining slots with zeros.
        let alpha_sq = alpha.abs_sq();
        if alpha_sq >= 1.0 || !alpha_sq.is_finite() {
            while verblunsky.len() < n - 1 {
                verblunsky.push(C64(0.0, 0.0));
            }
            return verblunsky;
        }

        a[k + 1] = alpha;
        verblunsky.push(alpha);

        p *= 1.0 - alpha_sq;
        if !p.is_finite() || p < 0.0 {
            while verblunsky.len() < n - 1 {
                verblunsky.push(C64(0.0, 0.0));
            }
            return verblunsky;
        }
    }

    verblunsky
}

/// Reconstruct moments from Verblunsky coefficients via the inverse Levinson
/// recursion (Yule-Walker equations).
///
/// Given α₀, …, αₙ₋₂, rebuilds the AR filter at each order and uses
/// ĉ[k] = −Σ_{j=1}^{k} a_k[j] · ĉ[k−j] to recover the moments.
fn reconstruct_moments_from_verblunsky(verblunsky: &[C64], n: usize) -> Vec<C64> {
    let mut c_hat = vec![C64(0.0, 0.0); n];
    c_hat[0] = C64(1.0, 0.0);

    if verblunsky.is_empty() || n < 2 {
        return c_hat;
    }

    let num_alpha = verblunsky.len().min(n - 1);
    let mut a: Vec<C64> = vec![C64(0.0, 0.0); n];

    for k in 1..=num_alpha {
        let alpha = verblunsky[k - 1];

        // Rebuild AR filter of order k from Verblunsky coefficients
        if k == 1 {
            a[1] = alpha;
        } else {
            // Save a[1..=k-1] before overwriting
            let a_old: Vec<C64> = a[1..k].to_vec();
            for j in 1..k {
                // a_new[j] = a_old[j] + α · conj(a_old[k−j])
                // a_old is 0-indexed: a_old[m] = a[m+1], so a[k−j] = a_old[k−j−1]
                let conj_prev = a_old[k - j - 1].conj();
                a[j] = a_old[j - 1] + alpha * conj_prev;
            }
            a[k] = alpha;
        }

        // Yule-Walker: ĉ[k] = −Σ_{j=1}^{k} a[j] · ĉ[k−j]
        let mut ck = C64(0.0, 0.0);
        for j in 1..=k {
            ck = ck - a[j] * c_hat[k - j];
        }
        c_hat[k] = ck;
    }

    c_hat
}

/// Relative L2 moment roundtrip error, ignoring m=0 (always 1.0 on both sides).
fn moment_roundtrip_error(original: &[C64], reconstructed: &[C64]) -> f64 {
    let n = original.len().min(reconstructed.len());
    if n < 2 {
        return 0.0;
    }

    let num_sq: f64 = (1..n)
        .map(|i| (original[i] - reconstructed[i]).abs_sq())
        .sum();
    let den_sq: f64 = (1..n).map(|i| original[i].abs_sq()).sum();

    if den_sq < 1e-30 {
        return if num_sq < 1e-30 { 0.0 } else { f64::INFINITY };
    }

    (num_sq / den_sq).sqrt()
}

/// Analyze the Verblunsky coefficient sequence for structural properties.
fn analyze_cmv_structure(magnitudes: &[f64], phases: &[f64]) -> CmvStructure {
    let n = magnitudes.len();

    let mean_mag = if n == 0 {
        0.0
    } else {
        magnitudes.iter().sum::<f64>() / n as f64
    };

    let max_mag = magnitudes.iter().copied().fold(0.0_f64, f64::max);

    // Coefficient of variation: std(|αₖ|) / mean(|αₖ|)
    let coeff_regularity = if n < 2 || mean_mag.abs() < 1e-30 {
        0.0
    } else {
        let var: f64 = magnitudes
            .iter()
            .map(|&m| (m - mean_mag) * (m - mean_mag))
            .sum::<f64>()
            / n as f64;
        var.sqrt() / mean_mag
    };

    // Power-law decay exponent β: |αₖ| ~ A·k^(−β)
    let decay_rate = fit_power_decay(magnitudes);

    // Phase regularity: std of consecutive phase differences
    let phase_regularity = if phases.len() < 2 {
        0.0
    } else {
        let diffs: Vec<f64> = phases.windows(2).map(|w| w[1] - w[0]).collect();
        let mean_d: f64 = diffs.iter().sum::<f64>() / diffs.len() as f64;
        let var: f64 = diffs
            .iter()
            .map(|&d| (d - mean_d) * (d - mean_d))
            .sum::<f64>()
            / diffs.len() as f64;
        var.sqrt()
    };

    CmvStructure {
        mean_coefficient_magnitude: mean_mag,
        coefficient_decay_rate: decay_rate,
        coefficient_regularity: coeff_regularity,
        phase_regularity,
        max_coefficient: max_mag,
        n,
    }
}

/// Fit |αₖ| ~ A·k^(−β) via log-log OLS regression.
///
/// Returns β (positive = decaying sequence). Skips k=0 (log undefined)
/// and any zero-magnitude entries.
fn fit_power_decay(magnitudes: &[f64]) -> f64 {
    let points: Vec<(f64, f64)> = magnitudes
        .iter()
        .enumerate()
        .skip(1) // k=0 excluded (log(0) undefined)
        .filter(|&(_, &m)| m > 1e-30)
        .map(|(k, &m)| ((k as f64).ln(), m.ln()))
        .collect();

    if points.len() < 2 {
        return 0.0;
    }

    let np = points.len() as f64;
    #[allow(clippy::suspicious_operation_groupings)]
    let (sum_x, sum_y, sum_xy, sum_x2) = points.iter().fold(
        (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
        |(sx, sy, sxy, sx2), &(x, y)| (sx + x, sy + y, sxy + x * y, sx2 + x * x),
    );

    #[allow(clippy::suspicious_operation_groupings)]
    let denom = np * sum_x2 - sum_x * sum_x;
    if denom.abs() < 1e-30 {
        return 0.0;
    }

    // slope of log(|αₖ|) vs log(k): negative slope → positive decay rate
    let slope = (np * sum_xy - sum_x * sum_y) / denom;
    -slope
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zeros::find_zeros_bracket;

    fn get_test_zeros() -> Vec<ZetaZero> {
        find_zeros_bracket(10.0, 200.0, 0.05).unwrap_or_default()
    }

    #[test]
    fn cmv_produces_valid_coefficient_lengths() {
        let zeros = get_test_zeros();
        let n = zeros.len();
        assert!(n >= 20, "expected >= 20 test zeros, got {n}");
        let cmv = reconstruct_cmv(&zeros).unwrap();
        assert_eq!(
            cmv.verblunsky_magnitudes.len(),
            n - 1,
            "expected {} magnitudes",
            n - 1
        );
        assert_eq!(
            cmv.verblunsky_phases.len(),
            n - 1,
            "expected {} phases",
            n - 1
        );
        assert_eq!(cmv.eigenvalues.len(), n);
    }

    #[test]
    fn verblunsky_coefficients_in_unit_disk() {
        let zeros = get_test_zeros();
        let cmv = reconstruct_cmv(&zeros).unwrap();
        for (k, &mag) in cmv.verblunsky_magnitudes.iter().enumerate() {
            assert!(mag >= 0.0, "α_{k} magnitude {mag:.6} is negative");
            assert!(
                mag < 1.0,
                "α_{k} magnitude {mag:.6} ≥ 1 — violates unit disk"
            );
        }
    }

    #[test]
    fn roundtrip_error_is_finite_and_bounded() {
        let zeros = get_test_zeros();
        let cmv = reconstruct_cmv(&zeros).unwrap();
        assert!(
            cmv.roundtrip_error.is_finite(),
            "roundtrip error is not finite: {}",
            cmv.roundtrip_error
        );
        assert!(
            cmv.roundtrip_error < 1.0,
            "roundtrip error {:.2e} ≥ 1.0 — poor conditioning",
            cmv.roundtrip_error
        );
        eprintln!(
            "CMV roundtrip error at n={}: {:.2e}",
            zeros.len(),
            cmv.roundtrip_error
        );
    }

    #[test]
    fn coefficient_decay_rate_is_finite() {
        let zeros = get_test_zeros();
        let cmv = reconstruct_cmv(&zeros).unwrap();
        assert!(
            cmv.structure.coefficient_decay_rate.is_finite(),
            "decay rate is not finite: {}",
            cmv.structure.coefficient_decay_rate
        );
        eprintln!(
            "CMV coefficient decay rate β = {:.4}",
            cmv.structure.coefficient_decay_rate
        );
    }

    #[test]
    fn rejects_too_few_zeros() {
        let zeros = vec![
            ZetaZero {
                ordinal: 1,
                t: 14.1,
                z_value: 0.0,
                on_critical_line: true,
            },
            ZetaZero {
                ordinal: 2,
                t: 21.0,
                z_value: 0.0,
                on_critical_line: true,
            },
        ];
        assert!(
            reconstruct_cmv(&zeros).is_err(),
            "expected error for < 3 zeros"
        );
    }

    #[test]
    fn structure_metrics_all_finite() {
        let zeros = get_test_zeros();
        let cmv = reconstruct_cmv(&zeros).unwrap();
        let s = &cmv.structure;
        assert!(
            s.mean_coefficient_magnitude.is_finite(),
            "mean_coefficient_magnitude not finite"
        );
        assert!(
            s.coefficient_decay_rate.is_finite(),
            "coefficient_decay_rate not finite"
        );
        assert!(s.max_coefficient.is_finite(), "max_coefficient not finite");
        assert!(s.n > 0, "n must be positive");
    }

    #[test]
    fn phase_regularity_is_finite() {
        let zeros = get_test_zeros();
        let cmv = reconstruct_cmv(&zeros).unwrap();
        assert!(
            cmv.structure.phase_regularity.is_finite(),
            "phase_regularity not finite: {}",
            cmv.structure.phase_regularity
        );
    }

    #[test]
    fn coefficient_regularity_is_nonnegative() {
        let zeros = get_test_zeros();
        let cmv = reconstruct_cmv(&zeros).unwrap();
        assert!(
            cmv.structure.coefficient_regularity >= 0.0,
            "coefficient_regularity {:.6} is negative",
            cmv.structure.coefficient_regularity
        );
    }
}
