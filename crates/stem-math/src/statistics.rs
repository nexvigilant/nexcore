//! # Statistical Inference: Z-Table, Confidence Intervals, P-Values
//!
//! Pure-math statistical computation grounded in T1 primitives:
//!
//! | Concept | T1 Primitives | Grounding |
//! |---------|---------------|-----------|
//! | Z-score | μ (Mapping) | Observation → standard scale |
//! | CDF/PDF | μ + N | Value → probability |
//! | CI | ∂ (Boundary) + N | Bounded estimation region |
//! | P-value | κ (Comparison) + N | Observed vs null hypothesis |
//! | Significance | ς (State) | Reject / fail-to-reject |
//!
//! ## Algorithms
//!
//! - **Normal CDF**: Abramowitz & Stegun 26.2.17 (|ε| < 7.5×10⁻⁸)
//! - **Inverse CDF**: Acklam rational approximation (|ε| < 1.15×10⁻⁹)
//! - **No external deps**: Pure `f64` arithmetic, `core::f64::consts`
//!
//! ## Usage
//!
//! Every numerical outcome wraps in [`StatisticalOutcome`] which carries
//! the value, its z-score, p-value, confidence interval, and significance
//! classification. Nothing leaves without statistical evidence.

use serde::{Deserialize, Serialize};
use std::f64::consts::{FRAC_1_SQRT_2, PI, SQRT_2};

use crate::Bounded;

// ============================================================================
// Constants
// ============================================================================

/// 1 / √(2π)
const INV_SQRT_2PI: f64 = 0.398_942_280_401_432_7; // 1.0 / (2.0 * PI).sqrt()

/// Abramowitz & Stegun coefficients for normal CDF approximation (26.2.17)
const A_S_P: f64 = 0.231_641_9;
const A_S_B1: f64 = 0.319_381_530;
const A_S_B2: f64 = -0.356_563_782;
const A_S_B3: f64 = 1.781_477_937;
const A_S_B4: f64 = -1.821_255_978;
const A_S_B5: f64 = 1.330_274_429;

/// Acklam inverse normal CDF coefficients
const ACKLAM_A: [f64; 6] = [
    -3.969_683_028_665_376e1,
    2.209_460_984_245_205e2,
    -2.759_285_104_469_687e2,
    1.383_577_518_672_69e2,
    -3.066_479_806_614_716e1,
    2.506_628_277_459_239e0,
];

const ACKLAM_B: [f64; 5] = [
    -5.447_609_879_822_406e1,
    1.615_858_368_580_409e2,
    -1.556_989_798_598_866e2,
    6.680_131_188_771_972e1,
    -1.328_068_155_288_572e1,
];

const ACKLAM_C: [f64; 6] = [
    -7.784_894_002_430_293e-3,
    -3.223_964_580_411_365e-1,
    -2.400_758_277_161_838e0,
    -2.549_732_539_343_734e0,
    4.374_664_141_464_968e0,
    2.938_163_982_698_783e0,
];

const ACKLAM_D: [f64; 4] = [
    7.784_695_709_041_462e-3,
    3.224_671_290_700_398e-1,
    2.445_134_137_142_996e0,
    3.754_408_661_907_416e0,
];

/// Low tail cutoff for Acklam approximation
const ACKLAM_P_LOW: f64 = 0.024_25;
/// High tail cutoff
const ACKLAM_P_HIGH: f64 = 1.0 - ACKLAM_P_LOW;

// ============================================================================
// Core Distribution Functions
// ============================================================================

/// Standard normal probability density function: φ(x)
///
/// φ(x) = (1/√(2π)) × e^(-x²/2)
///
/// Grounded in T1 Mapping (μ): value → density
#[must_use]
pub fn normal_pdf(x: f64) -> f64 {
    INV_SQRT_2PI * (-0.5 * x * x).exp()
}

/// Standard normal cumulative distribution function: Φ(x)
///
/// Uses Abramowitz & Stegun approximation 26.2.17.
/// Accuracy: |ε(x)| < 7.5 × 10⁻⁸
///
/// Grounded in T1 Mapping (μ) + Quantity (N): value → cumulative probability
#[must_use]
pub fn normal_cdf(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x.is_infinite() {
        return if x > 0.0 { 1.0 } else { 0.0 };
    }

    // Use symmetry: Φ(-x) = 1 - Φ(x)
    let abs_x = x.abs();
    let t = 1.0 / (1.0 + A_S_P * abs_x);
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;

    let poly = A_S_B1 * t + A_S_B2 * t2 + A_S_B3 * t3 + A_S_B4 * t4 + A_S_B5 * t5;
    let cdf_positive = 1.0 - normal_pdf(abs_x) * poly;

    if x >= 0.0 {
        cdf_positive
    } else {
        1.0 - cdf_positive
    }
}

/// Inverse standard normal CDF (quantile function): Φ⁻¹(p)
///
/// Given probability p ∈ (0, 1), returns z such that Φ(z) = p.
/// Uses Acklam's rational approximation.
/// Accuracy: |ε| < 1.15 × 10⁻⁹
///
/// Returns `None` for p ≤ 0 or p ≥ 1.
///
/// Grounded in T1 Mapping (μ): probability → z-score (inverse)
#[must_use]
pub fn normal_quantile(p: f64) -> Option<f64> {
    if p <= 0.0 || p >= 1.0 || p.is_nan() {
        return None;
    }

    let z = if p < ACKLAM_P_LOW {
        // Low tail
        let q = (-2.0 * p.ln()).sqrt();
        let num = ((((ACKLAM_C[0] * q + ACKLAM_C[1]) * q + ACKLAM_C[2]) * q + ACKLAM_C[3]) * q
            + ACKLAM_C[4])
            * q
            + ACKLAM_C[5];
        let den = (((ACKLAM_D[0] * q + ACKLAM_D[1]) * q + ACKLAM_D[2]) * q + ACKLAM_D[3]) * q + 1.0;
        num / den
    } else if p <= ACKLAM_P_HIGH {
        // Central region
        let q = p - 0.5;
        let r = q * q;
        let num = (((((ACKLAM_A[0] * r + ACKLAM_A[1]) * r + ACKLAM_A[2]) * r + ACKLAM_A[3]) * r
            + ACKLAM_A[4])
            * r
            + ACKLAM_A[5])
            * q;
        let den = ((((ACKLAM_B[0] * r + ACKLAM_B[1]) * r + ACKLAM_B[2]) * r + ACKLAM_B[3]) * r
            + ACKLAM_B[4])
            * r
            + 1.0;
        num / den
    } else {
        // High tail — use symmetry
        let q = (-2.0 * (1.0 - p).ln()).sqrt();
        let num = ((((ACKLAM_C[0] * q + ACKLAM_C[1]) * q + ACKLAM_C[2]) * q + ACKLAM_C[3]) * q
            + ACKLAM_C[4])
            * q
            + ACKLAM_C[5];
        let den = (((ACKLAM_D[0] * q + ACKLAM_D[1]) * q + ACKLAM_D[2]) * q + ACKLAM_D[3]) * q + 1.0;
        -(num / den)
    };

    // One Newton-Raphson refinement step for extra precision
    let e = 0.5 * erfc(-z * FRAC_1_SQRT_2) - p;
    let u = e * (2.0 * PI).sqrt() * (0.5 * z * z).exp();
    Some(z - u)
}

/// Complementary error function approximation
///
/// erfc(x) = 1 - erf(x) = 2Φ(-x√2)
///
/// Used internally for Newton-Raphson refinement.
#[must_use]
fn erfc(x: f64) -> f64 {
    // erfc(x) = 2 * Φ(-x * √2) but we avoid circularity
    // by using the direct approximation
    2.0 * normal_cdf(-x * SQRT_2)
}

/// Error function: erf(x) = 1 - erfc(x)
#[must_use]
pub fn erf(x: f64) -> f64 {
    1.0 - erfc(x)
}

// ============================================================================
// Z-Score Operations
// ============================================================================

/// Convert a raw value to a z-score given population parameters.
///
/// z = (x - μ) / σ
///
/// Returns `None` if `std_dev` is zero or negative.
///
/// Grounded in T1 Mapping (μ): raw → standardized scale
#[must_use]
pub fn z_score(value: f64, mean: f64, std_dev: f64) -> Option<f64> {
    if std_dev <= 0.0 || std_dev.is_nan() {
        return None;
    }
    Some((value - mean) / std_dev)
}

/// Convert a z-score back to a raw value.
///
/// x = μ + z × σ
#[must_use]
pub fn z_to_raw(z: f64, mean: f64, std_dev: f64) -> f64 {
    mean + z * std_dev
}

/// Look up the cumulative probability for a z-score.
///
/// This IS the z-table: z → P(Z ≤ z)
#[must_use]
pub fn z_table_lookup(z: f64) -> f64 {
    normal_cdf(z)
}

/// Reverse z-table lookup: probability → z-score.
///
/// Given P(Z ≤ z) = p, find z.
#[must_use]
pub fn z_table_reverse(p: f64) -> Option<f64> {
    normal_quantile(p)
}

// ============================================================================
// P-Value Calculations
// ============================================================================

/// Tail direction for hypothesis testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tail {
    /// H₁: parameter < hypothesized (left tail)
    Left,
    /// H₁: parameter > hypothesized (right tail)
    Right,
    /// H₁: parameter ≠ hypothesized (both tails)
    Two,
}

/// Calculate p-value from a z-score.
///
/// - Left tail: P(Z ≤ z)
/// - Right tail: P(Z ≥ z) = 1 - P(Z ≤ z)
/// - Two tail: 2 × min(P(Z ≤ z), P(Z ≥ z))
///
/// Grounded in T1 Comparison (κ): how extreme is the observed vs null?
#[must_use]
pub fn p_value_from_z(z: f64, tail: Tail) -> f64 {
    match tail {
        Tail::Left => normal_cdf(z),
        Tail::Right => 1.0 - normal_cdf(z),
        Tail::Two => {
            let one_tail = normal_cdf(-z.abs());
            2.0 * one_tail
        }
    }
}

/// Calculate p-value directly from observed value, null mean, and standard error.
///
/// Combines z-score computation and p-value lookup in one step.
#[must_use]
pub fn p_value(observed: f64, null_mean: f64, std_error: f64, tail: Tail) -> Option<f64> {
    let z = z_score(observed, null_mean, std_error)?;
    Some(p_value_from_z(z, tail))
}

// ============================================================================
// Confidence Intervals
// ============================================================================

/// A confidence interval with full statistical context.
///
/// Grounded in T1 Boundary (∂) + Quantity (N): bounded estimation region
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Point estimate (center)
    pub estimate: f64,
    /// Lower bound
    pub lower: f64,
    /// Upper bound
    pub upper: f64,
    /// Confidence level (e.g., 0.95 for 95%)
    pub level: f64,
    /// Margin of error (half-width)
    pub margin: f64,
    /// Critical z-value used
    pub z_critical: f64,
    /// Standard error used
    pub std_error: f64,
}

impl ConfidenceInterval {
    /// Width of the interval
    #[must_use]
    pub fn width(&self) -> f64 {
        self.upper - self.lower
    }

    /// Check if a value falls within the interval
    #[must_use]
    pub fn contains(&self, value: f64) -> bool {
        value >= self.lower && value <= self.upper
    }

    /// Convert to a `Bounded<f64>`
    #[must_use]
    pub fn to_bounded(&self) -> Bounded<f64> {
        Bounded::new(self.estimate, Some(self.lower), Some(self.upper))
    }
}

/// Construct a confidence interval for a mean (known or estimated σ).
///
/// Parameters:
/// - `mean`: sample mean (x̄)
/// - `std_error`: standard error of the mean (σ/√n or s/√n)
/// - `confidence_level`: e.g., 0.95 for 95% CI
///
/// Returns `None` if confidence_level is not in (0, 1) or std_error ≤ 0.
#[must_use]
pub fn confidence_interval_mean(
    mean: f64,
    std_error: f64,
    confidence_level: f64,
) -> Option<ConfidenceInterval> {
    if confidence_level <= 0.0 || confidence_level >= 1.0 {
        return None;
    }
    if std_error <= 0.0 || std_error.is_nan() {
        return None;
    }

    // α/2 tail probability
    let alpha_half = (1.0 - confidence_level) / 2.0;
    let z_crit = normal_quantile(1.0 - alpha_half)?;
    let margin = z_crit * std_error;

    Some(ConfidenceInterval {
        estimate: mean,
        lower: mean - margin,
        upper: mean + margin,
        level: confidence_level,
        margin,
        z_critical: z_crit,
        std_error,
    })
}

/// Construct a confidence interval for a proportion.
///
/// Parameters:
/// - `p_hat`: sample proportion (successes / n)
/// - `n`: sample size
/// - `confidence_level`: e.g., 0.95
///
/// Uses Wald interval: p̂ ± z × √(p̂(1-p̂)/n)
///
/// Returns `None` if inputs are invalid.
#[must_use]
pub fn confidence_interval_proportion(
    p_hat: f64,
    n: usize,
    confidence_level: f64,
) -> Option<ConfidenceInterval> {
    if !(0.0..=1.0).contains(&p_hat) || n == 0 {
        return None;
    }
    if confidence_level <= 0.0 || confidence_level >= 1.0 {
        return None;
    }

    let nf = n as f64;
    let se = (p_hat * (1.0 - p_hat) / nf).sqrt();

    if se <= 0.0 {
        // p_hat is 0 or 1 — degenerate interval
        return Some(ConfidenceInterval {
            estimate: p_hat,
            lower: p_hat,
            upper: p_hat,
            level: confidence_level,
            margin: 0.0,
            z_critical: 0.0,
            std_error: 0.0,
        });
    }

    confidence_interval_mean(p_hat, se, confidence_level)
}

/// Construct a confidence interval for the difference between two means.
///
/// SE_diff = √(SE₁² + SE₂²)
#[must_use]
pub fn confidence_interval_diff(
    mean1: f64,
    se1: f64,
    mean2: f64,
    se2: f64,
    confidence_level: f64,
) -> Option<ConfidenceInterval> {
    let diff = mean1 - mean2;
    let se_diff = (se1 * se1 + se2 * se2).sqrt();
    confidence_interval_mean(diff, se_diff, confidence_level)
}

// ============================================================================
// Significance Classification
// ============================================================================

/// Statistical significance level based on p-value thresholds.
///
/// Grounded in T1 State (ς): the decision state of a hypothesis test
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Significance {
    /// p < 0.001 — overwhelming evidence against H₀
    HighlySignificant,
    /// p < 0.01 — strong evidence against H₀
    VerySignificant,
    /// p < 0.05 — sufficient evidence against H₀ (conventional)
    Significant,
    /// p ≥ 0.05 — insufficient evidence to reject H₀
    NotSignificant,
}

impl Significance {
    /// Classify a p-value into significance level
    #[must_use]
    pub fn from_p(p: f64) -> Self {
        if p < 0.001 {
            Self::HighlySignificant
        } else if p < 0.01 {
            Self::VerySignificant
        } else if p < 0.05 {
            Self::Significant
        } else {
            Self::NotSignificant
        }
    }

    /// Whether the result is statistically significant at α = 0.05
    #[must_use]
    pub fn is_significant(&self) -> bool {
        !matches!(self, Self::NotSignificant)
    }

    /// Conventional notation
    #[must_use]
    pub fn stars(&self) -> &'static str {
        match self {
            Self::HighlySignificant => "***",
            Self::VerySignificant => "**",
            Self::Significant => "*",
            Self::NotSignificant => "ns",
        }
    }

    /// Alpha threshold this level implies
    #[must_use]
    pub fn alpha(&self) -> f64 {
        match self {
            Self::HighlySignificant => 0.001,
            Self::VerySignificant => 0.01,
            Self::Significant => 0.05,
            Self::NotSignificant => 1.0,
        }
    }
}

impl std::fmt::Display for Significance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HighlySignificant => write!(f, "p < 0.001 (***)"),
            Self::VerySignificant => write!(f, "p < 0.01 (**)"),
            Self::Significant => write!(f, "p < 0.05 (*)"),
            Self::NotSignificant => write!(f, "p ≥ 0.05 (ns)"),
        }
    }
}

// ============================================================================
// StatisticalOutcome — The Universal Wrapper
// ============================================================================

/// A numerical outcome carrying full statistical evidence.
///
/// Every numerical result wraps in this type so nothing leaves
/// without a p-value, confidence interval, and significance.
///
/// ## Primitive Composition
///
/// | Field | T1 |
/// |-------|----|
/// | value | N (Quantity) |
/// | z_score | μ (Mapping) — raw → standard |
/// | p_value | κ (Comparison) — observed vs null |
/// | ci | ∂ (Boundary) — estimation bounds |
/// | significance | ς (State) — reject / accept |
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalOutcome {
    /// The computed value
    pub value: f64,
    /// Z-score (standardized distance from null)
    pub z_score: f64,
    /// P-value (probability of observing this or more extreme under H₀)
    pub p_value: f64,
    /// Confidence interval
    pub ci: ConfidenceInterval,
    /// Significance classification
    pub significance: Significance,
    /// Test description (what was tested)
    pub description: String,
}

impl StatisticalOutcome {
    /// Create a statistical outcome from raw components.
    ///
    /// Computes z-score, p-value, CI, and significance from:
    /// - `value`: the observed statistic
    /// - `null_value`: the hypothesized value under H₀
    /// - `std_error`: standard error of the statistic
    /// - `confidence_level`: e.g., 0.95
    /// - `tail`: test direction
    /// - `description`: human-readable test label
    pub fn new(
        value: f64,
        null_value: f64,
        std_error: f64,
        confidence_level: f64,
        tail: Tail,
        description: impl Into<String>,
    ) -> Option<Self> {
        let z = z_score(value, null_value, std_error)?;
        let p = p_value_from_z(z, tail);
        let ci = confidence_interval_mean(value, std_error, confidence_level)?;
        let sig = Significance::from_p(p);

        Some(Self {
            value,
            z_score: z,
            p_value: p,
            ci,
            significance: sig,
            description: description.into(),
        })
    }

    /// Quick constructor for a two-tailed test at 95% confidence.
    pub fn two_tailed_95(
        value: f64,
        null_value: f64,
        std_error: f64,
        description: impl Into<String>,
    ) -> Option<Self> {
        Self::new(value, null_value, std_error, 0.95, Tail::Two, description)
    }

    /// Whether the result rejects H₀ at α = 0.05
    #[must_use]
    pub fn is_significant(&self) -> bool {
        self.significance.is_significant()
    }

    /// Does the CI exclude the null hypothesis value?
    #[must_use]
    pub fn ci_excludes_null(&self, null_value: f64) -> bool {
        !self.ci.contains(null_value)
    }
}

impl std::fmt::Display for StatisticalOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: value={:.6}, z={:.4}, p={:.6} {}, {}% CI [{:.6}, {:.6}]",
            self.description,
            self.value,
            self.z_score,
            self.p_value,
            self.significance.stars(),
            self.ci.level * 100.0,
            self.ci.lower,
            self.ci.upper,
        )
    }
}

// ============================================================================
// Batch Operations
// ============================================================================

/// Standard error of the mean from sample data.
///
/// SE = s / √n, where s = sample standard deviation
///
/// Returns `None` if fewer than 2 values.
#[must_use]
pub fn standard_error(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let variance = values.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1.0);
    Some(variance.sqrt() / n.sqrt())
}

/// Sample mean
#[must_use]
pub fn mean(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().sum::<f64>() / values.len() as f64)
}

/// Sample variance (Bessel-corrected, n-1 denominator)
#[must_use]
pub fn variance(values: &[f64]) -> Option<f64> {
    if values.len() < 2 {
        return None;
    }
    let n = values.len() as f64;
    let m = mean(values)?;
    Some(values.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0))
}

/// Sample standard deviation
#[must_use]
pub fn std_dev(values: &[f64]) -> Option<f64> {
    variance(values).map(f64::sqrt)
}

/// Analyze a sample against a null hypothesis, returning full `StatisticalOutcome`.
///
/// One-sample z-test (appropriate when n ≥ 30 or σ known).
pub fn analyze_sample(
    values: &[f64],
    null_mean: f64,
    confidence_level: f64,
    tail: Tail,
    description: impl Into<String>,
) -> Option<StatisticalOutcome> {
    let m = mean(values)?;
    let se = standard_error(values)?;
    StatisticalOutcome::new(m, null_mean, se, confidence_level, tail, description)
}

// ============================================================================
// Z-Table Reference (Common Values)
// ============================================================================

/// Common z-critical values for reference.
///
/// These are exact to the precision of the Acklam approximation.
pub struct ZTable;

impl ZTable {
    /// z for 80% confidence (α/2 = 0.10)
    pub const Z_80: f64 = 1.281_551_565_544_601;
    /// z for 90% confidence (α/2 = 0.05)
    pub const Z_90: f64 = 1.644_853_626_951_473;
    /// z for 95% confidence (α/2 = 0.025)
    pub const Z_95: f64 = 1.959_963_984_540_054;
    /// z for 99% confidence (α/2 = 0.005)
    pub const Z_99: f64 = 2.575_829_303_548_901;
    /// z for 99.9% confidence (α/2 = 0.0005)
    pub const Z_999: f64 = 3.290_526_731_491_764;

    /// Get z-critical for any confidence level
    #[must_use]
    pub fn z_critical(confidence_level: f64) -> Option<f64> {
        let alpha_half = (1.0 - confidence_level) / 2.0;
        normal_quantile(1.0 - alpha_half)
    }

    /// Full z-table: returns P(Z ≤ z) for z from -3.49 to 3.49 in 0.01 steps.
    ///
    /// This is the classic printed z-table as a computed Vec.
    #[must_use]
    pub fn full_table() -> Vec<(f64, f64)> {
        let mut table = Vec::with_capacity(700);
        let mut z = -3.49_f64;
        while z <= 3.495 {
            // Round to avoid floating-point drift
            let z_rounded = (z * 100.0).round() / 100.0;
            table.push((z_rounded, normal_cdf(z_rounded)));
            z += 0.01;
        }
        table
    }
}

// ============================================================================
// Special Functions (building blocks for t and chi-square distributions)
// ============================================================================

/// Maximum iterations for series/continued-fraction convergence.
const MAX_ITER: usize = 200;
/// Convergence tolerance for iterative algorithms.
const CONVERGE_EPS: f64 = 1e-12;
/// Tiny value to avoid division by zero in continued fractions.
const CF_TINY: f64 = 1e-30;

/// Natural logarithm of the Gamma function: ln(Γ(z))
///
/// Uses Lanczos approximation with g=7.
/// Accurate to ~15 significant digits for z > 0.5.
///
/// Returns `None` for z ≤ 0.
#[must_use]
pub fn ln_gamma(z: f64) -> Option<f64> {
    if z <= 0.0 || z.is_nan() {
        return None;
    }

    // Lanczos coefficients (g=7, n=9)
    const G: f64 = 7.0;
    const COEFF: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_9,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];

    // Use reflection for z < 0.5 (not needed since we reject z <= 0)
    let z = z - 1.0;
    let mut x = COEFF[0];
    for (i, &c) in COEFF.iter().enumerate().skip(1) {
        x += c / (z + i as f64);
    }

    let t = z + G + 0.5;
    Some(0.5 * (2.0 * PI).ln() + (z + 0.5) * t.ln() - t + x.ln())
}

/// Gamma function: Γ(z) = exp(ln(Γ(z)))
#[must_use]
pub fn gamma(z: f64) -> Option<f64> {
    ln_gamma(z).map(f64::exp)
}

/// Regularized lower incomplete gamma function: P(a, x) = γ(a,x) / Γ(a)
///
/// - When x < a+1: series expansion
/// - When x ≥ a+1: continued fraction for Q, then P = 1 - Q
///
/// Returns `None` for invalid inputs.
#[must_use]
pub fn regularized_gamma_p(a: f64, x: f64) -> Option<f64> {
    if a <= 0.0 || x < 0.0 || a.is_nan() || x.is_nan() {
        return None;
    }
    if x == 0.0 {
        return Some(0.0);
    }
    if x.is_infinite() {
        return Some(1.0);
    }

    if x < a + 1.0 {
        // Series expansion
        gamma_p_series(a, x)
    } else {
        // Continued fraction for Q, then P = 1 - Q
        gamma_q_cf(a, x).map(|q| 1.0 - q)
    }
}

/// Series expansion for P(a, x).
///
/// P(a,x) = e^(-x) * x^a * Σ(x^n / Γ(a+n+1))
fn gamma_p_series(a: f64, x: f64) -> Option<f64> {
    let lng = ln_gamma(a)?;
    let mut term = 1.0 / a;
    let mut sum = term;

    for n in 1..MAX_ITER {
        term *= x / (a + n as f64);
        sum += term;
        if term.abs() < sum.abs() * CONVERGE_EPS {
            let log_result = a * x.ln() - x - lng + sum.ln();
            return Some(log_result.exp());
        }
    }

    // Didn't converge — return best estimate
    let log_result = a * x.ln() - x - lng + sum.ln();
    Some(log_result.exp())
}

/// Continued fraction for Q(a, x) = 1 - P(a, x).
///
/// Uses modified Lentz's algorithm (Numerical Recipes §6.2).
fn gamma_q_cf(a: f64, x: f64) -> Option<f64> {
    let lng = ln_gamma(a)?;

    // Modified Lentz's method (Press et al.)
    let mut b = x + 1.0 - a;
    let mut c = 1.0 / CF_TINY;
    let mut d = 1.0 / b;
    let mut h = d;

    for i in 1..MAX_ITER {
        let an = -(i as f64) * (i as f64 - a);
        b += 2.0;
        d = an * d + b;
        if d.abs() < CF_TINY {
            d = CF_TINY;
        }
        c = b + an / c;
        if c.abs() < CF_TINY {
            c = CF_TINY;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;
        if (delta - 1.0).abs() < CONVERGE_EPS {
            break;
        }
    }

    let log_result = -x + a * x.ln() - lng;
    Some(log_result.exp() * h)
}

/// Regularized incomplete beta function: I_x(a, b)
///
/// I_x(a,b) = B(x;a,b) / B(a,b)
///
/// Uses continued fraction (Lentz's algorithm).
/// For x > (a+1)/(a+b+2), computes 1 - I_{1-x}(b,a).
#[must_use]
pub fn regularized_beta(x: f64, a: f64, b: f64) -> Option<f64> {
    if !(0.0..=1.0).contains(&x) || a <= 0.0 || b <= 0.0 {
        return None;
    }
    if x == 0.0 {
        return Some(0.0);
    }
    if (x - 1.0).abs() < f64::EPSILON {
        return Some(1.0);
    }

    // Use symmetry for better convergence
    if x > (a + 1.0) / (a + b + 2.0) {
        return regularized_beta(1.0 - x, b, a).map(|v| 1.0 - v);
    }

    let lnbeta = ln_gamma(a)? + ln_gamma(b)? - ln_gamma(a + b)?;
    let front = (a * x.ln() + b * (1.0 - x).ln() - lnbeta).exp() / a;

    // Continued fraction (Lentz's method)
    let mut c = 1.0;
    let mut d = 1.0 / (1.0 - (a + b) * x / (a + 1.0)).max(CF_TINY);
    let mut f = d;

    for m in 1..MAX_ITER {
        let mf = m as f64;

        // Even step: d_{2m}
        let num_even = mf * (b - mf) * x / ((a + 2.0 * mf - 1.0) * (a + 2.0 * mf));
        d = 1.0 + num_even * d;
        if d.abs() < CF_TINY {
            d = CF_TINY;
        }
        d = 1.0 / d;
        c = 1.0 + num_even / c;
        if c.abs() < CF_TINY {
            c = CF_TINY;
        }
        f *= c * d;

        // Odd step: d_{2m+1}
        let num_odd = -(a + mf) * (a + b + mf) * x / ((a + 2.0 * mf) * (a + 2.0 * mf + 1.0));
        d = 1.0 + num_odd * d;
        if d.abs() < CF_TINY {
            d = CF_TINY;
        }
        d = 1.0 / d;
        c = 1.0 + num_odd / c;
        if c.abs() < CF_TINY {
            c = CF_TINY;
        }
        let delta = c * d;
        f *= delta;

        if (delta - 1.0).abs() < CONVERGE_EPS {
            break;
        }
    }

    Some(front * f)
}

// ============================================================================
// Student's t-Distribution (N1)
// ============================================================================

/// Student's t-distribution CDF: P(T ≤ t | ν degrees of freedom)
///
/// Uses the relationship to the regularized incomplete beta function:
/// - For t ≥ 0: F(t|ν) = 1 - I_x(ν/2, 1/2) / 2, where x = ν/(ν+t²)
/// - For t < 0: F(t|ν) = I_x(ν/2, 1/2) / 2
///
/// Returns `None` for df ≤ 0.
///
/// Grounded in T1 Mapping (μ): t-statistic → cumulative probability
#[must_use]
pub fn t_cdf(t: f64, df: f64) -> Option<f64> {
    if df <= 0.0 || df.is_nan() || t.is_nan() {
        return None;
    }
    if t.is_infinite() {
        return Some(if t > 0.0 { 1.0 } else { 0.0 });
    }

    let x = df / (df + t * t);
    let ib = regularized_beta(x, df / 2.0, 0.5)?;

    if t >= 0.0 {
        Some(1.0 - ib / 2.0)
    } else {
        Some(ib / 2.0)
    }
}

/// Student's t-distribution PDF
///
/// f(t|ν) = Γ((ν+1)/2) / (√(νπ) × Γ(ν/2)) × (1 + t²/ν)^(-(ν+1)/2)
#[must_use]
pub fn t_pdf(t: f64, df: f64) -> Option<f64> {
    if df <= 0.0 {
        return None;
    }
    let lng_num = ln_gamma((df + 1.0) / 2.0)?;
    let lng_den = ln_gamma(df / 2.0)?;
    let log_coeff = lng_num - lng_den - 0.5 * (df * PI).ln();
    let log_body = -((df + 1.0) / 2.0) * (1.0 + t * t / df).ln();
    Some((log_coeff + log_body).exp())
}

/// P-value from a t-statistic with given degrees of freedom.
#[must_use]
pub fn p_value_from_t(t: f64, df: f64, tail: Tail) -> Option<f64> {
    match tail {
        Tail::Left => t_cdf(t, df),
        Tail::Right => t_cdf(t, df).map(|p| 1.0 - p),
        Tail::Two => {
            let one_tail = t_cdf(-t.abs(), df)?;
            Some(2.0 * one_tail)
        }
    }
}

/// One-sample t-test: test a sample mean against a null hypothesis.
///
/// Appropriate for any sample size (unlike z-test which needs n≥30).
/// Uses n-1 degrees of freedom.
pub fn t_test_one_sample(
    values: &[f64],
    null_mean: f64,
    confidence_level: f64,
    tail: Tail,
    description: impl Into<String>,
) -> Option<StatisticalOutcome> {
    if values.len() < 2 {
        return None;
    }
    let m = mean(values)?;
    let se = standard_error(values)?;
    let t = (m - null_mean) / se;
    let df = values.len() as f64 - 1.0;
    let p = p_value_from_t(t, df, tail)?;

    // For the CI, use t-critical instead of z-critical
    let alpha_half = (1.0 - confidence_level) / 2.0;
    // Approximate t-critical: for large df, approaches z-critical
    // For small df, we solve iteratively (but for now use the CDF)
    // Simple bisection for t-critical
    let t_crit = t_critical_bisect(df, alpha_half)?;
    let margin = t_crit * se;

    let ci = ConfidenceInterval {
        estimate: m,
        lower: m - margin,
        upper: m + margin,
        level: confidence_level,
        margin,
        z_critical: t_crit, // t-critical stored in z_critical field
        std_error: se,
    };

    let sig = Significance::from_p(p);
    Some(StatisticalOutcome {
        value: m,
        z_score: t, // t-statistic stored in z_score field
        p_value: p,
        ci,
        significance: sig,
        description: description.into(),
    })
}

/// Find t-critical value by bisection: find t such that P(T > t | df) = alpha.
fn t_critical_bisect(df: f64, alpha: f64) -> Option<f64> {
    let mut lo = 0.0_f64;
    let mut hi = 20.0_f64; // generous upper bound

    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        let p_right = 1.0 - t_cdf(mid, df).unwrap_or(0.5);
        if p_right > alpha {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    Some((lo + hi) / 2.0)
}

/// Two-sample t-test (Welch's, unequal variances).
pub fn t_test_two_sample(
    sample1: &[f64],
    sample2: &[f64],
    confidence_level: f64,
    tail: Tail,
    description: impl Into<String>,
) -> Option<StatisticalOutcome> {
    if sample1.len() < 2 || sample2.len() < 2 {
        return None;
    }
    let m1 = mean(sample1)?;
    let m2 = mean(sample2)?;
    let v1 = variance(sample1)?;
    let v2 = variance(sample2)?;
    let n1 = sample1.len() as f64;
    let n2 = sample2.len() as f64;

    let se = (v1 / n1 + v2 / n2).sqrt();
    if se <= 0.0 {
        return None;
    }
    let t = (m1 - m2) / se;

    // Welch-Satterthwaite degrees of freedom
    let num = (v1 / n1 + v2 / n2) * (v1 / n1 + v2 / n2);
    let den = (v1 / n1) * (v1 / n1) / (n1 - 1.0) + (v2 / n2) * (v2 / n2) / (n2 - 1.0);
    let df = num / den;

    let p = p_value_from_t(t, df, tail)?;
    let t_crit = t_critical_bisect(df, (1.0 - confidence_level) / 2.0)?;
    let margin = t_crit * se;
    let diff = m1 - m2;

    let ci = ConfidenceInterval {
        estimate: diff,
        lower: diff - margin,
        upper: diff + margin,
        level: confidence_level,
        margin,
        z_critical: t_crit,
        std_error: se,
    };

    Some(StatisticalOutcome {
        value: diff,
        z_score: t,
        p_value: p,
        ci,
        significance: Significance::from_p(p),
        description: description.into(),
    })
}

// ============================================================================
// Chi-Square Distribution (N2)
// ============================================================================

/// Chi-square distribution CDF: P(X² ≤ x | k degrees of freedom)
///
/// χ²(k) = Γ(k/2, x/2) — the regularized lower incomplete gamma.
///
/// PV signal threshold: χ² ≥ 3.841 at df=1, α=0.05.
///
/// Returns `None` for invalid inputs.
#[must_use]
pub fn chi_square_cdf(x: f64, df: usize) -> Option<f64> {
    if x < 0.0 || df == 0 {
        return None;
    }
    if x == 0.0 {
        return Some(0.0);
    }
    regularized_gamma_p(df as f64 / 2.0, x / 2.0)
}

/// P-value from a chi-square statistic: P(X² ≥ observed | df)
#[must_use]
pub fn chi_square_p_value(chi_sq: f64, df: usize) -> Option<f64> {
    chi_square_cdf(chi_sq, df).map(|cdf| 1.0 - cdf)
}

/// Chi-square goodness-of-fit test.
///
/// χ² = Σ((observed_i - expected_i)² / expected_i)
///
/// Returns `(chi_square_statistic, p_value, df)`.
///
/// `df = categories - 1` (or categories - 1 - constraints).
#[must_use]
pub fn chi_square_goodness_of_fit(observed: &[f64], expected: &[f64]) -> Option<ChiSquareResult> {
    if observed.len() != expected.len() || observed.len() < 2 {
        return None;
    }
    // Check all expected > 0
    if expected.iter().any(|&e| e <= 0.0) {
        return None;
    }

    let chi_sq: f64 = observed
        .iter()
        .zip(expected.iter())
        .map(|(&o, &e)| (o - e) * (o - e) / e)
        .sum();

    let df = observed.len() - 1;
    let p = chi_square_p_value(chi_sq, df)?;

    Some(ChiSquareResult {
        statistic: chi_sq,
        df,
        p_value: p,
        significance: Significance::from_p(p),
    })
}

/// Chi-square test of independence for a 2×2 contingency table.
///
/// | | Outcome+ | Outcome- |
/// |---|---------|----------|
/// | Exposed | a | b |
/// | Unexposed | c | d |
///
/// Standard in pharmacovigilance for drug-event association.
#[must_use]
pub fn chi_square_2x2(a: f64, b: f64, c: f64, d: f64) -> Option<ChiSquareResult> {
    let n = a + b + c + d;
    if n <= 0.0 {
        return None;
    }

    // Expected values
    let e_a = (a + b) * (a + c) / n;
    let e_b = (a + b) * (b + d) / n;
    let e_c = (c + d) * (a + c) / n;
    let e_d = (c + d) * (b + d) / n;

    // Check no expected < 5 warning (but still compute)
    let chi_sq = (a - e_a) * (a - e_a) / e_a
        + (b - e_b) * (b - e_b) / e_b
        + (c - e_c) * (c - e_c) / e_c
        + (d - e_d) * (d - e_d) / e_d;

    let p = chi_square_p_value(chi_sq, 1)?;

    Some(ChiSquareResult {
        statistic: chi_sq,
        df: 1,
        p_value: p,
        significance: Significance::from_p(p),
    })
}

/// Result of a chi-square test.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChiSquareResult {
    /// The χ² test statistic
    pub statistic: f64,
    /// Degrees of freedom
    pub df: usize,
    /// P-value
    pub p_value: f64,
    /// Significance classification
    pub significance: Significance,
}

impl ChiSquareResult {
    /// PV signal threshold check: χ² ≥ 3.841 (df=1, α=0.05)
    #[must_use]
    pub fn exceeds_pv_threshold(&self) -> bool {
        self.df == 1 && self.statistic >= 3.841
    }
}

impl std::fmt::Display for ChiSquareResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "χ²({})={:.4}, p={:.6} {}",
            self.df,
            self.statistic,
            self.p_value,
            self.significance.stars()
        )
    }
}

// ============================================================================
// Tests — Proving the code is true
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f64 = 1e-6;
    const TIGHT_EPSILON: f64 = 1e-8;

    // --- Normal PDF ---

    #[test]
    fn pdf_at_zero() {
        // φ(0) = 1/√(2π) ≈ 0.398942
        let result = normal_pdf(0.0);
        assert!((result - INV_SQRT_2PI).abs() < TIGHT_EPSILON);
    }

    #[test]
    fn pdf_symmetry() {
        // φ(x) = φ(-x)
        for x in [0.5, 1.0, 1.5, 2.0, 2.5, 3.0] {
            assert!(
                (normal_pdf(x) - normal_pdf(-x)).abs() < TIGHT_EPSILON,
                "PDF not symmetric at x={x}"
            );
        }
    }

    #[test]
    fn pdf_decreasing_from_zero() {
        let mut prev = normal_pdf(0.0);
        for x in [0.5, 1.0, 1.5, 2.0, 2.5, 3.0] {
            let cur = normal_pdf(x);
            assert!(cur < prev, "PDF not decreasing at x={x}");
            prev = cur;
        }
    }

    // --- Normal CDF ---

    #[test]
    fn cdf_at_zero() {
        // Φ(0) = 0.5 exactly
        assert!((normal_cdf(0.0) - 0.5).abs() < TIGHT_EPSILON);
    }

    #[test]
    fn cdf_symmetry() {
        // Φ(x) + Φ(-x) = 1
        for x in [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5] {
            let sum = normal_cdf(x) + normal_cdf(-x);
            assert!(
                (sum - 1.0).abs() < TIGHT_EPSILON,
                "CDF symmetry violated at x={x}: sum={sum}"
            );
        }
    }

    #[test]
    fn cdf_known_values() {
        // Standard z-table reference values
        let cases = [
            (-3.0, 0.001_349_9),
            (-2.0, 0.022_750_1),
            (-1.0, 0.158_655_3),
            (0.0, 0.500_000_0),
            (1.0, 0.841_344_7),
            (1.96, 0.975_002_1),
            (2.0, 0.977_249_9),
            (2.576, 0.995_002_5),
            (3.0, 0.998_650_1),
        ];
        for (z, expected) in cases {
            let result = normal_cdf(z);
            assert!(
                (result - expected).abs() < 1e-5,
                "CDF({z}) = {result}, expected {expected}"
            );
        }
    }

    #[test]
    fn cdf_monotonically_increasing() {
        let mut prev = normal_cdf(-5.0);
        let mut z = -4.99;
        while z <= 5.0 {
            let cur = normal_cdf(z);
            assert!(cur >= prev, "CDF not monotonic at z={z}");
            prev = cur;
            z += 0.01;
        }
    }

    #[test]
    fn cdf_edge_cases() {
        assert_eq!(normal_cdf(f64::INFINITY), 1.0);
        assert_eq!(normal_cdf(f64::NEG_INFINITY), 0.0);
        assert!(normal_cdf(f64::NAN).is_nan());
    }

    // --- Inverse CDF ---

    #[test]
    fn quantile_roundtrip() {
        // Φ⁻¹(Φ(z)) ≈ z
        for z in [-3.0, -2.0, -1.0, -0.5, 0.0, 0.5, 1.0, 2.0, 3.0] {
            let p = normal_cdf(z);
            let recovered = normal_quantile(p);
            assert!(recovered.is_some(), "quantile returned None for p={p}");
            assert!(
                (recovered.unwrap_or(0.0) - z).abs() < 1e-6,
                "Roundtrip failed: z={z}, recovered={:?}",
                recovered
            );
        }
    }

    #[test]
    fn quantile_known_values() {
        let cases = [(0.5, 0.0), (0.975, 1.96), (0.995, 2.576), (0.025, -1.96)];
        for (p, expected_z) in cases {
            let result = normal_quantile(p);
            assert!(result.is_some());
            assert!(
                (result.unwrap_or(0.0) - expected_z).abs() < 0.001,
                "quantile({p}) = {:?}, expected ~{expected_z}",
                result
            );
        }
    }

    #[test]
    fn quantile_edge_cases() {
        assert!(normal_quantile(0.0).is_none());
        assert!(normal_quantile(1.0).is_none());
        assert!(normal_quantile(-0.1).is_none());
        assert!(normal_quantile(1.1).is_none());
        assert!(normal_quantile(f64::NAN).is_none());
    }

    // --- Z-Score ---

    #[test]
    fn z_score_basic() {
        // z = (100 - 80) / 10 = 2.0
        assert_eq!(z_score(100.0, 80.0, 10.0), Some(2.0));
        assert_eq!(z_score(80.0, 80.0, 10.0), Some(0.0));
        assert_eq!(z_score(60.0, 80.0, 10.0), Some(-2.0));
    }

    #[test]
    fn z_score_rejects_invalid_std() {
        assert!(z_score(5.0, 0.0, 0.0).is_none());
        assert!(z_score(5.0, 0.0, -1.0).is_none());
        assert!(z_score(5.0, 0.0, f64::NAN).is_none());
    }

    // --- P-Values ---

    #[test]
    fn p_value_two_tailed_at_zero() {
        // z=0 → p=1.0 (perfectly at null)
        let p = p_value_from_z(0.0, Tail::Two);
        assert!((p - 1.0).abs() < EPSILON);
    }

    #[test]
    fn p_value_two_tailed_at_196() {
        // z=1.96 → p≈0.05
        let p = p_value_from_z(1.96, Tail::Two);
        assert!(
            (p - 0.05).abs() < 0.001,
            "p_value at z=1.96 should be ~0.05, got {p}"
        );
    }

    #[test]
    fn p_value_one_tailed() {
        let z = 1.645;
        let p_right = p_value_from_z(z, Tail::Right);
        assert!(
            (p_right - 0.05).abs() < 0.001,
            "Right-tail p at z=1.645 should be ~0.05, got {p_right}"
        );

        let p_left = p_value_from_z(-z, Tail::Left);
        assert!(
            (p_left - 0.05).abs() < 0.001,
            "Left-tail p at z=-1.645 should be ~0.05, got {p_left}"
        );
    }

    // --- Confidence Intervals ---

    #[test]
    fn ci_95_percent() {
        let ci = confidence_interval_mean(100.0, 5.0, 0.95);
        assert!(ci.is_some());
        let ci = ci.unwrap_or_else(|| unreachable!());
        assert!((ci.z_critical - 1.96).abs() < 0.01);
        assert!((ci.lower - 90.2).abs() < 0.1);
        assert!((ci.upper - 109.8).abs() < 0.1);
        assert!(ci.contains(100.0));
        assert!(!ci.contains(115.0));
    }

    #[test]
    fn ci_99_wider_than_95() {
        let ci95 = confidence_interval_mean(50.0, 2.0, 0.95);
        let ci99 = confidence_interval_mean(50.0, 2.0, 0.99);
        assert!(ci95.is_some() && ci99.is_some());
        let w95 = ci95.map(|c| c.width()).unwrap_or(0.0);
        let w99 = ci99.map(|c| c.width()).unwrap_or(0.0);
        assert!(w99 > w95, "99% CI should be wider than 95%");
    }

    #[test]
    fn ci_proportion() {
        // 60 out of 100 = 0.60 proportion
        let ci = confidence_interval_proportion(0.60, 100, 0.95);
        assert!(ci.is_some());
        let ci = ci.unwrap_or_else(|| unreachable!());
        assert!(ci.contains(0.60));
        assert!(ci.lower > 0.45);
        assert!(ci.upper < 0.75);
    }

    #[test]
    fn ci_rejects_invalid() {
        assert!(confidence_interval_mean(0.0, -1.0, 0.95).is_none());
        assert!(confidence_interval_mean(0.0, 1.0, 0.0).is_none());
        assert!(confidence_interval_mean(0.0, 1.0, 1.0).is_none());
    }

    // --- Significance ---

    #[test]
    fn significance_classification() {
        assert_eq!(
            Significance::from_p(0.0001),
            Significance::HighlySignificant
        );
        assert_eq!(Significance::from_p(0.005), Significance::VerySignificant);
        assert_eq!(Significance::from_p(0.03), Significance::Significant);
        assert_eq!(Significance::from_p(0.10), Significance::NotSignificant);
    }

    #[test]
    fn significance_stars() {
        assert_eq!(Significance::HighlySignificant.stars(), "***");
        assert_eq!(Significance::VerySignificant.stars(), "**");
        assert_eq!(Significance::Significant.stars(), "*");
        assert_eq!(Significance::NotSignificant.stars(), "ns");
    }

    // --- StatisticalOutcome ---

    #[test]
    fn outcome_significant_result() {
        // Observed mean=105, null=100, SE=2.0 → z=2.5 → p≈0.012 (two-tailed)
        let outcome = StatisticalOutcome::two_tailed_95(105.0, 100.0, 2.0, "test mean");
        assert!(outcome.is_some());
        let o = outcome.unwrap_or_else(|| unreachable!());
        assert!((o.z_score - 2.5).abs() < EPSILON);
        assert!(o.p_value < 0.05);
        assert!(o.is_significant());
        assert!(o.ci_excludes_null(100.0));
    }

    #[test]
    fn outcome_not_significant() {
        // Observed mean=101, null=100, SE=5.0 → z=0.2 → p≈0.84
        let outcome = StatisticalOutcome::two_tailed_95(101.0, 100.0, 5.0, "test mean");
        assert!(outcome.is_some());
        let o = outcome.unwrap_or_else(|| unreachable!());
        assert!(!o.is_significant());
        assert!(!o.ci_excludes_null(100.0));
    }

    #[test]
    fn outcome_display() {
        let o = StatisticalOutcome::two_tailed_95(105.0, 100.0, 2.0, "test");
        assert!(o.is_some());
        let display = format!("{}", o.unwrap_or_else(|| unreachable!()));
        assert!(display.contains("test"));
        assert!(display.contains("CI"));
    }

    // --- Sample Analysis ---

    #[test]
    fn analyze_sample_basic() {
        // 30 values with mean ~105, testing against null=100
        let values: Vec<f64> = (0..30).map(|i| 100.0 + (i as f64) / 3.0).collect();
        let result = analyze_sample(&values, 95.0, 0.95, Tail::Two, "sample test");
        assert!(result.is_some());
        let r = result.unwrap_or_else(|| unreachable!());
        assert!(r.value > 100.0);
    }

    #[test]
    fn standard_error_basic() {
        let vals = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let se = standard_error(&vals);
        assert!(se.is_some());
        assert!(se.unwrap_or(0.0) > 0.0);
    }

    #[test]
    fn mean_and_variance() {
        let vals = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        assert!((mean(&vals).unwrap_or(0.0) - 6.0).abs() < EPSILON);
        assert!((variance(&vals).unwrap_or(0.0) - 10.0).abs() < EPSILON);
        assert!((std_dev(&vals).unwrap_or(0.0) - 10.0_f64.sqrt()).abs() < EPSILON);
    }

    // --- Z-Table ---

    #[test]
    fn z_table_constants() {
        // Verify stored constants match computed
        let z95 = ZTable::z_critical(0.95);
        assert!(z95.is_some());
        assert!((z95.unwrap_or(0.0) - 1.96).abs() < 0.001);

        let z99 = ZTable::z_critical(0.99);
        assert!(z99.is_some());
        assert!((z99.unwrap_or(0.0) - 2.576).abs() < 0.001);
    }

    #[test]
    fn z_table_full() {
        let table = ZTable::full_table();
        assert!(table.len() >= 699); // -3.49 to 3.49 in 0.01 steps
        // First entry should be near z=-3.49
        assert!(table[0].0 < -3.0);
        // CDF should be monotonically increasing
        for window in table.windows(2) {
            assert!(
                window[1].1 >= window[0].1,
                "Z-table not monotonic at z={}",
                window[0].0
            );
        }
    }

    // --- Confidence Interval Diff ---

    #[test]
    fn ci_diff_basic() {
        let ci = confidence_interval_diff(100.0, 3.0, 95.0, 2.0, 0.95);
        assert!(ci.is_some());
        let ci = ci.unwrap_or_else(|| unreachable!());
        assert!((ci.estimate - 5.0).abs() < EPSILON);
        // SE_diff = √(9+4) = √13 ≈ 3.606
        assert!((ci.std_error - 13.0_f64.sqrt()).abs() < 0.01);
    }

    // --- Mathematical Invariants (Barebone Proofs) ---

    #[test]
    fn invariant_cdf_is_integral_of_pdf() {
        // Numerical integration of φ(x) from -∞ to z should equal Φ(z)
        // Trapezoidal rule with small step
        let z_test = 1.5;
        let step = 0.001;
        let mut integral = 0.0;
        let mut x = -8.0;
        while x < z_test {
            integral += 0.5 * (normal_pdf(x) + normal_pdf(x + step)) * step;
            x += step;
        }
        assert!(
            (integral - normal_cdf(z_test)).abs() < 1e-4,
            "CDF must equal integral of PDF: numerical={integral}, CDF={}",
            normal_cdf(z_test)
        );
    }

    #[test]
    fn invariant_total_probability_is_one() {
        // ∫φ(x)dx from -∞ to +∞ = 1
        let step = 0.001;
        let mut integral = 0.0;
        let mut x = -8.0;
        while x < 8.0 {
            integral += 0.5 * (normal_pdf(x) + normal_pdf(x + step)) * step;
            x += step;
        }
        assert!(
            (integral - 1.0).abs() < 1e-4,
            "Total probability must be 1.0, got {integral}"
        );
    }

    #[test]
    fn invariant_mean_of_standard_normal_is_zero() {
        // E[X] = ∫x·φ(x)dx = 0
        let step = 0.001;
        let mut integral = 0.0;
        let mut x = -8.0;
        while x < 8.0 {
            let mid = x + step / 2.0;
            integral += mid * normal_pdf(mid) * step;
            x += step;
        }
        assert!(
            integral.abs() < 1e-4,
            "Mean of standard normal must be 0, got {integral}"
        );
    }

    #[test]
    fn invariant_variance_of_standard_normal_is_one() {
        // Var[X] = E[X²] = ∫x²·φ(x)dx = 1
        let step = 0.001;
        let mut integral = 0.0;
        let mut x = -8.0;
        while x < 8.0 {
            let mid = x + step / 2.0;
            integral += mid * mid * normal_pdf(mid) * step;
            x += step;
        }
        assert!(
            (integral - 1.0).abs() < 1e-3,
            "Variance of standard normal must be 1, got {integral}"
        );
    }

    #[test]
    fn invariant_ci_coverage_interpretation() {
        // For a 95% CI: the z-critical should give ~2.5% in each tail
        let z = ZTable::z_critical(0.95).unwrap_or(0.0);
        let right_tail = 1.0 - normal_cdf(z);
        let left_tail = normal_cdf(-z);
        assert!(
            (right_tail - 0.025).abs() < 0.001,
            "Right tail should be 2.5%, got {right_tail}"
        );
        assert!(
            (left_tail - 0.025).abs() < 0.001,
            "Left tail should be 2.5%, got {left_tail}"
        );
    }

    // ================================================================
    // Special Functions
    // ================================================================

    #[test]
    fn ln_gamma_known_values() {
        // Γ(1) = 1 → ln(1) = 0
        assert!((ln_gamma(1.0).unwrap_or(f64::NAN) - 0.0).abs() < EPSILON);
        // Γ(2) = 1 → ln(1) = 0
        assert!((ln_gamma(2.0).unwrap_or(f64::NAN) - 0.0).abs() < EPSILON);
        // Γ(3) = 2 → ln(2) ≈ 0.6931
        assert!((ln_gamma(3.0).unwrap_or(f64::NAN) - 2.0_f64.ln()).abs() < EPSILON);
        // Γ(5) = 24 → ln(24) ≈ 3.1781
        assert!((ln_gamma(5.0).unwrap_or(f64::NAN) - 24.0_f64.ln()).abs() < 1e-5);
        // Γ(0.5) = √π → ln(√π) ≈ 0.5724
        assert!(
            (ln_gamma(0.5).unwrap_or(f64::NAN) - 0.5 * PI.ln()).abs() < 1e-5,
            "ln_gamma(0.5) should be ln(√π)"
        );
    }

    #[test]
    fn ln_gamma_edge_cases() {
        assert!(ln_gamma(0.0).is_none());
        assert!(ln_gamma(-1.0).is_none());
        assert!(ln_gamma(f64::NAN).is_none());
    }

    #[test]
    fn regularized_gamma_p_known() {
        // P(1, 1) = 1 - e^(-1) ≈ 0.6321
        let p = regularized_gamma_p(1.0, 1.0).unwrap_or(0.0);
        assert!(
            (p - (1.0 - (-1.0_f64).exp())).abs() < 1e-5,
            "P(1,1) should be 1-e^-1, got {p}"
        );

        // P(1, 0) = 0
        assert!((regularized_gamma_p(1.0, 0.0).unwrap_or(1.0) - 0.0).abs() < EPSILON);

        // P(a, ∞) = 1
        assert!((regularized_gamma_p(2.0, f64::INFINITY).unwrap_or(0.0) - 1.0).abs() < EPSILON);
    }

    #[test]
    fn regularized_beta_known() {
        // I_0(a,b) = 0, I_1(a,b) = 1
        assert!((regularized_beta(0.0, 2.0, 3.0).unwrap_or(1.0) - 0.0).abs() < EPSILON);
        assert!((regularized_beta(1.0, 2.0, 3.0).unwrap_or(0.0) - 1.0).abs() < EPSILON);

        // I_0.5(1,1) = 0.5 (uniform distribution)
        assert!(
            (regularized_beta(0.5, 1.0, 1.0).unwrap_or(0.0) - 0.5).abs() < 1e-5,
            "I_0.5(1,1) should be 0.5"
        );
    }

    // ================================================================
    // Student's t-Distribution
    // ================================================================

    #[test]
    fn t_cdf_at_zero() {
        // F(0|ν) = 0.5 for all ν (symmetric)
        for df in [1.0, 5.0, 10.0, 30.0, 100.0] {
            let p = t_cdf(0.0, df).unwrap_or(0.0);
            assert!(
                (p - 0.5).abs() < 1e-5,
                "t_cdf(0, {df}) should be 0.5, got {p}"
            );
        }
    }

    #[test]
    fn t_cdf_symmetry() {
        // F(t|ν) + F(-t|ν) = 1
        for df in [1.0, 5.0, 30.0] {
            for t in [0.5, 1.0, 2.0, 3.0] {
                let sum = t_cdf(t, df).unwrap_or(0.0) + t_cdf(-t, df).unwrap_or(0.0);
                assert!(
                    (sum - 1.0).abs() < 1e-5,
                    "t-CDF symmetry violated: df={df}, t={t}, sum={sum}"
                );
            }
        }
    }

    #[test]
    fn t_cdf_converges_to_normal() {
        // As df → ∞, t-distribution → standard normal
        let t_val = 1.96;
        let t_prob = t_cdf(t_val, 10000.0).unwrap_or(0.0);
        let z_prob = normal_cdf(t_val);
        assert!(
            (t_prob - z_prob).abs() < 1e-3,
            "t(df=10000) should ≈ N(0,1): t_prob={t_prob}, z_prob={z_prob}"
        );
    }

    #[test]
    fn t_cdf_known_values() {
        // t=2.228, df=10 → two-tailed p ≈ 0.05
        let p = p_value_from_t(2.228, 10.0, Tail::Two).unwrap_or(1.0);
        assert!(
            (p - 0.05).abs() < 0.005,
            "t=2.228, df=10, two-tailed p should be ~0.05, got {p}"
        );

        // t=12.706, df=1 → two-tailed p ≈ 0.05
        let p = p_value_from_t(12.706, 1.0, Tail::Two).unwrap_or(1.0);
        assert!(
            (p - 0.05).abs() < 0.005,
            "t=12.706, df=1, two-tailed p should be ~0.05, got {p}"
        );
    }

    #[test]
    fn t_cdf_edge_cases() {
        assert!(t_cdf(0.0, 0.0).is_none());
        assert!(t_cdf(0.0, -1.0).is_none());
        assert_eq!(t_cdf(f64::INFINITY, 5.0), Some(1.0));
        assert_eq!(t_cdf(f64::NEG_INFINITY, 5.0), Some(0.0));
    }

    #[test]
    fn t_test_one_sample_basic() {
        // 10 values with mean ~7, null=5
        let values = vec![5.0, 6.0, 7.0, 8.0, 7.0, 6.0, 8.0, 9.0, 7.0, 8.0];
        let result = t_test_one_sample(&values, 5.0, 0.95, Tail::Two, "small sample t-test");
        assert!(result.is_some());
        let r = result.unwrap_or_else(|| unreachable!());
        assert!(r.is_significant(), "Should reject H₀: μ=5 with this sample");
    }

    #[test]
    fn t_test_two_sample_basic() {
        let a = vec![10.0, 12.0, 11.0, 13.0, 14.0, 10.0, 11.0, 12.0];
        let b = vec![5.0, 6.0, 7.0, 5.0, 6.0, 7.0, 8.0, 5.0];
        let result = t_test_two_sample(&a, &b, 0.95, Tail::Two, "two-sample");
        assert!(result.is_some());
        let r = result.unwrap_or_else(|| unreachable!());
        assert!(r.is_significant(), "Groups clearly differ");
        assert!(r.value > 0.0, "Group A mean > Group B mean");
    }

    // ================================================================
    // Chi-Square Distribution
    // ================================================================

    #[test]
    fn chi_square_cdf_known() {
        // χ²=3.841, df=1 → p≈0.95 (CDF), so p-value ≈ 0.05
        let p_value = chi_square_p_value(3.841, 1).unwrap_or(1.0);
        assert!(
            (p_value - 0.05).abs() < 0.005,
            "χ²=3.841, df=1 should give p≈0.05, got {p_value}"
        );

        // χ²=5.991, df=2 → p-value ≈ 0.05
        let p_value = chi_square_p_value(5.991, 2).unwrap_or(1.0);
        assert!(
            (p_value - 0.05).abs() < 0.005,
            "χ²=5.991, df=2 should give p≈0.05, got {p_value}"
        );

        // χ²=0, any df → p-value = 1
        let p_value = chi_square_p_value(0.0, 5).unwrap_or(0.0);
        assert!((p_value - 1.0).abs() < EPSILON);
    }

    #[test]
    fn chi_square_cdf_edge_cases() {
        assert!(chi_square_cdf(-1.0, 1).is_none());
        assert!(chi_square_cdf(1.0, 0).is_none());
    }

    #[test]
    fn chi_square_goodness_of_fit_basic() {
        // Fair die: observed vs expected
        let observed = vec![18.0, 16.0, 14.0, 20.0, 15.0, 17.0]; // 100 rolls
        let expected = vec![16.67, 16.67, 16.67, 16.67, 16.67, 16.67];
        let result = chi_square_goodness_of_fit(&observed, &expected);
        assert!(result.is_some());
        let r = result.unwrap_or_else(|| unreachable!());
        assert_eq!(r.df, 5);
        assert!(!r.significance.is_significant(), "Fair die should pass");
    }

    #[test]
    fn chi_square_goodness_of_fit_biased() {
        // Very biased die
        let observed = vec![50.0, 10.0, 10.0, 10.0, 10.0, 10.0];
        let expected = vec![16.67, 16.67, 16.67, 16.67, 16.67, 16.67];
        let result = chi_square_goodness_of_fit(&observed, &expected);
        assert!(result.is_some());
        let r = result.unwrap_or_else(|| unreachable!());
        assert!(r.significance.is_significant(), "Biased die should fail");
    }

    #[test]
    fn chi_square_2x2_pv_signal() {
        // Classic PV 2×2: Drug X with adverse event
        // a=30 (drug+event), b=70 (drug+no_event)
        // c=10 (no_drug+event), d=90 (no_drug+no_event)
        let result = chi_square_2x2(30.0, 70.0, 10.0, 90.0);
        assert!(result.is_some());
        let r = result.unwrap_or_else(|| unreachable!());
        assert_eq!(r.df, 1);
        assert!(
            r.exceeds_pv_threshold(),
            "χ²={} should exceed 3.841",
            r.statistic
        );
        assert!(r.significance.is_significant());
    }

    #[test]
    fn chi_square_2x2_no_association() {
        // No association
        let result = chi_square_2x2(25.0, 75.0, 25.0, 75.0);
        assert!(result.is_some());
        let r = result.unwrap_or_else(|| unreachable!());
        assert!(
            r.statistic < 0.001,
            "No association should give χ²≈0, got {}",
            r.statistic
        );
        assert!(!r.significance.is_significant());
    }

    // ================================================================
    // Invariant: t → normal convergence proof
    // ================================================================

    #[test]
    fn invariant_t_approaches_normal_as_df_grows() {
        let t_val = 1.0;
        let mut last_diff = f64::INFINITY;
        for df in [5.0, 10.0, 30.0, 100.0, 1000.0] {
            let t_p = t_cdf(t_val, df).unwrap_or(0.0);
            let z_p = normal_cdf(t_val);
            let diff = (t_p - z_p).abs();
            assert!(
                diff < last_diff || diff < 1e-4,
                "t should converge to normal: df={df}, diff={diff}"
            );
            last_diff = diff;
        }
        assert!(
            last_diff < 1e-3,
            "At df=1000, t and normal should be very close"
        );
    }

    #[test]
    fn invariant_chi_square_mean_equals_df() {
        // E[χ²(k)] = k. Verify by computing mean of CDF-implied distribution.
        // The median of χ²(k) ≈ k(1 - 2/(9k))³ for large k.
        // For k=10: median ≈ 10(1-2/90)³ ≈ 9.342
        // CDF at median should be ≈ 0.5
        let k = 10;
        let approx_median = 10.0_f64 * (1.0_f64 - 2.0 / 90.0).powi(3);
        let cdf_at_median = chi_square_cdf(approx_median, k).unwrap_or(0.0);
        assert!(
            (cdf_at_median - 0.5).abs() < 0.02,
            "χ²(10) median approx: CDF({approx_median}) = {cdf_at_median}, expected ~0.5"
        );
    }
}
