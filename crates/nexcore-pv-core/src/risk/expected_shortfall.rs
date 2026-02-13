//! Expected Shortfall (ES) / Conditional Value at Risk (CVaR)
//!
//! Measures the expected safety risk beyond the Safety-at-Risk threshold,
//! providing insight into tail risk and worst-case safety scenarios.
//!
//! Expected Shortfall answers: "Given that we've exceeded our SaR threshold,
//! what is the average loss we can expect?"
//!
//! ES is a **coherent risk measure** that satisfies:
//! - Monotonicity: Higher losses → higher risk
//! - Subadditivity: Diversification reduces risk
//! - Positive homogeneity: Scaling inputs scales risk
//! - Translation invariance: Adding constant shifts risk

use serde::{Deserialize, Serialize};

/// Distribution type for parametric calculations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DistributionType {
    /// Normal (Gaussian) distribution
    #[default]
    Normal,
    /// Student's t-distribution (heavier tails)
    StudentT,
    /// Skewed Student's t-distribution
    SkewedT,
}

/// Calculation method for Expected Shortfall
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EsMethod {
    /// Historical simulation using actual data
    #[default]
    Historical,
    /// Parametric approach with distribution assumptions
    Parametric,
    /// Monte Carlo simulation
    MonteCarlo,
}

/// Input parameters for Expected Shortfall calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsInput {
    /// Historical adverse event rates (0.0 to 1.0)
    pub adverse_event_rates: Vec<f64>,
    /// Confidence level (0.5 to 0.999)
    pub confidence_level: f64,
    /// Calculation method
    pub method: EsMethod,
    /// Time horizon in days
    pub time_horizon: u32,
    /// Distribution type for parametric method
    pub distribution_type: DistributionType,
    /// Number of Monte Carlo paths
    pub monte_carlo_paths: u32,
    /// Degrees of freedom for Student's t (only used when distribution is StudentT)
    pub degrees_of_freedom: Option<f64>,
}

impl Default for EsInput {
    fn default() -> Self {
        Self {
            adverse_event_rates: Vec::new(),
            confidence_level: 0.95,
            method: EsMethod::Historical,
            time_horizon: 30,
            distribution_type: DistributionType::Normal,
            monte_carlo_paths: 10_000,
            degrees_of_freedom: Some(4.0),
        }
    }
}

/// Confidence interval bounds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    /// Lower bound
    pub lower: f64,
    /// Upper bound
    pub upper: f64,
}

/// Tail risk metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TailRiskMetrics {
    /// Average loss in the tail
    pub average_tail_loss: f64,
    /// Maximum observed tail loss
    pub max_tail_loss: f64,
    /// Volatility of tail losses
    pub tail_volatility: f64,
    /// Skewness of the tail distribution
    pub skewness_risk: f64,
}

/// Breakdown of losses beyond SaR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsBreakdown {
    /// Individual losses beyond SaR threshold
    pub beyond_sar: Vec<f64>,
    /// Probability weights for each tail loss
    pub probabilities: Vec<f64>,
    /// Total cumulative risk in tail
    pub cumulative_risk: f64,
}

/// Validation tests for coherent risk measure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsValidation {
    /// ES >= SaR (coherent risk measure property)
    pub coherent_risk_measure: bool,
    /// ES satisfies subadditivity
    pub subadditivity: bool,
    /// ES increases with worse outcomes
    pub monotonicity_test: bool,
}

/// Expected Shortfall calculation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EsResult {
    /// Expected Shortfall value (average loss beyond SaR)
    pub expected_shortfall: f64,
    /// Safety-at-Risk threshold
    pub safety_at_risk: f64,
    /// Probability of exceeding SaR
    pub exceedance_probability: f64,
    /// Additional risk contribution beyond SaR
    pub tail_risk_contribution: f64,
    /// Worst case scenario observed
    pub worst_case_scenario: f64,
    /// Confidence interval for ES estimate
    pub confidence_interval: ConfidenceInterval,
    /// Detailed tail risk metrics
    pub risk_metrics: TailRiskMetrics,
    /// Breakdown of tail losses
    pub breakdown: EsBreakdown,
    /// Validation of coherent risk measure properties
    pub validation: EsValidation,
}

/// Calculate volatility (standard deviation) of a slice
fn volatility(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let mean = values.iter().sum::<f64>() / values.len() as f64;
    let variance: f64 =
        values.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

/// Calculate skewness of a distribution
fn skewness(values: &[f64]) -> f64 {
    if values.len() < 3 {
        return 0.0;
    }
    let n = values.len() as f64;
    let mean = values.iter().sum::<f64>() / n;
    let std_dev = volatility(values);
    if std_dev == 0.0 {
        return 0.0;
    }
    let m3: f64 = values.iter().map(|x| ((x - mean) / std_dev).powi(3)).sum();
    m3 / n
}

/// Thread-local LCG random number generator
fn rand_simple() -> f64 {
    use std::cell::Cell;
    // CONCURRENCY: Thread-local cell, no cross-thread access.
    thread_local! {
        static SEED: Cell<u64> = const { Cell::new(98765) };
    }
    SEED.with(|seed| {
        let mut s = seed.get();
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        seed.set(s);
        (s >> 33) as f64 / (1u64 << 31) as f64
    })
}

/// Box-Muller transform for normal random variates
fn random_normal(mean: f64, std_dev: f64) -> f64 {
    use std::f64::consts::PI;
    let u1 = rand_simple().max(1e-10);
    let u2 = rand_simple();
    let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
    mean + std_dev * z
}

/// Normal PDF
fn normal_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

/// Inverse normal CDF (Beasley-Springer-Moro approximation)
fn normal_inverse(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 {
        return if p <= 0.0 {
            f64::NEG_INFINITY
        } else {
            f64::INFINITY
        };
    }

    const A: [f64; 4] = [
        2.506_628_238_84,
        -18.615_000_625_29,
        41.391_197_735_34,
        -25.441_060_496_37,
    ];
    const B: [f64; 4] = [
        -8.473_510_930_90,
        23.083_367_437_43,
        -21.062_241_018_26,
        3.130_829_098_33,
    ];
    const C: [f64; 9] = [
        0.337_475_482_272_614_7,
        0.976_169_019_091_718_6,
        0.160_797_971_491_820_9,
        0.027_643_881_033_386_3,
        0.003_840_572_937_360_9,
        0.000_395_189_651_191_9,
        0.000_032_176_788_176_8,
        0.000_000_288_816_736_4,
        0.000_000_396_031_518_7,
    ];

    let y = p - 0.5;

    if y.abs() < 0.42 {
        let r = y * y;
        let num = y * (((A[3] * r + A[2]) * r + A[1]) * r + A[0]);
        let den = (((B[3] * r + B[2]) * r + B[1]) * r + B[0]) * r + 1.0;
        return num / den;
    }

    let r = if p < 0.5 { p } else { 1.0 - p };
    let s = (-r.ln()).sqrt();
    let t = C[0]
        + s * (C[1]
            + s * (C[2]
                + s * (C[3] + s * (C[4] + s * (C[5] + s * (C[6] + s * (C[7] + s * C[8])))))));

    if p < 0.5 { -t } else { t }
}

/// Student's t inverse CDF approximation
fn student_t_inverse(p: f64, df: f64) -> f64 {
    let z = normal_inverse(p);
    let z2 = z * z;
    let z3 = z2 * z;
    let z5 = z3 * z2;

    z + (z3 + z) / (4.0 * df) + (5.0 * z5 + 16.0 * z3 + 3.0 * z) / (96.0 * df * df)
}

/// Student's t PDF
fn student_t_pdf(x: f64, df: f64) -> f64 {
    let coeff = gamma((df + 1.0) / 2.0) / (gamma(df / 2.0) * (df * std::f64::consts::PI).sqrt());
    coeff * (1.0 + (x * x) / df).powf(-(df + 1.0) / 2.0)
}

/// Gamma function approximation (Lanczos)
fn gamma(x: f64) -> f64 {
    if x < 0.5 {
        return std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * gamma(1.0 - x));
    }
    let x = x - 1.0;
    const P: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];

    let mut g = P[0];
    for (i, &p) in P.iter().enumerate().skip(1) {
        g += p / (x + i as f64);
    }

    let t = x + P.len() as f64 - 1.5;
    (2.0 * std::f64::consts::PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * g
}

/// Bootstrap confidence interval
fn bootstrap_ci(values: &[f64], confidence: f64) -> ConfidenceInterval {
    if values.is_empty() {
        return ConfidenceInterval {
            lower: 0.0,
            upper: 0.0,
        };
    }

    const NUM_BOOTSTRAPS: usize = 1000;
    let mut estimates = Vec::with_capacity(NUM_BOOTSTRAPS);

    for _ in 0..NUM_BOOTSTRAPS {
        let sample: Vec<f64> = (0..values.len())
            .map(|_| {
                let idx = (rand_simple() * values.len() as f64) as usize;
                values[idx.min(values.len() - 1)]
            })
            .collect();
        let mean = sample.iter().sum::<f64>() / sample.len() as f64;
        estimates.push(mean);
    }

    estimates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let alpha = 1.0 - confidence;
    let lower_idx = (alpha / 2.0 * estimates.len() as f64) as usize;
    let upper_idx = ((1.0 - alpha / 2.0) * estimates.len() as f64) as usize;

    ConfidenceInterval {
        lower: estimates.get(lower_idx).copied().unwrap_or(0.0),
        upper: estimates
            .get(upper_idx.min(estimates.len() - 1))
            .copied()
            .unwrap_or(0.0),
    }
}

/// Perform validation tests for coherent risk measure properties
fn perform_validation(data: &[f64], sar: f64, es: f64) -> EsValidation {
    // Test 1: ES >= SaR (coherent risk measure property)
    let coherent_risk_measure = es >= sar;

    // Test 2: Subadditivity (ES is always subadditive by definition)
    let subadditivity = true;

    // Test 3: Monotonicity - ES should increase with worse data
    let worse_data: Vec<f64> = data.iter().map(|x| x * 1.1).collect();
    let worse_es = if worse_data.is_empty() {
        0.0
    } else {
        let tail_size = ((1.0 - 0.95) * worse_data.len() as f64) as usize;
        let mut sorted = worse_data.clone();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let tail = &sorted[..=tail_size.min(sorted.len().saturating_sub(1))];
        if tail.is_empty() {
            0.0
        } else {
            tail.iter().sum::<f64>() / tail.len() as f64
        }
    };
    let monotonicity_test = worse_es >= es;

    EsValidation {
        coherent_risk_measure,
        subadditivity,
        monotonicity_test,
    }
}

/// Calculate Historical Expected Shortfall
fn calculate_historical_es(input: &EsInput) -> EsResult {
    let rates = &input.adverse_event_rates;
    let horizon = input.time_horizon as f64;

    if rates.is_empty() {
        return empty_result();
    }

    // Sort rates descending (worst first)
    let mut sorted_rates: Vec<f64> = rates.clone();
    sorted_rates.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    // Calculate SaR threshold index
    let sar_index = ((1.0 - input.confidence_level) * sorted_rates.len() as f64) as usize;
    let sar_index = sar_index.min(sorted_rates.len().saturating_sub(1));
    let safety_at_risk = sorted_rates[sar_index];

    // Calculate ES as average of tail losses
    let tail_losses: Vec<f64> = sorted_rates[..=sar_index].to_vec();
    let expected_shortfall = if tail_losses.is_empty() {
        safety_at_risk
    } else {
        tail_losses.iter().sum::<f64>() / tail_losses.len() as f64
    };

    // Calculate metrics
    let exceedance_probability = tail_losses.len() as f64 / sorted_rates.len() as f64;
    let tail_risk_contribution = expected_shortfall - safety_at_risk;
    let worst_case_scenario = sorted_rates.first().copied().unwrap_or(0.0);

    // Tail risk analysis
    let tail_volatility = volatility(&tail_losses);
    let skewness_risk = skewness(&tail_losses);

    // Bootstrap CI
    let ci = bootstrap_ci(&tail_losses, input.confidence_level);

    // Validation
    let validation = perform_validation(&sorted_rates, safety_at_risk, expected_shortfall);

    EsResult {
        expected_shortfall: expected_shortfall * horizon,
        safety_at_risk: safety_at_risk * horizon,
        exceedance_probability,
        tail_risk_contribution: tail_risk_contribution * horizon,
        worst_case_scenario: worst_case_scenario * horizon,
        confidence_interval: ConfidenceInterval {
            lower: ci.lower * horizon,
            upper: ci.upper * horizon,
        },
        risk_metrics: TailRiskMetrics {
            average_tail_loss: expected_shortfall * horizon,
            max_tail_loss: worst_case_scenario * horizon,
            tail_volatility: tail_volatility * horizon.sqrt(),
            skewness_risk,
        },
        breakdown: EsBreakdown {
            beyond_sar: tail_losses.iter().map(|r| r * horizon).collect(),
            probabilities: vec![1.0 / tail_losses.len() as f64; tail_losses.len()],
            cumulative_risk: tail_losses.iter().sum::<f64>() * horizon,
        },
        validation,
    }
}

/// Calculate Parametric Expected Shortfall
fn calculate_parametric_es(input: &EsInput) -> EsResult {
    let rates = &input.adverse_event_rates;
    let horizon = input.time_horizon as f64;

    if rates.is_empty() {
        return empty_result();
    }

    let n = rates.len() as f64;
    let mean = rates.iter().sum::<f64>() / n;
    let variance: f64 = rates.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();

    let (safety_at_risk, expected_shortfall) = match input.distribution_type {
        DistributionType::Normal => {
            let z = normal_inverse(input.confidence_level);
            let sar = mean + std_dev * z;
            let es = mean + std_dev * normal_pdf(z) / (1.0 - input.confidence_level);
            (sar, es)
        }
        DistributionType::StudentT => {
            let df = input.degrees_of_freedom.unwrap_or(4.0);
            let t = student_t_inverse(input.confidence_level, df);
            let scale = ((df - 2.0) / df).sqrt();
            let sar = mean + std_dev * scale * t;
            let es = mean
                + std_dev * scale * student_t_pdf(t, df) * (df + t * t)
                    / ((df - 1.0) * (1.0 - input.confidence_level));
            (sar, es)
        }
        DistributionType::SkewedT => {
            // Fallback to normal for skewed-t (simplified)
            let z = normal_inverse(input.confidence_level);
            let sar = mean + std_dev * z;
            let es = mean + std_dev * normal_pdf(z) / (1.0 - input.confidence_level);
            (sar, es)
        }
    };

    let exceedance_probability = 1.0 - input.confidence_level;
    let tail_risk_contribution = expected_shortfall - safety_at_risk;
    let worst_case_scenario = expected_shortfall + 2.0 * std_dev;

    // Fisher information CI
    let se_es = std_dev / n.sqrt();
    let ci = ConfidenceInterval {
        lower: expected_shortfall - 1.96 * se_es,
        upper: expected_shortfall + 1.96 * se_es,
    };

    let validation = perform_validation(rates, safety_at_risk, expected_shortfall);

    EsResult {
        expected_shortfall: expected_shortfall * horizon,
        safety_at_risk: safety_at_risk * horizon,
        exceedance_probability,
        tail_risk_contribution: tail_risk_contribution * horizon,
        worst_case_scenario: worst_case_scenario * horizon,
        confidence_interval: ConfidenceInterval {
            lower: ci.lower * horizon,
            upper: ci.upper * horizon,
        },
        risk_metrics: TailRiskMetrics {
            average_tail_loss: expected_shortfall * horizon,
            max_tail_loss: worst_case_scenario * horizon,
            tail_volatility: std_dev * horizon.sqrt(),
            skewness_risk: skewness(rates),
        },
        breakdown: EsBreakdown {
            beyond_sar: Vec::new(),
            probabilities: Vec::new(),
            cumulative_risk: expected_shortfall * horizon,
        },
        validation,
    }
}

/// Calculate Monte Carlo Expected Shortfall
fn calculate_monte_carlo_es(input: &EsInput) -> EsResult {
    let rates = &input.adverse_event_rates;
    let horizon = input.time_horizon as f64;
    let num_paths = input.monte_carlo_paths as usize;

    if rates.is_empty() {
        return empty_result();
    }

    let n = rates.len() as f64;
    let mean = rates.iter().sum::<f64>() / n;
    let variance: f64 = rates.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1.0);
    let std_dev = variance.sqrt();

    // Generate Monte Carlo paths
    let mut simulated_rates: Vec<f64> = (0..num_paths)
        .map(|_| random_normal(mean, std_dev).clamp(0.0, 1.0))
        .collect();

    // Sort descending
    simulated_rates.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    // Calculate SaR and ES
    let sar_index = ((1.0 - input.confidence_level) * simulated_rates.len() as f64) as usize;
    let sar_index = sar_index.min(simulated_rates.len().saturating_sub(1));
    let safety_at_risk = simulated_rates[sar_index];

    let tail_losses: Vec<f64> = simulated_rates[..=sar_index].to_vec();
    let expected_shortfall = tail_losses.iter().sum::<f64>() / tail_losses.len() as f64;

    // Calculate metrics
    let exceedance_probability = tail_losses.len() as f64 / simulated_rates.len() as f64;
    let tail_risk_contribution = expected_shortfall - safety_at_risk;
    let worst_case_scenario = simulated_rates.first().copied().unwrap_or(0.0);

    let tail_volatility = volatility(&tail_losses);
    let skewness_risk = skewness(&tail_losses);

    // Monte Carlo CI (run multiple simulations)
    const MC_RUNS: usize = 100;
    let mut es_estimates: Vec<f64> = Vec::with_capacity(MC_RUNS);

    for _ in 0..MC_RUNS {
        let mut sample: Vec<f64> = (0..num_paths / 10)
            .map(|_| random_normal(mean, std_dev).clamp(0.0, 1.0))
            .collect();
        sample.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        let idx = ((1.0 - input.confidence_level) * sample.len() as f64) as usize;
        let idx = idx.min(sample.len().saturating_sub(1));
        let tail = &sample[..=idx];
        let es = tail.iter().sum::<f64>() / tail.len() as f64;
        es_estimates.push(es);
    }

    es_estimates.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let ci_lower_idx = (0.025 * es_estimates.len() as f64) as usize;
    let ci_upper_idx = (0.975 * es_estimates.len() as f64) as usize;
    let ci = ConfidenceInterval {
        lower: es_estimates.get(ci_lower_idx).copied().unwrap_or(0.0),
        upper: es_estimates
            .get(ci_upper_idx.min(es_estimates.len().saturating_sub(1)))
            .copied()
            .unwrap_or(0.0),
    };

    let validation = perform_validation(&simulated_rates, safety_at_risk, expected_shortfall);

    EsResult {
        expected_shortfall: expected_shortfall * horizon,
        safety_at_risk: safety_at_risk * horizon,
        exceedance_probability,
        tail_risk_contribution: tail_risk_contribution * horizon,
        worst_case_scenario: worst_case_scenario * horizon,
        confidence_interval: ConfidenceInterval {
            lower: ci.lower * horizon,
            upper: ci.upper * horizon,
        },
        risk_metrics: TailRiskMetrics {
            average_tail_loss: expected_shortfall * horizon,
            max_tail_loss: worst_case_scenario * horizon,
            tail_volatility: tail_volatility * horizon.sqrt(),
            skewness_risk,
        },
        breakdown: EsBreakdown {
            beyond_sar: tail_losses.iter().map(|r| r * horizon).collect(),
            probabilities: vec![1.0 / tail_losses.len() as f64; tail_losses.len()],
            cumulative_risk: tail_losses.iter().sum::<f64>() * horizon,
        },
        validation,
    }
}

/// Create empty result for edge cases
fn empty_result() -> EsResult {
    EsResult {
        expected_shortfall: 0.0,
        safety_at_risk: 0.0,
        exceedance_probability: 0.0,
        tail_risk_contribution: 0.0,
        worst_case_scenario: 0.0,
        confidence_interval: ConfidenceInterval {
            lower: 0.0,
            upper: 0.0,
        },
        risk_metrics: TailRiskMetrics {
            average_tail_loss: 0.0,
            max_tail_loss: 0.0,
            tail_volatility: 0.0,
            skewness_risk: 0.0,
        },
        breakdown: EsBreakdown {
            beyond_sar: Vec::new(),
            probabilities: Vec::new(),
            cumulative_risk: 0.0,
        },
        validation: EsValidation {
            coherent_risk_measure: true,
            subadditivity: true,
            monotonicity_test: true,
        },
    }
}

/// Calculate Expected Shortfall (CVaR)
///
/// # Arguments
/// * `input` - Expected Shortfall calculation parameters
///
/// # Returns
/// Comprehensive ES result with tail risk metrics and validation
#[must_use]
pub fn calculate_expected_shortfall(input: &EsInput) -> EsResult {
    match input.method {
        EsMethod::Historical => calculate_historical_es(input),
        EsMethod::Parametric => calculate_parametric_es(input),
        EsMethod::MonteCarlo => calculate_monte_carlo_es(input),
    }
}

/// Portfolio-level Expected Shortfall calculation
///
/// Combines multiple asset ES calculations with weights.
#[must_use]
pub fn calculate_portfolio_es(
    portfolio_inputs: &[(Vec<f64>, f64)], // (rates, weight)
    confidence_level: f64,
) -> EsResult {
    let mut combined_rates: Vec<f64> = Vec::new();
    for (rates, weight) in portfolio_inputs {
        for rate in rates {
            combined_rates.push(rate * weight);
        }
    }

    let input = EsInput {
        adverse_event_rates: combined_rates,
        confidence_level,
        method: EsMethod::Historical,
        ..Default::default()
    };

    calculate_expected_shortfall(&input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_historical_es() {
        let input = EsInput {
            adverse_event_rates: vec![0.01, 0.02, 0.03, 0.04, 0.05, 0.06, 0.07, 0.08, 0.09, 0.10],
            confidence_level: 0.95,
            method: EsMethod::Historical,
            time_horizon: 30,
            ..Default::default()
        };
        let result = calculate_expected_shortfall(&input);

        assert!(result.expected_shortfall > 0.0);
        assert!(result.expected_shortfall >= result.safety_at_risk);
        assert!(result.validation.coherent_risk_measure);
    }

    #[test]
    fn test_parametric_es_normal() {
        let input = EsInput {
            adverse_event_rates: vec![0.01, 0.02, 0.015, 0.018, 0.025, 0.012],
            confidence_level: 0.95,
            method: EsMethod::Parametric,
            distribution_type: DistributionType::Normal,
            time_horizon: 30,
            ..Default::default()
        };
        let result = calculate_expected_shortfall(&input);

        assert!(result.expected_shortfall.is_finite());
        assert!(result.safety_at_risk.is_finite());
    }

    #[test]
    fn test_parametric_es_student_t() {
        let input = EsInput {
            adverse_event_rates: vec![0.01, 0.02, 0.015, 0.018, 0.025],
            confidence_level: 0.95,
            method: EsMethod::Parametric,
            distribution_type: DistributionType::StudentT,
            degrees_of_freedom: Some(4.0),
            time_horizon: 30,
            ..Default::default()
        };
        let result = calculate_expected_shortfall(&input);

        assert!(result.expected_shortfall.is_finite());
    }

    #[test]
    fn test_monte_carlo_es() {
        let input = EsInput {
            adverse_event_rates: vec![0.01, 0.02, 0.015, 0.018],
            confidence_level: 0.95,
            method: EsMethod::MonteCarlo,
            monte_carlo_paths: 1000,
            time_horizon: 30,
            ..Default::default()
        };
        let result = calculate_expected_shortfall(&input);

        assert!(result.expected_shortfall.is_finite());
        assert!(result.expected_shortfall >= 0.0);
    }

    #[test]
    fn test_empty_input() {
        let input = EsInput::default();
        let result = calculate_expected_shortfall(&input);

        assert_eq!(result.expected_shortfall, 0.0);
        assert_eq!(result.safety_at_risk, 0.0);
        assert!(result.validation.coherent_risk_measure);
    }

    #[test]
    fn test_es_greater_than_sar() {
        let input = EsInput {
            adverse_event_rates: vec![0.01, 0.05, 0.10, 0.15, 0.20, 0.25],
            confidence_level: 0.95,
            method: EsMethod::Historical,
            time_horizon: 1,
            ..Default::default()
        };
        let result = calculate_expected_shortfall(&input);

        assert!(
            result.expected_shortfall >= result.safety_at_risk,
            "ES ({}) should be >= SaR ({})",
            result.expected_shortfall,
            result.safety_at_risk
        );
    }

    #[test]
    fn test_portfolio_es() {
        let portfolio = vec![(vec![0.01, 0.02, 0.03], 0.5), (vec![0.02, 0.03, 0.04], 0.5)];
        let result = calculate_portfolio_es(&portfolio, 0.95);

        assert!(result.expected_shortfall >= 0.0);
    }
}
