//! # Signal Analytics — Finance-Transferred PV Variables
//!
//! 9 measurable variables transferred from stem-finance to pharmacovigilance,
//! each grounded to T1 Lex Primitiva through formal cross-domain transfer.
//!
//! ## Transfer Map
//!
//! | PV Variable | Finance Analog | STEM Trait | T1 Grounding | Confidence |
//! |-------------|----------------|------------|--------------|------------|
//! | SignalVolatility | Historical Volatility | Hedge | ∂ (Boundary) | 0.92 |
//! | SignalVaR | Value at Risk | Hedge+Appraise | ∂+N | 0.88 |
//! | CrossSignalCorrelation | Asset Correlation | Diversify | Σ (Sum) | 0.85 |
//! | ReportingMomentum | Price Momentum | Compound | ρ (Recursion) | 0.90 |
//! | DataLiquidity | Bid-Ask Spread | Arbitrage | κ (Comparison) | 0.78 |
//! | NumberNeededToHarm | P/E Ratio | Appraise | N (Quantity) | 0.95 |
//! | SignalDiversification | Portfolio Diversify | Diversify | Σ (Sum) | 0.82 |
//! | RegulatoryMaturity | Bond Maturity | Mature | ∝ (Irreversibility) | 0.93 |
//! | InvestigationLeverage | Financial Leverage | Leverage | × (Product) | 0.80 |
//!
//! ## Tier Classification
//!
//! - **T2-P**: NNH, RegulatoryMaturity, SignalVolatility, DataLiquidity (single T1 dominant)
//! - **T2-C**: SignalVaR, ReportingMomentum, CrossSignalCorrelation,
//!             SignalDiversification, InvestigationLeverage (multi-T1 composite)

use crate::lex_primitiva::{GroundsTo, LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::fmt;

// ============================================================================
// 1. SignalVolatility — Variance of PRR/ROR over reporting windows
//    Finance analog: Historical Volatility
//    STEM trait: Hedge (∂)
//    Priority: P1 (Signal Integrity)
// ============================================================================

/// Variance of a disproportionality metric (PRR, ROR, IC) across reporting
/// windows. High volatility indicates an unstable signal — either emerging
/// or confounded.
///
/// # Lex Primitiva
/// - **∂ (Boundary)** dominant: volatility defines the width of the signal's
///   behavioral envelope
/// - **ν (Frequency)**: computed over periodic windows
///
/// # Finance Transfer
/// - Source: Historical volatility (σ² of returns)
/// - Confidence: 0.92 (direct structural mapping)
///
/// # Interpretation
/// - Low (< 0.1): Stable signal — consistent across windows
/// - Medium (0.1–0.5): Emerging — may be strengthening or weakening
/// - High (> 0.5): Erratic — investigate confounders
///
/// Tier: T2-P (∂)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SignalVolatility {
    /// Variance of the metric across windows
    variance: f64,
    /// Standard deviation (sqrt of variance)
    std_dev: f64,
    /// Number of windows observed
    window_count: u32,
}

impl SignalVolatility {
    /// Compute volatility from a series of metric values (e.g., PRR per quarter).
    ///
    /// Returns `None` if fewer than 2 observations.
    #[must_use]
    pub fn from_observations(values: &[f64]) -> Option<Self> {
        if values.len() < 2 {
            return None;
        }

        let n = values.len() as f64;
        let mean = values.iter().sum::<f64>() / n;
        let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1.0);
        let variance = variance.max(0.0); // Guard against negative from float rounding
        let std_dev = variance.sqrt();

        Some(Self {
            variance,
            std_dev,
            window_count: values.len() as u32,
        })
    }

    /// Get the variance.
    #[inline]
    #[must_use]
    pub fn variance(self) -> f64 {
        self.variance
    }

    /// Get the standard deviation.
    #[inline]
    #[must_use]
    pub fn std_dev(self) -> f64 {
        self.std_dev
    }

    /// Number of observation windows.
    #[inline]
    #[must_use]
    pub fn window_count(self) -> u32 {
        self.window_count
    }

    /// Is the signal stable (low volatility)?
    #[inline]
    #[must_use]
    pub fn is_stable(self) -> bool {
        self.variance < 0.1
    }

    /// Is the signal erratic (high volatility)?
    #[inline]
    #[must_use]
    pub fn is_erratic(self) -> bool {
        self.variance > 0.5
    }

    /// Coefficient of variation: std_dev / mean.
    /// Returns `None` if mean is zero.
    #[must_use]
    pub fn coefficient_of_variation(self, mean: f64) -> Option<f64> {
        if mean.abs() < f64::EPSILON {
            None
        } else {
            Some(self.std_dev / mean.abs())
        }
    }
}

impl GroundsTo for SignalVolatility {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Frequency])
    }
}

impl fmt::Display for SignalVolatility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Volatility(σ²={:.4}, σ={:.4}, n={})",
            self.variance, self.std_dev, self.window_count
        )
    }
}

// ============================================================================
// 2. SignalVaR — Worst-case signal strength at confidence level
//    Finance analog: Value at Risk
//    STEM traits: Hedge (∂) + Appraise (N)
//    Priority: P1 (Signal Integrity)
// ============================================================================

/// Worst-case disproportionality metric at a given confidence level.
/// Answers: "How bad could this signal get?"
///
/// # Lex Primitiva
/// - **∂ (Boundary)** dominant: VaR is an extreme boundary
/// - **N (Quantity)**: numeric metric value
///
/// # Finance Transfer
/// - Source: Value at Risk (VaR)
/// - Confidence: 0.88 (direct structural mapping)
///
/// # Interpretation
/// At 95% confidence, VaR = 5.2 means: "In the worst 5% of reporting
/// periods, we expect the PRR to exceed 5.2."
///
/// Tier: T2-C (∂ + N)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SignalVaR {
    /// The VaR value (worst-case metric level)
    value: f64,
    /// Confidence level (e.g., 0.95 for 95%)
    confidence: f64,
    /// Number of observations used
    observation_count: u32,
}

impl SignalVaR {
    /// Compute VaR from historical observations using the historical method.
    ///
    /// Sorts values and returns the (1-confidence) percentile.
    /// Returns `None` if insufficient data.
    #[must_use]
    pub fn from_historical(values: &[f64], confidence: f64) -> Option<Self> {
        if values.len() < 3 || !(0.0..1.0).contains(&confidence) {
            return None;
        }

        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let index = ((values.len() as f64) * confidence).floor() as usize;
        let index = index.min(sorted.len() - 1);

        Some(Self {
            value: sorted[index],
            confidence,
            observation_count: values.len() as u32,
        })
    }

    /// Parametric VaR using normal distribution approximation.
    ///
    /// VaR = mean + z * std_dev
    /// where z is the z-score for the confidence level.
    #[must_use]
    pub fn parametric(mean: f64, std_dev: f64, confidence: f64, count: u32) -> Option<Self> {
        if !(0.0..1.0).contains(&confidence) || count < 3 {
            return None;
        }

        // Common z-scores
        let z = match () {
            () if (confidence - 0.90).abs() < 0.001 => 1.282,
            () if (confidence - 0.95).abs() < 0.001 => 1.645,
            () if (confidence - 0.99).abs() < 0.001 => 2.326,
            _ => 1.645, // Default to 95%
        };

        Some(Self {
            value: mean + z * std_dev,
            confidence,
            observation_count: count,
        })
    }

    /// Get the VaR value.
    #[inline]
    #[must_use]
    pub fn value(self) -> f64 {
        self.value
    }

    /// Get the confidence level.
    #[inline]
    #[must_use]
    pub fn confidence(self) -> f64 {
        self.confidence
    }

    /// Observation count.
    #[inline]
    #[must_use]
    pub fn observation_count(self) -> u32 {
        self.observation_count
    }

    /// Does the VaR exceed a threshold?
    #[inline]
    #[must_use]
    pub fn exceeds_threshold(self, threshold: f64) -> bool {
        self.value >= threshold
    }
}

impl GroundsTo for SignalVaR {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Quantity])
    }
}

impl fmt::Display for SignalVaR {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "VaR({:.3} @ {:.0}% confidence, n={})",
            self.value,
            self.confidence * 100.0,
            self.observation_count
        )
    }
}

// ============================================================================
// 3. CrossSignalCorrelation — Pairwise correlation between drug-event pairs
//    Finance analog: Asset Correlation Matrix
//    STEM trait: Diversify (Σ)
//    Priority: P1 (Signal Integrity)
// ============================================================================

/// Pearson correlation coefficient between two signal time series.
/// Detects drug-class effects (correlated signals across related drugs).
///
/// # Lex Primitiva
/// - **Σ (Sum)** dominant: aggregation of co-movement across time
/// - **κ (Comparison)**: pairwise comparison of signal trajectories
///
/// # Finance Transfer
/// - Source: Asset correlation matrix (ρ_ij)
/// - Confidence: 0.85 (structural mapping)
///
/// # Interpretation
/// - ρ > 0.7: Strongly correlated — likely class effect
/// - 0.3 < ρ < 0.7: Moderate — shared mechanism possible
/// - ρ < 0.3: Independent signals
///
/// Tier: T2-C (Σ + κ)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CrossSignalCorrelation {
    /// Pearson correlation coefficient [-1.0, 1.0]
    coefficient: f64,
    /// First signal identifier
    signal_a: String,
    /// Second signal identifier
    signal_b: String,
    /// Number of overlapping time points
    overlap_count: u32,
}

impl CrossSignalCorrelation {
    /// Compute Pearson correlation from two aligned time series.
    ///
    /// Both series must have the same length and at least 3 points.
    #[must_use]
    pub fn compute(
        signal_a: impl Into<String>,
        signal_b: impl Into<String>,
        series_a: &[f64],
        series_b: &[f64],
    ) -> Option<Self> {
        if series_a.len() != series_b.len() || series_a.len() < 3 {
            return None;
        }

        let n = series_a.len() as f64;
        let mean_a = series_a.iter().sum::<f64>() / n;
        let mean_b = series_b.iter().sum::<f64>() / n;

        let mut cov = 0.0;
        let mut var_a = 0.0;
        let mut var_b = 0.0;

        for (a, b) in series_a.iter().zip(series_b.iter()) {
            let da = a - mean_a;
            let db = b - mean_b;
            cov += da * db;
            var_a += da * da;
            var_b += db * db;
        }

        let denom = (var_a * var_b).sqrt();
        if denom < f64::EPSILON {
            return None;
        }

        let coefficient = cov / denom;

        Some(Self {
            coefficient,
            signal_a: signal_a.into(),
            signal_b: signal_b.into(),
            overlap_count: series_a.len() as u32,
        })
    }

    /// Get the correlation coefficient.
    #[inline]
    #[must_use]
    pub fn coefficient(&self) -> f64 {
        self.coefficient
    }

    /// First signal identifier.
    #[must_use]
    pub fn signal_a(&self) -> &str {
        &self.signal_a
    }

    /// Second signal identifier.
    #[must_use]
    pub fn signal_b(&self) -> &str {
        &self.signal_b
    }

    /// Number of overlapping observations.
    #[inline]
    #[must_use]
    pub fn overlap_count(&self) -> u32 {
        self.overlap_count
    }

    /// Are the signals strongly correlated (class effect likely)?
    #[inline]
    #[must_use]
    pub fn is_class_effect(&self) -> bool {
        self.coefficient.abs() > 0.7
    }

    /// Are the signals independent?
    #[inline]
    #[must_use]
    pub fn is_independent(&self) -> bool {
        self.coefficient.abs() < 0.3
    }
}

impl GroundsTo for CrossSignalCorrelation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Comparison])
    }
}

impl fmt::Display for CrossSignalCorrelation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Correlation({} × {}: ρ={:.3}, n={})",
            self.signal_a, self.signal_b, self.coefficient, self.overlap_count
        )
    }
}

// ============================================================================
// 4. ReportingMomentum — Rate of change of reporting rate
//    Finance analog: Price Momentum
//    STEM trait: Compound (ρ)
//    Priority: P1 (Signal Integrity)
// ============================================================================

/// Rate of change (acceleration/deceleration) of case reporting over time.
/// A positive momentum indicates an emerging signal; negative indicates waning.
///
/// # Lex Primitiva
/// - **ρ (Recursion)** dominant: growth applied to growth rate
/// - **ν (Frequency)**: computed from reporting rates
/// - **σ (Sequence)**: requires temporal ordering
///
/// # Finance Transfer
/// - Source: Price momentum (rate of return of returns)
/// - Confidence: 0.90 (direct mapping)
///
/// # Formulas
/// - Absolute momentum: Δrate = rate_current - rate_previous
/// - Relative momentum: Δrate / rate_previous (% change of rate)
///
/// Tier: T2-C (ρ + ν + σ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReportingMomentum {
    /// Absolute change in reporting rate
    absolute: f64,
    /// Relative change (fraction, e.g., 0.15 = 15% increase)
    relative: f64,
    /// Current period rate
    current_rate: f64,
    /// Previous period rate
    previous_rate: f64,
}

impl ReportingMomentum {
    /// Compute momentum from two consecutive reporting rates.
    ///
    /// Returns `None` if previous rate is zero (relative momentum undefined).
    #[must_use]
    pub fn from_rates(current_rate: f64, previous_rate: f64) -> Option<Self> {
        if previous_rate.abs() < f64::EPSILON {
            return None;
        }

        let absolute = current_rate - previous_rate;
        let relative = absolute / previous_rate;

        Some(Self {
            absolute,
            relative,
            current_rate,
            previous_rate,
        })
    }

    /// Compute momentum from a time series of rates.
    /// Uses the last two values.
    #[must_use]
    pub fn from_series(rates: &[f64]) -> Option<Self> {
        if rates.len() < 2 {
            return None;
        }
        let current = rates[rates.len() - 1];
        let previous = rates[rates.len() - 2];
        Self::from_rates(current, previous)
    }

    /// Absolute change in rate.
    #[inline]
    #[must_use]
    pub fn absolute(self) -> f64 {
        self.absolute
    }

    /// Relative change (fraction).
    #[inline]
    #[must_use]
    pub fn relative(self) -> f64 {
        self.relative
    }

    /// Current reporting rate.
    #[inline]
    #[must_use]
    pub fn current_rate(self) -> f64 {
        self.current_rate
    }

    /// Previous reporting rate.
    #[inline]
    #[must_use]
    pub fn previous_rate(self) -> f64 {
        self.previous_rate
    }

    /// Is the signal accelerating (positive momentum)?
    #[inline]
    #[must_use]
    pub fn is_accelerating(self) -> bool {
        self.absolute > f64::EPSILON
    }

    /// Is the signal decelerating (negative momentum)?
    #[inline]
    #[must_use]
    pub fn is_decelerating(self) -> bool {
        self.absolute < -f64::EPSILON
    }

    /// Is momentum significant (> 20% change)?
    #[inline]
    #[must_use]
    pub fn is_significant(self) -> bool {
        self.relative.abs() > 0.20
    }
}

impl GroundsTo for ReportingMomentum {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion,
            LexPrimitiva::Frequency,
            LexPrimitiva::Sequence,
        ])
    }
}

impl fmt::Display for ReportingMomentum {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let direction = if self.is_accelerating() {
            "accelerating"
        } else if self.is_decelerating() {
            "decelerating"
        } else {
            "stable"
        };
        write!(
            f,
            "Momentum({}: Δ={:+.4}, {:.1}%)",
            direction,
            self.absolute,
            self.relative * 100.0
        )
    }
}

// ============================================================================
// 5. DataLiquidity — Sparseness/quality metric for observed counts
//    Finance analog: Bid-Ask Spread / Market Depth
//    STEM trait: Arbitrage (κ)
//    Priority: P3 (Data Quality)
// ============================================================================

/// Data sparseness quality metric. Low liquidity means wide uncertainty;
/// signals in low-liquidity data need wider thresholds.
///
/// # Lex Primitiva
/// - **κ (Comparison)** dominant: comparing observed vs minimum viable counts
///
/// # Finance Transfer
/// - Source: Bid-ask spread (market liquidity)
/// - Confidence: 0.78 (metaphorical)
///
/// # Formula
/// Liquidity = observed / (observed + missing)
/// where missing = expected_total - observed
///
/// # Interpretation
/// - Liquidity = 1.0: Complete data
/// - Liquidity > 0.8: Adequate for standard thresholds
/// - Liquidity < 0.5: Sparse — use sensitive thresholds
/// - Liquidity < 0.2: Critically sparse — flag for expert review
///
/// Tier: T2-P (κ)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DataLiquidity {
    /// Liquidity score [0.0, 1.0]
    score: f64,
    /// Observed count
    observed: u64,
    /// Expected total (background + observed)
    expected_total: u64,
}

impl DataLiquidity {
    /// Compute liquidity from observed and expected total.
    ///
    /// Returns `None` if expected_total is zero.
    #[must_use]
    pub fn new(observed: u64, expected_total: u64) -> Option<Self> {
        if expected_total == 0 {
            return None;
        }

        let score = (observed as f64 / expected_total as f64).min(1.0);

        Some(Self {
            score,
            observed,
            expected_total,
        })
    }

    /// Create from cell counts in a 2x2 table.
    /// Uses total N as the expected total and cell a as observed.
    #[must_use]
    pub fn from_contingency(cell_a: u64, total_n: u64) -> Option<Self> {
        Self::new(cell_a, total_n)
    }

    /// Get the liquidity score.
    #[inline]
    #[must_use]
    pub fn score(self) -> f64 {
        self.score
    }

    /// Observed count.
    #[inline]
    #[must_use]
    pub fn observed(self) -> u64 {
        self.observed
    }

    /// Expected total.
    #[inline]
    #[must_use]
    pub fn expected_total(self) -> u64 {
        self.expected_total
    }

    /// Is data adequate for standard thresholds?
    #[inline]
    #[must_use]
    pub fn is_adequate(self) -> bool {
        self.score >= 0.8
    }

    /// Is data critically sparse?
    #[inline]
    #[must_use]
    pub fn is_sparse(self) -> bool {
        self.score < 0.5
    }

    /// Is data critically deficient (needs expert review)?
    #[inline]
    #[must_use]
    pub fn is_critical(self) -> bool {
        self.score < 0.2
    }

    /// Recommended threshold profile based on liquidity.
    #[must_use]
    pub fn recommended_threshold_profile(&self) -> &'static str {
        if self.score >= 0.8 {
            "standard"
        } else if self.score >= 0.5 {
            "sensitive"
        } else {
            "expert_review"
        }
    }
}

impl GroundsTo for DataLiquidity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
    }
}

impl fmt::Display for DataLiquidity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Liquidity({:.2}, {}/{}, {})",
            self.score,
            self.observed,
            self.expected_total,
            self.recommended_threshold_profile()
        )
    }
}

// ============================================================================
// 6. NumberNeededToHarm (NNH) — Clinical interpretability of signal magnitude
//    Finance analog: Price-to-Earnings Ratio
//    STEM trait: Appraise (N)
//    Priority: P0 (Patient Safety)
// ============================================================================

/// Number Needed to Harm: how many patients must be exposed before one
/// additional adverse event occurs. The primary clinical interpretability
/// metric for disproportionality signals.
///
/// # Lex Primitiva
/// - **N (Quantity)** dominant: a single interpretable number
///
/// # Finance Transfer
/// - Source: Price-to-Earnings ratio (single summary metric)
/// - Confidence: 0.95 (direct structural mapping)
///
/// # Formula
/// NNH = 1 / ARI
/// where ARI = Absolute Risk Increase = risk_exposed - risk_unexposed
///
/// # Interpretation
/// - NNH = 10: 1 in 10 exposed patients harmed
/// - NNH = 100: 1 in 100 exposed patients harmed
/// - NNH = 1000: 1 in 1000 exposed patients harmed
/// - Lower NNH = more dangerous signal
///
/// Tier: T2-P (N)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NumberNeededToHarm {
    /// The NNH value (positive = harmful, negative = protective)
    value: f64,
    /// Absolute risk increase used in calculation
    absolute_risk_increase: f64,
    /// Risk in exposed group
    risk_exposed: f64,
    /// Risk in unexposed group
    risk_unexposed: f64,
}

impl NumberNeededToHarm {
    /// Compute NNH from exposed and unexposed risks.
    ///
    /// Returns `None` if absolute risk increase is zero.
    #[must_use]
    pub fn from_risks(risk_exposed: f64, risk_unexposed: f64) -> Option<Self> {
        let ari = risk_exposed - risk_unexposed;
        if ari.abs() < f64::EPSILON {
            return None;
        }

        Some(Self {
            value: 1.0 / ari,
            absolute_risk_increase: ari,
            risk_exposed,
            risk_unexposed,
        })
    }

    /// Compute NNH from a 2x2 contingency table.
    ///
    /// risk_exposed = a / (a+b), risk_unexposed = c / (c+d)
    #[must_use]
    pub fn from_contingency(a: u64, b: u64, c: u64, d: u64) -> Option<Self> {
        let exposed_total = a + b;
        let unexposed_total = c + d;

        if exposed_total == 0 || unexposed_total == 0 {
            return None;
        }

        let risk_exposed = a as f64 / exposed_total as f64;
        let risk_unexposed = c as f64 / unexposed_total as f64;

        Self::from_risks(risk_exposed, risk_unexposed)
    }

    /// Get the NNH value.
    #[inline]
    #[must_use]
    pub fn value(self) -> f64 {
        self.value
    }

    /// Get the absolute risk increase.
    #[inline]
    #[must_use]
    pub fn absolute_risk_increase(self) -> f64 {
        self.absolute_risk_increase
    }

    /// Risk in exposed group.
    #[inline]
    #[must_use]
    pub fn risk_exposed(self) -> f64 {
        self.risk_exposed
    }

    /// Risk in unexposed group.
    #[inline]
    #[must_use]
    pub fn risk_unexposed(self) -> f64 {
        self.risk_unexposed
    }

    /// Is this a harmful signal (positive ARI)?
    #[inline]
    #[must_use]
    pub fn is_harmful(self) -> bool {
        self.absolute_risk_increase > 0.0
    }

    /// Is this a protective signal (negative ARI — NNT)?
    #[inline]
    #[must_use]
    pub fn is_protective(self) -> bool {
        self.absolute_risk_increase < 0.0
    }

    /// Clinical severity: lower NNH = more urgent.
    #[must_use]
    pub fn severity_category(self) -> &'static str {
        let abs_nnh = self.value.abs();
        if abs_nnh < 10.0 {
            "critical"
        } else if abs_nnh < 100.0 {
            "high"
        } else if abs_nnh < 1000.0 {
            "moderate"
        } else {
            "low"
        }
    }
}

impl GroundsTo for NumberNeededToHarm {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity])
    }
}

impl fmt::Display for NumberNeededToHarm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = if self.is_harmful() { "NNH" } else { "NNT" };
        write!(
            f,
            "{}={:.1} (ARI={:.4}, severity={})",
            label,
            self.value.abs(),
            self.absolute_risk_increase,
            self.severity_category()
        )
    }
}

// ============================================================================
// 7. SignalDiversification — Multi-source confirmation benefit
//    Finance analog: Portfolio Diversification
//    STEM trait: Diversify (Σ)
//    Priority: P3 (Data Quality)
// ============================================================================

/// Quantifies the information gain from receiving the same signal across
/// multiple independent databases (FAERS, EudraVigilance, VigiBase).
///
/// # Lex Primitiva
/// - **Σ (Sum)** dominant: aggregation of independent evidence
///
/// # Finance Transfer
/// - Source: Portfolio diversification benefit
/// - Confidence: 0.82 (structural mapping)
///
/// # Formula
/// Diversification benefit = 1.0 - (combined_variance / sum_individual_variances)
/// where combined_variance uses correlation-weighted aggregation.
///
/// # Interpretation
/// - Benefit > 0.5: Strong multi-source confirmation
/// - Benefit 0.2–0.5: Moderate confirmation
/// - Benefit < 0.2: Sources are highly correlated (little new info)
///
/// Tier: T2-C (Σ + κ)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalDiversification {
    /// Diversification benefit [0.0, 1.0]
    benefit: f64,
    /// Number of independent sources confirming the signal
    source_count: u32,
    /// Source names
    sources: Vec<String>,
}

impl SignalDiversification {
    /// Compute diversification from individual signal variances and a common
    /// average pairwise correlation.
    ///
    /// Uses the simplified portfolio variance formula:
    /// σ²_portfolio = σ²_avg * (1/n + (1-1/n) * ρ_avg)
    /// benefit = 1 - σ²_portfolio / σ²_avg
    #[must_use]
    pub fn from_sources(
        sources: Vec<String>,
        variances: &[f64],
        avg_correlation: f64,
    ) -> Option<Self> {
        if sources.len() < 2 || variances.len() != sources.len() {
            return None;
        }

        let n = sources.len() as f64;
        let avg_variance = variances.iter().sum::<f64>() / n;

        if avg_variance < f64::EPSILON {
            return None;
        }

        // Portfolio variance under equal weighting
        let portfolio_var = avg_variance * (1.0 / n + (1.0 - 1.0 / n) * avg_correlation);
        let benefit = (1.0 - portfolio_var / avg_variance).max(0.0).min(1.0);

        Some(Self {
            benefit,
            source_count: sources.len() as u32,
            sources,
        })
    }

    /// Quick computation from source count and average correlation only.
    /// Assumes equal variances across sources.
    #[must_use]
    pub fn quick(source_count: u32, avg_correlation: f64) -> Option<Self> {
        if source_count < 2 {
            return None;
        }

        let n = f64::from(source_count);
        let benefit = ((1.0 - 1.0 / n) * (1.0 - avg_correlation))
            .max(0.0)
            .min(1.0);

        Some(Self {
            benefit,
            source_count,
            sources: Vec::new(),
        })
    }

    /// Get the diversification benefit.
    #[inline]
    #[must_use]
    pub fn benefit(&self) -> f64 {
        self.benefit
    }

    /// Number of confirming sources.
    #[inline]
    #[must_use]
    pub fn source_count(&self) -> u32 {
        self.source_count
    }

    /// Source names.
    #[must_use]
    pub fn sources(&self) -> &[String] {
        &self.sources
    }

    /// Is the multi-source confirmation strong?
    #[inline]
    #[must_use]
    pub fn is_strong_confirmation(&self) -> bool {
        self.benefit > 0.5
    }

    /// Is the multi-source confirmation weak (sources are correlated)?
    #[inline]
    #[must_use]
    pub fn is_weak_confirmation(&self) -> bool {
        self.benefit < 0.2
    }
}

impl GroundsTo for SignalDiversification {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Comparison])
    }
}

impl fmt::Display for SignalDiversification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Diversification(benefit={:.3}, sources={})",
            self.benefit, self.source_count
        )
    }
}

// ============================================================================
// 8. RegulatoryMaturity — ICH E2A deadline as typed temporal boundary
//    Finance analog: Bond Maturity
//    STEM trait: Mature (∝)
//    Priority: P2 (Regulatory Compliance)
// ============================================================================

/// ICH E2A reporting deadline as a first-class typed value with countdown.
/// Once expired, the event is irreversibly non-compliant.
///
/// # Lex Primitiva
/// - **∝ (Irreversibility)** dominant: expiration is permanent
/// - **σ (Sequence)**: time-ordered deadline
///
/// # Finance Transfer
/// - Source: Bond maturity date
/// - Confidence: 0.93 (direct mapping)
///
/// # ICH E2A Deadlines
/// - Fatal: 0h (immediate)
/// - Life-threatening: 4h
/// - Disability/Hospitalization: 24h-72h
/// - Non-serious: 30d (720h)
///
/// Tier: T2-P (∝)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegulatoryMaturity {
    /// Deadline in hours from event receipt
    deadline_hours: u64,
    /// Elapsed hours since receipt
    elapsed_hours: u64,
    /// Seriousness category
    seriousness: SeriousnessCategory,
}

/// ICH E2A seriousness categories with regulatory deadlines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SeriousnessCategory {
    /// Fatal — immediate reporting (0h)
    Fatal,
    /// Life-threatening — 4h
    LifeThreatening,
    /// Disability — 24h
    Disability,
    /// Hospitalization — 72h
    Hospitalization,
    /// Non-serious — 30 days (720h)
    NonSerious,
}

impl SeriousnessCategory {
    /// Get the regulatory deadline in hours.
    #[must_use]
    pub const fn deadline_hours(self) -> u64 {
        match self {
            Self::Fatal => 0,
            Self::LifeThreatening => 4,
            Self::Disability => 24,
            Self::Hospitalization => 72,
            Self::NonSerious => 720,
        }
    }

    /// All categories ordered by urgency.
    pub const ALL: &'static [SeriousnessCategory] = &[
        Self::Fatal,
        Self::LifeThreatening,
        Self::Disability,
        Self::Hospitalization,
        Self::NonSerious,
    ];
}

impl fmt::Display for SeriousnessCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fatal => write!(f, "Fatal"),
            Self::LifeThreatening => write!(f, "Life-threatening"),
            Self::Disability => write!(f, "Disability"),
            Self::Hospitalization => write!(f, "Hospitalization"),
            Self::NonSerious => write!(f, "Non-serious"),
        }
    }
}

impl RegulatoryMaturity {
    /// Create a new regulatory maturity tracker.
    #[must_use]
    pub fn new(seriousness: SeriousnessCategory, elapsed_hours: u64) -> Self {
        Self {
            deadline_hours: seriousness.deadline_hours(),
            elapsed_hours,
            seriousness,
        }
    }

    /// Create for fatal event (immediate deadline).
    #[must_use]
    pub fn fatal(elapsed_hours: u64) -> Self {
        Self::new(SeriousnessCategory::Fatal, elapsed_hours)
    }

    /// Create for life-threatening event (4h deadline).
    #[must_use]
    pub fn life_threatening(elapsed_hours: u64) -> Self {
        Self::new(SeriousnessCategory::LifeThreatening, elapsed_hours)
    }

    /// Deadline in hours.
    #[inline]
    #[must_use]
    pub fn deadline_hours(self) -> u64 {
        self.deadline_hours
    }

    /// Elapsed hours since event receipt.
    #[inline]
    #[must_use]
    pub fn elapsed_hours(self) -> u64 {
        self.elapsed_hours
    }

    /// Seriousness category.
    #[inline]
    #[must_use]
    pub fn seriousness(self) -> SeriousnessCategory {
        self.seriousness
    }

    /// Remaining hours until deadline. Returns 0 if expired.
    #[inline]
    #[must_use]
    pub fn remaining_hours(self) -> u64 {
        self.deadline_hours.saturating_sub(self.elapsed_hours)
    }

    /// Has the deadline expired? (Irreversible — ∝)
    #[inline]
    #[must_use]
    pub fn is_expired(self) -> bool {
        self.elapsed_hours >= self.deadline_hours
    }

    /// Time-to-maturity as a fraction [0.0, 1.0] where 1.0 = expired.
    #[must_use]
    pub fn maturity_fraction(self) -> f64 {
        if self.deadline_hours == 0 {
            return 1.0; // Fatal is always at maturity
        }
        (self.elapsed_hours as f64 / self.deadline_hours as f64).min(1.0)
    }

    /// Urgency score (0.0 = no urgency, 1.0 = critical).
    /// Non-linear: urgency accelerates as deadline approaches.
    #[must_use]
    pub fn urgency(self) -> f64 {
        let fraction = self.maturity_fraction();
        // Quadratic acceleration: urgency rises sharply near deadline
        fraction * fraction
    }

    /// Advance elapsed time. Returns new state.
    #[must_use]
    pub fn advance(self, additional_hours: u64) -> Self {
        Self {
            elapsed_hours: self.elapsed_hours.saturating_add(additional_hours),
            ..self
        }
    }
}

impl GroundsTo for RegulatoryMaturity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Irreversibility, LexPrimitiva::Sequence])
    }
}

impl fmt::Display for RegulatoryMaturity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_expired() {
            write!(
                f,
                "EXPIRED({}, {}h past deadline)",
                self.seriousness,
                self.elapsed_hours - self.deadline_hours
            )
        } else {
            write!(
                f,
                "Maturity({}, {}h remaining, urgency={:.2})",
                self.seriousness,
                self.remaining_hours(),
                self.urgency()
            )
        }
    }
}

// ============================================================================
// 9. InvestigationLeverage — ROI of one investigation preventing N harms
//    Finance analog: Financial Leverage
//    STEM trait: Leverage (×)
//    Priority: P4 (Operational Efficiency)
// ============================================================================

/// Multiplier effect of a single investigation: how many future harm events
/// does one investigation potentially prevent?
///
/// # Lex Primitiva
/// - **× (Product)** dominant: multiplication of prevention effect
/// - **N (Quantity)**: numeric count of prevented harms
///
/// # Finance Transfer
/// - Source: Financial leverage (equity × multiplier → exposure)
/// - Confidence: 0.80 (metaphorical mapping)
///
/// # Formula
/// Leverage = estimated_harms_prevented / investigation_cost_units
///
/// # Interpretation
/// - Leverage > 10: High-value investigation
/// - Leverage 1–10: Standard value
/// - Leverage < 1: Low-value (cost exceeds expected prevention)
///
/// Tier: T2-C (× + N)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct InvestigationLeverage {
    /// Leverage ratio (prevented / cost)
    ratio: f64,
    /// Estimated harm events prevented
    estimated_prevented: f64,
    /// Investigation cost in work-units (person-hours)
    cost_units: f64,
}

impl InvestigationLeverage {
    /// Compute leverage from estimated prevention and cost.
    ///
    /// Returns `None` if cost is zero.
    #[must_use]
    pub fn new(estimated_prevented: f64, cost_units: f64) -> Option<Self> {
        if cost_units.abs() < f64::EPSILON {
            return None;
        }

        Some(Self {
            ratio: estimated_prevented / cost_units,
            estimated_prevented,
            cost_units,
        })
    }

    /// Quick estimation from signal case count and standard investigation cost.
    ///
    /// Uses heuristic: prevented ≈ case_count × 0.3 (30% prevention rate)
    /// Standard investigation cost: 40 person-hours.
    #[must_use]
    pub fn estimate(case_count: u64, prevention_rate: f64) -> Option<Self> {
        let estimated = case_count as f64 * prevention_rate;
        Self::new(estimated, 40.0) // Standard 40 person-hour investigation
    }

    /// Get the leverage ratio.
    #[inline]
    #[must_use]
    pub fn ratio(self) -> f64 {
        self.ratio
    }

    /// Estimated harm events prevented.
    #[inline]
    #[must_use]
    pub fn estimated_prevented(self) -> f64 {
        self.estimated_prevented
    }

    /// Investigation cost in work-units.
    #[inline]
    #[must_use]
    pub fn cost_units(self) -> f64 {
        self.cost_units
    }

    /// Is this a high-value investigation?
    #[inline]
    #[must_use]
    pub fn is_high_value(self) -> bool {
        self.ratio > 10.0
    }

    /// Is the cost justified (leverage > 1)?
    #[inline]
    #[must_use]
    pub fn is_justified(self) -> bool {
        self.ratio > 1.0
    }
}

impl GroundsTo for InvestigationLeverage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Product, LexPrimitiva::Quantity])
    }
}

impl fmt::Display for InvestigationLeverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Leverage({:.1}x, prevented={:.1}, cost={:.1}h)",
            self.ratio, self.estimated_prevented, self.cost_units
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    mod volatility_tests {
        use super::*;

        #[test]
        fn stable_signal() {
            let values = [2.1, 2.0, 2.05, 2.02, 1.98];
            let vol = SignalVolatility::from_observations(&values);
            assert!(vol.is_some());
            let vol = vol.unwrap_or_else(|| unreachable!());
            assert!(vol.is_stable());
            assert!(!vol.is_erratic());
            assert_eq!(vol.window_count(), 5);
        }

        #[test]
        fn erratic_signal() {
            let values = [1.0, 5.0, 0.5, 8.0, 2.0];
            let vol = SignalVolatility::from_observations(&values);
            assert!(vol.is_some());
            let vol = vol.unwrap_or_else(|| unreachable!());
            assert!(vol.is_erratic());
        }

        #[test]
        fn insufficient_data() {
            assert!(SignalVolatility::from_observations(&[]).is_none());
            assert!(SignalVolatility::from_observations(&[1.0]).is_none());
        }

        #[test]
        fn coefficient_of_variation() {
            let values = [10.0, 12.0, 11.0, 9.0, 10.5];
            let vol = SignalVolatility::from_observations(&values);
            assert!(vol.is_some());
            let vol = vol.unwrap_or_else(|| unreachable!());
            let cv = vol.coefficient_of_variation(10.5);
            assert!(cv.is_some());
            assert!(cv.unwrap_or(0.0) > 0.0);
            assert!(vol.coefficient_of_variation(0.0).is_none());
        }
    }

    mod var_tests {
        use super::*;

        #[test]
        fn historical_var() {
            let values = [1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0];
            let var = SignalVaR::from_historical(&values, 0.95);
            assert!(var.is_some());
            let var = var.unwrap_or_else(|| unreachable!());
            assert!(var.value() >= 5.0);
            assert_eq!(var.observation_count(), 10);
        }

        #[test]
        fn parametric_var() {
            let var = SignalVaR::parametric(2.0, 0.5, 0.95, 100);
            assert!(var.is_some());
            let var = var.unwrap_or_else(|| unreachable!());
            // VaR = 2.0 + 1.645 * 0.5 = 2.8225
            assert!((var.value() - 2.8225).abs() < 0.001);
        }

        #[test]
        fn insufficient_data() {
            assert!(SignalVaR::from_historical(&[1.0, 2.0], 0.95).is_none());
            assert!(SignalVaR::parametric(2.0, 0.5, 0.95, 2).is_none());
        }
    }

    mod correlation_tests {
        use super::*;

        #[test]
        fn perfect_positive_correlation() {
            let a = [1.0, 2.0, 3.0, 4.0, 5.0];
            let b = [2.0, 4.0, 6.0, 8.0, 10.0];
            let corr = CrossSignalCorrelation::compute("drug_a", "drug_b", &a, &b);
            assert!(corr.is_some());
            let corr = corr.unwrap_or_else(|| unreachable!());
            assert!((corr.coefficient() - 1.0).abs() < 0.001);
            assert!(corr.is_class_effect());
        }

        #[test]
        fn negative_correlation() {
            let a = [1.0, 2.0, 3.0, 4.0, 5.0];
            let b = [5.0, 4.0, 3.0, 2.0, 1.0];
            let corr = CrossSignalCorrelation::compute("drug_a", "drug_b", &a, &b);
            assert!(corr.is_some());
            let corr = corr.unwrap_or_else(|| unreachable!());
            assert!((corr.coefficient() + 1.0).abs() < 0.001);
        }

        #[test]
        fn mismatched_lengths() {
            assert!(CrossSignalCorrelation::compute("a", "b", &[1.0, 2.0], &[1.0]).is_none());
        }
    }

    mod momentum_tests {
        use super::*;

        #[test]
        fn accelerating_signal() {
            let mom = ReportingMomentum::from_rates(0.15, 0.10);
            assert!(mom.is_some());
            let mom = mom.unwrap_or_else(|| unreachable!());
            assert!(mom.is_accelerating());
            assert!(!mom.is_decelerating());
            assert!((mom.relative() - 0.5).abs() < f64::EPSILON);
            assert!(mom.is_significant());
        }

        #[test]
        fn decelerating_signal() {
            let mom = ReportingMomentum::from_rates(0.05, 0.10);
            assert!(mom.is_some());
            let mom = mom.unwrap_or_else(|| unreachable!());
            assert!(mom.is_decelerating());
            assert!((mom.relative() + 0.5).abs() < f64::EPSILON);
        }

        #[test]
        fn from_series() {
            let rates = [0.08, 0.10, 0.15];
            let mom = ReportingMomentum::from_series(&rates);
            assert!(mom.is_some());
            let mom = mom.unwrap_or_else(|| unreachable!());
            assert!(mom.is_accelerating());
        }

        #[test]
        fn zero_previous_rate() {
            assert!(ReportingMomentum::from_rates(0.10, 0.0).is_none());
        }
    }

    mod liquidity_tests {
        use super::*;

        #[test]
        fn adequate_data() {
            let liq = DataLiquidity::new(80, 100);
            assert!(liq.is_some());
            let liq = liq.unwrap_or_else(|| unreachable!());
            assert!(liq.is_adequate());
            assert!(!liq.is_sparse());
            assert_eq!(liq.recommended_threshold_profile(), "standard");
        }

        #[test]
        fn sparse_data() {
            let liq = DataLiquidity::new(3, 100);
            assert!(liq.is_some());
            let liq = liq.unwrap_or_else(|| unreachable!());
            assert!(liq.is_sparse());
            assert!(liq.is_critical());
            assert_eq!(liq.recommended_threshold_profile(), "expert_review");
        }

        #[test]
        fn zero_expected() {
            assert!(DataLiquidity::new(5, 0).is_none());
        }
    }

    mod nnh_tests {
        use super::*;

        #[test]
        fn harmful_signal() {
            // 10% exposed vs 5% unexposed → ARI = 0.05 → NNH = 20
            let nnh = NumberNeededToHarm::from_risks(0.10, 0.05);
            assert!(nnh.is_some());
            let nnh = nnh.unwrap_or_else(|| unreachable!());
            assert!((nnh.value() - 20.0).abs() < 0.001);
            assert!(nnh.is_harmful());
            assert_eq!(nnh.severity_category(), "high");
        }

        #[test]
        fn critical_nnh() {
            // 50% vs 10% → ARI = 0.40 → NNH = 2.5
            let nnh = NumberNeededToHarm::from_risks(0.50, 0.10);
            assert!(nnh.is_some());
            let nnh = nnh.unwrap_or_else(|| unreachable!());
            assert!((nnh.value() - 2.5).abs() < 0.001);
            assert_eq!(nnh.severity_category(), "critical");
        }

        #[test]
        fn protective_signal() {
            // 5% exposed vs 10% unexposed → NNT
            let nnh = NumberNeededToHarm::from_risks(0.05, 0.10);
            assert!(nnh.is_some());
            let nnh = nnh.unwrap_or_else(|| unreachable!());
            assert!(nnh.is_protective());
        }

        #[test]
        fn from_contingency_table() {
            // a=15, b=85, c=5, d=895 → RE=0.15, RU=0.00556 → ARI≈0.1444 → NNH≈6.9
            let nnh = NumberNeededToHarm::from_contingency(15, 85, 5, 895);
            assert!(nnh.is_some());
            let nnh = nnh.unwrap_or_else(|| unreachable!());
            assert!(nnh.value() > 0.0);
            assert!(nnh.value() < 10.0);
            assert!(nnh.is_harmful());
        }

        #[test]
        fn equal_risks() {
            assert!(NumberNeededToHarm::from_risks(0.10, 0.10).is_none());
        }
    }

    mod diversification_tests {
        use super::*;

        #[test]
        fn independent_sources() {
            let sources = vec!["FAERS".into(), "EudraVigilance".into(), "VigiBase".into()];
            let variances = [0.5, 0.6, 0.4];
            let div = SignalDiversification::from_sources(sources, &variances, 0.1);
            assert!(div.is_some());
            let div = div.unwrap_or_else(|| unreachable!());
            assert!(div.is_strong_confirmation());
            assert_eq!(div.source_count(), 3);
        }

        #[test]
        fn correlated_sources() {
            let sources = vec!["Source1".into(), "Source2".into()];
            let variances = [0.5, 0.5];
            let div = SignalDiversification::from_sources(sources, &variances, 0.9);
            assert!(div.is_some());
            let div = div.unwrap_or_else(|| unreachable!());
            assert!(div.is_weak_confirmation());
        }

        #[test]
        fn quick_computation() {
            let div = SignalDiversification::quick(3, 0.2);
            assert!(div.is_some());
            let div = div.unwrap_or_else(|| unreachable!());
            assert!(div.benefit() > 0.0);
        }

        #[test]
        fn single_source() {
            assert!(SignalDiversification::quick(1, 0.0).is_none());
        }
    }

    mod maturity_tests {
        use super::*;

        #[test]
        fn fatal_immediate() {
            let mat = RegulatoryMaturity::fatal(0);
            assert!(mat.is_expired()); // Fatal = 0h deadline = always at maturity
            assert_eq!(mat.remaining_hours(), 0);
            assert!((mat.maturity_fraction() - 1.0).abs() < f64::EPSILON);
        }

        #[test]
        fn life_threatening_within_deadline() {
            let mat = RegulatoryMaturity::life_threatening(2);
            assert!(!mat.is_expired());
            assert_eq!(mat.remaining_hours(), 2);
            assert!((mat.maturity_fraction() - 0.5).abs() < f64::EPSILON);
        }

        #[test]
        fn expired_deadline() {
            let mat = RegulatoryMaturity::new(SeriousnessCategory::Hospitalization, 100);
            assert!(mat.is_expired()); // 72h deadline, 100h elapsed
            assert_eq!(mat.remaining_hours(), 0);
        }

        #[test]
        fn urgency_increases_quadratically() {
            let early = RegulatoryMaturity::new(SeriousnessCategory::NonSerious, 100);
            let late = RegulatoryMaturity::new(SeriousnessCategory::NonSerious, 600);
            assert!(late.urgency() > early.urgency());
        }

        #[test]
        fn advance_time() {
            let mat = RegulatoryMaturity::life_threatening(1);
            assert!(!mat.is_expired());
            let mat = mat.advance(5);
            assert!(mat.is_expired());
            assert_eq!(mat.elapsed_hours(), 6);
        }

        #[test]
        fn all_seriousness_categories() {
            assert_eq!(SeriousnessCategory::Fatal.deadline_hours(), 0);
            assert_eq!(SeriousnessCategory::LifeThreatening.deadline_hours(), 4);
            assert_eq!(SeriousnessCategory::Disability.deadline_hours(), 24);
            assert_eq!(SeriousnessCategory::Hospitalization.deadline_hours(), 72);
            assert_eq!(SeriousnessCategory::NonSerious.deadline_hours(), 720);
            assert_eq!(SeriousnessCategory::ALL.len(), 5);
        }
    }

    mod leverage_tests {
        use super::*;

        #[test]
        fn high_value_investigation() {
            let lev = InvestigationLeverage::new(500.0, 40.0);
            assert!(lev.is_some());
            let lev = lev.unwrap_or_else(|| unreachable!());
            assert!((lev.ratio() - 12.5).abs() < f64::EPSILON);
            assert!(lev.is_high_value());
            assert!(lev.is_justified());
        }

        #[test]
        fn low_value_investigation() {
            let lev = InvestigationLeverage::new(10.0, 40.0);
            assert!(lev.is_some());
            let lev = lev.unwrap_or_else(|| unreachable!());
            assert!(!lev.is_high_value());
        }

        #[test]
        fn estimate_from_case_count() {
            let lev = InvestigationLeverage::estimate(200, 0.3);
            assert!(lev.is_some());
            let lev = lev.unwrap_or_else(|| unreachable!());
            // 200 * 0.3 = 60 prevented, / 40 cost = 1.5x leverage
            assert!((lev.ratio() - 1.5).abs() < f64::EPSILON);
        }

        #[test]
        fn zero_cost() {
            assert!(InvestigationLeverage::new(100.0, 0.0).is_none());
        }
    }

    mod grounding_tests {
        use super::*;

        #[test]
        fn all_types_grounded() {
            // Verify every type has a GroundsTo impl
            let _vol = SignalVolatility::primitive_composition();
            let _var = SignalVaR::primitive_composition();
            let _corr = CrossSignalCorrelation::primitive_composition();
            let _mom = ReportingMomentum::primitive_composition();
            let _liq = DataLiquidity::primitive_composition();
            let _nnh = NumberNeededToHarm::primitive_composition();
            let _div = SignalDiversification::primitive_composition();
            let _mat = RegulatoryMaturity::primitive_composition();
            let _lev = InvestigationLeverage::primitive_composition();
        }

        #[test]
        fn correct_dominant_primitives() {
            let vol = SignalVolatility::primitive_composition();
            assert!(vol.primitives.contains(&LexPrimitiva::Boundary));

            let nnh = NumberNeededToHarm::primitive_composition();
            assert!(nnh.primitives.contains(&LexPrimitiva::Quantity));

            let mat = RegulatoryMaturity::primitive_composition();
            assert!(mat.primitives.contains(&LexPrimitiva::Irreversibility));

            let lev = InvestigationLeverage::primitive_composition();
            assert!(lev.primitives.contains(&LexPrimitiva::Product));
        }

        #[test]
        fn t1_coverage() {
            // Collectively, these 9 types cover 9 distinct T1 primitives:
            // ∂, N, Σ, ρ, κ, ∝, ×, ν, σ
            let all_primitives: Vec<Vec<LexPrimitiva>> = vec![
                SignalVolatility::primitive_composition()
                    .primitives
                    .to_vec(),
                SignalVaR::primitive_composition().primitives.to_vec(),
                CrossSignalCorrelation::primitive_composition()
                    .primitives
                    .to_vec(),
                ReportingMomentum::primitive_composition()
                    .primitives
                    .to_vec(),
                DataLiquidity::primitive_composition().primitives.to_vec(),
                NumberNeededToHarm::primitive_composition()
                    .primitives
                    .to_vec(),
                SignalDiversification::primitive_composition()
                    .primitives
                    .to_vec(),
                RegulatoryMaturity::primitive_composition()
                    .primitives
                    .to_vec(),
                InvestigationLeverage::primitive_composition()
                    .primitives
                    .to_vec(),
            ];

            let mut unique: std::collections::HashSet<LexPrimitiva> =
                std::collections::HashSet::new();
            for prims in &all_primitives {
                for p in prims {
                    unique.insert(*p);
                }
            }

            // Should cover at least 9 distinct T1 primitives
            assert!(
                unique.len() >= 9,
                "Expected >= 9 unique primitives, got {}",
                unique.len()
            );
        }
    }

    mod display_tests {
        use super::*;

        #[test]
        fn all_display_impls() {
            let vol = SignalVolatility::from_observations(&[1.0, 2.0, 3.0]);
            assert!(vol.is_some());
            let s = format!("{}", vol.unwrap_or_else(|| unreachable!()));
            assert!(s.contains("Volatility"));

            let var = SignalVaR::parametric(2.0, 0.5, 0.95, 10);
            assert!(var.is_some());
            let s = format!("{}", var.unwrap_or_else(|| unreachable!()));
            assert!(s.contains("VaR"));

            let mom = ReportingMomentum::from_rates(0.15, 0.10);
            assert!(mom.is_some());
            let s = format!("{}", mom.unwrap_or_else(|| unreachable!()));
            assert!(s.contains("Momentum"));

            let nnh = NumberNeededToHarm::from_risks(0.10, 0.05);
            assert!(nnh.is_some());
            let s = format!("{}", nnh.unwrap_or_else(|| unreachable!()));
            assert!(s.contains("NNH"));

            let mat = RegulatoryMaturity::life_threatening(2);
            let s = format!("{mat}");
            assert!(s.contains("Maturity"));

            let lev = InvestigationLeverage::new(100.0, 40.0);
            assert!(lev.is_some());
            let s = format!("{}", lev.unwrap_or_else(|| unreachable!()));
            assert!(s.contains("Leverage"));
        }
    }
}
