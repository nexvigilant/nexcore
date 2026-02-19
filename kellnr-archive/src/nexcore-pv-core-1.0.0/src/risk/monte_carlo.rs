//! Monte Carlo Simulation Engine for Pharmacovigilance
//!
//! Advanced stochastic modeling for safety risk assessment, signal validation,
//! and regulatory decision support using Monte Carlo methods.
//!
//! # Applications
//!
//! - Signal strength validation under uncertainty
//! - Risk-benefit analysis with confidence intervals
//! - Regulatory submission success probability
//! - Clinical trial safety outcome predictions

use crate::signals::core::types::ContingencyTable;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;

/// Distribution type for adverse event rate sampling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum McDistribution {
    /// Normal (Gaussian) distribution
    Normal,
    /// Log-normal distribution (default for rates)
    #[default]
    LogNormal,
    /// Beta distribution (bounded 0-1)
    Beta,
    /// Gamma distribution (positive values)
    Gamma,
}

/// Adverse event rate parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdverseEventRates {
    /// Background AE rate in population
    pub baseline: f64,
    /// Drug-specific additional AE rate
    pub drug_specific: f64,
    /// Uncertainty coefficient (0-1)
    pub uncertainty: f64,
    /// Distribution for sampling
    pub distribution: McDistribution,
}

impl Default for AdverseEventRates {
    fn default() -> Self {
        Self {
            baseline: 0.01,
            drug_specific: 0.02,
            uncertainty: 0.2,
            distribution: McDistribution::LogNormal,
        }
    }
}

/// Patient exposure parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientExposure {
    /// Average daily patient count
    pub daily_patients: f64,
    /// Exposure duration in days
    pub exposure_duration: u32,
    /// Population growth rate (annual)
    pub population_growth: f64,
    /// Apply seasonal variation
    pub seasonality: bool,
    /// Uncertainty range for exposure (0-1)
    pub uncertainty_range: f64,
}

impl Default for PatientExposure {
    fn default() -> Self {
        Self {
            daily_patients: 1000.0,
            exposure_duration: 365,
            population_growth: 0.0,
            seasonality: false,
            uncertainty_range: 0.2,
        }
    }
}

/// Signal detection thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalThresholds {
    /// ROR threshold for signal detection
    pub ror_threshold: f64,
    /// PRR threshold for signal detection
    pub prr_threshold: f64,
    /// IC (BCPNN) threshold for signal detection
    pub bcpnn_threshold: f64,
    /// Minimum case count for signal
    pub minimum_cases: u32,
    /// Confidence level (0.5-0.999)
    pub confidence_level: f64,
}

impl Default for SignalThresholds {
    fn default() -> Self {
        Self {
            ror_threshold: 2.0,
            prr_threshold: 2.0,
            bcpnn_threshold: 0.0,
            minimum_cases: 3,
            confidence_level: 0.95,
        }
    }
}

/// Time horizon configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeHorizon {
    /// Total simulation days
    pub days: u32,
    /// Milestone days for interim analysis
    pub milestones: Vec<u32>,
}

impl Default for TimeHorizon {
    fn default() -> Self {
        Self {
            days: 365,
            milestones: vec![30, 90, 180, 365],
        }
    }
}

/// Monte Carlo simulation input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McInput {
    /// Number of simulations to run
    pub simulations: u32,
    /// Random seed for reproducibility
    pub random_seed: Option<u64>,
    /// Adverse event rate parameters
    pub adverse_event_rates: AdverseEventRates,
    /// Patient exposure parameters
    pub patient_exposure: PatientExposure,
    /// Signal detection thresholds
    pub signal_thresholds: SignalThresholds,
    /// Time horizon configuration
    pub time_horizon: TimeHorizon,
}

impl Default for McInput {
    fn default() -> Self {
        Self {
            simulations: 10_000,
            random_seed: None,
            adverse_event_rates: AdverseEventRates::default(),
            patient_exposure: PatientExposure::default(),
            signal_thresholds: SignalThresholds::default(),
            time_horizon: TimeHorizon::default(),
        }
    }
}

/// Summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McSummary {
    /// Mean adverse events across simulations
    pub mean_adverse_events: f64,
    /// Median adverse events
    pub median_adverse_events: f64,
    /// Standard deviation
    pub standard_deviation: f64,
    /// 5th percentile
    pub p5: f64,
    /// 95th percentile
    pub p95: f64,
    /// 99th percentile
    pub p99: f64,
}

/// Signal detection analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McSignalDetection {
    /// Probability of detecting a signal
    pub probability_of_signal: f64,
    /// Expected time to signal detection (days)
    pub expected_time_to_signal: f64,
    /// Mean ROR across simulations
    pub mean_ror: f64,
    /// Mean PRR across simulations
    pub mean_prr: f64,
    /// Mean IC (BCPNN) across simulations
    pub mean_bcpnn: f64,
}

/// Risk metrics (VaR-style for safety)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McRiskMetrics {
    /// Safety Value at Risk
    pub value_at_risk: f64,
    /// Expected Shortfall (CVaR)
    pub expected_shortfall: f64,
    /// Maximum drawdown
    pub max_drawdown: f64,
    /// Safety Sharpe ratio
    pub sharpe_ratio: f64,
}

/// Scenario analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McScenarios {
    /// Best case events
    pub best_case: f64,
    /// Worst case events
    pub worst_case: f64,
    /// Stress test (99th percentile)
    pub stress_test: f64,
}

/// Monte Carlo simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McResult {
    /// Number of simulations run
    pub simulations: u32,
    /// Summary statistics
    pub summary: McSummary,
    /// Signal detection analysis
    pub signal_detection: McSignalDetection,
    /// Risk metrics
    pub risk_metrics: McRiskMetrics,
    /// Scenario analysis
    pub scenarios: McScenarios,
    /// Generated recommendations
    pub recommendations: Vec<String>,
}

/// Seeded random number generator
struct SeededRng {
    state: u64,
}

impl SeededRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> f64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (self.state >> 33) as f64 / (1u64 << 31) as f64
    }

    fn next_normal(&mut self, mean: f64, std_dev: f64) -> f64 {
        use std::f64::consts::PI;
        let u1 = self.next().max(1e-10);
        let u2 = self.next();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();
        mean + std_dev * z
    }

    fn next_poisson(&mut self, lambda: f64) -> u32 {
        if lambda < 10.0 {
            let l = (-lambda).exp();
            let mut k = 0u32;
            let mut p = 1.0;
            loop {
                k += 1;
                p *= self.next();
                if p <= l {
                    break;
                }
            }
            k.saturating_sub(1)
        } else {
            self.next_normal(lambda, lambda.sqrt()).max(0.0).round() as u32
        }
    }
}

/// Monte Carlo simulation engine
pub struct MonteCarloEngine {
    rng: RefCell<SeededRng>,
}

impl MonteCarloEngine {
    /// Create a new Monte Carlo engine with optional seed
    #[must_use]
    pub fn new(seed: Option<u64>) -> Self {
        Self {
            rng: RefCell::new(SeededRng::new(seed.unwrap_or(12345))),
        }
    }

    /// Run Monte Carlo simulation
    #[must_use]
    pub fn run_simulation(&self, input: &McInput) -> McResult {
        let mut sim_results: Vec<f64> = Vec::with_capacity(input.simulations as usize);
        let mut signal_results: Vec<SignalSimResult> =
            Vec::with_capacity(input.simulations as usize);

        for _ in 0..input.simulations {
            let result = self.run_single_simulation(input);
            sim_results.push(result.total_events);
            signal_results.push(result.signal_metrics);
        }

        let summary = self.calculate_summary(&sim_results);
        let signal_detection = self.analyze_signal_detection(&signal_results);
        let risk_metrics = self.calculate_risk_metrics(&sim_results, input);
        let scenarios = self.calculate_scenarios(&sim_results);
        let recommendations =
            self.generate_recommendations(&summary, &signal_detection, &risk_metrics, &scenarios);

        McResult {
            simulations: input.simulations,
            summary,
            signal_detection,
            risk_metrics,
            scenarios,
            recommendations,
        }
    }

    fn run_single_simulation(&self, input: &McInput) -> SingleSimResult {
        let mut total_events = 0.0;
        let mut current_patients = input.patient_exposure.daily_patients;
        let mut signal_detected = false;
        let mut time_to_signal = input.time_horizon.days;
        let mut contingency = ContingencyTable::default();

        for day in 1..=input.time_horizon.days {
            current_patients = self.update_patient_exposure(current_patients, day, input);
            let daily_rate = self.sample_ae_rate(input);
            let expected = current_patients * daily_rate;
            let daily_events = self.rng.borrow_mut().next_poisson(expected);
            total_events += f64::from(daily_events);

            contingency.a += u64::from(daily_events);
            contingency.b += u64::from((current_patients as u32).saturating_sub(daily_events));
            let bg = self
                .rng
                .borrow_mut()
                .next_poisson(input.adverse_event_rates.baseline * 10000.0);
            contingency.c += u64::from(bg);
            contingency.d += u64::from(10000u32.saturating_sub(bg));

            if !signal_detected && contingency.a >= u64::from(input.signal_thresholds.minimum_cases)
            {
                let signals = self.check_signal(&contingency, &input.signal_thresholds);
                if signals.detected {
                    signal_detected = true;
                    time_to_signal = day;
                }
            }
        }

        let final_signals = self.check_signal(&contingency, &input.signal_thresholds);

        SingleSimResult {
            total_events,
            signal_metrics: SignalSimResult {
                signal_detected,
                time_to_signal,
                ror: final_signals.ror,
                prr: final_signals.prr,
                bcpnn: final_signals.bcpnn,
            },
        }
    }

    fn update_patient_exposure(&self, current: f64, day: u32, input: &McInput) -> f64 {
        let mut patients = current;

        if input.patient_exposure.population_growth != 0.0 {
            patients *= 1.0 + input.patient_exposure.population_growth / 365.0;
        }

        if input.patient_exposure.seasonality {
            patients *= 1.0 + 0.2 * (2.0 * std::f64::consts::PI * f64::from(day) / 365.0).sin();
        }

        let uncertainty = self.rng.borrow_mut().next() * 2.0 - 1.0;
        patients *= 1.0 + uncertainty * input.patient_exposure.uncertainty_range;

        patients.max(0.0)
    }

    fn sample_ae_rate(&self, input: &McInput) -> f64 {
        let rates = &input.adverse_event_rates;
        let mean = rates.baseline + rates.drug_specific;
        let variance = (mean * rates.uncertainty).powi(2);
        let mut rng = self.rng.borrow_mut();

        match rates.distribution {
            McDistribution::Normal => rng.next_normal(mean, variance.sqrt()).max(0.0),
            McDistribution::LogNormal => {
                let mu = mean.ln() - 0.5 * (1.0 + variance / (mean * mean)).ln();
                let sigma = (1.0 + variance / (mean * mean)).ln().sqrt();
                (mu + sigma * rng.next_normal(0.0, 1.0)).exp()
            }
            McDistribution::Beta | McDistribution::Gamma => {
                rng.next_normal(mean, variance.sqrt()).clamp(0.0, 1.0)
            }
        }
    }

    fn check_signal(&self, table: &ContingencyTable, thresholds: &SignalThresholds) -> SignalCheck {
        let a = table.a as f64;
        let b = table.b as f64;
        let c = table.c as f64;
        let d = table.d as f64;

        if b == 0.0 || c == 0.0 || (a + b) == 0.0 || (c + d) == 0.0 {
            return SignalCheck::default();
        }

        let ror = (a * d) / (b * c);
        let prr = (a / (a + b)) / (c / (c + d));
        let n = a + b + c + d;
        let bcpnn = if (a + b) > 0.0 && (a + c) > 0.0 && n > 0.0 {
            (a * n / ((a + b) * (a + c))).ln() / std::f64::consts::LN_2
        } else {
            0.0
        };

        let min_cases = u64::from(thresholds.minimum_cases);
        let detected = table.a >= min_cases
            && (ror >= thresholds.ror_threshold
                || prr >= thresholds.prr_threshold
                || bcpnn >= thresholds.bcpnn_threshold);

        SignalCheck {
            detected,
            ror,
            prr,
            bcpnn,
        }
    }

    fn calculate_summary(&self, results: &[f64]) -> McSummary {
        let mut sorted = results.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = sorted.len();
        if n == 0 {
            return McSummary {
                mean_adverse_events: 0.0,
                median_adverse_events: 0.0,
                standard_deviation: 0.0,
                p5: 0.0,
                p95: 0.0,
                p99: 0.0,
            };
        }

        let mean = sorted.iter().sum::<f64>() / n as f64;
        let median = if n % 2 == 0 {
            (sorted[n / 2 - 1] + sorted[n / 2]) / 2.0
        } else {
            sorted[n / 2]
        };
        let variance: f64 =
            sorted.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1).max(1) as f64;

        McSummary {
            mean_adverse_events: mean,
            median_adverse_events: median,
            standard_deviation: variance.sqrt(),
            p5: sorted[(n as f64 * 0.05) as usize],
            p95: sorted[(n as f64 * 0.95).min((n - 1) as f64) as usize],
            p99: sorted[(n as f64 * 0.99).min((n - 1) as f64) as usize],
        }
    }

    fn analyze_signal_detection(&self, results: &[SignalSimResult]) -> McSignalDetection {
        let total = results.len() as f64;
        if total == 0.0 {
            return McSignalDetection {
                probability_of_signal: 0.0,
                expected_time_to_signal: 0.0,
                mean_ror: 0.0,
                mean_prr: 0.0,
                mean_bcpnn: 0.0,
            };
        }

        let signals_detected = results.iter().filter(|r| r.signal_detected).count() as f64;
        let detected_results: Vec<_> = results.iter().filter(|r| r.signal_detected).collect();
        let expected_time = if detected_results.is_empty() {
            0.0
        } else {
            detected_results
                .iter()
                .map(|r| f64::from(r.time_to_signal))
                .sum::<f64>()
                / detected_results.len() as f64
        };

        McSignalDetection {
            probability_of_signal: signals_detected / total,
            expected_time_to_signal: expected_time,
            mean_ror: results.iter().map(|r| r.ror).sum::<f64>() / total,
            mean_prr: results.iter().map(|r| r.prr).sum::<f64>() / total,
            mean_bcpnn: results.iter().map(|r| r.bcpnn).sum::<f64>() / total,
        }
    }

    fn calculate_risk_metrics(&self, results: &[f64], input: &McInput) -> McRiskMetrics {
        let mut sorted = results.to_vec();
        sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));

        let n = sorted.len();
        if n == 0 {
            return McRiskMetrics {
                value_at_risk: 0.0,
                expected_shortfall: 0.0,
                max_drawdown: 0.0,
                sharpe_ratio: 0.0,
            };
        }

        let var_idx = ((1.0 - input.signal_thresholds.confidence_level) * n as f64) as usize;
        let value_at_risk = sorted.get(var_idx).copied().unwrap_or(0.0);
        let tail = &sorted[..=var_idx.min(n - 1)];
        let expected_shortfall = tail.iter().sum::<f64>() / tail.len() as f64;

        let mut max_dd = 0.0f64;
        let mut peak = results.first().copied().unwrap_or(0.0);
        for &val in results {
            if val > peak {
                peak = val;
            } else if peak > 0.0 {
                max_dd = max_dd.max((peak - val) / peak);
            }
        }

        let mean = results.iter().sum::<f64>() / n as f64;
        let variance: f64 =
            results.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1).max(1) as f64;

        McRiskMetrics {
            value_at_risk,
            expected_shortfall,
            max_drawdown: max_dd,
            sharpe_ratio: if variance > 0.0 {
                mean / variance.sqrt()
            } else {
                0.0
            },
        }
    }

    fn calculate_scenarios(&self, results: &[f64]) -> McScenarios {
        let mut sorted = results.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = sorted.len();
        if n == 0 {
            return McScenarios {
                best_case: 0.0,
                worst_case: 0.0,
                stress_test: 0.0,
            };
        }

        McScenarios {
            best_case: sorted[0],
            worst_case: sorted[n - 1],
            stress_test: sorted[(n as f64 * 0.99).min((n - 1) as f64) as usize],
        }
    }

    fn generate_recommendations(
        &self,
        summary: &McSummary,
        signal: &McSignalDetection,
        risk: &McRiskMetrics,
        scenarios: &McScenarios,
    ) -> Vec<String> {
        let mut recs = Vec::new();

        if signal.probability_of_signal > 0.8 {
            recs.push("HIGH: Signal detection highly likely - enhanced monitoring needed".into());
        }
        if risk.value_at_risk > summary.mean_adverse_events * 2.0 {
            recs.push("Consider risk mitigation strategies for tail scenarios".into());
        }
        if signal.expected_time_to_signal < 30.0 && signal.expected_time_to_signal > 0.0 {
            recs.push("Early signal expected - implement rapid response protocols".into());
        }
        if scenarios.worst_case > summary.mean_adverse_events * 5.0 {
            recs.push("Stress test shows high risk - review safety profile".into());
        }
        if recs.is_empty() {
            recs.push("No immediate concerns - continue standard monitoring".into());
        }

        recs
    }
}

struct SingleSimResult {
    total_events: f64,
    signal_metrics: SignalSimResult,
}

#[derive(Clone)]
struct SignalSimResult {
    signal_detected: bool,
    time_to_signal: u32,
    ror: f64,
    prr: f64,
    bcpnn: f64,
}

#[derive(Default)]
struct SignalCheck {
    detected: bool,
    ror: f64,
    prr: f64,
    bcpnn: f64,
}

/// Convenience function for quick Monte Carlo simulation
#[must_use]
pub fn run_monte_carlo(input: &McInput) -> McResult {
    let engine = MonteCarloEngine::new(input.random_seed);
    engine.run_simulation(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_simulation() {
        let input = McInput {
            simulations: 100,
            random_seed: Some(42),
            ..Default::default()
        };

        let result = run_monte_carlo(&input);

        assert_eq!(result.simulations, 100);
        assert!(result.summary.mean_adverse_events > 0.0);
        assert!(result.summary.standard_deviation >= 0.0);
    }

    #[test]
    fn test_signal_detection() {
        let input = McInput {
            simulations: 100,
            random_seed: Some(42),
            adverse_event_rates: AdverseEventRates {
                baseline: 0.01,
                drug_specific: 0.05,
                uncertainty: 0.1,
                ..Default::default()
            },
            ..Default::default()
        };

        let result = run_monte_carlo(&input);

        assert!(result.signal_detection.probability_of_signal >= 0.0);
        assert!(result.signal_detection.probability_of_signal <= 1.0);
    }

    #[test]
    fn test_risk_metrics() {
        let input = McInput {
            simulations: 100,
            random_seed: Some(42),
            ..Default::default()
        };

        let result = run_monte_carlo(&input);

        assert!(result.risk_metrics.value_at_risk >= 0.0);
        assert!(result.risk_metrics.expected_shortfall >= 0.0);
    }

    #[test]
    fn test_deterministic_with_seed() {
        let input = McInput {
            simulations: 50,
            random_seed: Some(12345),
            ..Default::default()
        };

        let result1 = run_monte_carlo(&input);
        let result2 = run_monte_carlo(&input);

        assert_eq!(
            result1.summary.mean_adverse_events,
            result2.summary.mean_adverse_events
        );
    }

    #[test]
    fn test_recommendations_generated() {
        let input = McInput::default();
        let result = run_monte_carlo(&input);

        assert!(!result.recommendations.is_empty());
    }
}
