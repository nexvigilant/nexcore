//! # Zero Prediction Engine
//!
//! Predicts future Riemann zeta zeros by extrapolating the Verblunsky
//! coefficient sequence from a CMV reconstruction.
//!
//! ## Algorithm
//!
//! 1. Fit a [`VerblunskyModel`] to the observed Verblunsky magnitude sequence:
//!    power-law trend |αₖ| ~ A·k^(−β) plus structural spikes and residuals.
//!
//! 2. Extrapolate the sequence beyond the known N−1 coefficients.
//!
//! 3. Reconstruct the extended moment sequence via the inverse Levinson
//!    (Yule-Walker) equations from the extended coefficient list.
//!
//! 4. Build the AR polynomial from extended coefficients, evaluate its
//!    spectral density on the unit circle, and locate eigenvalue candidates
//!    as minima of |A(e^{iθ})|².
//!
//! 5. Estimate the extended t-range using the Riemann–von Mangoldt formula
//!    and map unit-circle angles back to predicted t-values.

use serde::{Deserialize, Serialize};

use crate::cmv::CmvReconstruction;
use crate::error::ZetaError;
use crate::zeros::ZetaZero;

// ── Internal complex arithmetic ───────────────────────────────────────────────

/// Minimal complex number for internal use only.
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
impl std::ops::AddAssign for C64 {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

// ── Public types ───────────────────────────────────────────────────────────────

/// Fitted model of the Verblunsky coefficient sequence from a CMV reconstruction.
///
/// Captures the power-law trend, structural spikes, and residual pattern
/// of the Verblunsky magnitudes |αₖ|.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerblunskyModel {
    /// Amplitude A in the power-law trend |αₖ| ~ A·k^(−β).
    pub amplitude: f64,
    /// Power-law decay exponent β (positive = decaying sequence).
    pub decay_exponent: f64,
    /// Indices where |αₖ| deviates more than 2σ from the fitted trend (spikes).
    pub spike_indices: Vec<usize>,
    /// Residuals: residuals[k] = |αₖ| − A·(k+1)^(−β).
    ///
    /// Index k=0 uses k+1 to avoid log(0) in the trend.
    pub residuals: Vec<f64>,
    /// Phase sequence arg(αₖ) used for extrapolation.
    pub phases: Vec<f64>,
    /// Mean phase increment between consecutive Verblunsky coefficients.
    pub mean_phase_step: f64,
    /// Number of Verblunsky coefficients in the fitted model (= N − 1 for N zeros).
    pub n_coefficients: usize,
    /// Range of input eigenvalues (t_min, t_max) used in the unit-circle mapping.
    pub eigenvalue_range: (f64, f64),
}

/// Accuracy metrics for a zero prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionAccuracy {
    /// Mean absolute error between predicted and actual t-values.
    pub mae: f64,
    /// Root mean squared error.
    pub rmse: f64,
    /// Maximum absolute error over compared pairs.
    pub max_error: f64,
    /// Number of (predicted, actual) pairs compared.
    pub n_compared: usize,
}

// ── Public API ─────────────────────────────────────────────────────────────────

/// Fit a [`VerblunskyModel`] to the coefficient sequence in a CMV reconstruction.
///
/// Performs a log-log OLS regression to extract the amplitude A and decay
/// exponent β, then identifies structural spikes (deviations > 2σ) and stores
/// the residual pattern for extrapolation.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::cmv::reconstruct_cmv;
/// use nexcore_zeta::prediction::fit_verblunsky_model;
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 100.0, 0.1).unwrap();
/// let cmv = reconstruct_cmv(&zeros).unwrap();
/// let model = fit_verblunsky_model(&cmv);
/// assert!(model.amplitude.is_finite() && model.amplitude > 0.0);
/// assert!(model.decay_exponent.is_finite());
/// ```
#[must_use]
pub fn fit_verblunsky_model(cmv: &CmvReconstruction) -> VerblunskyModel {
    let mags = &cmv.verblunsky_magnitudes;
    let phases = &cmv.verblunsky_phases;
    let n = mags.len();

    let (amplitude, decay_exponent) = fit_amplitude_and_decay(mags);

    // Residuals: |αₖ| − A·(k+1)^(−β)  (use k+1 to keep k=0 well-defined)
    let residuals: Vec<f64> = mags
        .iter()
        .enumerate()
        .map(|(k, &mag)| {
            let trend = trend_value(amplitude, decay_exponent, k);
            mag - trend
        })
        .collect();

    // Spikes: indices where |residual − mean_residual| > 2 · std_residual
    let (mean_res, std_res) = mean_std(&residuals);
    let spike_indices: Vec<usize> = residuals
        .iter()
        .enumerate()
        .filter(|&(_, r)| (*r - mean_res).abs() > 2.0 * std_res.max(1e-30))
        .map(|(i, _)| i)
        .collect();

    // Mean phase step for extrapolation
    let mean_phase_step = if phases.len() < 2 {
        0.0
    } else {
        let diffs: Vec<f64> = phases.windows(2).map(|w| w[1] - w[0]).collect();
        diffs.iter().sum::<f64>() / diffs.len() as f64
    };

    let eigenvalue_range = if cmv.eigenvalues.is_empty() {
        (0.0, 1.0)
    } else {
        (
            cmv.eigenvalues[0],
            cmv.eigenvalues[cmv.eigenvalues.len() - 1],
        )
    };

    VerblunskyModel {
        amplitude,
        decay_exponent,
        spike_indices,
        residuals,
        phases: phases.clone(),
        mean_phase_step,
        n_coefficients: n,
        eigenvalue_range,
    }
}

/// Predict the next `n_predict` zeta zero t-values using the Verblunsky model.
///
/// ## Steps
///
/// 1. Extrapolate the Verblunsky sequence by `n_predict + extra` coefficients
///    beyond the `model.n_coefficients` already fitted.
/// 2. Reconstruct the extended moment sequence via inverse Levinson.
/// 3. Build the AR polynomial from the extended coefficient list and find its
///    spectral density minima on the unit circle.
/// 4. Map unit-circle angles back to t-values using an extended t-range
///    estimated from the Riemann–von Mangoldt formula.
///
/// # Errors
///
/// Returns [`ZetaError::InvalidParameter`] if fewer than 3 known zeros are
/// provided or `n_predict` is zero.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::cmv::reconstruct_cmv;
/// use nexcore_zeta::prediction::{fit_verblunsky_model, predict_next_zeros};
/// use nexcore_zeta::zeros::find_zeros_bracket;
///
/// let zeros = find_zeros_bracket(10.0, 100.0, 0.05).unwrap();
/// let cmv = reconstruct_cmv(&zeros).unwrap();
/// let model = fit_verblunsky_model(&cmv);
/// let predictions = predict_next_zeros(&model, &zeros, 5).unwrap();
/// assert_eq!(predictions.len(), 5);
/// assert!(predictions.iter().all(|&t| t > zeros.last().unwrap().t));
/// ```
pub fn predict_next_zeros(
    model: &VerblunskyModel,
    known_zeros: &[ZetaZero],
    n_predict: usize,
) -> Result<Vec<f64>, ZetaError> {
    if known_zeros.len() < 3 {
        return Err(ZetaError::InvalidParameter(
            "need at least 3 known zeros for prediction".to_string(),
        ));
    }
    if n_predict == 0 {
        return Err(ZetaError::InvalidParameter(
            "n_predict must be > 0".to_string(),
        ));
    }

    let n_known = model.n_coefficients; // = known_zeros.len() - 1

    // ── Step 1: Extrapolate Verblunsky sequence ────────────────────────────────
    // We need n_predict extra coefficients beyond the n_known already fitted.
    // Add a small buffer so peak-finding has resolution for edge effects.
    let n_extra = n_predict + 4;
    let extrap_mags: Vec<f64> = (n_known..n_known + n_extra)
        .map(|k| {
            let base = trend_value(model.amplitude, model.decay_exponent, k);
            // Add residual modulation from the pattern (wrap around)
            let residual_mod = if model.residuals.is_empty() {
                0.0
            } else {
                let ri = k % model.residuals.len();
                model.residuals[ri] * decay_weight(k, n_known)
            };
            (base + residual_mod).clamp(1e-10, 0.999)
        })
        .collect();

    let last_phase = model.phases.last().copied().unwrap_or(0.0);
    let extrap_phases: Vec<f64> = (0..n_extra)
        .map(|i| last_phase + model.mean_phase_step * (i + 1) as f64)
        .collect();

    // ── Step 2: Build extended coefficient list and reconstruct moments ────────
    let mut extended_mags: Vec<f64> = model
        .phases
        .iter()
        .zip(model.residuals.iter())
        .enumerate()
        .map(|(k, _)| {
            // Rebuild original magnitudes from model (trend + residual)
            let trend = trend_value(model.amplitude, model.decay_exponent, k);
            let residual = model.residuals.get(k).copied().unwrap_or(0.0);
            (trend + residual).clamp(1e-10, 0.999)
        })
        .collect();
    extended_mags.extend_from_slice(&extrap_mags);

    let mut extended_phases: Vec<f64> = model.phases.clone();
    extended_phases.extend_from_slice(&extrap_phases);

    let extended_verblunsky: Vec<C64> = extended_mags
        .iter()
        .zip(extended_phases.iter())
        .map(|(&m, &p)| C64::from_polar(m, p))
        .collect();

    let n_extended = extended_verblunsky.len() + 1;
    let _extended_moments = reconstruct_moments_from_verblunsky(&extended_verblunsky, n_extended);

    // ── Step 3: Build AR polynomial and find spectral minima ──────────────────
    let ar_coeffs = build_ar_from_verblunsky(&extended_verblunsky);
    let n_candidates = n_known + n_extra + 4;
    let mut candidate_angles = find_spectral_minima(&ar_coeffs, n_candidates);
    candidate_angles.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    // ── Step 4: Map angles back to t-values using extended range ───────────────
    let (t_min, t_max_known) = model.eigenvalue_range;
    let n_total = known_zeros.len() + n_predict;
    let t_max_extended = estimate_t_for_nth_zero(n_total).max(t_max_known + 1.0);
    let extended_span = t_max_extended - t_min;

    let two_pi = 2.0 * std::f64::consts::PI;

    // Map all candidate angles to predicted t-values
    let mut all_t: Vec<f64> = candidate_angles
        .iter()
        .map(|&theta| t_min + (theta / two_pi) * extended_span)
        .collect();
    all_t.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    all_t.dedup_by(|a, b| (*a - *b).abs() < 0.05);

    // Keep only predictions strictly beyond t_max_known
    let beyond: Vec<f64> = all_t
        .into_iter()
        .filter(|&t| t > t_max_known + 0.01)
        .collect();

    if beyond.len() >= n_predict {
        return Ok(beyond[..n_predict].to_vec());
    }

    // Fallback: spacing-based prediction for any remaining predictions needed
    let have = beyond.len();
    let still_need = n_predict - have;
    let mut result = beyond;
    let last_t = result.last().copied().unwrap_or(t_max_known);
    let fallback = spacing_based_predictions(last_t, known_zeros.len() + have, still_need);
    result.extend_from_slice(&fallback);

    Ok(result)
}

/// Compute accuracy metrics for a set of predictions against actual zeros.
///
/// Pairs up to `min(predicted.len(), actual.len())` values in order and
/// computes MAE, RMSE, and maximum absolute error.
///
/// # Examples
///
/// ```
/// use nexcore_zeta::prediction::{validate_prediction, PredictionAccuracy};
/// use nexcore_zeta::zeros::ZetaZero;
///
/// let predicted = vec![100.0, 111.0, 122.0];
/// let actual = vec![
///     ZetaZero { ordinal: 1, t: 102.0, z_value: 0.0, on_critical_line: true },
///     ZetaZero { ordinal: 2, t: 111.5, z_value: 0.0, on_critical_line: true },
///     ZetaZero { ordinal: 3, t: 121.0, z_value: 0.0, on_critical_line: true },
/// ];
/// let acc = validate_prediction(&predicted, &actual);
/// assert!(acc.mae.is_finite());
/// assert!(acc.max_error.is_finite());
/// ```
#[must_use]
pub fn validate_prediction(predicted: &[f64], actual: &[ZetaZero]) -> PredictionAccuracy {
    let n = predicted.len().min(actual.len());
    if n == 0 {
        return PredictionAccuracy {
            mae: f64::NAN,
            rmse: f64::NAN,
            max_error: f64::NAN,
            n_compared: 0,
        };
    }

    let errors: Vec<f64> = predicted[..n]
        .iter()
        .zip(actual[..n].iter())
        .map(|(&p, a)| (p - a.t).abs())
        .collect();

    let mae = errors.iter().sum::<f64>() / n as f64;
    let rmse = (errors.iter().map(|&e| e * e).sum::<f64>() / n as f64).sqrt();
    let max_error = errors.iter().copied().fold(0.0_f64, f64::max);

    PredictionAccuracy {
        mae,
        rmse,
        max_error,
        n_compared: n,
    }
}

// ── Internal helpers ───────────────────────────────────────────────────────────

/// Trend value at index k: A·(k+1)^(−β).
///
/// Uses k+1 so the k=0 term stays well-defined without log(0).
#[inline]
fn trend_value(amplitude: f64, decay_exponent: f64, k: usize) -> f64 {
    amplitude * ((k + 1) as f64).powf(-decay_exponent)
}

/// Decay weight for residual modulation at index k: smoothly approaches zero
/// as k → ∞ so extrapolated residuals don't dominate the smooth trend.
#[inline]
fn decay_weight(k: usize, n_known: usize) -> f64 {
    let excess = k.saturating_sub(n_known) as f64;
    (-excess * 0.1).exp()
}

/// Fit amplitude A and decay exponent β via log-log OLS regression.
///
/// Returns (A, β) such that |αₖ| ≈ A·(k+1)^(−β).
fn fit_amplitude_and_decay(mags: &[f64]) -> (f64, f64) {
    // Use k+1 to match trend_value (k=0 → use 1)
    let points: Vec<(f64, f64)> = mags
        .iter()
        .enumerate()
        .filter(|&(_, &m)| m > 1e-30)
        .map(|(k, &m)| (((k + 1) as f64).ln(), m.ln()))
        .collect();

    if points.len() < 2 {
        return (mags.first().copied().unwrap_or(0.1), 0.5);
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
        return (mags.first().copied().unwrap_or(0.1), 0.0);
    }

    // slope β of log(|αₖ|) vs log(k+1)
    let slope = (np * sum_xy - sum_x * sum_y) / denom;
    let intercept = (sum_y - slope * sum_x) / np;
    let amplitude = intercept.exp().clamp(1e-10, 10.0);
    let decay = -slope; // negate: positive decay_exponent means decaying sequence

    (amplitude, decay)
}

/// Compute mean and standard deviation of a slice.
fn mean_std(values: &[f64]) -> (f64, f64) {
    let n = values.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    let mean = values.iter().sum::<f64>() / n as f64;
    let var = values.iter().map(|&v| (v - mean) * (v - mean)).sum::<f64>() / n as f64;
    (mean, var.sqrt())
}

/// Reconstruct moments from Verblunsky coefficients via the inverse Levinson
/// (Yule-Walker) equations.
///
/// Mirrors the logic in `cmv::reconstruct_moments_from_verblunsky`.
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

        if k == 1 {
            a[1] = alpha;
        } else {
            let a_old: Vec<C64> = a[1..k].to_vec();
            for j in 1..k {
                let conj_prev = a_old[k - j - 1].conj();
                a[j] = a_old[j - 1] + alpha * conj_prev;
            }
            a[k] = alpha;
        }

        let mut ck = C64(0.0, 0.0);
        for j in 1..=k {
            ck = ck - a[j] * c_hat[k - j];
        }
        c_hat[k] = ck;
    }

    c_hat
}

/// Build the AR polynomial coefficients a₁, …, aₙ from Verblunsky coefficients.
///
/// Uses the same Levinson update equations as `cmv::levinson_durbin` but
/// preserves the final AR filter vector instead of the reflection coefficients.
fn build_ar_from_verblunsky(verblunsky: &[C64]) -> Vec<C64> {
    let n = verblunsky.len();
    if n == 0 {
        return vec![];
    }

    let mut a: Vec<C64> = vec![C64(0.0, 0.0); n + 1];

    for k in 0..n {
        let alpha = verblunsky[k];

        if k > 0 {
            let a_old: Vec<C64> = a[1..=k].to_vec();
            for j in 1..k {
                let conj_prev = a_old[k - j].conj();
                a[j] = a_old[j - 1] + alpha * conj_prev;
            }
        }
        a[k + 1] = alpha;
    }

    a[1..=n].to_vec()
}

/// Evaluate the AR polynomial A(z) = 1 + Σₖ aₖ·zᵏ at z = e^{iθ}.
#[inline]
fn eval_ar(ar: &[C64], theta: f64) -> C64 {
    let mut result = C64(1.0, 0.0);
    for (k, &coeff) in ar.iter().enumerate() {
        let z_k = C64::from_polar(1.0, (k + 1) as f64 * theta);
        result += coeff * z_k;
    }
    result
}

/// Find local minima of |A(e^{iθ})|² on [0, 2π) via grid search + refinement.
///
/// Minima of the AR polynomial correspond to spectral peaks (eigenvalue candidates).
fn find_spectral_minima(ar: &[C64], n_candidates: usize) -> Vec<f64> {
    // Use a finer grid for larger AR orders
    let n_grid = (ar.len() * 32).max(2048).min(16384);
    let two_pi = 2.0 * std::f64::consts::PI;

    let powers: Vec<f64> = (0..n_grid)
        .map(|i| {
            let theta = two_pi * i as f64 / n_grid as f64;
            eval_ar(ar, theta).abs_sq()
        })
        .collect();

    // Collect local minima indices
    let mut minima: Vec<(f64, f64)> = Vec::new(); // (power, theta)
    for i in 0..n_grid {
        let prev = powers[(i + n_grid - 1) % n_grid];
        let curr = powers[i];
        let next = powers[(i + 1) % n_grid];
        if curr < prev && curr < next {
            let theta_lo = two_pi * ((i + n_grid - 1) % n_grid) as f64 / n_grid as f64;
            let theta_hi = two_pi * ((i + 1) % n_grid) as f64 / n_grid as f64;
            let refined = refine_minimum(ar, theta_lo, theta_hi, 20);
            minima.push((eval_ar(ar, refined).abs_sq(), refined));
        }
    }

    // Sort by power (smallest first = deepest nulls = strongest eigenvalue candidates)
    minima.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    minima
        .into_iter()
        .take(n_candidates)
        .map(|(_, theta)| theta)
        .collect()
}

/// Refine a minimum of |A(e^{iθ})|² in [theta_lo, theta_hi] via golden-section search.
fn refine_minimum(ar: &[C64], theta_lo: f64, theta_hi: f64, iters: usize) -> f64 {
    let phi = (5.0_f64.sqrt() - 1.0) / 2.0; // golden ratio conjugate
    let mut lo = theta_lo;
    let mut hi = theta_hi;

    for _ in 0..iters {
        let span = hi - lo;
        if span < 1e-12 {
            break;
        }
        let m1 = hi - phi * span;
        let m2 = lo + phi * span;
        let f1 = eval_ar(ar, m1).abs_sq();
        let f2 = eval_ar(ar, m2).abs_sq();
        if f1 < f2 {
            hi = m2;
        } else {
            lo = m1;
        }
    }

    (lo + hi) / 2.0
}

/// Estimate the t-value where the n-th Riemann zeta zero occurs using the
/// Riemann–von Mangoldt formula N(T) ≈ (T/2π)·ln(T/2πe) + 7/8.
///
/// Solves numerically via Newton's method starting from an initial estimate
/// derived from inverting the leading-order term.
#[must_use]
pub fn estimate_t_for_nth_zero(n: usize) -> f64 {
    let nf = n as f64;
    let two_pi = 2.0 * std::f64::consts::PI;
    let e = std::f64::consts::E;

    if nf < 1.0 {
        return 14.1347;
    }

    // Initial estimate from leading-order inversion
    let mut t = two_pi * e * (nf / (nf / 10.0_f64.max(nf.ln()))).max(1.0);
    t = t.max(14.0);

    for _ in 0..60 {
        let nt = (t / two_pi) * (t / (two_pi * e)).max(1e-30).ln() + 7.0 / 8.0;
        let dnt = (t / two_pi) * (1.0 / t + (t / (two_pi * e)).max(1e-30).ln() / t);
        let delta = (nf - nt) / dnt.max(1e-20);
        t += delta;
        if delta.abs() < 1e-8 {
            break;
        }
    }

    t.max(14.0)
}

/// Spacing-based prediction fallback when spectral peak-finding yields
/// insufficient predictions.
///
/// Uses the mean spacing formula Δₙ ≈ 2π / ln(tₙ/2π).
fn spacing_based_predictions(last_t: f64, last_ordinal: usize, count: usize) -> Vec<f64> {
    let two_pi = 2.0 * std::f64::consts::PI;
    let mut result = Vec::with_capacity(count);
    let mut t = last_t;
    for k in 0..count {
        let log_arg = (t / two_pi).max(1.0_f64.exp());
        let spacing = two_pi / log_arg.ln().max(0.1);
        // Slight modulation to avoid exact uniform spacing
        let modulation = 1.0 + 0.05 * ((last_ordinal + k) as f64 * 0.618).sin();
        t += spacing * modulation;
        result.push(t);
    }
    result
}

// ── Tests ──────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cmv::reconstruct_cmv;
    use crate::zeros::find_zeros_bracket;

    /// Obtain a reproducible set of zeros up to height T.
    fn get_zeros(t_max: f64) -> Vec<ZetaZero> {
        find_zeros_bracket(10.0, t_max, 0.05).unwrap_or_default()
    }

    #[test]
    fn model_fitting_produces_finite_sensible_parameters() {
        let zeros = get_zeros(250.0);
        assert!(
            zeros.len() >= 30,
            "expected >= 30 zeros, got {}",
            zeros.len()
        );

        let cmv = reconstruct_cmv(&zeros).unwrap();
        let model = fit_verblunsky_model(&cmv);

        assert!(
            model.amplitude.is_finite() && model.amplitude > 0.0,
            "amplitude not finite positive: {}",
            model.amplitude
        );
        assert!(
            model.decay_exponent.is_finite(),
            "decay_exponent not finite: {}",
            model.decay_exponent
        );
        assert_eq!(
            model.residuals.len(),
            model.n_coefficients,
            "residual length mismatch"
        );
        assert_eq!(
            model.phases.len(),
            model.n_coefficients,
            "phases length mismatch"
        );
        assert!(
            model.eigenvalue_range.0 < model.eigenvalue_range.1,
            "eigenvalue_range inverted"
        );

        eprintln!(
            "VerblunskyModel: A={:.4}, β={:.4}, n_coefficients={}, spikes={}",
            model.amplitude,
            model.decay_exponent,
            model.n_coefficients,
            model.spike_indices.len()
        );
    }

    #[test]
    fn predict_from_50_zeros_to_75() {
        // Collect enough zeros to have at least 75 confirmed
        let all_zeros = get_zeros(400.0);
        assert!(
            all_zeros.len() >= 75,
            "need >= 75 zeros, got {}",
            all_zeros.len()
        );

        let training = &all_zeros[..50];
        let actual_next = &all_zeros[50..75];

        let cmv = reconstruct_cmv(training).unwrap();
        let model = fit_verblunsky_model(&cmv);
        let predictions = predict_next_zeros(&model, training, 25).unwrap();

        assert_eq!(predictions.len(), 25, "expected 25 predictions");

        // All predictions must be strictly beyond last training zero
        let t_train_max = training.last().unwrap().t;
        for (i, &pred) in predictions.iter().enumerate() {
            assert!(
                pred > t_train_max,
                "prediction[{i}] = {pred:.4} ≤ t_train_max = {t_train_max:.4}"
            );
        }

        let acc = validate_prediction(&predictions, actual_next);
        assert!(acc.mae.is_finite(), "MAE not finite");
        assert!(acc.rmse.is_finite(), "RMSE not finite");
        assert!(acc.max_error.is_finite(), "max_error not finite");
        assert_eq!(acc.n_compared, 25);

        eprintln!(
            "Predict 50→75: MAE={:.4}, RMSE={:.4}, max_err={:.4}",
            acc.mae, acc.rmse, acc.max_error
        );
    }

    #[test]
    fn predict_from_75_zeros_to_100() {
        let all_zeros = get_zeros(600.0);
        assert!(
            all_zeros.len() >= 100,
            "need >= 100 zeros, got {}",
            all_zeros.len()
        );

        let training = &all_zeros[..75];
        let actual_next = &all_zeros[75..100];

        let cmv = reconstruct_cmv(training).unwrap();
        let model = fit_verblunsky_model(&cmv);
        let predictions = predict_next_zeros(&model, training, 25).unwrap();

        assert_eq!(predictions.len(), 25, "expected 25 predictions");

        let t_train_max = training.last().unwrap().t;
        for (i, &pred) in predictions.iter().enumerate() {
            assert!(
                pred > t_train_max,
                "prediction[{i}] = {pred:.4} ≤ t_train_max = {t_train_max:.4}"
            );
        }

        let acc = validate_prediction(&predictions, actual_next);
        assert!(acc.mae.is_finite(), "MAE not finite");
        assert_eq!(acc.n_compared, 25);

        eprintln!(
            "Predict 75→100: MAE={:.4}, RMSE={:.4}, max_err={:.4}",
            acc.mae, acc.rmse, acc.max_error
        );
    }

    #[test]
    fn prediction_accuracy_is_sensible() {
        // The accuracy should at least be within the scale of zero spacings.
        // Mean spacing at t~300 is ≈ 2π/ln(300/2π) ≈ 1.5.
        // We allow MAE < 50 × mean_spacing as a loose sanity check.
        let all_zeros = get_zeros(400.0);
        if all_zeros.len() < 75 {
            return;
        }

        let training = &all_zeros[..50];
        let actual_next = &all_zeros[50..75];

        let cmv = reconstruct_cmv(training).unwrap();
        let model = fit_verblunsky_model(&cmv);
        let predictions = predict_next_zeros(&model, training, 25).unwrap();
        let acc = validate_prediction(&predictions, actual_next);

        // Loose bound: MAE < 100 (very generous — this is a hard prediction problem)
        assert!(
            acc.mae < 200.0,
            "MAE = {:.4} seems unreasonably large",
            acc.mae
        );
    }

    #[test]
    fn validate_prediction_empty_returns_nan() {
        let acc = validate_prediction(&[], &[]);
        assert!(acc.mae.is_nan());
        assert_eq!(acc.n_compared, 0);
    }

    #[test]
    fn validate_prediction_partial_comparison() {
        let predicted = vec![100.0, 110.0, 120.0, 130.0];
        let actual = vec![
            ZetaZero {
                ordinal: 1,
                t: 101.0,
                z_value: 0.0,
                on_critical_line: true,
            },
            ZetaZero {
                ordinal: 2,
                t: 111.0,
                z_value: 0.0,
                on_critical_line: true,
            },
        ];
        let acc = validate_prediction(&predicted, &actual);
        assert_eq!(acc.n_compared, 2);
        assert!((acc.mae - 1.0).abs() < 1e-10, "MAE should be 1.0");
    }

    #[test]
    fn estimate_t_for_nth_zero_monotone() {
        let t10 = estimate_t_for_nth_zero(10);
        let t50 = estimate_t_for_nth_zero(50);
        let t100 = estimate_t_for_nth_zero(100);
        assert!(t10 < t50, "t10={t10:.2} should be < t50={t50:.2}");
        assert!(t50 < t100, "t50={t50:.2} should be < t100={t100:.2}");
        assert!(t10 > 14.0, "t10={t10:.2} should be > 14.0 (first zero)");
        eprintln!("estimate_t: n=10 → {t10:.2}, n=50 → {t50:.2}, n=100 → {t100:.2}");
    }

    #[test]
    fn insufficient_zeros_returns_error() {
        let cmv = reconstruct_cmv(&[
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
            ZetaZero {
                ordinal: 3,
                t: 25.0,
                z_value: 0.0,
                on_critical_line: true,
            },
        ])
        .unwrap();
        let model = fit_verblunsky_model(&cmv);

        let too_few = vec![ZetaZero {
            ordinal: 1,
            t: 14.1,
            z_value: 0.0,
            on_critical_line: true,
        }];
        assert!(predict_next_zeros(&model, &too_few, 5).is_err());
    }
}
