/// Beta distribution confidence intervals and hypothesis testing.
///
/// Zero-dependency implementation of:
/// - Regularized incomplete beta function (Lentz continued fraction)
/// - Beta CDF and quantile (inverse CDF via bisection)
/// - Credible intervals and exceedance probability
///
/// Fixes: Fallacy #5 (Neglect of Probability), Fallacy #6 (Hasty Generalization),
/// Gap #5 (Confidence Intervals).
///
/// Tier: T2-P (Quantity N + Boundary d)

/// Natural log of the Gamma function via Lanczos approximation (g=7, n=9).
/// Accurate to ~15 digits for positive real arguments.
fn ln_gamma(z: f64) -> f64 {
    const COEFFS: [f64; 9] = [
        0.999_999_999_999_809_93,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_13,
        -176.615_029_162_140_59,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_571_6e-6,
        1.505_632_735_149_311_6e-7,
    ];

    if z < 0.5 {
        let pi = core::f64::consts::PI;
        let sin_pz = (pi * z).sin();
        if sin_pz.abs() < 1e-300 {
            return f64::INFINITY;
        }
        return (pi / sin_pz).ln() - ln_gamma(1.0 - z);
    }

    let x = z - 1.0;
    let mut sum = COEFFS[0];
    for i in 1..9 {
        sum += COEFFS[i] / (x + i as f64);
    }
    let t = x + 7.5;
    0.5 * (2.0 * core::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + sum.ln()
}

/// Continued fraction evaluation for the regularized incomplete beta function.
/// Uses the modified Lentz algorithm.
fn beta_continued_fraction(a: f64, b: f64, x: f64) -> f64 {
    const MAX_ITER: u32 = 200;
    const EPS: f64 = 3.0e-12;
    const FPMIN: f64 = 1.0e-30;

    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;

    let mut c = 1.0_f64;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < FPMIN {
        d = FPMIN;
    }
    d = 1.0 / d;
    let mut h = d;

    for m in 1..=MAX_ITER {
        let mf = f64::from(m);
        let m2 = 2.0 * mf;

        // Even step
        let aa = mf * (b - mf) * x / ((qam + m2) * (a + m2));
        d = 1.0 + aa * d;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = 1.0 + aa / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1.0 / d;
        h *= d * c;

        // Odd step
        let aa = -(a + mf) * (qab + mf) * x / ((a + m2) * (qap + m2));
        d = 1.0 + aa * d;
        if d.abs() < FPMIN {
            d = FPMIN;
        }
        c = 1.0 + aa / c;
        if c.abs() < FPMIN {
            c = FPMIN;
        }
        d = 1.0 / d;
        let del = d * c;
        h *= del;

        if (del - 1.0).abs() <= EPS {
            return h;
        }
    }
    h
}

/// Cumulative distribution function of the Beta distribution.
///
/// Returns P(X <= x) for X ~ Beta(alpha, beta).
///
/// Uses the regularized incomplete beta function via continued fraction.
pub fn beta_cdf(x: f64, alpha: f64, beta: f64) -> f64 {
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }
    if alpha <= 0.0 || beta <= 0.0 {
        return f64::NAN;
    }

    let log_beta = ln_gamma(alpha) + ln_gamma(beta) - ln_gamma(alpha + beta);
    let bt = (alpha * x.ln() + beta * (1.0 - x).ln() - log_beta).exp();

    // Use symmetry relation for numerical stability
    if x < (alpha + 1.0) / (alpha + beta + 2.0) {
        bt * beta_continued_fraction(alpha, beta, x) / alpha
    } else {
        1.0 - bt * beta_continued_fraction(beta, alpha, 1.0 - x) / beta
    }
}

/// Quantile function (inverse CDF) of the Beta distribution.
///
/// Returns x such that P(X <= x) = p for X ~ Beta(alpha, beta).
/// Uses bisection search on [0, 1] (monotonic CDF).
pub fn beta_quantile(p: f64, alpha: f64, beta: f64) -> f64 {
    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return 1.0;
    }
    if alpha <= 0.0 || beta <= 0.0 {
        return f64::NAN;
    }

    let mut lo = 0.0_f64;
    let mut hi = 1.0_f64;
    let eps = 1e-10;

    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        let cdf_mid = beta_cdf(mid, alpha, beta);
        if (cdf_mid - p).abs() < eps {
            return mid;
        }
        if cdf_mid < p {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

/// Probability that the true trust score exceeds a threshold.
///
/// Returns P(X > threshold) for X ~ Beta(alpha, beta).
/// This is the key decision-support function: "how confident are we
/// that this entity's trust exceeds the required level?"
pub fn prob_exceeds(alpha: f64, beta: f64, threshold: f64) -> f64 {
    1.0 - beta_cdf(threshold, alpha, beta)
}

/// Mode of the Beta distribution (most likely value).
///
/// Defined when alpha > 1 and beta > 1:
/// mode = (alpha - 1) / (alpha + beta - 2)
///
/// Returns None when the mode is not uniquely defined
/// (uniform, U-shaped, or J-shaped distributions).
pub fn beta_mode(alpha: f64, beta: f64) -> Option<f64> {
    if alpha > 1.0 && beta > 1.0 {
        Some((alpha - 1.0) / (alpha + beta - 2.0))
    } else {
        None
    }
}

/// Bayesian credible interval for the Beta distribution.
///
/// The equal-tailed interval [lower, upper] such that
/// P(lower <= X <= upper) = level for X ~ Beta(alpha, beta).
#[derive(Debug, Clone, Copy)]
pub struct CredibleInterval {
    /// Lower bound of the interval
    pub lower: f64,
    /// Upper bound of the interval
    pub upper: f64,
    /// Credibility level (e.g., 0.95 for 95%)
    pub level: f64,
}

impl core::fmt::Display for CredibleInterval {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "[{:.4}, {:.4}] ({:.0}% credible)",
            self.lower,
            self.upper,
            self.level * 100.0
        )
    }
}

/// Compute an equal-tailed Bayesian credible interval.
///
/// For a given credibility level (e.g., 0.95), returns [lower, upper]
/// such that P(lower <= score <= upper) = level.
pub fn credible_interval(alpha: f64, beta: f64, level: f64) -> CredibleInterval {
    let level = level.clamp(0.0, 1.0);
    let tail = (1.0 - level) / 2.0;
    CredibleInterval {
        lower: beta_quantile(tail, alpha, beta),
        upper: beta_quantile(1.0 - tail, alpha, beta),
        level,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-4;

    #[test]
    fn ln_gamma_known_values() {
        // Gamma(1) = 0! = 1, ln(1) = 0
        assert!((ln_gamma(1.0)).abs() < 1e-10);
        // Gamma(2) = 1! = 1, ln(1) = 0
        assert!((ln_gamma(2.0)).abs() < 1e-10);
        // Gamma(6) = 5! = 120, ln(120) ≈ 4.7875
        assert!((ln_gamma(6.0) - (120.0_f64).ln()).abs() < 1e-10);
        // Gamma(0.5) = sqrt(pi)
        assert!((ln_gamma(0.5) - (core::f64::consts::PI.sqrt()).ln()).abs() < 1e-10);
    }

    #[test]
    fn cdf_boundary_values() {
        assert!((beta_cdf(0.0, 2.0, 3.0)).abs() < EPS);
        assert!((beta_cdf(1.0, 2.0, 3.0) - 1.0).abs() < EPS);
    }

    #[test]
    fn cdf_uniform_distribution() {
        // Beta(1,1) is uniform on [0,1], so CDF(x) = x
        for &x in &[0.1, 0.25, 0.5, 0.75, 0.9] {
            assert!(
                (beta_cdf(x, 1.0, 1.0) - x).abs() < EPS,
                "Beta(1,1) CDF({x}) should equal {x}, got {}",
                beta_cdf(x, 1.0, 1.0)
            );
        }
    }

    #[test]
    fn cdf_symmetric_at_midpoint() {
        // Beta(a,a) is symmetric around 0.5, so CDF(0.5) = 0.5
        for &a in &[2.0, 5.0, 10.0, 50.0] {
            assert!(
                (beta_cdf(0.5, a, a) - 0.5).abs() < EPS,
                "Beta({a},{a}) CDF(0.5) should be 0.5, got {}",
                beta_cdf(0.5, a, a)
            );
        }
    }

    #[test]
    fn quantile_inverts_cdf() {
        let alpha = 5.0;
        let beta = 3.0;
        for &p in &[0.1, 0.25, 0.5, 0.75, 0.9] {
            let x = beta_quantile(p, alpha, beta);
            let cdf_x = beta_cdf(x, alpha, beta);
            assert!(
                (cdf_x - p).abs() < EPS,
                "quantile({p}) = {x}, but CDF({x}) = {cdf_x}"
            );
        }
    }

    #[test]
    fn prob_exceeds_complements_cdf() {
        let alpha = 10.0;
        let beta = 5.0;
        let threshold = 0.6;
        let p = prob_exceeds(alpha, beta, threshold);
        let cdf = beta_cdf(threshold, alpha, beta);
        assert!((p + cdf - 1.0).abs() < EPS);
    }

    #[test]
    fn prob_exceeds_high_trust_entity() {
        // After 50 positive, 2 negative: Beta(51, 3)
        // Should be very confident score exceeds 0.6
        let p = prob_exceeds(51.0, 3.0, 0.6);
        assert!(
            p > 0.99,
            "P(score > 0.6 | a=51, b=3) should be > 0.99, got {p:.4}"
        );
    }

    #[test]
    fn prob_exceeds_uncertain_entity() {
        // Fresh prior Beta(1, 1): should NOT be confident about exceeding 0.6
        let p = prob_exceeds(1.0, 1.0, 0.6);
        assert!(
            p < 0.5,
            "P(score > 0.6 | a=1, b=1) should be < 0.5, got {p:.4}"
        );
    }

    #[test]
    fn credible_interval_contains_mean() {
        let alpha = 10.0;
        let beta = 5.0;
        let mean = alpha / (alpha + beta);
        let ci = credible_interval(alpha, beta, 0.95);
        assert!(ci.lower < mean && mean < ci.upper);
    }

    #[test]
    fn credible_interval_narrows_with_evidence() {
        let ci_low = credible_interval(2.0, 2.0, 0.95);
        let ci_high = credible_interval(200.0, 200.0, 0.95);
        let width_low = ci_low.upper - ci_low.lower;
        let width_high = ci_high.upper - ci_high.lower;
        assert!(
            width_high < width_low,
            "more evidence should narrow CI: {width_low:.4} vs {width_high:.4}"
        );
    }

    #[test]
    fn beta_mode_symmetric() {
        // Beta(5,5) mode should be 0.5
        let mode = beta_mode(5.0, 5.0);
        assert!(mode.is_some());
        if let Some(m) = mode {
            assert!((m - 0.5).abs() < EPS);
        }
    }

    #[test]
    fn beta_mode_undefined_for_uniform() {
        // Beta(1,1) has no unique mode
        assert!(beta_mode(1.0, 1.0).is_none());
    }

    #[test]
    fn credible_interval_display() {
        let ci = credible_interval(10.0, 5.0, 0.95);
        let s = format!("{ci}");
        assert!(s.contains("95% credible"));
    }
}
