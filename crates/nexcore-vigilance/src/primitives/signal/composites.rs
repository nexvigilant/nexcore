//! # T2-C Signal Composites
//!
//! Composite structures built from T2-P atoms.
//!
//! ## Tier Classification
//!
//! All types in this module are **T2-C (Cross-Domain Composite)**:
//! - Composed from T1/T2-P primitives
//! - Include trait implementations
//! - Transferable across signal detection domains

use super::atoms::{
    Association, Count, Detected, Frequency, Method, Ratio, Source, Threshold, Timestamp,
};
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// Private MGPS (Multi-item Gamma Poisson Shrinker) helpers for EBGM
// ============================================================================
//
// DuMouchel W (1999). "Bayesian data mining in large frequency tables."
// The American Statistician 53(3):177-190.
//
// These are self-contained implementations to keep the primitive layer
// independent of nexcore-pv-core internals.

/// DuMouchel recommended prior: shape for signal distribution.
const MGPS_ALPHA1: f64 = 0.2;
/// DuMouchel recommended prior: rate for signal distribution.
const MGPS_BETA1: f64 = 0.1;
/// DuMouchel recommended prior: shape for null distribution.
const MGPS_ALPHA2: f64 = 2.0;
/// DuMouchel recommended prior: rate for null distribution.
const MGPS_BETA2: f64 = 4.0;
/// DuMouchel recommended prior: mixing proportion for signal component.
const MGPS_P: f64 = 0.1;

/// Stirling's approximation for log-gamma. O(1).
fn log_gamma_approx(x: f64) -> f64 {
    if x <= 0.0 {
        return f64::INFINITY;
    }
    (x - 0.5) * x.ln() - x + 0.5 * (2.0 * std::f64::consts::PI).ln() + 1.0 / (12.0 * x)
}

/// Inverse standard normal CDF (Abramowitz & Stegun 26.2.23). O(1).
#[allow(clippy::suboptimal_flops)]
fn normal_quantile_approx(p: f64) -> f64 {
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

    let (sign, t) = if p < 0.5 {
        (-1.0_f64, (-2.0 * p.ln()).sqrt())
    } else {
        (1.0_f64, (-2.0 * (1.0 - p).ln()).sqrt())
    };

    let numer = c0 + c1 * t + c2 * t * t;
    let denom = 1.0 + d1 * t + d2 * t * t + d3 * t * t * t;

    sign * (t - numer / denom)
}

/// Weight for a gamma-Poisson mixture component (log-space). O(1).
#[allow(clippy::suboptimal_flops)]
fn gamma_poisson_weight(n: f64, e: f64, alpha: f64, beta: f64, prior_p: f64) -> f64 {
    if prior_p <= 0.0 || alpha <= 0.0 || beta <= 0.0 {
        return 0.0;
    }
    if n > 1e6 || e > 1e6 {
        return 0.0;
    }

    let log_weight = log_gamma_approx(n + alpha) - log_gamma_approx(alpha) + alpha * beta.ln()
        - (n + alpha) * (e + beta).ln()
        + prior_p.ln();

    if log_weight.is_nan() || log_weight < -700.0 {
        0.0
    } else if log_weight > 700.0 {
        f64::MAX
    } else {
        log_weight.exp()
    }
}

/// Approximate posterior variance of log(lambda) for MGPS. O(1).
fn mgps_posterior_variance(n: f64) -> f64 {
    let var1 = if n + MGPS_ALPHA1 > 0.0 {
        1.0 / (n + MGPS_ALPHA1)
    } else {
        1.0
    };

    let var2 = if n + MGPS_ALPHA2 > 0.0 {
        1.0 / (n + MGPS_ALPHA2)
    } else {
        1.0
    };

    MGPS_P * var1 + (1.0 - MGPS_P) * var2
}

// ============================================================================
// T2-C: Table (Composite of N)
// ============================================================================

/// 2x2 contingency table (T2-C: Composite of Count).
///
/// Grounds to: 4 x Count (T1: Quantity).
///
/// ```text
///              Outcome+    Outcome-
/// Exposure+  |    a     |    b     |  (a+b)
/// Exposure-  |    c     |    d     |  (c+d)
///            |  (a+c)   |  (b+d)   |    N
/// ```
///
/// # Cross-Domain Instantiation
///
/// | Domain | Exposure | Outcome |
/// |--------|----------|---------|
/// | Pharmacovigilance | Drug | Adverse Event |
/// | Epidemiology | Risk Factor | Disease |
/// | Finance | Market Condition | Price Movement |
/// | Cybersecurity | Attack Vector | Breach |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Table {
    /// Exposure+ Outcome+ (cases of interest)
    pub a: Count,
    /// Exposure+ Outcome-
    pub b: Count,
    /// Exposure- Outcome+
    pub c: Count,
    /// Exposure- Outcome-
    pub d: Count,
}

impl Table {
    /// Create a new contingency table.
    #[inline]
    #[must_use]
    pub const fn new(a: Count, b: Count, c: Count, d: Count) -> Self {
        Self { a, b, c, d }
    }

    /// Create from raw u64 values.
    #[inline]
    #[must_use]
    pub const fn from_raw(a: u64, b: u64, c: u64, d: u64) -> Self {
        Self {
            a: Count::new(a),
            b: Count::new(b),
            c: Count::new(c),
            d: Count::new(d),
        }
    }

    /// Create from u32 values (for 32-bit ARM targets).
    ///
    /// Widening conversion — no data loss.
    #[inline]
    #[must_use]
    pub const fn from_u32(a: u32, b: u32, c: u32, d: u32) -> Self {
        Self {
            a: Count::new(a as u64),
            b: Count::new(b as u64),
            c: Count::new(c as u64),
            d: Count::new(d as u64),
        }
    }

    /// Total count (N).
    #[inline]
    #[must_use]
    pub fn total(&self) -> Count {
        self.a + self.b + self.c + self.d
    }

    /// Exposure+ total (a + b).
    #[inline]
    #[must_use]
    pub fn exposed_total(&self) -> Count {
        self.a + self.b
    }

    /// Exposure- total (c + d).
    #[inline]
    #[must_use]
    pub fn unexposed_total(&self) -> Count {
        self.c + self.d
    }

    /// Outcome+ total (a + c).
    #[inline]
    #[must_use]
    pub fn outcome_total(&self) -> Count {
        self.a + self.c
    }

    /// Outcome- total (b + d).
    #[inline]
    #[must_use]
    pub fn no_outcome_total(&self) -> Count {
        self.b + self.d
    }

    /// Frequency of outcome among exposed: a / (a+b).
    #[inline]
    #[must_use]
    pub fn freq_exposed(&self) -> Option<Frequency> {
        Frequency::from_count(self.a, self.exposed_total())
    }

    /// Frequency of outcome among unexposed: c / (c+d).
    #[inline]
    #[must_use]
    pub fn freq_unexposed(&self) -> Option<Frequency> {
        Frequency::from_count(self.c, self.unexposed_total())
    }

    /// Expected count under independence: E = (a+b)(a+c) / N.
    #[inline]
    #[must_use]
    pub fn expected(&self) -> Option<f64> {
        let n = self.total().as_f64();
        if n == 0.0 {
            None
        } else {
            Some(self.exposed_total().as_f64() * self.outcome_total().as_f64() / n)
        }
    }

    /// Check if table is valid (N > 0).
    #[inline]
    #[must_use]
    pub fn is_valid(&self) -> bool {
        !self.total().is_zero()
    }

    /// Proportional Reporting Ratio: (a/(a+b)) / (c/(c+d)).
    ///
    /// # Lex Primitiva Derivation
    /// ```text
    /// PRR = f_exposed / f_unexposed
    ///     = (a / (a+b)) / (c / (c+d))
    ///     = kappa(f_exposed, f_unexposed)
    /// ```
    #[must_use]
    pub fn prr(&self) -> Option<Ratio> {
        let f_exp = self.freq_exposed()?;
        let f_unexp = self.freq_unexposed()?;
        Ratio::from_frequencies(f_exp, f_unexp)
    }

    /// Reporting Odds Ratio: (a*d) / (b*c).
    ///
    /// # Lex Primitiva Derivation
    /// ```text
    /// ROR = (a/b) / (c/d)
    ///     = (a*d) / (b*c)
    ///     = kappa(odds_exposed, odds_unexposed)
    /// ```
    #[must_use]
    pub fn ror(&self) -> Option<Ratio> {
        let denom = self.b.value().checked_mul(self.c.value())?;
        if denom == 0 {
            return None;
        }
        let numer = self.a.value().checked_mul(self.d.value())?;
        Ratio::new(numer as f64 / denom as f64)
    }

    /// Chi-square statistic (Pearson's).
    ///
    /// # Formula
    /// ```text
    /// chi^2 = N * (|ad - bc|)^2 / ((a+b)(c+d)(a+c)(b+d))
    /// ```
    #[must_use]
    pub fn chi_square(&self) -> Option<f64> {
        let n = self.total().as_f64();
        if n == 0.0 {
            return None;
        }

        let ad = self.a.as_f64() * self.d.as_f64();
        let bc = self.b.as_f64() * self.c.as_f64();
        let diff = (ad - bc).abs();

        let denom = self.exposed_total().as_f64()
            * self.unexposed_total().as_f64()
            * self.outcome_total().as_f64()
            * self.no_outcome_total().as_f64();

        if denom == 0.0 {
            return None;
        }

        Some(n * diff * diff / denom)
    }

    /// Information Component: log2(observed / expected).
    ///
    /// # Lex Primitiva Derivation
    /// IC measures the information gained by observing the association.
    #[must_use]
    pub fn information_component(&self) -> Option<f64> {
        let observed = self.a.as_f64();
        let expected = self.expected()?;
        if expected == 0.0 || observed == 0.0 {
            return None;
        }
        let ic = (observed / expected).log2();
        if ic.is_finite() { Some(ic) } else { None }
    }

    /// PRR with 95% confidence interval using Woolf's method.
    ///
    /// # Lex Primitiva Derivation
    /// ```text
    /// ln(PRR) ± 1.96 * sqrt(1/a - 1/(a+b) + 1/c - 1/(c+d))
    /// ```
    #[must_use]
    pub fn prr_with_ci(&self) -> Option<(Ratio, Interval)> {
        let prr = self.prr()?;
        let ln_prr = prr.value().ln();

        // Woolf's variance estimate
        let a = self.a.as_f64();
        let b = self.b.as_f64();
        let c = self.c.as_f64();
        let d = self.d.as_f64();

        // Avoid division by zero
        if a == 0.0 || c == 0.0 {
            return None;
        }

        let ab = a + b;
        let cd = c + d;

        if ab == 0.0 || cd == 0.0 {
            return None;
        }

        let variance = 1.0 / a - 1.0 / ab + 1.0 / c - 1.0 / cd;
        if variance < 0.0 {
            return None;
        }

        let se = variance.sqrt();
        let z = 1.96; // 95% CI

        let lower = (ln_prr - z * se).exp();
        let upper = (ln_prr + z * se).exp();

        let interval = Interval::ci95(lower, upper)?;
        Some((prr, interval))
    }

    /// ROR with 95% confidence interval using Woolf's method.
    ///
    /// # Lex Primitiva Derivation
    /// ```text
    /// ln(ROR) ± 1.96 * sqrt(1/a + 1/b + 1/c + 1/d)
    /// ```
    #[must_use]
    pub fn ror_with_ci(&self) -> Option<(Ratio, Interval)> {
        let ror = self.ror()?;
        let ln_ror = ror.value().ln();

        let a = self.a.as_f64();
        let b = self.b.as_f64();
        let c = self.c.as_f64();
        let d = self.d.as_f64();

        // Avoid division by zero
        if a == 0.0 || b == 0.0 || c == 0.0 || d == 0.0 {
            return None;
        }

        let variance = 1.0 / a + 1.0 / b + 1.0 / c + 1.0 / d;
        let se = variance.sqrt();
        let z = 1.96; // 95% CI

        let lower = (ln_ror - z * se).exp();
        let upper = (ln_ror + z * se).exp();

        let interval = Interval::ci95(lower, upper)?;
        Some((ror, interval))
    }

    /// Empirical Bayes Geometric Mean (EBGM) using MGPS.
    ///
    /// DuMouchel's Multi-item Gamma Poisson Shrinker provides Bayesian
    /// shrinkage estimation to reduce false positives from sparse data.
    /// The prior is a mixture of two gamma distributions:
    ///
    /// ```text
    /// Prior = p * Gamma(alpha1, beta1) + (1-p) * Gamma(alpha2, beta2)
    /// ```
    ///
    /// # Lex Primitiva Derivation
    /// ```text
    /// EBGM = exp(Q1 * ln(mu1) + Q2 * ln(mu2))
    /// where mu_k = (n + alpha_k) / (E + beta_k)  (posterior means)
    ///       Q_k  = w_k / (w1 + w2)               (posterior weights)
    /// ```
    ///
    /// # References
    ///
    /// DuMouchel W (1999). "Bayesian data mining in large frequency tables."
    /// The American Statistician 53(3):177-190.
    #[must_use]
    #[allow(clippy::suboptimal_flops)]
    pub fn ebgm(&self) -> Option<Ratio> {
        let observed = self.a.as_f64();
        let expected = self.expected()?;

        if expected <= 0.0 {
            return None;
        }

        // Compute posterior weights for each gamma component
        let w1 = gamma_poisson_weight(observed, expected, MGPS_ALPHA1, MGPS_BETA1, MGPS_P);
        let w2 = gamma_poisson_weight(observed, expected, MGPS_ALPHA2, MGPS_BETA2, 1.0 - MGPS_P);

        let total_w = w1 + w2;
        if total_w <= 0.0 {
            return Ratio::new(observed / expected);
        }

        let q1 = w1 / total_w;
        let q2 = w2 / total_w;

        if q1.is_nan() || q2.is_nan() {
            return Ratio::new(observed / expected);
        }

        // Posterior means for each component
        let mean1 = (observed + MGPS_ALPHA1) / (expected + MGPS_BETA1);
        let mean2 = (observed + MGPS_ALPHA2) / (expected + MGPS_BETA2);

        // EBGM = geometric mean of posterior
        let ebgm = if mean1 > 0.0 && mean2 > 0.0 {
            let log_ebgm = q1 * mean1.ln() + q2 * mean2.ln();
            if log_ebgm.is_nan() || log_ebgm > 700.0 {
                observed / expected
            } else if log_ebgm < -700.0 {
                0.0
            } else {
                log_ebgm.exp()
            }
        } else {
            mean1.max(mean2)
        };

        let ebgm = if ebgm.is_nan() || ebgm.is_infinite() {
            observed / expected
        } else {
            ebgm
        };

        Ratio::new(ebgm)
    }

    /// EBGM with 90% credibility interval (EB05 to EB95).
    ///
    /// Returns the EBGM point estimate and the credibility interval
    /// `[EB05, EB95]` where EB05 is the 5th percentile (lower bound for
    /// signal detection).
    ///
    /// Signal criteria: `EB05 >= 2.0` (standard threshold).
    ///
    /// # Lex Primitiva Derivation
    /// ```text
    /// EB05 = exp(ln(EBGM) + z_0.05 * sigma)
    /// EB95 = exp(ln(EBGM) + z_0.95 * sigma)
    /// where sigma = sqrt(posterior_variance)
    /// ```
    #[must_use]
    pub fn ebgm_with_interval(&self) -> Option<(Ratio, Interval)> {
        let ebgm = self.ebgm()?;
        let observed = self.a.as_f64();
        let variance = mgps_posterior_variance(observed);

        let (eb05, eb95) = if variance > 0.0 && ebgm.value() > 0.0 {
            let sd = variance.sqrt();
            let z_05 = normal_quantile_approx(0.05);
            let z_95 = normal_quantile_approx(0.95);
            let log_ebgm = ebgm.value().ln();
            if log_ebgm.is_finite() {
                let eb05_val = z_05.mul_add(sd, log_ebgm).exp();
                let eb95_val = z_95.mul_add(sd, log_ebgm).exp();
                (
                    if eb05_val.is_finite() {
                        eb05_val
                    } else {
                        ebgm.value() * 0.5
                    },
                    if eb95_val.is_finite() {
                        eb95_val
                    } else {
                        ebgm.value() * 2.0
                    },
                )
            } else {
                (ebgm.value() * 0.5, ebgm.value() * 2.0)
            }
        } else {
            (ebgm.value() * 0.5, ebgm.value() * 2.0)
        };

        // EB05 to EB95 = 90% credibility interval
        let interval = Interval::new(eb05, eb95, 0.90)?;
        Some((ebgm, interval))
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::from_raw(0, 0, 0, 0)
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Table(a={}, b={}, c={}, d={}, N={})",
            self.a,
            self.b,
            self.c,
            self.d,
            self.total()
        )
    }
}

// ============================================================================
// T2-C: Interval (Composite of Ratio + Threshold)
// ============================================================================

/// Confidence/credibility interval (T2-C: Bounds).
///
/// Grounds to: 2 x f64 (lower, upper) + f64 (level).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Interval {
    /// Lower bound.
    lower: f64,
    /// Upper bound.
    upper: f64,
    /// Confidence level (e.g., 0.95 for 95%).
    level: f64,
}

impl Interval {
    /// Create a new interval.
    ///
    /// # Errors
    /// Returns `None` if lower > upper or level not in (0, 1).
    #[must_use]
    pub fn new(lower: f64, upper: f64, level: f64) -> Option<Self> {
        if lower > upper || !(0.0..=1.0).contains(&level) {
            None
        } else {
            Some(Self {
                lower,
                upper,
                level,
            })
        }
    }

    /// Create 95% confidence interval.
    #[must_use]
    pub fn ci95(lower: f64, upper: f64) -> Option<Self> {
        Self::new(lower, upper, 0.95)
    }

    /// Lower bound.
    #[inline]
    #[must_use]
    pub const fn lower(&self) -> f64 {
        self.lower
    }

    /// Upper bound.
    #[inline]
    #[must_use]
    pub const fn upper(&self) -> f64 {
        self.upper
    }

    /// Confidence level.
    #[inline]
    #[must_use]
    pub const fn level(&self) -> f64 {
        self.level
    }

    /// Check if interval excludes a value.
    #[inline]
    #[must_use]
    pub fn excludes(&self, value: f64) -> bool {
        value < self.lower || value > self.upper
    }

    /// Check if interval excludes null (1.0 for ratios, 0.0 for differences).
    #[inline]
    #[must_use]
    pub fn excludes_null_ratio(&self) -> bool {
        self.lower > 1.0
    }

    /// Width of the interval.
    #[inline]
    #[must_use]
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }

    /// Midpoint of the interval.
    #[inline]
    #[must_use]
    pub fn midpoint(&self) -> f64 {
        (self.lower + self.upper) / 2.0
    }
}

impl fmt::Display for Interval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{:.3}, {:.3}] ({:.0}% CI)",
            self.lower,
            self.upper,
            self.level * 100.0
        )
    }
}

// ============================================================================
// T2-C: Signal (Full Composite)
// ============================================================================

/// Complete signal detection result (T2-C: Full Composite).
///
/// # Lex Primitiva (Canonical Composition)
///
/// - **∃ (Existence)**: Signal detected/not — the core semantic output
/// - **κ (Comparison)**: Observed vs expected ratio (PRR, ROR, IC)
/// - **N (Quantity)**: Numeric metric values
/// - **∂ (Boundary)**: Confidence intervals and thresholds
/// - **Σ (Sum)**: Detection method selection coproduct
///
/// Dominant: **∃ (Existence)** @ 0.85 — signal detection fundamentally
/// answers "does this disproportionate association exist?"
///
/// # Composition Tree
/// ```text
/// Signal (T2-C) — dominant: ∃ (Existence)
/// +-- table: Table (T2-C)           — N (Quantity)
/// |   +-- a, b, c, d: Count (T1)
/// +-- ratio: Ratio (T2-P)           — κ (Comparison)
/// +-- interval: Option<Interval>    — ∂ (Boundary)
/// +-- chi_square: Option<f64>       — N (Quantity)
/// +-- detected: Detected (T1)       — ∃ (Existence) ← DOMINANT
/// +-- source: Source (T2-P)         — λ (Location)
/// +-- method: Method (T2-P)         — Σ (Sum)
/// +-- association: Option<Assoc>    — → (Causality)
/// +-- timestamp: Timestamp (T2-P)   — σ (Sequence)
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Signal {
    /// Contingency table.
    pub table: Table,
    /// Point estimate (PRR, ROR, etc.).
    pub ratio: Ratio,
    /// Confidence interval.
    pub interval: Option<Interval>,
    /// Chi-square statistic.
    pub chi_square: Option<f64>,
    /// Detection result.
    pub detected: Detected,
    /// Data source.
    pub source: Source,
    /// Detection method used.
    pub method: Method,
    /// The exposure-outcome association being tested.
    pub association: Option<Association>,
    /// When this signal was detected.
    pub timestamp: Timestamp,
}

impl Signal {
    /// Create a new signal result.
    #[must_use]
    pub fn new(
        table: Table,
        ratio: Ratio,
        interval: Option<Interval>,
        chi_square: Option<f64>,
        detected: Detected,
        source: Source,
    ) -> Self {
        Self {
            table,
            ratio,
            interval,
            chi_square,
            detected,
            source,
            method: Method::PRR,
            association: None,
            timestamp: Timestamp::default(),
        }
    }

    /// Create a new signal with full context.
    #[must_use]
    pub fn with_context(
        table: Table,
        ratio: Ratio,
        interval: Option<Interval>,
        chi_square: Option<f64>,
        detected: Detected,
        source: Source,
        method: Method,
        association: Option<Association>,
        timestamp: Timestamp,
    ) -> Self {
        Self {
            table,
            ratio,
            interval,
            chi_square,
            detected,
            source,
            method,
            association,
            timestamp,
        }
    }

    /// Create signal from table using Evans criteria.
    ///
    /// Evans criteria:
    /// - PRR >= 2.0
    /// - Chi-square >= 3.841 (p < 0.05)
    /// - n >= 3 cases
    #[must_use]
    pub fn from_table_evans(table: Table, source: Source) -> Option<Self> {
        let (ratio, interval) = table.prr_with_ci().unzip();
        let ratio = ratio?;
        let chi_square = table.chi_square();

        // Evans criteria
        let prr_pass = ratio.value() >= Threshold::STANDARD.value();
        let chi_pass = chi_square.map_or(false, |x| x >= Threshold::CHI_SQUARE_CRITICAL.value());
        let n_pass = table.a.value() >= 3;

        let detected = Detected::new(prr_pass && chi_pass && n_pass);

        Some(Self {
            table,
            ratio,
            interval,
            chi_square,
            detected,
            source,
            method: Method::PRR,
            association: None,
            timestamp: Timestamp::default(),
        })
    }

    /// Create signal from table using EBGM (Bayesian shrinkage).
    ///
    /// Signal criteria: EBGM >= 2.0 AND EB05 >= 2.0 AND n >= 3.
    #[must_use]
    pub fn from_table_ebgm(table: Table, source: Source) -> Option<Self> {
        let (ebgm, interval) = table.ebgm_with_interval()?;

        // EBGM signal criteria
        let ebgm_pass = ebgm.value() >= Threshold::STANDARD.value();
        let eb05_pass = interval.lower() >= Threshold::STANDARD.value();
        let n_pass = table.a.value() >= 3;

        let detected = Detected::new(ebgm_pass && eb05_pass && n_pass);

        Some(Self {
            table,
            ratio: ebgm,
            interval: Some(interval),
            chi_square: None, // EBGM is Bayesian, no chi-square
            detected,
            source,
            method: Method::EBGM,
            association: None,
            timestamp: Timestamp::default(),
        })
    }

    /// Create signal with association context.
    #[must_use]
    pub fn from_table_with_association(
        table: Table,
        source: Source,
        association: Association,
    ) -> Option<Self> {
        let mut signal = Self::from_table_evans(table, source)?;
        signal.association = Some(association);
        Some(signal)
    }

    /// Set the detection method.
    #[must_use]
    pub fn with_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Set the timestamp.
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: Timestamp) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Set the association.
    #[must_use]
    pub fn with_association(mut self, association: Association) -> Self {
        self.association = Some(association);
        self
    }

    /// Get the detection method.
    #[inline]
    #[must_use]
    pub const fn method(&self) -> Method {
        self.method
    }

    /// Get the association if set.
    #[inline]
    #[must_use]
    pub fn association(&self) -> Option<&Association> {
        self.association.as_ref()
    }

    /// Get the detection timestamp.
    #[inline]
    #[must_use]
    pub const fn timestamp(&self) -> Timestamp {
        self.timestamp
    }

    /// Case count (cell a).
    #[inline]
    #[must_use]
    pub fn case_count(&self) -> Count {
        self.table.a
    }

    /// Is this a detected signal?
    #[inline]
    #[must_use]
    pub fn is_signal(&self) -> bool {
        self.detected.is_signal()
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Signal({} | ratio={} | {} | source={})",
            self.table, self.ratio, self.detected, self.source
        )
    }
}

// ============================================================================
// T2-C: SignalStrength (Ordinal Classification)
// ============================================================================

/// Signal strength classification (T2-C: Ordinal).
///
/// Grounds to: Enum with ordered variants.
///
/// # Lex Primitiva
/// - Symbol: **Sigma** (Sum/Coproduct)
/// - Maps ratio magnitude to ordinal category
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SignalStrength {
    /// No signal detected.
    None,
    /// Weak signal - monitoring warranted.
    Weak,
    /// Moderate signal - investigation warranted.
    Moderate,
    /// Strong signal - action warranted.
    Strong,
    /// Critical signal - immediate action.
    Critical,
}

impl SignalStrength {
    /// Classify from ratio value using standard thresholds.
    #[must_use]
    pub fn from_ratio(ratio: Ratio) -> Self {
        let v = ratio.value();
        match v {
            x if x < 1.5 => Self::None,
            x if x < 2.0 => Self::Weak,
            x if x < 3.0 => Self::Moderate,
            x if x < 5.0 => Self::Strong,
            _ => Self::Critical,
        }
    }

    /// Check if signal warrants action.
    #[inline]
    #[must_use]
    pub const fn warrants_action(self) -> bool {
        matches!(self, Self::Strong | Self::Critical)
    }

    /// Check if signal warrants investigation.
    #[inline]
    #[must_use]
    pub const fn warrants_investigation(self) -> bool {
        matches!(self, Self::Moderate | Self::Strong | Self::Critical)
    }

    /// Check if signal warrants monitoring.
    #[inline]
    #[must_use]
    pub const fn warrants_monitoring(self) -> bool {
        !matches!(self, Self::None)
    }
}

impl Default for SignalStrength {
    fn default() -> Self {
        Self::None
    }
}

impl fmt::Display for SignalStrength {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Weak => write!(f, "Weak"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Strong => write!(f, "Strong"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

// ============================================================================
// T2-C: PvLoopStage (pharmacovigilance pipeline stage)
// ============================================================================

/// PV loop pipeline stage — which stage of the pharmacovigilance cycle
/// a signal is currently being processed through.
///
/// Mirrors `nexcore-pvos::typestate::SignalDetected/Evaluated/Validated/Actioned/Monitoring`
/// at the runtime level, for consumers that need loop stage awareness without
/// compile-time typestate enforcement.
///
/// # PV Loop
/// ```text
/// Detect → Evaluate → Assess → Act → Monitor
///   ↑                                    │
///   └──────────── feedback ──────────────┘
/// ```
///
/// # Lex Primitiva
/// - **ς (State)**: Each stage is a discrete pipeline position
/// - **σ (Sequence)**: Stages follow the PV loop ordering
/// - **∂ (Boundary)**: Thresholds gate transitions between stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PvLoopStage {
    /// Stage 1: Statistical signal detection from FAERS/spontaneous data.
    Detect,
    /// Stage 2: Evidence gathering (literature, labeling, clinical trials).
    Evaluate,
    /// Stage 3: Causality assessment (Naranjo, WHO-UMC).
    Assess,
    /// Stage 4: Regulatory/clinical action determination.
    Act,
    /// Stage 5: Ongoing surveillance. Non-terminal — can feedback to Detect.
    Monitor,
}

impl PvLoopStage {
    /// Next stage in the loop. Monitor returns None (decision point: feedback or close).
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Detect => Some(Self::Evaluate),
            Self::Evaluate => Some(Self::Assess),
            Self::Assess => Some(Self::Act),
            Self::Act => Some(Self::Monitor),
            Self::Monitor => None, // Decision point: feedback() → Detect or close
        }
    }

    /// All stages in order.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::Detect,
            Self::Evaluate,
            Self::Assess,
            Self::Act,
            Self::Monitor,
        ]
    }

    /// Stage name matching nexcore-pvos convention.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Detect => "detected",
            Self::Evaluate => "evaluated",
            Self::Assess => "validated",
            Self::Act => "actioned",
            Self::Monitor => "monitoring",
        }
    }
}

impl fmt::Display for PvLoopStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Detect => write!(f, "Detect"),
            Self::Evaluate => write!(f, "Evaluate"),
            Self::Assess => write!(f, "Assess"),
            Self::Act => write!(f, "Act"),
            Self::Monitor => write!(f, "Monitor"),
        }
    }
}

// ============================================================================
// T2-C: SignalLifecycle (directed state machine — review workflow)
// ============================================================================

/// Signal lifecycle state machine (T2-C: State Machine).
///
/// Tracks a signal through its **review workflow** from detection to closure.
/// This is the administrative review process, orthogonal to [`PvLoopStage`]
/// which tracks which PV pipeline stage the signal is being processed through.
///
/// # Lex Primitiva (Canonical Composition)
///
/// - **ς (State)**: Each variant is a discrete context-at-a-point-in-time
/// - **σ (Sequence)**: Transitions follow a directed temporal ordering
/// - **∝ (Irreversibility)**: Terminal state (Closed) cannot be exited
///
/// Dominant: **ς (State)** @ 0.80 — each variant captures a distinct
/// lifecycle position; transitions are the grammar connecting them.
///
/// # Transitions
/// ```text
/// New -> UnderReview -> Confirmed -> Closed
///              \-----> Rejected -> Closed
///              \-----> Escalated -> Confirmed -> Closed
/// ```
///
/// # Relationship to PvLoopStage
///
/// A signal can be in any review state during any loop stage:
/// - `New` + `Detect`: Fresh signal just identified
/// - `UnderReview` + `Assess`: Being reviewed during causality assessment
/// - `Confirmed` + `Monitor`: Confirmed signal under ongoing surveillance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalLifecycle {
    /// Newly detected, not yet reviewed.
    New,
    /// Under active review.
    UnderReview,
    /// Confirmed as genuine signal.
    Confirmed,
    /// Rejected as false positive.
    Rejected,
    /// Escalated for senior review.
    Escalated,
    /// Closed (terminal state — irreversible).
    Closed,
}

/// Backwards-compatible type alias.
/// Tier: T2-C (ς + σ + ∝)
#[deprecated(
    since = "0.1.0",
    note = "Use SignalLifecycle instead — State collides with T1 primitive"
)]
pub type State = SignalLifecycle;

impl SignalLifecycle {
    /// Check if lifecycle is in terminal state.
    #[inline]
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Closed)
    }

    /// Check if lifecycle is in an actionable state.
    #[inline]
    #[must_use]
    pub const fn is_actionable(self) -> bool {
        matches!(self, Self::Confirmed | Self::Escalated)
    }

    /// Check if lifecycle is pending review.
    #[inline]
    #[must_use]
    pub const fn is_pending(self) -> bool {
        matches!(self, Self::New | Self::UnderReview | Self::Escalated)
    }

    /// Get valid next states (state machine transitions).
    #[must_use]
    pub const fn valid_transitions(self) -> &'static [SignalLifecycle] {
        match self {
            Self::New => &[Self::UnderReview, Self::Rejected],
            Self::UnderReview => &[Self::Confirmed, Self::Rejected, Self::Escalated],
            Self::Confirmed => &[Self::Closed],
            Self::Rejected => &[Self::Closed, Self::UnderReview], // Can reopen
            Self::Escalated => &[Self::Confirmed, Self::Rejected],
            Self::Closed => &[], // Terminal — irreversible (∝)
        }
    }

    /// Check if transition to target lifecycle state is valid.
    #[must_use]
    pub fn can_transition_to(self, target: Self) -> bool {
        self.valid_transitions().contains(&target)
    }

    /// PV loop stages that are compatible with this review state.
    ///
    /// A signal can be in any review state during any loop stage, but some
    /// combinations are more common. This returns the typical stages.
    #[must_use]
    pub const fn compatible_loop_stages(self) -> &'static [PvLoopStage] {
        match self {
            Self::New => &[PvLoopStage::Detect],
            Self::UnderReview => &[PvLoopStage::Evaluate, PvLoopStage::Assess],
            Self::Confirmed => &[PvLoopStage::Act, PvLoopStage::Monitor],
            Self::Rejected => &[PvLoopStage::Assess],
            Self::Escalated => &[PvLoopStage::Assess, PvLoopStage::Act],
            Self::Closed => &[], // Terminal — no active loop stage
        }
    }
}

impl Default for SignalLifecycle {
    fn default() -> Self {
        Self::New
    }
}

impl fmt::Display for SignalLifecycle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::New => write!(f, "New"),
            Self::UnderReview => write!(f, "UnderReview"),
            Self::Confirmed => write!(f, "Confirmed"),
            Self::Rejected => write!(f, "Rejected"),
            Self::Escalated => write!(f, "Escalated"),
            Self::Closed => write!(f, "Closed"),
        }
    }
}
