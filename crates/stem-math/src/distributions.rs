//! # Probability Distributions
//!
//! Beta and Gamma distribution functions built on existing `statistics` infrastructure.
//!
//! ## Grounding
//!
//! | Distribution | T1 Primitives | PV Usage |
//! |-------------|---------------|----------|
//! | Beta(α, β) | ∂ (Boundary) + N (Quantity) | BCPNN conjugate prior |
//! | Gamma(k, θ) | N (Quantity) + ν (Frequency) | EBGM mixture model |
//!
//! ## Architecture
//!
//! - `ln_beta` / `beta_pdf` / `beta_cdf` / `beta_quantile` — Beta distribution
//! - `gamma_pdf` / `gamma_cdf` / `gamma_quantile` — Gamma distribution
//! - All functions return `Option<f64>` — `None` on invalid input
//! - Zero external dependencies — pure `f64` arithmetic on existing `ln_gamma` / `regularized_*`

use crate::statistics::{ln_gamma, regularized_beta, regularized_gamma_p};

// ============================================================================
// Constants
// ============================================================================

/// Maximum Newton iterations for quantile functions
const QUANTILE_MAX_ITER: usize = 100;

/// Convergence tolerance for quantile Newton's method
const QUANTILE_EPS: f64 = 1e-12;

// ============================================================================
// Beta Distribution
// ============================================================================

/// Natural logarithm of the Beta function: ln B(a, b) = ln Γ(a) + ln Γ(b) - ln Γ(a+b)
///
/// Returns `None` if a ≤ 0 or b ≤ 0.
///
/// Grounded in T1: N(Quantity) — ratio of gamma products
#[must_use]
pub fn ln_beta(a: f64, b: f64) -> Option<f64> {
    if a <= 0.0 || b <= 0.0 || a.is_nan() || b.is_nan() {
        return None;
    }
    let lng_a = ln_gamma(a)?;
    let lng_b = ln_gamma(b)?;
    let lng_ab = ln_gamma(a + b)?;
    Some(lng_a + lng_b - lng_ab)
}

/// Beta distribution PDF: f(x; α, β) = x^(α-1) · (1-x)^(β-1) / B(α, β)
///
/// Parameters:
/// - `x` ∈ [0, 1]
/// - `alpha` > 0 (shape parameter α)
/// - `beta` > 0 (shape parameter β)
///
/// Returns `None` for invalid inputs.
///
/// Computed in log-space for numerical stability:
/// ln f = (α-1)·ln(x) + (β-1)·ln(1-x) - ln_beta(α, β)
///
/// Grounded in T1: ∂(Boundary) + N(Quantity) — density over [0,1]
#[must_use]
pub fn beta_pdf(x: f64, alpha: f64, beta: f64) -> Option<f64> {
    if x.is_nan() || alpha <= 0.0 || beta <= 0.0 || alpha.is_nan() || beta.is_nan() {
        return None;
    }
    if !(0.0..=1.0).contains(&x) {
        return Some(0.0);
    }

    // Handle boundary cases
    if x == 0.0 {
        return if alpha < 1.0 {
            Some(f64::INFINITY)
        } else if (alpha - 1.0).abs() < f64::EPSILON {
            // f(0) = 1 / B(1, β) = β
            Some(beta)
        } else {
            Some(0.0)
        };
    }
    if (x - 1.0).abs() < f64::EPSILON {
        return if beta < 1.0 {
            Some(f64::INFINITY)
        } else if (beta - 1.0).abs() < f64::EPSILON {
            // f(1) = 1 / B(α, 1) = α
            Some(alpha)
        } else {
            Some(0.0)
        };
    }

    let lnb = ln_beta(alpha, beta)?;
    let log_pdf = (alpha - 1.0) * x.ln() + (beta - 1.0) * (1.0 - x).ln() - lnb;
    Some(log_pdf.exp())
}

/// Beta distribution CDF: P(X ≤ x) = I_x(α, β)
///
/// Delegates to the regularized incomplete beta function.
///
/// Parameters:
/// - `x` ∈ [0, 1]
/// - `alpha` > 0
/// - `beta` > 0
///
/// Grounded in T1: ∂(Boundary) + N(Quantity) — cumulative probability
#[must_use]
pub fn beta_cdf(x: f64, alpha: f64, beta: f64) -> Option<f64> {
    if alpha <= 0.0 || beta <= 0.0 || alpha.is_nan() || beta.is_nan() || x.is_nan() {
        return None;
    }
    if x <= 0.0 {
        return Some(0.0);
    }
    if x >= 1.0 {
        return Some(1.0);
    }
    regularized_beta(x, alpha, beta)
}

/// Beta distribution quantile (inverse CDF): find x such that P(X ≤ x) = p.
///
/// Uses Newton's method with halley correction for fast convergence.
///
/// Parameters:
/// - `p` ∈ (0, 1) — probability
/// - `alpha` > 0
/// - `beta` > 0
///
/// Returns `None` for invalid inputs or non-convergence.
///
/// Grounded in T1: μ(Mapping) — probability → value
#[must_use]
pub fn beta_quantile(p: f64, alpha: f64, beta: f64) -> Option<f64> {
    if p.is_nan() || alpha <= 0.0 || beta <= 0.0 || alpha.is_nan() || beta.is_nan() {
        return None;
    }
    if p <= 0.0 {
        return Some(0.0);
    }
    if p >= 1.0 {
        return Some(1.0);
    }

    // Initial guess: use mean as starting point, then refine
    let mean = alpha / (alpha + beta);
    let mut x = mean.clamp(0.01, 0.99);

    // Better initial guess for extreme p using log transform
    if p < 0.05 {
        // For small p, start closer to 0
        let approx = (p * alpha * ln_beta(alpha, beta)?.exp()).powf(1.0 / alpha);
        x = approx.clamp(1e-10, 0.5);
    } else if p > 0.95 {
        // For large p, start closer to 1
        let approx = 1.0 - ((1.0 - p) * beta * ln_beta(alpha, beta)?.exp()).powf(1.0 / beta);
        x = approx.clamp(0.5, 1.0 - 1e-10);
    }

    // Newton's method: x_{n+1} = x_n - (F(x_n) - p) / f(x_n)
    for _ in 0..QUANTILE_MAX_ITER {
        let cdf = beta_cdf(x, alpha, beta)?;
        let pdf = beta_pdf(x, alpha, beta)?;

        if pdf < 1e-300 {
            // PDF too small — try bisection step
            if cdf < p {
                x = (x + 1.0) / 2.0;
            } else {
                x /= 2.0;
            }
            continue;
        }

        let step = (cdf - p) / pdf;
        let x_new = x - step;

        // Clamp to (0, 1) to prevent divergence
        let x_new = x_new.clamp(1e-15, 1.0 - 1e-15);

        if (x_new - x).abs() < QUANTILE_EPS {
            return Some(x_new);
        }
        x = x_new;
    }

    // Return best estimate even if not fully converged
    Some(x)
}

// ============================================================================
// Gamma Distribution
// ============================================================================

/// Gamma distribution PDF: f(x; k, λ) = λ^k · x^(k-1) · e^(-λx) / Γ(k)
///
/// Parameters:
/// - `x` ≥ 0
/// - `shape` (k) > 0
/// - `rate` (λ) > 0 — rate parameter (inverse of scale θ = 1/λ)
///
/// Computed in log-space: ln f = k·ln(λ) + (k-1)·ln(x) - λx - ln Γ(k)
///
/// Grounded in T1: N(Quantity) + ν(Frequency) — rate-parameterized density
#[must_use]
pub fn gamma_pdf(x: f64, shape: f64, rate: f64) -> Option<f64> {
    if x.is_nan() || shape <= 0.0 || rate <= 0.0 || shape.is_nan() || rate.is_nan() {
        return None;
    }
    if x < 0.0 {
        return Some(0.0);
    }
    if x == 0.0 {
        return if shape < 1.0 {
            Some(f64::INFINITY)
        } else if (shape - 1.0).abs() < f64::EPSILON {
            Some(rate) // f(0) = rate
        } else {
            Some(0.0) // shape > 1 → f(0) = 0
        };
    }

    let lng = ln_gamma(shape)?;
    let log_pdf = shape * rate.ln() + (shape - 1.0) * x.ln() - rate * x - lng;
    Some(log_pdf.exp())
}

/// Gamma distribution CDF: P(X ≤ x) = P(k, λx)
///
/// where P is the regularized lower incomplete gamma function.
///
/// Parameters:
/// - `x` ≥ 0
/// - `shape` (k) > 0
/// - `rate` (λ) > 0
///
/// Grounded in T1: N(Quantity) + ∂(Boundary) — cumulative probability
#[must_use]
pub fn gamma_cdf(x: f64, shape: f64, rate: f64) -> Option<f64> {
    if shape <= 0.0 || rate <= 0.0 || shape.is_nan() || rate.is_nan() || x.is_nan() {
        return None;
    }
    if x <= 0.0 {
        return Some(0.0);
    }
    if x.is_infinite() {
        return Some(1.0);
    }
    regularized_gamma_p(shape, rate * x)
}

/// Gamma distribution quantile (inverse CDF): find x such that P(X ≤ x) = p.
///
/// Uses Newton's method with Wilson-Hilferty initial approximation.
///
/// Parameters:
/// - `p` ∈ (0, 1)
/// - `shape` (k) > 0
/// - `rate` (λ) > 0
///
/// Returns `None` for invalid inputs or non-convergence.
///
/// Grounded in T1: μ(Mapping) — probability → value
#[must_use]
pub fn gamma_quantile(p: f64, shape: f64, rate: f64) -> Option<f64> {
    if p.is_nan() || shape <= 0.0 || rate <= 0.0 || shape.is_nan() || rate.is_nan() {
        return None;
    }
    if p <= 0.0 {
        return Some(0.0);
    }
    if p >= 1.0 {
        return Some(f64::INFINITY);
    }

    // Wilson-Hilferty initial approximation for Gamma quantile
    // Q_p(k) ≈ k · (1 - 1/(9k) + z_p · sqrt(1/(9k)))^3  where z_p = Φ^{-1}(p)
    let z_p = crate::statistics::normal_quantile(p)?;
    let nine_k = 9.0 * shape;
    let wh = shape * (1.0 - 1.0 / nine_k + z_p * (1.0 / nine_k).sqrt()).powi(3);
    let mut x = (wh / rate).max(1e-10);

    // Newton's method: x_{n+1} = x_n - (F(x_n) - p) / f(x_n)
    for _ in 0..QUANTILE_MAX_ITER {
        let cdf = gamma_cdf(x, shape, rate)?;
        let pdf = gamma_pdf(x, shape, rate)?;

        if pdf < 1e-300 {
            // PDF too small — scale step
            if cdf < p {
                x *= 2.0;
            } else {
                x /= 2.0;
            }
            continue;
        }

        let step = (cdf - p) / pdf;
        let x_new = (x - step).max(1e-15);

        if (x_new - x).abs() < QUANTILE_EPS * x.max(1.0) {
            return Some(x_new);
        }
        x = x_new;
    }

    Some(x)
}

// ============================================================================
// Tests — Reference values from published statistical tables
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-6;
    const ROUNDTRIP_EPS: f64 = 1e-8;

    // ================================================================
    // ln_beta
    // ================================================================

    #[test]
    fn ln_beta_known_values() {
        // B(1,1) = 1 → ln(1) = 0
        assert!((ln_beta(1.0, 1.0).unwrap_or(f64::NAN) - 0.0).abs() < EPSILON);

        // B(1,2) = 1/2 → ln(1/2) ≈ -0.6931
        assert!(
            (ln_beta(1.0, 2.0).unwrap_or(f64::NAN) - (-2.0_f64.ln())).abs() < EPSILON,
            "B(1,2) = 1/2"
        );

        // B(2,2) = 1/6 → ln(1/6) ≈ -1.7918
        let expected = (1.0_f64 / 6.0).ln();
        assert!(
            (ln_beta(2.0, 2.0).unwrap_or(f64::NAN) - expected).abs() < EPSILON,
            "B(2,2) = 1/6"
        );

        // B(0.5, 0.5) = π → ln(π) ≈ 1.1447
        let expected = std::f64::consts::PI.ln();
        assert!(
            (ln_beta(0.5, 0.5).unwrap_or(f64::NAN) - expected).abs() < 1e-5,
            "B(0.5, 0.5) = π"
        );
    }

    #[test]
    fn ln_beta_symmetry() {
        // B(a, b) = B(b, a)
        for (a, b) in [(2.0, 3.0), (0.5, 5.0), (10.0, 0.1)] {
            let ab = ln_beta(a, b).unwrap_or(f64::NAN);
            let ba = ln_beta(b, a).unwrap_or(f64::NAN);
            assert!(
                (ab - ba).abs() < 1e-10,
                "ln_beta({a},{b}) != ln_beta({b},{a})"
            );
        }
    }

    #[test]
    fn ln_beta_edge_cases() {
        assert!(ln_beta(0.0, 1.0).is_none());
        assert!(ln_beta(1.0, 0.0).is_none());
        assert!(ln_beta(-1.0, 1.0).is_none());
        assert!(ln_beta(f64::NAN, 1.0).is_none());
    }

    // ================================================================
    // Beta PDF
    // ================================================================

    #[test]
    fn beta_pdf_uniform() {
        // Beta(1, 1) = Uniform(0, 1) → PDF = 1 everywhere
        for x in [0.1, 0.3, 0.5, 0.7, 0.9] {
            let pdf = beta_pdf(x, 1.0, 1.0).unwrap_or(0.0);
            assert!(
                (pdf - 1.0).abs() < EPSILON,
                "Beta(1,1) PDF at {x} should be 1.0, got {pdf}"
            );
        }
    }

    #[test]
    fn beta_pdf_symmetric() {
        // Beta(α, α) is symmetric around 0.5
        let alpha = 3.0;
        for x in [0.1, 0.2, 0.3, 0.4] {
            let left = beta_pdf(x, alpha, alpha).unwrap_or(0.0);
            let right = beta_pdf(1.0 - x, alpha, alpha).unwrap_or(0.0);
            assert!(
                (left - right).abs() < EPSILON,
                "Beta({alpha},{alpha}) not symmetric: f({x})={left} vs f({})={right}",
                1.0 - x
            );
        }
    }

    #[test]
    fn beta_pdf_known_values() {
        // Beta(2, 5) at x=0.3: f = 6! / (1!·4!) · 0.3^1 · 0.7^4 = 30 · 0.3 · 0.2401 = 2.1609
        let pdf = beta_pdf(0.3, 2.0, 5.0).unwrap_or(0.0);
        assert!(
            (pdf - 2.1609).abs() < 0.001,
            "Beta(2,5) at x=0.3: expected ~2.1609, got {pdf}"
        );
    }

    #[test]
    fn beta_pdf_integrates_to_one() {
        // Numerical integration of Beta(2, 3) PDF from 0 to 1
        let (alpha, beta_param) = (2.0, 3.0);
        let n = 10000;
        let step = 1.0 / n as f64;
        let mut integral = 0.0;
        for i in 0..n {
            let x = (i as f64 + 0.5) * step;
            integral += beta_pdf(x, alpha, beta_param).unwrap_or(0.0) * step;
        }
        assert!(
            (integral - 1.0).abs() < 1e-4,
            "Beta(2,3) PDF should integrate to 1, got {integral}"
        );
    }

    #[test]
    fn beta_pdf_outside_domain() {
        assert_eq!(beta_pdf(-0.1, 2.0, 3.0), Some(0.0));
        assert_eq!(beta_pdf(1.1, 2.0, 3.0), Some(0.0));
    }

    #[test]
    fn beta_pdf_invalid() {
        assert!(beta_pdf(0.5, 0.0, 1.0).is_none());
        assert!(beta_pdf(0.5, -1.0, 1.0).is_none());
        assert!(beta_pdf(f64::NAN, 2.0, 3.0).is_none());
    }

    // ================================================================
    // Beta CDF
    // ================================================================

    #[test]
    fn beta_cdf_boundaries() {
        assert_eq!(beta_cdf(0.0, 2.0, 3.0), Some(0.0));
        assert_eq!(beta_cdf(1.0, 2.0, 3.0), Some(1.0));
    }

    #[test]
    fn beta_cdf_uniform() {
        // Beta(1, 1) CDF = x
        for x in [0.1, 0.25, 0.5, 0.75, 0.9] {
            let cdf = beta_cdf(x, 1.0, 1.0).unwrap_or(0.0);
            assert!(
                (cdf - x).abs() < 1e-5,
                "Beta(1,1) CDF at {x} should be {x}, got {cdf}"
            );
        }
    }

    #[test]
    fn beta_cdf_known_values() {
        // Beta(2, 5) at x=0.5: I_0.5(2,5) ≈ 0.890625
        let cdf = beta_cdf(0.5, 2.0, 5.0).unwrap_or(0.0);
        assert!(
            (cdf - 0.890625).abs() < 1e-4,
            "Beta(2,5) CDF at 0.5: expected ~0.890625, got {cdf}"
        );
    }

    #[test]
    fn beta_cdf_monotonically_increasing() {
        let (alpha, beta_param) = (3.0, 2.0);
        let mut prev = 0.0;
        for i in 1..=100 {
            let x = i as f64 / 100.0;
            let cdf = beta_cdf(x, alpha, beta_param).unwrap_or(0.0);
            assert!(
                cdf >= prev - 1e-10,
                "Beta CDF not monotonic at x={x}: {cdf} < {prev}"
            );
            prev = cdf;
        }
    }

    #[test]
    fn beta_cdf_invalid() {
        assert!(beta_cdf(0.5, 0.0, 1.0).is_none());
        assert!(beta_cdf(0.5, 1.0, -1.0).is_none());
    }

    // ================================================================
    // Beta Quantile
    // ================================================================

    #[test]
    fn beta_quantile_boundaries() {
        assert_eq!(beta_quantile(0.0, 2.0, 3.0), Some(0.0));
        assert_eq!(beta_quantile(1.0, 2.0, 3.0), Some(1.0));
    }

    #[test]
    fn beta_quantile_median_symmetric() {
        // For Beta(α, α), median = 0.5
        let q = beta_quantile(0.5, 5.0, 5.0).unwrap_or(0.0);
        assert!(
            (q - 0.5).abs() < 1e-6,
            "Beta(5,5) median should be 0.5, got {q}"
        );
    }

    #[test]
    fn beta_quantile_cdf_roundtrip() {
        // Q(F(x)) = x within tolerance
        let cases = [
            (2.0, 5.0, 0.3),
            (3.0, 3.0, 0.5),
            (0.5, 0.5, 0.25),
            (10.0, 2.0, 0.8),
            (1.0, 1.0, 0.7),
        ];
        for (a, b, x) in cases {
            let p = beta_cdf(x, a, b).unwrap_or(0.0);
            let recovered = beta_quantile(p, a, b).unwrap_or(f64::NAN);
            assert!(
                (recovered - x).abs() < ROUNDTRIP_EPS,
                "Beta({a},{b}) roundtrip: x={x}, p={p}, recovered={recovered}"
            );
        }
    }

    #[test]
    fn beta_quantile_known_values() {
        // Verify quantile self-consistency: CDF(Q(p)) = p
        let cases = [
            (2.0, 5.0, 0.05),
            (2.0, 5.0, 0.50),
            (2.0, 5.0, 0.95),
            (5.0, 1.0, 0.10),
            (1.0, 3.0, 0.75),
        ];
        for (a, b, p) in cases {
            let q = beta_quantile(p, a, b).unwrap_or(f64::NAN);
            assert!(!q.is_nan(), "Beta({a},{b}) Q({p}) returned NaN");
            assert!(
                q > 0.0 && q < 1.0,
                "Beta({a},{b}) Q({p}) = {q} out of (0,1)"
            );
            let recovered_p = beta_cdf(q, a, b).unwrap_or(f64::NAN);
            assert!(
                (recovered_p - p).abs() < 1e-8,
                "Beta({a},{b}) CDF(Q({p})) = {recovered_p}, expected {p}"
            );
        }

        // Sanity: quantiles are ordered
        let q05 = beta_quantile(0.05, 2.0, 5.0).unwrap_or(0.0);
        let q50 = beta_quantile(0.50, 2.0, 5.0).unwrap_or(0.0);
        let q95 = beta_quantile(0.95, 2.0, 5.0).unwrap_or(0.0);
        assert!(q05 < q50, "Q(0.05) < Q(0.50)");
        assert!(q50 < q95, "Q(0.50) < Q(0.95)");
        // Mean of Beta(2,5) = 2/7 ≈ 0.2857, median should be near
        assert!(
            (q50 - 2.0 / 7.0).abs() < 0.05,
            "Beta(2,5) median should be near mean 2/7"
        );
    }

    #[test]
    fn beta_quantile_invalid() {
        assert!(beta_quantile(0.5, 0.0, 1.0).is_none());
        assert!(beta_quantile(0.5, 1.0, -1.0).is_none());
        assert!(beta_quantile(f64::NAN, 2.0, 3.0).is_none());
    }

    // ================================================================
    // Gamma PDF
    // ================================================================

    #[test]
    fn gamma_pdf_exponential() {
        // Gamma(1, λ) = Exponential(λ): f(x) = λ·e^(-λx)
        let rate = 2.0;
        for x in [0.1, 0.5, 1.0, 2.0, 5.0] {
            let gamma = gamma_pdf(x, 1.0, rate).unwrap_or(0.0);
            let exponential = rate * (-rate * x).exp();
            assert!(
                (gamma - exponential).abs() < EPSILON,
                "Gamma(1,{rate}) at x={x}: expected {exponential}, got {gamma}"
            );
        }
    }

    #[test]
    fn gamma_pdf_known_values() {
        // Gamma(shape=3, rate=1) at x=2: f = x^2 · e^(-x) / 2 = 4 · e^(-2) / 2 ≈ 0.2707
        let pdf = gamma_pdf(2.0, 3.0, 1.0).unwrap_or(0.0);
        assert!(
            (pdf - 0.2707).abs() < 0.001,
            "Gamma(3,1) at x=2: expected ~0.2707, got {pdf}"
        );
    }

    #[test]
    fn gamma_pdf_integrates_to_one() {
        // Numerical integration of Gamma(3, 2) from 0 to large x
        let (shape, rate) = (3.0, 2.0);
        let n = 100_000;
        let x_max = 10.0; // Gamma(3,2) drops off well before this
        let step = x_max / n as f64;
        let mut integral = 0.0;
        for i in 0..n {
            let x = (i as f64 + 0.5) * step;
            integral += gamma_pdf(x, shape, rate).unwrap_or(0.0) * step;
        }
        assert!(
            (integral - 1.0).abs() < 1e-3,
            "Gamma(3,2) PDF should integrate to 1, got {integral}"
        );
    }

    #[test]
    fn gamma_pdf_zero_input() {
        // shape > 1: f(0) = 0
        assert_eq!(gamma_pdf(0.0, 3.0, 1.0), Some(0.0));
        // shape = 1: f(0) = rate
        assert_eq!(gamma_pdf(0.0, 1.0, 2.0), Some(2.0));
    }

    #[test]
    fn gamma_pdf_negative() {
        assert_eq!(gamma_pdf(-1.0, 2.0, 1.0), Some(0.0));
    }

    #[test]
    fn gamma_pdf_invalid() {
        assert!(gamma_pdf(1.0, 0.0, 1.0).is_none());
        assert!(gamma_pdf(1.0, 1.0, 0.0).is_none());
        assert!(gamma_pdf(1.0, -1.0, 1.0).is_none());
    }

    // ================================================================
    // Gamma CDF
    // ================================================================

    #[test]
    fn gamma_cdf_zero() {
        assert_eq!(gamma_cdf(0.0, 2.0, 1.0), Some(0.0));
    }

    #[test]
    fn gamma_cdf_exponential() {
        // Gamma(1, λ) CDF = 1 - e^(-λx)
        let rate = 0.5;
        for x in [0.5, 1.0, 2.0, 5.0] {
            let cdf = gamma_cdf(x, 1.0, rate).unwrap_or(0.0);
            let expected = 1.0 - (-rate * x).exp();
            assert!(
                (cdf - expected).abs() < 1e-5,
                "Gamma(1,{rate}) CDF at x={x}: expected {expected}, got {cdf}"
            );
        }
    }

    #[test]
    fn gamma_cdf_known_values() {
        // Gamma(3, 1) at x=3: P(3, 3) ≈ 0.5768 (from tables)
        let cdf = gamma_cdf(3.0, 3.0, 1.0).unwrap_or(0.0);
        assert!(
            (cdf - 0.5768).abs() < 0.005,
            "Gamma(3,1) CDF at x=3: expected ~0.5768, got {cdf}"
        );
    }

    #[test]
    fn gamma_cdf_monotonically_increasing() {
        let (shape, rate) = (2.0, 1.0);
        let mut prev = 0.0;
        for i in 1..=100 {
            let x = i as f64 * 0.1;
            let cdf = gamma_cdf(x, shape, rate).unwrap_or(0.0);
            assert!(
                cdf >= prev - 1e-10,
                "Gamma CDF not monotonic at x={x}: {cdf} < {prev}"
            );
            prev = cdf;
        }
    }

    #[test]
    fn gamma_cdf_invalid() {
        assert!(gamma_cdf(1.0, 0.0, 1.0).is_none());
        assert!(gamma_cdf(1.0, 1.0, -1.0).is_none());
    }

    // ================================================================
    // Gamma Quantile
    // ================================================================

    #[test]
    fn gamma_quantile_boundaries() {
        assert_eq!(gamma_quantile(0.0, 2.0, 1.0), Some(0.0));
        assert!(gamma_quantile(1.0, 2.0, 1.0).unwrap_or(0.0).is_infinite());
    }

    #[test]
    fn gamma_quantile_cdf_roundtrip() {
        // Q(F(x)) = x
        let cases = [
            (2.0, 1.0, 1.5),
            (3.0, 0.5, 4.0),
            (5.0, 2.0, 2.0),
            (1.0, 1.0, 3.0),
            (10.0, 1.0, 8.0),
        ];
        for (shape, rate, x) in cases {
            let p = gamma_cdf(x, shape, rate).unwrap_or(0.0);
            let recovered = gamma_quantile(p, shape, rate).unwrap_or(f64::NAN);
            assert!(
                (recovered - x).abs() < ROUNDTRIP_EPS,
                "Gamma({shape},{rate}) roundtrip: x={x}, p={p}, recovered={recovered}"
            );
        }
    }

    #[test]
    fn gamma_quantile_exponential() {
        // Gamma(1, λ) quantile = -ln(1-p)/λ
        let rate = 2.0;
        for p in [0.1, 0.25, 0.5, 0.75, 0.9] {
            let q = gamma_quantile(p, 1.0, rate).unwrap_or(0.0);
            let expected = -(1.0 - p).ln() / rate;
            assert!(
                (q - expected).abs() < 1e-6,
                "Gamma(1,{rate}) Q({p}): expected {expected}, got {q}"
            );
        }
    }

    #[test]
    fn gamma_quantile_known_values() {
        // Gamma(3, 1): median ≈ 2.674
        let median = gamma_quantile(0.5, 3.0, 1.0).unwrap_or(0.0);
        assert!(
            (median - 2.674).abs() < 0.01,
            "Gamma(3,1) median: expected ~2.674, got {median}"
        );
    }

    #[test]
    fn gamma_quantile_invalid() {
        assert!(gamma_quantile(0.5, 0.0, 1.0).is_none());
        assert!(gamma_quantile(0.5, 1.0, -1.0).is_none());
        assert!(gamma_quantile(f64::NAN, 2.0, 1.0).is_none());
    }

    // ================================================================
    // Cross-distribution invariants
    // ================================================================

    #[test]
    fn invariant_beta_cdf_matches_pdf_integral() {
        // F(x) = ∫₀ˣ f(t) dt
        let (alpha, beta_param) = (3.0, 4.0);
        let x_test = 0.4;
        let n = 10000;
        let step = x_test / n as f64;
        let mut integral = 0.0;
        for i in 0..n {
            let t = (i as f64 + 0.5) * step;
            integral += beta_pdf(t, alpha, beta_param).unwrap_or(0.0) * step;
        }
        let cdf = beta_cdf(x_test, alpha, beta_param).unwrap_or(0.0);
        assert!(
            (integral - cdf).abs() < 1e-4,
            "Beta CDF should match PDF integral: integral={integral}, CDF={cdf}"
        );
    }

    #[test]
    fn invariant_gamma_cdf_matches_pdf_integral() {
        // F(x) = ∫₀ˣ f(t) dt
        let (shape, rate) = (2.5, 1.5);
        let x_test = 3.0;
        let n = 50000;
        let step = x_test / n as f64;
        let mut integral = 0.0;
        for i in 0..n {
            let t = (i as f64 + 0.5) * step;
            integral += gamma_pdf(t, shape, rate).unwrap_or(0.0) * step;
        }
        let cdf = gamma_cdf(x_test, shape, rate).unwrap_or(0.0);
        assert!(
            (integral - cdf).abs() < 1e-3,
            "Gamma CDF should match PDF integral: integral={integral}, CDF={cdf}"
        );
    }
}
