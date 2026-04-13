//! Statistical helper functions — all O(1), no allocations, no I/O.
//!
//! WASM-safe: no `std::time`, no networking, no threading.

use std::f64::consts::PI;

/// Z-score for 95% confidence interval.
pub const Z_95: f64 = 1.96;

/// Chi-square critical value for p < 0.05 with df = 1.
/// CRITICAL: Use exact value 3.841, NOT 4.0.
pub const CHI_SQUARE_CRITICAL_05: f64 = 3.841;

/// Natural logarithm of 2.
const LN_2: f64 = 0.693_147_180_559_945_3;

/// Base-2 logarithm.
#[must_use]
pub fn log2(x: f64) -> f64 {
    x.ln() / LN_2
}

/// Standard normal CDF — Abramowitz & Stegun approximation 26.2.17.
#[must_use]
#[allow(clippy::suboptimal_flops)] // formula clarity
pub fn normal_cdf(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    let t = 1.0 / (1.0 + 0.231_641_9 * x.abs());
    let d = 0.398_942_3 * (-x * x / 2.0).exp();
    let p = d
        * t
        * (0.319_381_5 + t * (-0.356_563_8 + t * (1.781_478 + t * (-1.821_256 + t * 1.330_274))));
    if x >= 0.0 { 1.0 - p } else { p }
}

/// Inverse standard normal CDF — Abramowitz & Stegun rational approximation 26.2.23.
#[must_use]
pub fn normal_quantile(p: f64) -> f64 {
    if p <= 0.0 {
        return f64::NEG_INFINITY;
    }
    if p >= 1.0 {
        return f64::INFINITY;
    }
    if (p - 0.5).abs() < f64::EPSILON {
        return 0.0;
    }

    let c0 = 2.515_517;
    let c1 = 0.802_853;
    let c2 = 0.010_328;
    let d1 = 1.432_788;
    let d2 = 0.189_269;
    let d3 = 0.001_308;

    let (t, sign) = if p < 0.5 {
        ((-2.0 * p.ln()).sqrt(), -1.0_f64)
    } else {
        ((-2.0 * (1.0 - p).ln()).sqrt(), 1.0_f64)
    };

    sign * (t - (c0 + c1 * t + c2 * t * t) / (1.0 + d1 * t + d2 * t * t + d3 * t * t * t))
}

/// Log-gamma using Stirling's approximation — O(1).
#[must_use]
pub fn log_gamma(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::INFINITY;
    }
    (x - 0.5) * x.ln() - x + 0.5 * (2.0 * PI).ln() + 1.0 / (12.0 * x)
}

/// Haldane-Anscombe correction: adds 0.5 to all cells when any cell is zero.
#[must_use]
pub fn apply_continuity_correction(a: f64, b: f64, c: f64, d: f64) -> (f64, f64, f64, f64) {
    if a == 0.0 || b == 0.0 || c == 0.0 || d == 0.0 {
        (a + 0.5, b + 0.5, c + 0.5, d + 0.5)
    } else {
        (a, b, c, d)
    }
}

/// Standard error of a log ratio: sqrt(1/a + 1/b + 1/c + 1/d).
#[must_use]
pub fn log_ratio_se(a: f64, b: f64, c: f64, d: f64) -> f64 {
    if a <= 0.0 || b <= 0.0 || c <= 0.0 || d <= 0.0 {
        return f64::INFINITY;
    }
    (1.0 / a + 1.0 / b + 1.0 / c + 1.0 / d).sqrt()
}

/// Pearson chi-square statistic for a 2×2 table: Σ (O - E)² / E.
#[must_use]
#[allow(clippy::many_single_char_names)]
pub fn chi_square_statistic(a: f64, b: f64, c: f64, d: f64) -> f64 {
    let n = a + b + c + d;
    if n == 0.0 {
        return 0.0;
    }
    let ea = (a + b) * (a + c) / n;
    let eb = (a + b) * (b + d) / n;
    let ec = (c + d) * (a + c) / n;
    let ed = (c + d) * (b + d) / n;
    let mut chi = 0.0;
    if ea > 0.0 {
        chi += (a - ea).powi(2) / ea;
    }
    if eb > 0.0 {
        chi += (b - eb).powi(2) / eb;
    }
    if ec > 0.0 {
        chi += (c - ec).powi(2) / ec;
    }
    if ed > 0.0 {
        chi += (d - ed).powi(2) / ed;
    }
    chi
}

/// Chi-square p-value for 1 degree of freedom.
#[must_use]
pub fn chi_square_p_value(chi_sq: f64) -> f64 {
    if chi_sq <= 0.0 {
        return 1.0;
    }
    2.0 * (1.0 - normal_cdf(chi_sq.sqrt()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_cdf_symmetry() {
        assert!((normal_cdf(0.0) - 0.5).abs() < 0.01);
        assert!((normal_cdf(1.96) - 0.975).abs() < 0.01);
        assert!((normal_cdf(-1.96) - 0.025).abs() < 0.01);
    }

    #[test]
    fn chi_square_critical_is_3_841() {
        let p = chi_square_p_value(CHI_SQUARE_CRITICAL_05);
        assert!((p - 0.05).abs() < 0.01, "p={p} expected ~0.05");
    }
}
