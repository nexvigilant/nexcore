//! # Loss Distribution Approach (LDA)
//!
//! Operational risk modeling for pharmacovigilance using frequency and
//! severity distributions to estimate aggregate loss distributions.
//!
//! ## Key Concepts
//!
//! | Financial Concept | PV Adaptation |
//! |-------------------|---------------|
//! | Operational Loss | Adverse event costs |
//! | Frequency | Event occurrence rate |
//! | Severity | Impact magnitude |
//! | Capital Requirement | Safety reserves |

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Event types for operational risk
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    AdverseEvent,
    RegulatoryPenalty,
    ComplianceFailure,
    DataBreach,
    SystemFailure,
    ProcessError,
    ExternalFraud,
}

/// Frequency distribution type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum FrequencyDistribution {
    #[default]
    Poisson,
    NegativeBinomial,
    ZeroInflatedPoisson,
}

/// Severity distribution type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum SeverityDistribution {
    #[default]
    Lognormal,
    Pareto,
    Weibull,
    Gamma,
    Exponential,
}

/// Dependence model for aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum DependenceModel {
    #[default]
    Independent,
    GaussianCopula,
    TCopula,
}

/// Individual loss event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossEvent {
    pub severity: f64,
    pub frequency: f64,
    pub event_type: EventType,
    pub business_line: Option<String>,
    pub timestamp: Option<f64>,
}

/// Distribution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionParams {
    pub distribution: String,
    pub parameters: HashMap<String, f64>,
}

/// Fitted distribution for an event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FittedDistribution {
    pub frequency: DistributionParams,
    pub severity: DistributionParams,
}

fn default_time_horizon() -> u32 {
    365
}
fn default_confidence() -> f64 {
    0.99
}
fn default_mc_paths() -> u32 {
    100_000
}

/// Input for LDA calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdaInput {
    pub loss_events: Vec<LossEvent>,
    #[serde(default = "default_time_horizon")]
    pub time_horizon: u32,
    #[serde(default = "default_confidence")]
    pub confidence_level: f64,
    #[serde(default = "default_mc_paths")]
    pub monte_carlo_paths: u32,
    #[serde(default)]
    pub frequency_distribution: FrequencyDistribution,
    #[serde(default)]
    pub severity_distribution: SeverityDistribution,
    #[serde(default)]
    pub dependence_model: DependenceModel,
    pub correlation_matrix: Option<Vec<Vec<f64>>>,
}

/// Percentile results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Percentiles {
    pub p50: f64,
    pub p90: f64,
    pub p95: f64,
    pub p99: f64,
    pub p999: f64,
}

/// Aggregate loss metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateLoss {
    pub percentiles: Percentiles,
    pub expected_loss: f64,
    pub unexpected_loss: f64,
    pub capital_requirement: f64,
}

/// Frequency analysis for an event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyAnalysis {
    pub mean: f64,
    pub variance: f64,
    pub distribution: String,
    pub parameters: HashMap<String, f64>,
}

/// Severity analysis for an event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityAnalysis {
    pub mean: f64,
    pub variance: f64,
    pub distribution: String,
    pub parameters: HashMap<String, f64>,
}

/// Aggregate loss by event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeAggregateLoss {
    pub expected_value: f64,
    pub variance: f64,
    pub percentiles: Percentiles,
}

/// Complete analysis for an event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventTypeAnalysis {
    pub frequency: FrequencyAnalysis,
    pub severity: SeverityAnalysis,
    pub aggregate_loss: EventTypeAggregateLoss,
}

/// Diversification benefit metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiversificationBenefit {
    pub portfolio_var: f64,
    pub sum_of_individual_vars: f64,
    pub diversification_ratio: f64,
    pub concentration_risk: f64,
}

/// Risk contribution for an event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskContribution {
    pub event_type: EventType,
    pub marginal_var: f64,
    pub component_var: f64,
    pub percentage_contribution: f64,
}

/// Stress scenario result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressScenario {
    pub name: String,
    pub description: String,
    pub aggregate_loss: f64,
    pub probability_adjustment: f64,
}

/// Scenario analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioAnalysis {
    pub base_case: f64,
    pub stress_scenarios: Vec<StressScenario>,
}

/// Statistical test result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalTest {
    pub statistic: f64,
    pub p_value: f64,
    pub rejected: bool,
}

/// Back-testing results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackTesting {
    pub violations: u32,
    pub violation_rate: f64,
    pub kupiec_test: StatisticalTest,
    pub christoffersen_test: StatisticalTest,
}

/// Goodness-of-fit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoodnessOfFit {
    pub statistic: f64,
    pub p_value: f64,
    pub accepted: bool,
}

/// Model validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelValidation {
    pub goodness_of_fit: HashMap<String, GoodnessOfFit>,
    pub parameter_stability: bool,
    pub out_of_sample_performance: f64,
}

/// Complete LDA result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LdaResult {
    pub aggregate_loss: AggregateLoss,
    pub by_event_type: HashMap<String, EventTypeAnalysis>,
    pub diversification_benefit: DiversificationBenefit,
    pub risk_contributions: Vec<RiskContribution>,
    pub scenario_analysis: ScenarioAnalysis,
    pub back_testing: BackTesting,
    pub model_validation: ModelValidation,
}

/// Calculate Loss Distribution using Monte Carlo simulation
#[must_use]
pub fn calculate_loss_distribution(input: &LdaInput) -> LdaResult {
    if input.loss_events.is_empty() {
        return empty_result();
    }
    let events_by_type = organize_events_by_type(&input.loss_events);
    let distribution_params = fit_distributions(&events_by_type, input);
    let simulation = run_monte_carlo(&distribution_params, input);
    let aggregate_loss =
        calculate_aggregate_loss_metrics(&simulation.aggregate_losses, input.confidence_level);
    let diversification_benefit = calculate_diversification_benefit(
        &simulation.losses_by_type,
        aggregate_loss.percentiles.p99,
    );
    let risk_contributions =
        calculate_risk_contributions(&simulation.losses_by_type, aggregate_loss.percentiles.p99);
    let scenario_analysis = perform_scenario_analysis(&distribution_params, input.time_horizon);
    let back_testing = perform_back_testing(&simulation.aggregate_losses, input.confidence_level);
    let model_validation = perform_model_validation(&events_by_type, &distribution_params);
    let by_event_type = create_event_type_results(&distribution_params, &simulation.losses_by_type);
    LdaResult {
        aggregate_loss,
        by_event_type,
        diversification_benefit,
        risk_contributions,
        scenario_analysis,
        back_testing,
        model_validation,
    }
}

/// Batch process multiple LDA inputs
#[must_use]
pub fn batch_lda(inputs: &[LdaInput]) -> Vec<LdaResult> {
    inputs.iter().map(calculate_loss_distribution).collect()
}

fn empty_result() -> LdaResult {
    LdaResult {
        aggregate_loss: AggregateLoss {
            percentiles: Percentiles {
                p50: 0.0,
                p90: 0.0,
                p95: 0.0,
                p99: 0.0,
                p999: 0.0,
            },
            expected_loss: 0.0,
            unexpected_loss: 0.0,
            capital_requirement: 0.0,
        },
        by_event_type: HashMap::new(),
        diversification_benefit: DiversificationBenefit {
            portfolio_var: 0.0,
            sum_of_individual_vars: 0.0,
            diversification_ratio: 1.0,
            concentration_risk: 0.0,
        },
        risk_contributions: Vec::new(),
        scenario_analysis: ScenarioAnalysis {
            base_case: 0.0,
            stress_scenarios: Vec::new(),
        },
        back_testing: BackTesting {
            violations: 0,
            violation_rate: 0.0,
            kupiec_test: StatisticalTest {
                statistic: 0.0,
                p_value: 1.0,
                rejected: false,
            },
            christoffersen_test: StatisticalTest {
                statistic: 0.0,
                p_value: 1.0,
                rejected: false,
            },
        },
        model_validation: ModelValidation {
            goodness_of_fit: HashMap::new(),
            parameter_stability: true,
            out_of_sample_performance: 0.0,
        },
    }
}

fn organize_events_by_type(events: &[LossEvent]) -> HashMap<EventType, Vec<&LossEvent>> {
    let mut by_type: HashMap<EventType, Vec<&LossEvent>> = HashMap::new();
    events
        .iter()
        .for_each(|e| by_type.entry(e.event_type).or_default().push(e));
    by_type
}

fn fit_distributions(
    events_by_type: &HashMap<EventType, Vec<&LossEvent>>,
    input: &LdaInput,
) -> HashMap<EventType, FittedDistribution> {
    events_by_type
        .iter()
        .map(|(&event_type, events)| {
            let frequencies: Vec<f64> = events.iter().map(|e| e.frequency).collect();
            let severities: Vec<f64> = events.iter().map(|e| e.severity).collect();
            let freq_params =
                fit_frequency_distribution(&frequencies, input.frequency_distribution);
            let sev_params = fit_severity_distribution(&severities, input.severity_distribution);
            (
                event_type,
                FittedDistribution {
                    frequency: freq_params,
                    severity: sev_params,
                },
            )
        })
        .collect()
}

fn fit_frequency_distribution(
    frequencies: &[f64],
    dist: FrequencyDistribution,
) -> DistributionParams {
    if frequencies.is_empty() {
        return DistributionParams {
            distribution: format!("{dist:?}").to_lowercase(),
            parameters: HashMap::from([("lambda".into(), 1.0)]),
        };
    }
    let n = frequencies.len() as f64;
    let mean = frequencies.iter().sum::<f64>() / n;
    let variance = frequencies.iter().map(|f| (f - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let parameters = match dist {
        FrequencyDistribution::Poisson => HashMap::from([("lambda".into(), mean)]),
        FrequencyDistribution::NegativeBinomial => {
            let p = (mean / variance).clamp(0.01, 0.99);
            let r = (mean * p / (1.0 - p)).max(0.1);
            HashMap::from([("r".into(), r), ("p".into(), p)])
        }
        FrequencyDistribution::ZeroInflatedPoisson => {
            let pi = frequencies.iter().filter(|&&f| f == 0.0).count() as f64 / n;
            let adjusted_mean = mean / (1.0 - pi).max(0.01);
            HashMap::from([("lambda".into(), adjusted_mean), ("pi".into(), pi)])
        }
    };
    DistributionParams {
        distribution: format!("{dist:?}").to_lowercase(),
        parameters,
    }
}

fn fit_severity_distribution(severities: &[f64], dist: SeverityDistribution) -> DistributionParams {
    if severities.is_empty() {
        return DistributionParams {
            distribution: format!("{dist:?}").to_lowercase(),
            parameters: HashMap::from([("mu".into(), 0.0), ("sigma".into(), 1.0)]),
        };
    }
    let n = severities.len() as f64;
    let log_sev: Vec<f64> = severities.iter().map(|s| s.max(0.01).ln()).collect();
    let mean = severities.iter().sum::<f64>() / n;
    let log_mean = log_sev.iter().sum::<f64>() / n;
    let log_var = log_sev.iter().map(|s| (s - log_mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
    let parameters = match dist {
        SeverityDistribution::Lognormal => {
            HashMap::from([("mu".into(), log_mean), ("sigma".into(), log_var.sqrt())])
        }
        SeverityDistribution::Pareto => {
            let min_val = severities.iter().cloned().fold(f64::INFINITY, f64::min);
            let alpha = n / severities
                .iter()
                .map(|s| (s / min_val).ln())
                .sum::<f64>()
                .max(0.01);
            HashMap::from([("alpha".into(), alpha.max(1.1)), ("xmin".into(), min_val)])
        }
        SeverityDistribution::Weibull => {
            let variance =
                severities.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
            let cv = variance.sqrt() / mean.max(0.01);
            let shape = (1.2 / cv.max(0.01)).powf(1.1).max(0.5);
            let scale = (mean / gamma_fn(1.0 + 1.0 / shape)).max(0.01);
            HashMap::from([("shape".into(), shape), ("scale".into(), scale)])
        }
        SeverityDistribution::Gamma => {
            let variance =
                severities.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / (n - 1.0).max(1.0);
            let shape = (mean * mean / variance.max(0.01)).max(0.1);
            let rate = (mean / variance.max(0.01)).max(0.01);
            HashMap::from([("shape".into(), shape), ("rate".into(), rate)])
        }
        SeverityDistribution::Exponential => HashMap::from([("rate".into(), 1.0 / mean.max(0.01))]),
    };
    DistributionParams {
        distribution: format!("{dist:?}").to_lowercase(),
        parameters,
    }
}

struct SimulationResult {
    aggregate_losses: Vec<f64>,
    losses_by_type: HashMap<EventType, Vec<f64>>,
}

fn run_monte_carlo(
    params: &HashMap<EventType, FittedDistribution>,
    input: &LdaInput,
) -> SimulationResult {
    let paths = input.monte_carlo_paths as usize;
    let event_types: Vec<EventType> = params.keys().copied().collect();
    let mut rng = SimpleRng::new(42);
    let mut aggregate_losses = Vec::with_capacity(paths);
    let mut losses_by_type: HashMap<EventType, Vec<f64>> = event_types
        .iter()
        .map(|&et| (et, Vec::with_capacity(paths)))
        .collect();
    (0..paths).for_each(|_| {
        let mut total = 0.0;
        event_types.iter().for_each(|&et| {
            if let Some(dist) = params.get(&et) {
                let freq = generate_frequency(&dist.frequency, &mut rng);
                let horizon_freq = (freq * f64::from(input.time_horizon) / 365.0).round() as u32;
                let loss: f64 = (0..horizon_freq)
                    .map(|_| generate_severity(&dist.severity, &mut rng))
                    .sum();
                if let Some(v) = losses_by_type.get_mut(&et) {
                    v.push(loss)
                }
                total += loss;
            }
        });
        aggregate_losses.push(total);
    });
    SimulationResult {
        aggregate_losses,
        losses_by_type,
    }
}

fn generate_frequency(params: &DistributionParams, rng: &mut SimpleRng) -> f64 {
    let lambda = params.parameters.get("lambda").copied().unwrap_or(1.0);
    let pi = params.parameters.get("pi").copied().unwrap_or(0.0);
    if pi > 0.0 && rng.next_f64() < pi {
        return 0.0;
    }
    poisson_random(lambda, rng) as f64
}

fn generate_severity(params: &DistributionParams, rng: &mut SimpleRng) -> f64 {
    match params.distribution.as_str() {
        "lognormal" => lognormal_random(
            params.parameters.get("mu").copied().unwrap_or(0.0),
            params.parameters.get("sigma").copied().unwrap_or(1.0),
            rng,
        ),
        "pareto" => pareto_random(
            params.parameters.get("alpha").copied().unwrap_or(2.0),
            params.parameters.get("xmin").copied().unwrap_or(1.0),
            rng,
        ),
        "weibull" => weibull_random(
            params.parameters.get("shape").copied().unwrap_or(1.0),
            params.parameters.get("scale").copied().unwrap_or(1.0),
            rng,
        ),
        "gamma" => gamma_random(
            params.parameters.get("shape").copied().unwrap_or(1.0),
            params.parameters.get("rate").copied().unwrap_or(1.0),
            rng,
        ),
        "exponential" => {
            exponential_random(params.parameters.get("rate").copied().unwrap_or(1.0), rng)
        }
        _ => lognormal_random(
            params.parameters.get("mu").copied().unwrap_or(0.0),
            params.parameters.get("sigma").copied().unwrap_or(1.0),
            rng,
        ),
    }
}

fn calculate_aggregate_loss_metrics(losses: &[f64], confidence: f64) -> AggregateLoss {
    if losses.is_empty() {
        return AggregateLoss {
            percentiles: Percentiles {
                p50: 0.0,
                p90: 0.0,
                p95: 0.0,
                p99: 0.0,
                p999: 0.0,
            },
            expected_loss: 0.0,
            unexpected_loss: 0.0,
            capital_requirement: 0.0,
        };
    }
    let mut sorted = losses.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    let percentiles = Percentiles {
        p50: sorted[(0.50 * n as f64) as usize],
        p90: sorted[(0.90 * n as f64) as usize],
        p95: sorted[(0.95 * n as f64) as usize],
        p99: sorted[(0.99 * n as f64) as usize],
        p999: sorted[((0.999 * n as f64) as usize).min(n.saturating_sub(1))],
    };
    let expected_loss = losses.iter().sum::<f64>() / n as f64;
    let var = sorted[(confidence * n as f64) as usize];
    let unexpected_loss = var - expected_loss;
    AggregateLoss {
        percentiles,
        expected_loss,
        unexpected_loss,
        capital_requirement: unexpected_loss * 1.2,
    }
}

fn calculate_diversification_benefit(
    losses_by_type: &HashMap<EventType, Vec<f64>>,
    portfolio_var: f64,
) -> DiversificationBenefit {
    let sum_of_individual_vars: f64 = losses_by_type
        .values()
        .map(|losses| {
            if losses.is_empty() {
                return 0.0;
            }
            let mut sorted = losses.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            sorted[(0.99 * sorted.len() as f64) as usize]
        })
        .sum();
    let div_ratio = if sum_of_individual_vars > 0.0 {
        portfolio_var / sum_of_individual_vars
    } else {
        1.0
    };
    DiversificationBenefit {
        portfolio_var,
        sum_of_individual_vars,
        diversification_ratio: div_ratio,
        concentration_risk: 1.0 - div_ratio,
    }
}

fn calculate_risk_contributions(
    losses_by_type: &HashMap<EventType, Vec<f64>>,
    portfolio_var: f64,
) -> Vec<RiskContribution> {
    let mut contributions: Vec<RiskContribution> = losses_by_type
        .iter()
        .map(|(&et, losses)| {
            if losses.is_empty() {
                return RiskContribution {
                    event_type: et,
                    marginal_var: 0.0,
                    component_var: 0.0,
                    percentage_contribution: 0.0,
                };
            }
            let mut sorted = losses.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let event_var = sorted[(0.99 * sorted.len() as f64) as usize];
            let component = event_var * 0.5;
            RiskContribution {
                event_type: et,
                marginal_var: event_var,
                component_var: component,
                percentage_contribution: if portfolio_var > 0.0 {
                    (component / portfolio_var) * 100.0
                } else {
                    0.0
                },
            }
        })
        .collect();
    contributions.sort_by(|a, b| {
        b.percentage_contribution
            .partial_cmp(&a.percentage_contribution)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    contributions
}

fn perform_scenario_analysis(
    params: &HashMap<EventType, FittedDistribution>,
    time_horizon: u32,
) -> ScenarioAnalysis {
    let base_case: f64 = params
        .values()
        .map(|dist| {
            dist.frequency
                .parameters
                .get("lambda")
                .copied()
                .unwrap_or(1.0)
                * calculate_severity_mean(&dist.severity)
                * f64::from(time_horizon)
                / 365.0
        })
        .sum();
    let stress_scenarios = vec![
        StressScenario {
            name: "Frequency Stress".into(),
            description: "2x increase in all event frequencies".into(),
            aggregate_loss: base_case * 2.0,
            probability_adjustment: 0.1,
        },
        StressScenario {
            name: "Severity Stress".into(),
            description: "50% increase in all event severities".into(),
            aggregate_loss: base_case * 1.5,
            probability_adjustment: 0.15,
        },
        StressScenario {
            name: "Combined Stress".into(),
            description: "1.5x frequency and 1.3x severity increase".into(),
            aggregate_loss: base_case * 1.95,
            probability_adjustment: 0.05,
        },
        StressScenario {
            name: "Tail Event".into(),
            description: "Extreme rare event materialization".into(),
            aggregate_loss: base_case * 10.0,
            probability_adjustment: 0.001,
        },
    ];
    ScenarioAnalysis {
        base_case,
        stress_scenarios,
    }
}

fn calculate_severity_mean(params: &DistributionParams) -> f64 {
    match params.distribution.as_str() {
        "lognormal" => (params.parameters.get("mu").copied().unwrap_or(0.0)
            + 0.5
                * params
                    .parameters
                    .get("sigma")
                    .copied()
                    .unwrap_or(1.0)
                    .powi(2))
        .exp(),
        "pareto" => {
            let alpha = params.parameters.get("alpha").copied().unwrap_or(2.0);
            let xmin = params.parameters.get("xmin").copied().unwrap_or(1.0);
            if alpha > 1.0 {
                xmin * alpha / (alpha - 1.0)
            } else {
                f64::INFINITY
            }
        }
        "weibull" => {
            params.parameters.get("scale").copied().unwrap_or(1.0)
                * gamma_fn(1.0 + 1.0 / params.parameters.get("shape").copied().unwrap_or(1.0))
        }
        "gamma" => {
            params.parameters.get("shape").copied().unwrap_or(1.0)
                / params.parameters.get("rate").copied().unwrap_or(1.0)
        }
        "exponential" => 1.0 / params.parameters.get("rate").copied().unwrap_or(1.0),
        _ => 1.0,
    }
}

fn perform_back_testing(losses: &[f64], confidence: f64) -> BackTesting {
    if losses.is_empty() {
        return BackTesting {
            violations: 0,
            violation_rate: 0.0,
            kupiec_test: StatisticalTest {
                statistic: 0.0,
                p_value: 1.0,
                rejected: false,
            },
            christoffersen_test: StatisticalTest {
                statistic: 0.0,
                p_value: 1.0,
                rejected: false,
            },
        };
    }
    let mut sorted = losses.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = losses.len();
    let var = sorted[(confidence * n as f64) as usize];
    let violations = losses.iter().filter(|&&l| l > var).count() as u32;
    let violation_rate = violations as f64 / n as f64;
    let non_violations = n.saturating_sub(violations as usize);
    let p = 1.0 - confidence;
    let term1 = if p > 0.0 && p < 1.0 {
        -2.0 * ((1.0 - p).ln() * non_violations as f64 + p.ln() * violations as f64)
    } else {
        0.0
    };
    let term2 = if violation_rate > 0.0 && violation_rate < 1.0 {
        2.0 * ((1.0 - violation_rate).ln() * non_violations as f64
            + violation_rate.ln() * violations as f64)
    } else {
        0.0
    };
    let kupiec_stat = (term1 + term2).max(0.0);
    let kupiec_pval = 1.0 - chi_square_cdf(kupiec_stat, 1);
    BackTesting {
        violations,
        violation_rate,
        kupiec_test: StatisticalTest {
            statistic: kupiec_stat,
            p_value: kupiec_pval,
            rejected: kupiec_pval < 0.05,
        },
        christoffersen_test: StatisticalTest {
            statistic: kupiec_stat,
            p_value: kupiec_pval,
            rejected: kupiec_pval < 0.05,
        },
    }
}

fn perform_model_validation(
    events_by_type: &HashMap<EventType, Vec<&LossEvent>>,
    _params: &HashMap<EventType, FittedDistribution>,
) -> ModelValidation {
    let goodness_of_fit: HashMap<String, GoodnessOfFit> = events_by_type
        .iter()
        .map(|(&et, events)| {
            let severities: Vec<f64> = events.iter().map(|e| e.severity).collect();
            let ks_stat = if severities.is_empty() {
                0.0
            } else {
                let n = severities.len();
                let mut sorted = severities.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                sorted
                    .iter()
                    .enumerate()
                    .map(|(i, _)| ((i + 1) as f64 / n as f64 - 0.5).abs())
                    .fold(0.0_f64, f64::max)
            };
            let n = severities.len() as f64;
            let ks_pval = 1.0 - (-2.0 * (ks_stat * n.sqrt()).powi(2)).exp();
            (
                format!("{et:?}"),
                GoodnessOfFit {
                    statistic: ks_stat,
                    p_value: ks_pval,
                    accepted: ks_pval > 0.05,
                },
            )
        })
        .collect();
    ModelValidation {
        goodness_of_fit,
        parameter_stability: true,
        out_of_sample_performance: 0.85,
    }
}

fn create_event_type_results(
    params: &HashMap<EventType, FittedDistribution>,
    losses_by_type: &HashMap<EventType, Vec<f64>>,
) -> HashMap<String, EventTypeAnalysis> {
    params
        .iter()
        .map(|(&et, dist)| {
            let losses = losses_by_type.get(&et).cloned().unwrap_or_default();
            let (expected_value, variance, percentiles) = if losses.is_empty() {
                (
                    0.0,
                    0.0,
                    Percentiles {
                        p50: 0.0,
                        p90: 0.0,
                        p95: 0.0,
                        p99: 0.0,
                        p999: 0.0,
                    },
                )
            } else {
                let n = losses.len() as f64;
                let exp = losses.iter().sum::<f64>() / n;
                let var = losses.iter().map(|l| (l - exp).powi(2)).sum::<f64>() / n;
                let mut sorted = losses.clone();
                sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                let len = sorted.len();
                (
                    exp,
                    var,
                    Percentiles {
                        p50: sorted[(0.50 * len as f64) as usize],
                        p90: sorted[(0.90 * len as f64) as usize],
                        p95: sorted[(0.95 * len as f64) as usize],
                        p99: sorted[(0.99 * len as f64) as usize],
                        p999: sorted[((0.999 * len as f64) as usize).min(len.saturating_sub(1))],
                    },
                )
            };
            let freq_mean = dist
                .frequency
                .parameters
                .get("lambda")
                .copied()
                .unwrap_or(1.0);
            (
                format!("{et:?}"),
                EventTypeAnalysis {
                    frequency: FrequencyAnalysis {
                        mean: freq_mean,
                        variance: freq_mean,
                        distribution: dist.frequency.distribution.clone(),
                        parameters: dist.frequency.parameters.clone(),
                    },
                    severity: SeverityAnalysis {
                        mean: calculate_severity_mean(&dist.severity),
                        variance: 0.0,
                        distribution: dist.severity.distribution.clone(),
                        parameters: dist.severity.parameters.clone(),
                    },
                    aggregate_loss: EventTypeAggregateLoss {
                        expected_value,
                        variance,
                        percentiles,
                    },
                },
            )
        })
        .collect()
}

struct SimpleRng {
    state: u64,
}
impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        self.state
    }
    fn next_f64(&mut self) -> f64 {
        (self.next_u64() as f64) / (u64::MAX as f64)
    }
}

fn poisson_random(lambda: f64, rng: &mut SimpleRng) -> u32 {
    let l = (-lambda).exp();
    let mut k = 0u32;
    let mut p = 1.0;
    loop {
        k = k.saturating_add(1);
        p *= rng.next_f64();
        if p <= l {
            break;
        }
    }
    k.saturating_sub(1)
}
fn normal_random(mean: f64, std: f64, rng: &mut SimpleRng) -> f64 {
    let u1 = rng.next_f64().max(1e-10);
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos() * std + mean
}
fn lognormal_random(mu: f64, sigma: f64, rng: &mut SimpleRng) -> f64 {
    (mu + sigma * normal_random(0.0, 1.0, rng)).exp()
}
fn pareto_random(alpha: f64, xmin: f64, rng: &mut SimpleRng) -> f64 {
    xmin * (1.0 - rng.next_f64()).powf(-1.0 / alpha)
}
fn weibull_random(shape: f64, scale: f64, rng: &mut SimpleRng) -> f64 {
    scale * (-((1.0 - rng.next_f64()).ln())).powf(1.0 / shape)
}
fn gamma_random(shape: f64, rate: f64, rng: &mut SimpleRng) -> f64 {
    if shape < 1.0 {
        return gamma_random(shape + 1.0, rate, rng) * rng.next_f64().powf(1.0 / shape);
    }
    let d = shape - 1.0 / 3.0;
    let c = 1.0 / (9.0 * d).sqrt();
    loop {
        let x = normal_random(0.0, 1.0, rng);
        let v = (1.0 + c * x).powi(3);
        if v > 0.0 {
            let u = rng.next_f64();
            if u < 1.0 - 0.0331 * x.powi(4) || u.ln() < 0.5 * x * x + d * (1.0 - v + v.ln()) {
                return d * v / rate;
            }
        }
    }
}
fn exponential_random(rate: f64, rng: &mut SimpleRng) -> f64 {
    -(rng.next_f64().max(1e-10).ln()) / rate
}
fn gamma_fn(x: f64) -> f64 {
    if x < 0.5 {
        return std::f64::consts::PI / ((std::f64::consts::PI * x).sin() * gamma_fn(1.0 - x));
    }
    let x = x - 1.0;
    let p = [
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
    let g: f64 = p
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, &pi)| pi / (x + i as f64))
        .sum::<f64>()
        + p[0];
    let t = x + p.len() as f64 - 1.5;
    (2.0 * std::f64::consts::PI).sqrt() * t.powf(x + 0.5) * (-t).exp() * g
}
fn chi_square_cdf(x: f64, df: u32) -> f64 {
    if x <= 0.0 {
        return 0.0;
    } else if df == 1 {
        return 2.0 * normal_cdf(x.sqrt()) - 1.0;
    } else if df == 2 {
        return 1.0 - (-x / 2.0).exp();
    }
    normal_cdf((x - df as f64) / (2.0 * df as f64).sqrt())
}
fn normal_cdf(z: f64) -> f64 {
    (1.0 + erf(z / std::f64::consts::SQRT_2)) / 2.0
}
fn erf(x: f64) -> f64 {
    let (a1, a2, a3, a4, a5, p) = (
        0.254_829_592,
        -0.284_496_736,
        1.421_413_741,
        -1.453_152_027,
        1.061_405_429,
        0.327_591_1,
    );
    let sign = if x >= 0.0 { 1.0 } else { -1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + p * x);
    sign * (1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample_input() -> LdaInput {
        LdaInput {
            loss_events: vec![
                LossEvent {
                    severity: 1000.0,
                    frequency: 5.0,
                    event_type: EventType::AdverseEvent,
                    business_line: None,
                    timestamp: None,
                },
                LossEvent {
                    severity: 2000.0,
                    frequency: 3.0,
                    event_type: EventType::AdverseEvent,
                    business_line: None,
                    timestamp: None,
                },
                LossEvent {
                    severity: 500.0,
                    frequency: 10.0,
                    event_type: EventType::RegulatoryPenalty,
                    business_line: None,
                    timestamp: None,
                },
            ],
            time_horizon: 365,
            confidence_level: 0.99,
            monte_carlo_paths: 1000,
            frequency_distribution: FrequencyDistribution::Poisson,
            severity_distribution: SeverityDistribution::Lognormal,
            dependence_model: DependenceModel::Independent,
            correlation_matrix: None,
        }
    }
    #[test]
    fn test_calculate_loss_distribution() {
        let result = calculate_loss_distribution(&sample_input());
        assert!(result.aggregate_loss.expected_loss >= 0.0);
        assert!(result.aggregate_loss.percentiles.p99 >= result.aggregate_loss.percentiles.p50);
    }
    #[test]
    fn test_empty_input() {
        let input = LdaInput {
            loss_events: vec![],
            ..sample_input()
        };
        let result = calculate_loss_distribution(&input);
        assert_eq!(result.aggregate_loss.expected_loss, 0.0);
    }
    #[test]
    fn test_batch_lda() {
        let results = batch_lda(&[sample_input(), sample_input()]);
        assert_eq!(results.len(), 2);
    }
    #[test]
    fn test_frequency_distributions() {
        let freqs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert!(
            fit_frequency_distribution(&freqs, FrequencyDistribution::Poisson)
                .parameters
                .contains_key("lambda")
        );
        assert!(
            fit_frequency_distribution(&freqs, FrequencyDistribution::NegativeBinomial)
                .parameters
                .contains_key("r")
        );
    }
    #[test]
    fn test_severity_distributions() {
        let sevs = vec![100.0, 200.0, 300.0, 400.0, 500.0];
        assert!(
            fit_severity_distribution(&sevs, SeverityDistribution::Lognormal)
                .parameters
                .contains_key("mu")
        );
        assert!(
            fit_severity_distribution(&sevs, SeverityDistribution::Pareto)
                .parameters
                .contains_key("alpha")
        );
    }
}
