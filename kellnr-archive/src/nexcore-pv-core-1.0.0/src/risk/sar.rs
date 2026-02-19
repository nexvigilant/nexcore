//! Safety-at-Risk (SaR) Calculator
//!
//! Pharmacovigilance adaptation of Value-at-Risk (VaR) for quantifying
//! safety risk exposure in patient populations and drug portfolios.
//!
//! SaR answers: "What is the maximum safety risk we might face
//! over a given time period with a given confidence level?"

use serde::{Deserialize, Serialize};

/// SaR calculation method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SarMethod {
    /// Historical simulation using actual data
    #[default]
    Historical,
    /// Parametric (assumes normal distribution)
    Parametric,
    /// Monte Carlo simulation
    MonteCarlo,
}

/// Input for Safety-at-Risk calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarInput {
    /// Historical adverse event rates
    pub adverse_event_rates: Vec<f64>,
    /// Confidence level (0.5 to 0.999)
    pub confidence_level: f64,
    /// Time horizon in days
    pub time_horizon: u32,
    /// Calculation method
    pub method: SarMethod,
}

impl Default for SarInput {
    fn default() -> Self {
        Self {
            adverse_event_rates: Vec::new(),
            confidence_level: 0.95,
            time_horizon: 30,
            method: SarMethod::Historical,
        }
    }
}

/// Backtest results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarBacktest {
    /// Number of violations
    pub violations: usize,
    /// Violation rate
    pub violation_rate: f64,
    /// Is model accurate
    pub is_accurate: bool,
}

/// Risk component breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskComponents {
    /// Market-wide safety trends
    pub systematic_risk: f64,
    /// Drug-specific risks
    pub idiosyncratic_risk: f64,
    /// Portfolio concentration
    pub concentration_risk: f64,
}

/// Safety-at-Risk result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SarResult {
    /// Calculation method used
    pub method: String,
    /// Safety-at-Risk value
    pub safety_at_risk: f64,
    /// Expected safety loss (ES/CVaR)
    pub expected_safety_loss: f64,
    /// Confidence level used
    pub confidence_level: f64,
    /// Time horizon in days
    pub time_horizon: u32,
    /// Maximum drawdown
    pub max_drawdown: f64,
    /// Risk-adjusted ratio
    pub sharpe_ratio: f64,
    /// Backtest results
    pub backtest: SarBacktest,
    /// Risk decomposition
    pub risk_components: RiskComponents,
}

/// Calculate variance of a slice
fn variance(data: &[f64]) -> f64 {
    if data.len() < 2 {
        return 0.0;
    }
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let sq_diffs: f64 = data.iter().map(|x| (x - mean).powi(2)).sum();
    sq_diffs / (data.len() - 1) as f64
}

/// Calculate standard deviation
fn volatility(data: &[f64]) -> f64 {
    variance(data).sqrt()
}

/// Calculate maximum drawdown
fn max_drawdown(rates: &[f64]) -> f64 {
    if rates.is_empty() {
        return 0.0;
    }
    let mut max_dd: f64 = 0.0;
    let mut peak = rates[0];

    for &rate in &rates[1..] {
        if rate > peak {
            peak = rate;
        } else if peak > 0.0 {
            let dd = (peak - rate) / peak;
            max_dd = max_dd.max(dd);
        }
    }
    max_dd
}

/// Get Z-score for confidence level
fn z_score(confidence: f64) -> f64 {
    match confidence {
        c if c >= 0.999 => -3.090,
        c if c >= 0.995 => -2.576,
        c if c >= 0.99 => -2.326,
        c if c >= 0.95 => -1.645,
        c if c >= 0.90 => -1.282,
        _ => -1.645,
    }
}

/// Normal PDF
fn normal_pdf(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

/// Simple LCG random number generator (0,1)
///
/// Uses a thread-local state for deterministic but varied results per thread.
fn rand_simple() -> f64 {
    use std::cell::Cell;
    // CONCURRENCY: Thread-local cell, no cross-thread access.
    // Each thread has its own seed, avoiding contention.
    thread_local! {
        static SEED: Cell<u64> = const { Cell::new(12345) };
    }
    SEED.with(|seed| {
        let mut s = seed.get();
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1);
        seed.set(s);
        (s >> 33) as f64 / (1u64 << 31) as f64
    })
}

/// Box-Muller normal random
fn random_normal() -> f64 {
    use std::f64::consts::PI;
    let u1 = rand_simple().max(1e-10); // Avoid log(0)
    let u2 = rand_simple();
    (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
}

/// Backtest SaR model
fn backtest(rates: &[f64], sar: f64, confidence: f64) -> SarBacktest {
    let violations = rates.iter().filter(|&&r| r > sar).count();
    let violation_rate = if rates.is_empty() {
        0.0
    } else {
        violations as f64 / rates.len() as f64
    };
    let expected = 1.0 - confidence;
    let is_accurate = (violation_rate - expected).abs() < 0.05;

    SarBacktest {
        violations,
        violation_rate,
        is_accurate,
    }
}

/// Decompose risk into components
fn decompose_risk(rates: &[f64]) -> RiskComponents {
    let total_var = variance(rates);
    RiskComponents {
        systematic_risk: total_var * 0.3,
        idiosyncratic_risk: total_var * 0.5,
        concentration_risk: total_var * 0.2,
    }
}

/// Calculate Historical SaR
fn historical_sar(input: &SarInput) -> SarResult {
    let rates = &input.adverse_event_rates;
    let horizon = input.time_horizon as f64;

    if rates.is_empty() {
        return empty_result("Historical Simulation", input);
    }

    let mut losses: Vec<f64> = rates.iter().map(|r| r * horizon).collect();
    losses.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    let var_idx = ((1.0 - input.confidence_level) * losses.len() as f64) as usize;
    let safety_at_risk = losses.get(var_idx).copied().unwrap_or(0.0);

    let tail = &losses[..=var_idx.min(losses.len().saturating_sub(1))];
    let expected_loss = if tail.is_empty() {
        0.0
    } else {
        tail.iter().sum::<f64>() / tail.len() as f64
    };

    let mean = rates.iter().sum::<f64>() / rates.len() as f64;
    let vol = volatility(rates);
    let sharpe = if vol > 0.0 { mean / vol } else { 0.0 };

    SarResult {
        method: "Historical Simulation".to_string(),
        safety_at_risk,
        expected_safety_loss: expected_loss,
        confidence_level: input.confidence_level,
        time_horizon: input.time_horizon,
        max_drawdown: max_drawdown(rates),
        sharpe_ratio: sharpe,
        backtest: backtest(rates, safety_at_risk, input.confidence_level),
        risk_components: decompose_risk(rates),
    }
}

/// Calculate Parametric SaR
fn parametric_sar(input: &SarInput) -> SarResult {
    let rates = &input.adverse_event_rates;
    let horizon = input.time_horizon as f64;

    if rates.is_empty() {
        return empty_result("Parametric (Normal)", input);
    }

    let mean = rates.iter().sum::<f64>() / rates.len() as f64;
    let vol = volatility(rates);
    let z = z_score(input.confidence_level);

    let safety_at_risk = (mean + z * vol) * horizon;
    let expected_loss = (mean + (normal_pdf(z) / (1.0 - input.confidence_level)) * vol) * horizon;
    let sharpe = if vol > 0.0 { mean / vol } else { 0.0 };

    SarResult {
        method: "Parametric (Normal)".to_string(),
        safety_at_risk,
        expected_safety_loss: expected_loss,
        confidence_level: input.confidence_level,
        time_horizon: input.time_horizon,
        max_drawdown: max_drawdown(rates),
        sharpe_ratio: sharpe,
        backtest: backtest(rates, safety_at_risk, input.confidence_level),
        risk_components: decompose_risk(rates),
    }
}

/// Calculate Monte Carlo SaR
fn monte_carlo_sar(input: &SarInput) -> SarResult {
    let rates = &input.adverse_event_rates;
    let horizon = input.time_horizon as f64;
    let simulations = 10_000;

    if rates.is_empty() {
        return empty_result("Monte Carlo Simulation", input);
    }

    let mean = rates.iter().sum::<f64>() / rates.len() as f64;
    let vol = volatility(rates);

    let mut scenarios: Vec<f64> = (0..simulations)
        .map(|_| (mean + random_normal() * vol) * horizon)
        .collect();

    scenarios.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

    let var_idx = ((1.0 - input.confidence_level) * simulations as f64) as usize;
    let safety_at_risk = scenarios[var_idx];

    let tail = &scenarios[..=var_idx];
    let expected_loss = tail.iter().sum::<f64>() / tail.len() as f64;
    let sharpe = if vol > 0.0 { mean / vol } else { 0.0 };

    SarResult {
        method: "Monte Carlo Simulation".to_string(),
        safety_at_risk,
        expected_safety_loss: expected_loss,
        confidence_level: input.confidence_level,
        time_horizon: input.time_horizon,
        max_drawdown: max_drawdown(rates),
        sharpe_ratio: sharpe,
        backtest: backtest(rates, safety_at_risk, input.confidence_level),
        risk_components: decompose_risk(rates),
    }
}

/// Create empty result for edge cases
fn empty_result(method: &str, input: &SarInput) -> SarResult {
    SarResult {
        method: method.to_string(),
        safety_at_risk: 0.0,
        expected_safety_loss: 0.0,
        confidence_level: input.confidence_level,
        time_horizon: input.time_horizon,
        max_drawdown: 0.0,
        sharpe_ratio: 0.0,
        backtest: SarBacktest {
            violations: 0,
            violation_rate: 0.0,
            is_accurate: true,
        },
        risk_components: RiskComponents {
            systematic_risk: 0.0,
            idiosyncratic_risk: 0.0,
            concentration_risk: 0.0,
        },
    }
}

/// Calculate Safety-at-Risk
#[must_use]
pub fn calculate_sar(input: &SarInput) -> SarResult {
    match input.method {
        SarMethod::Historical => historical_sar(input),
        SarMethod::Parametric => parametric_sar(input),
        SarMethod::MonteCarlo => monte_carlo_sar(input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_historical_sar() {
        let input = SarInput {
            adverse_event_rates: vec![0.01, 0.02, 0.015, 0.018, 0.025, 0.012],
            confidence_level: 0.95,
            time_horizon: 30,
            method: SarMethod::Historical,
        };
        let result = calculate_sar(&input);
        assert!(result.safety_at_risk > 0.0);
        assert_eq!(result.method, "Historical Simulation");
    }

    #[test]
    fn test_parametric_sar() {
        let input = SarInput {
            adverse_event_rates: vec![0.01, 0.02, 0.015, 0.018],
            confidence_level: 0.95,
            time_horizon: 30,
            method: SarMethod::Parametric,
        };
        let result = calculate_sar(&input);
        assert!(result.safety_at_risk.is_finite());
    }

    #[test]
    fn test_monte_carlo_sar() {
        let input = SarInput {
            adverse_event_rates: vec![0.01, 0.02, 0.015],
            confidence_level: 0.95,
            time_horizon: 30,
            method: SarMethod::MonteCarlo,
        };
        let result = calculate_sar(&input);
        assert!(result.safety_at_risk.is_finite());
    }

    #[test]
    fn test_empty_input() {
        let input = SarInput::default();
        let result = calculate_sar(&input);
        assert_eq!(result.safety_at_risk, 0.0);
    }
}
