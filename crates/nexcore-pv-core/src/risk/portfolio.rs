//! # Portfolio Optimization for Drug Safety Risk Management
//!
//! Optimizes drug portfolios to minimize overall safety risk while maintaining
//! therapeutic efficacy, considering drug interactions and population exposure.
//!
//! ## Key Concepts
//!
//! | MPT Concept | PV Adaptation |
//! |-------------|---------------|
//! | Expected Return | Therapeutic benefit |
//! | Volatility | Safety risk variance |
//! | Sharpe Ratio | Benefit-risk ratio |
//! | Portfolio VaR | Safety-at-Risk |

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Optimization objective
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum OptimizationObjective {
    MinimizeRisk,
    #[default]
    MaximizeBenefitRiskRatio,
    MaximizeTherapeuticValue,
    MinimizePortfolioVar,
    MaximizeSharpeRatio,
}

/// Optimization method
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum OptimizationMethod {
    #[default]
    MeanVariance,
    BlackLitterman,
    RiskParity,
    MinimumVariance,
    MaximumDiversification,
    ConditionalValueAtRisk,
}

/// Rebalancing frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum RebalancingFrequency {
    Monthly,
    #[default]
    Quarterly,
    Annually,
}

/// Regulatory status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum RegulatoryStatus {
    #[default]
    Approved,
    Investigational,
    Restricted,
    Withdrawn,
}

/// Safety profile for a drug
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyProfile {
    pub expected_adverse_event_rate: f64,
    pub volatility: f64,
    #[serde(default = "default_severity")]
    pub severity_score: f64,
    #[serde(default = "default_freq")]
    pub frequency: f64,
}

fn default_severity() -> f64 {
    5.0
}
fn default_freq() -> f64 {
    1.0
}

/// Efficacy profile for a drug
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficacyProfile {
    pub therapeutic_benefit: f64,
    pub treatment_success_rate: f64,
    #[serde(default = "default_qaly")]
    pub quality_adjusted_life_years: f64,
}

fn default_qaly() -> f64 {
    1.0
}

/// Market data for a drug
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketData {
    pub patient_exposure: f64,
    #[serde(default = "default_share")]
    pub market_share: f64,
    #[serde(default)]
    pub regulatory_status: RegulatoryStatus,
    #[serde(default = "default_cost")]
    pub cost_per_treatment: f64,
}

fn default_share() -> f64 {
    0.1
}
fn default_cost() -> f64 {
    1000.0
}

/// Allocation constraints for a drug
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugConstraints {
    #[serde(default)]
    pub minimum_allocation: f64,
    #[serde(default = "default_max")]
    pub maximum_allocation: f64,
    #[serde(default)]
    pub required_in_portfolio: bool,
}

fn default_max() -> f64 {
    1.0
}

/// Drug in the portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drug {
    pub id: String,
    pub name: String,
    pub therapeutic_class: String,
    pub safety_profile: SafetyProfile,
    pub efficacy_profile: EfficacyProfile,
    pub market_data: MarketData,
    pub constraints: Option<DrugConstraints>,
}

/// Population constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PopulationConstraints {
    pub total_patient_capacity: Option<f64>,
    pub therapeutic_coverage_requirements: Option<HashMap<String, f64>>,
    pub regulatory_limitations: Option<Vec<String>>,
}

fn default_horizon() -> u32 {
    12
}
fn default_confidence() -> f64 {
    0.95
}
fn default_mc_paths() -> u32 {
    10_000
}
fn default_risk_tol() -> f64 {
    0.1
}

/// Input for portfolio optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioInput {
    pub drugs: Vec<Drug>,
    pub correlation_matrix: Option<Vec<Vec<f64>>>,
    #[serde(default)]
    pub optimization_objective: OptimizationObjective,
    #[serde(default = "default_risk_tol")]
    pub risk_tolerance: f64,
    pub population_constraints: Option<PopulationConstraints>,
    #[serde(default)]
    pub optimization_method: OptimizationMethod,
    #[serde(default)]
    pub rebalancing_frequency: RebalancingFrequency,
    #[serde(default = "default_horizon")]
    pub forecast_horizon: u32,
    #[serde(default = "default_confidence")]
    pub confidence_level: f64,
    #[serde(default = "default_mc_paths")]
    pub monte_carlo_paths: u32,
}

/// Individual drug allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugAllocation {
    pub drug_id: String,
    pub drug_name: String,
    pub weight: f64,
    pub patient_allocation: f64,
    pub expected_return: f64,
    pub individual_risk: f64,
    pub contribution_to_portfolio_risk: f64,
}

/// Portfolio-level metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMetrics {
    pub expected_portfolio_return: f64,
    pub portfolio_risk: f64,
    pub sharpe_ratio: f64,
    pub benefit_risk_ratio: f64,
    pub diversification_ratio: f64,
    pub maximum_drawdown: f64,
}

/// Risk metrics for the portfolio
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRiskMetrics {
    pub value_at_risk: f64,
    pub conditional_value_at_risk: f64,
    pub expected_shortfall: f64,
    pub probability_of_adverse_outcome: f64,
}

/// Optimal portfolio result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalPortfolio {
    pub allocations: Vec<DrugAllocation>,
    pub portfolio_metrics: PortfolioMetrics,
    pub risk_metrics: PortfolioRiskMetrics,
}

/// Point on the efficient frontier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficientFrontierPoint {
    pub risk_level: f64,
    pub expected_return: f64,
    pub allocations: HashMap<String, f64>,
    pub sharpe_ratio: f64,
}

/// Risk contribution for a drug
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrugRiskContribution {
    pub drug_id: String,
    pub marginal_contribution: f64,
    pub component_contribution: f64,
    pub percentage_contribution: f64,
    pub diversification_benefit: f64,
}

/// Base case scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseCase {
    pub portfolio_value: f64,
    pub risk_level: f64,
    pub therapeutic_outcome: f64,
}

/// Recovery info for stress scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recovery {
    pub time_to_recover: u32,
    pub probability_of_recovery: f64,
}

/// Stress scenario result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StressScenarioResult {
    pub name: String,
    pub portfolio_value: f64,
    pub risk_level: f64,
    pub therapeutic_outcome: f64,
    pub recovery: Recovery,
}

/// Worst case scenario from Monte Carlo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorstCase {
    pub probability: f64,
    pub portfolio_value: f64,
    pub affected_drugs: Vec<String>,
}

/// Monte Carlo simulation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloResult {
    pub percentiles: HashMap<String, f64>,
    pub probability_distribution: Vec<f64>,
    pub worst_case_scenario: WorstCase,
}

/// Complete scenario analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioAnalysis {
    pub base_case: BaseCase,
    pub stress_scenarios: Vec<StressScenarioResult>,
    pub monte_carlo: MonteCarloResult,
}

/// Coverage for a therapeutic class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassCoverage {
    pub coverage: f64,
    pub risk_level: f64,
    pub patient_benefit: f64,
}

/// Gap in therapeutic coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageGap {
    pub therapeutic_class: String,
    pub unmet_need: f64,
    pub risk_of_no_treatment: f64,
}

/// Redundancy in coverage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageRedundancy {
    pub therapeutic_class: String,
    pub overallocation: f64,
    pub consolidation_opportunity: f64,
}

/// Therapeutic coverage analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TherapeuticCoverage {
    pub by_class: HashMap<String, ClassCoverage>,
    pub gaps: Vec<CoverageGap>,
    pub redundancies: Vec<CoverageRedundancy>,
}

/// Rebalancing trigger
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalancingTrigger {
    pub condition: String,
    pub threshold: f64,
    pub action: String,
}

/// Implementation phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPhase {
    pub phase: String,
    pub duration: u32,
    pub critical_path: Vec<String>,
}

/// Rebalancing strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebalancingStrategy {
    pub triggers: Vec<RebalancingTrigger>,
    pub expected_rebalancing_frequency: u32,
    pub transaction_costs: f64,
    pub implementation_timeline: Vec<ImplementationPhase>,
}

/// Risk tolerance impact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskToleranceImpact {
    pub risk_tolerance: f64,
    pub optimal_allocation: HashMap<String, f64>,
    pub expected_outcome: f64,
}

/// Correlation impact analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrelationImpact {
    pub base_correlation: f64,
    pub stressed_correlation: f64,
    pub allocation_change: HashMap<String, f64>,
}

/// Regulatory impact scenario
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryImpact {
    pub scenario: String,
    pub affected_drugs: Vec<String>,
    pub portfolio_impact: f64,
}

/// Sensitivity analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensitivityAnalysis {
    pub risk_tolerance_impact: Vec<RiskToleranceImpact>,
    pub correlation_impact: CorrelationImpact,
    pub regulatory_impact: Vec<RegulatoryImpact>,
}

/// Implementation phase detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanPhase {
    pub name: String,
    pub duration: u32,
    pub allocations: HashMap<String, f64>,
    pub risk_controls: Vec<String>,
    pub success_metrics: Vec<String>,
}

/// Risk management strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskManagement {
    pub hedging_strategies: Vec<String>,
    pub monitoring_requirements: Vec<String>,
    pub contingency_plans: Vec<String>,
}

/// Implementation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationPlan {
    pub phases: Vec<PlanPhase>,
    pub risk_management: RiskManagement,
}

/// Complete portfolio optimization result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioResult {
    pub optimal_portfolio: OptimalPortfolio,
    pub efficient_frontier: Vec<EfficientFrontierPoint>,
    pub risk_contributions: Vec<DrugRiskContribution>,
    pub scenario_analysis: ScenarioAnalysis,
    pub therapeutic_coverage: TherapeuticCoverage,
    pub rebalancing_strategy: RebalancingStrategy,
    pub sensitivity_analysis: SensitivityAnalysis,
    pub implementation_plan: ImplementationPlan,
}

/// Optimize drug portfolio for safety and efficacy
#[must_use]
pub fn optimize_portfolio(input: &PortfolioInput) -> PortfolioResult {
    if input.drugs.is_empty() {
        return empty_result();
    }
    let expected_returns = calculate_expected_returns(&input.drugs);
    let covariance = calculate_covariance(&input.drugs, &input.correlation_matrix);
    let efficient_frontier = generate_efficient_frontier(&expected_returns, &covariance, input);
    let optimal_portfolio =
        find_optimal_portfolio(&expected_returns, &covariance, input, &efficient_frontier);
    let risk_contributions = calculate_risk_contributions(
        &optimal_portfolio
            .allocations
            .iter()
            .map(|a| a.weight)
            .collect::<Vec<_>>(),
        &covariance,
        &input.drugs,
    );
    let scenario_analysis = perform_scenario_analysis(
        &optimal_portfolio,
        &input.drugs,
        &covariance,
        input.monte_carlo_paths,
    );
    let therapeutic_coverage = analyze_therapeutic_coverage(
        &optimal_portfolio,
        &input.drugs,
        &input.population_constraints,
    );
    let rebalancing_strategy =
        design_rebalancing_strategy(&optimal_portfolio, input.rebalancing_frequency);
    let sensitivity_analysis = conduct_sensitivity_analysis(&expected_returns, &covariance, input);
    let implementation_plan = create_implementation_plan(&optimal_portfolio);
    PortfolioResult {
        optimal_portfolio,
        efficient_frontier,
        risk_contributions,
        scenario_analysis,
        therapeutic_coverage,
        rebalancing_strategy,
        sensitivity_analysis,
        implementation_plan,
    }
}

/// Batch process multiple portfolio inputs
#[must_use]
pub fn batch_optimize(inputs: &[PortfolioInput]) -> Vec<PortfolioResult> {
    inputs.iter().map(optimize_portfolio).collect()
}

fn empty_result() -> PortfolioResult {
    PortfolioResult {
        optimal_portfolio: OptimalPortfolio {
            allocations: vec![],
            portfolio_metrics: PortfolioMetrics {
                expected_portfolio_return: 0.0,
                portfolio_risk: 0.0,
                sharpe_ratio: 0.0,
                benefit_risk_ratio: 0.0,
                diversification_ratio: 1.0,
                maximum_drawdown: 0.0,
            },
            risk_metrics: PortfolioRiskMetrics {
                value_at_risk: 0.0,
                conditional_value_at_risk: 0.0,
                expected_shortfall: 0.0,
                probability_of_adverse_outcome: 0.0,
            },
        },
        efficient_frontier: vec![],
        risk_contributions: vec![],
        scenario_analysis: ScenarioAnalysis {
            base_case: BaseCase {
                portfolio_value: 0.0,
                risk_level: 0.0,
                therapeutic_outcome: 0.0,
            },
            stress_scenarios: vec![],
            monte_carlo: MonteCarloResult {
                percentiles: HashMap::new(),
                probability_distribution: vec![],
                worst_case_scenario: WorstCase {
                    probability: 0.0,
                    portfolio_value: 0.0,
                    affected_drugs: vec![],
                },
            },
        },
        therapeutic_coverage: TherapeuticCoverage {
            by_class: HashMap::new(),
            gaps: vec![],
            redundancies: vec![],
        },
        rebalancing_strategy: RebalancingStrategy {
            triggers: vec![],
            expected_rebalancing_frequency: 0,
            transaction_costs: 0.0,
            implementation_timeline: vec![],
        },
        sensitivity_analysis: SensitivityAnalysis {
            risk_tolerance_impact: vec![],
            correlation_impact: CorrelationImpact {
                base_correlation: 0.0,
                stressed_correlation: 0.0,
                allocation_change: HashMap::new(),
            },
            regulatory_impact: vec![],
        },
        implementation_plan: ImplementationPlan {
            phases: vec![],
            risk_management: RiskManagement {
                hedging_strategies: vec![],
                monitoring_requirements: vec![],
                contingency_plans: vec![],
            },
        },
    }
}

fn calculate_expected_returns(drugs: &[Drug]) -> Vec<f64> {
    drugs
        .iter()
        .map(|d| {
            let benefit = d.efficacy_profile.therapeutic_benefit;
            let safety = 1.0 - d.safety_profile.expected_adverse_event_rate;
            let efficacy = d.efficacy_profile.treatment_success_rate;
            (benefit * safety * efficacy) / 10.0
        })
        .collect()
}

fn calculate_covariance(drugs: &[Drug], corr: &Option<Vec<Vec<f64>>>) -> Vec<Vec<f64>> {
    let n = drugs.len();
    let vols: Vec<f64> = drugs.iter().map(|d| d.safety_profile.volatility).collect();
    let correlation = corr.clone().unwrap_or_else(|| estimate_correlation(drugs));
    (0..n)
        .map(|i| {
            (0..n)
                .map(|j| vols[i] * vols[j] * correlation[i][j])
                .collect()
        })
        .collect()
}

fn estimate_correlation(drugs: &[Drug]) -> Vec<Vec<f64>> {
    let n = drugs.len();
    (0..n)
        .map(|i| {
            (0..n)
                .map(|j| {
                    if i == j {
                        1.0
                    } else {
                        let mut corr = 0.1;
                        if drugs[i].therapeutic_class == drugs[j].therapeutic_class {
                            corr = 0.6;
                        }
                        corr += drugs[i]
                            .market_data
                            .market_share
                            .min(drugs[j].market_data.market_share)
                            * 0.3;
                        if drugs[i].market_data.regulatory_status
                            == drugs[j].market_data.regulatory_status
                        {
                            corr += 0.1;
                        }
                        corr.min(0.9)
                    }
                })
                .collect()
        })
        .collect()
}

fn generate_efficient_frontier(
    returns: &[f64],
    cov: &[Vec<f64>],
    input: &PortfolioInput,
) -> Vec<EfficientFrontierPoint> {
    let min_ret = returns.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_ret = returns.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    (0..=20)
        .map(|i| {
            let target = min_ret + (i as f64 / 20.0) * (max_ret - min_ret);
            let weights = solve_min_variance(returns, cov, target, &input.drugs);
            let ret = weights.iter().zip(returns.iter()).map(|(w, r)| w * r).sum();
            let var = portfolio_variance(&weights, cov);
            let risk = var.sqrt();
            let sharpe = if risk > 0.0 { ret / risk } else { 0.0 };
            let allocs: HashMap<String, f64> = input
                .drugs
                .iter()
                .zip(weights.iter())
                .map(|(d, &w)| (d.id.clone(), w))
                .collect();
            EfficientFrontierPoint {
                risk_level: risk,
                expected_return: ret,
                allocations: allocs,
                sharpe_ratio: sharpe,
            }
        })
        .collect()
}

fn solve_min_variance(returns: &[f64], cov: &[Vec<f64>], _target: f64, drugs: &[Drug]) -> Vec<f64> {
    let n = returns.len();
    let constraints: Vec<(f64, f64)> = drugs
        .iter()
        .map(|d| {
            d.constraints
                .as_ref()
                .map(|c| (c.minimum_allocation, c.maximum_allocation))
                .unwrap_or((0.0, 1.0))
        })
        .collect();
    let mut weights = vec![1.0 / n as f64; n];
    (0..100).for_each(|_| {
        let grad = compute_gradient(&weights, cov, returns);
        let step = 0.01;
        weights = weights
            .iter()
            .zip(grad.iter())
            .map(|(&w, &g)| w - step * g)
            .collect();
        weights = apply_constraints(&weights, &constraints);
    });
    weights
}

fn compute_gradient(weights: &[f64], cov: &[Vec<f64>], _returns: &[f64]) -> Vec<f64> {
    let n = weights.len();
    (0..n)
        .map(|i| 2.0 * (0..n).map(|j| weights[j] * cov[i][j]).sum::<f64>())
        .collect()
}

fn apply_constraints(weights: &[f64], constraints: &[(f64, f64)]) -> Vec<f64> {
    let constrained: Vec<f64> = weights
        .iter()
        .zip(constraints.iter())
        .map(|(&w, &(min, max))| w.clamp(min, max))
        .collect();
    let sum: f64 = constrained.iter().sum();
    if sum > 0.0 {
        constrained.iter().map(|&w| w / sum).collect()
    } else {
        vec![1.0 / weights.len() as f64; weights.len()]
    }
}

fn portfolio_variance(weights: &[f64], cov: &[Vec<f64>]) -> f64 {
    let n = weights.len();
    (0..n)
        .flat_map(|i| (0..n).map(move |j| weights[i] * weights[j] * cov[i][j]))
        .sum()
}

fn find_optimal_portfolio(
    returns: &[f64],
    cov: &[Vec<f64>],
    input: &PortfolioInput,
    frontier: &[EfficientFrontierPoint],
) -> OptimalPortfolio {
    let weights = match input.optimization_objective {
        OptimizationObjective::MinimizeRisk => find_min_variance(cov, &input.drugs),
        OptimizationObjective::MaximizeSharpeRatio => frontier
            .iter()
            .max_by(|a, b| {
                a.sharpe_ratio
                    .partial_cmp(&b.sharpe_ratio)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|p| p.allocations.values().cloned().collect())
            .unwrap_or_else(|| vec![1.0 / returns.len() as f64; returns.len()]),
        OptimizationObjective::MaximizeBenefitRiskRatio => find_max_benefit_risk(returns, cov),
        _ => find_risk_adjusted(returns, cov, input.risk_tolerance),
    };
    let portfolio_return: f64 = weights.iter().zip(returns.iter()).map(|(w, r)| w * r).sum();
    let portfolio_var = portfolio_variance(&weights, cov);
    let portfolio_risk = portfolio_var.sqrt();
    let sharpe = if portfolio_risk > 0.0 {
        portfolio_return / portfolio_risk
    } else {
        0.0
    };
    let benefit_risk = calculate_benefit_risk(&weights, &input.drugs);
    let div_ratio = calculate_diversification(&weights, cov);
    let max_dd = portfolio_risk * 2.33;
    let cap = input
        .population_constraints
        .as_ref()
        .and_then(|p| p.total_patient_capacity)
        .unwrap_or(100_000.0);
    let allocations: Vec<DrugAllocation> = input
        .drugs
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let ind_risk = cov[i][i].sqrt();
            let contrib = risk_contribution(i, &weights, cov);
            DrugAllocation {
                drug_id: d.id.clone(),
                drug_name: d.name.clone(),
                weight: weights[i],
                patient_allocation: weights[i] * cap,
                expected_return: returns[i],
                individual_risk: ind_risk,
                contribution_to_portfolio_risk: contrib,
            }
        })
        .collect();
    let z = normal_inverse(input.confidence_level);
    let var = portfolio_return - z * portfolio_risk;
    let cvar =
        portfolio_return - portfolio_risk * normal_density(z) / (1.0 - input.confidence_level);
    OptimalPortfolio {
        allocations,
        portfolio_metrics: PortfolioMetrics {
            expected_portfolio_return: portfolio_return,
            portfolio_risk,
            sharpe_ratio: sharpe,
            benefit_risk_ratio: benefit_risk,
            diversification_ratio: div_ratio,
            maximum_drawdown: max_dd,
        },
        risk_metrics: PortfolioRiskMetrics {
            value_at_risk: var,
            conditional_value_at_risk: cvar,
            expected_shortfall: cvar,
            probability_of_adverse_outcome: 1.0 - input.confidence_level,
        },
    }
}

fn find_min_variance(cov: &[Vec<f64>], _drugs: &[Drug]) -> Vec<f64> {
    vec![1.0 / cov.len() as f64; cov.len()]
}

fn find_max_benefit_risk(returns: &[f64], cov: &[Vec<f64>]) -> Vec<f64> {
    let ratios: Vec<f64> = returns
        .iter()
        .enumerate()
        .map(|(i, &r)| {
            let risk = cov[i][i].sqrt().max(0.01);
            r / risk
        })
        .collect();
    let sum: f64 = ratios.iter().map(|r| r.max(0.0)).sum::<f64>().max(0.01);
    ratios.iter().map(|r| r.max(0.0) / sum).collect()
}

fn find_risk_adjusted(returns: &[f64], cov: &[Vec<f64>], risk_tol: f64) -> Vec<f64> {
    let weights: Vec<f64> = returns
        .iter()
        .enumerate()
        .map(|(i, &r)| {
            let risk = cov[i][i].sqrt();
            (r - risk_tol * risk).max(0.0)
        })
        .collect();
    let sum = weights.iter().sum::<f64>();
    if sum > 0.0 {
        weights.iter().map(|w| w / sum).collect()
    } else {
        vec![1.0 / returns.len() as f64; returns.len()]
    }
}

fn calculate_benefit_risk(weights: &[f64], drugs: &[Drug]) -> f64 {
    let (benefit, risk) = weights
        .iter()
        .zip(drugs.iter())
        .fold((0.0, 0.0), |(b, r), (&w, d)| {
            (
                b + w * d.efficacy_profile.therapeutic_benefit,
                r + w * d.safety_profile.expected_adverse_event_rate,
            )
        });
    if risk > 0.0 { benefit / risk } else { 0.0 }
}

fn calculate_diversification(weights: &[f64], cov: &[Vec<f64>]) -> f64 {
    let n = weights.len();
    let weighted_vol: f64 = (0..n).map(|i| weights[i] * cov[i][i].sqrt()).sum();
    let portfolio_vol = portfolio_variance(weights, cov).sqrt();
    if portfolio_vol > 0.0 {
        weighted_vol / portfolio_vol
    } else {
        1.0
    }
}

fn risk_contribution(i: usize, weights: &[f64], cov: &[Vec<f64>]) -> f64 {
    let n = weights.len();
    let contrib: f64 = (0..n).map(|j| weights[j] * cov[i][j]).sum();
    let portfolio_var = portfolio_variance(weights, cov);
    if portfolio_var > 0.0 {
        weights[i] * contrib / portfolio_var.sqrt()
    } else {
        0.0
    }
}

fn calculate_risk_contributions(
    weights: &[f64],
    cov: &[Vec<f64>],
    drugs: &[Drug],
) -> Vec<DrugRiskContribution> {
    let n = weights.len();
    let portfolio_risk = portfolio_variance(weights, cov).sqrt();
    drugs
        .iter()
        .enumerate()
        .map(|(i, d)| {
            let marginal: f64 =
                (0..n).map(|j| weights[j] * cov[i][j]).sum::<f64>() / portfolio_risk.max(0.01);
            let component = weights[i] * marginal;
            let pct = if portfolio_risk > 0.0 {
                component / portfolio_risk * 100.0
            } else {
                0.0
            };
            let standalone = cov[i][i].sqrt();
            DrugRiskContribution {
                drug_id: d.id.clone(),
                marginal_contribution: marginal,
                component_contribution: component,
                percentage_contribution: pct,
                diversification_benefit: standalone - marginal,
            }
        })
        .collect()
}

fn perform_scenario_analysis(
    portfolio: &OptimalPortfolio,
    drugs: &[Drug],
    cov: &[Vec<f64>],
    paths: u32,
) -> ScenarioAnalysis {
    let weights: Vec<f64> = portfolio.allocations.iter().map(|a| a.weight).collect();
    let base = BaseCase {
        portfolio_value: 1.0,
        risk_level: portfolio.portfolio_metrics.portfolio_risk,
        therapeutic_outcome: portfolio.portfolio_metrics.expected_portfolio_return,
    };
    let stress = vec![
        StressScenarioResult {
            name: "Regulatory Crackdown".into(),
            portfolio_value: simulate_regulatory_stress(&weights, drugs),
            risk_level: base.risk_level * 1.5,
            therapeutic_outcome: base.therapeutic_outcome * 0.8,
            recovery: Recovery {
                time_to_recover: 24,
                probability_of_recovery: 0.7,
            },
        },
        StressScenarioResult {
            name: "Safety Signal".into(),
            portfolio_value: simulate_safety_stress(&weights, drugs),
            risk_level: base.risk_level * 2.0,
            therapeutic_outcome: base.therapeutic_outcome * 0.6,
            recovery: Recovery {
                time_to_recover: 36,
                probability_of_recovery: 0.5,
            },
        },
        StressScenarioResult {
            name: "Competition".into(),
            portfolio_value: simulate_competition_stress(&weights, drugs),
            risk_level: base.risk_level * 1.2,
            therapeutic_outcome: base.therapeutic_outcome * 0.9,
            recovery: Recovery {
                time_to_recover: 12,
                probability_of_recovery: 0.9,
            },
        },
    ];
    let mc = run_monte_carlo(&weights, cov, drugs, paths);
    ScenarioAnalysis {
        base_case: base,
        stress_scenarios: stress,
        monte_carlo: mc,
    }
}

fn simulate_regulatory_stress(weights: &[f64], drugs: &[Drug]) -> f64 {
    1.0 - weights
        .iter()
        .zip(drugs.iter())
        .map(|(&w, d)| match d.market_data.regulatory_status {
            RegulatoryStatus::Investigational => w * 0.3,
            _ if d.safety_profile.severity_score > 7.0 => w * 0.2,
            _ => 0.0,
        })
        .sum::<f64>()
}

fn simulate_safety_stress(weights: &[f64], drugs: &[Drug]) -> f64 {
    1.0 - weights
        .iter()
        .zip(drugs.iter())
        .map(|(&w, d)| {
            w * d.safety_profile.expected_adverse_event_rate * d.safety_profile.severity_score
                / 10.0
                * 0.5
        })
        .sum::<f64>()
}

fn simulate_competition_stress(weights: &[f64], drugs: &[Drug]) -> f64 {
    1.0 - weights
        .iter()
        .zip(drugs.iter())
        .map(|(&w, d)| w * (1.0 - d.market_data.market_share) * 0.1)
        .sum::<f64>()
        .min(0.5)
}

fn run_monte_carlo(
    weights: &[f64],
    cov: &[Vec<f64>],
    drugs: &[Drug],
    paths: u32,
) -> MonteCarloResult {
    let mut rng = SimpleRng::new(42);
    let values: Vec<f64> = (0..paths)
        .map(|_| {
            let returns = generate_correlated_returns(cov, &mut rng);
            1.0 + weights
                .iter()
                .zip(returns.iter())
                .map(|(&w, &r)| w * r)
                .sum::<f64>()
        })
        .collect();
    let mut sorted = values.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = sorted.len();
    let percentiles: HashMap<String, f64> = [
        ("p5", 0.05),
        ("p10", 0.10),
        ("p25", 0.25),
        ("p50", 0.50),
        ("p75", 0.75),
        ("p90", 0.90),
        ("p95", 0.95),
    ]
    .iter()
    .map(|(k, p)| (k.to_string(), sorted[(p * n as f64) as usize]))
    .collect();
    MonteCarloResult {
        percentiles,
        probability_distribution: values,
        worst_case_scenario: WorstCase {
            probability: 0.01,
            portfolio_value: sorted[(0.01 * n as f64) as usize],
            affected_drugs: drugs.iter().take(2).map(|d| d.id.clone()).collect(),
        },
    }
}

fn generate_correlated_returns(cov: &[Vec<f64>], rng: &mut SimpleRng) -> Vec<f64> {
    let n = cov.len();
    let z: Vec<f64> = (0..n).map(|_| normal_random(0.0, 1.0, rng)).collect();
    let l = cholesky(cov);
    (0..n)
        .map(|i| (0..=i).map(|j| l[i][j] * z[j]).sum())
        .collect()
}

fn cholesky(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = matrix.len();
    let mut l: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
    (0..n).for_each(|i| {
        (0..=i).for_each(|j| {
            if i == j {
                let sum: f64 = (0..j).map(|k| l[j][k] * l[j][k]).sum();
                l[j][j] = (matrix[j][j] - sum).max(1e-10).sqrt();
            } else {
                let sum: f64 = (0..j).map(|k| l[i][k] * l[j][k]).sum();
                l[i][j] = (matrix[i][j] - sum) / l[j][j].max(1e-10);
            }
        });
    });
    l
}

fn analyze_therapeutic_coverage(
    portfolio: &OptimalPortfolio,
    drugs: &[Drug],
    constraints: &Option<PopulationConstraints>,
) -> TherapeuticCoverage {
    let mut by_class: HashMap<String, ClassCoverage> = HashMap::new();
    drugs
        .iter()
        .zip(portfolio.allocations.iter())
        .for_each(|(d, a)| {
            let e = by_class
                .entry(d.therapeutic_class.clone())
                .or_insert(ClassCoverage {
                    coverage: 0.0,
                    risk_level: 0.0,
                    patient_benefit: 0.0,
                });
            e.coverage += a.weight;
            e.risk_level = (e.risk_level + d.safety_profile.expected_adverse_event_rate) / 2.0;
            e.patient_benefit = (e.patient_benefit + d.efficacy_profile.therapeutic_benefit) / 2.0;
        });
    let reqs = constraints
        .as_ref()
        .and_then(|c| c.therapeutic_coverage_requirements.as_ref());
    let (gaps, redundancies) = if let Some(req) = reqs {
        let g: Vec<CoverageGap> = req
            .iter()
            .filter_map(|(cls, &required)| {
                let actual = by_class.get(cls).map(|c| c.coverage).unwrap_or(0.0);
                if actual < required {
                    Some(CoverageGap {
                        therapeutic_class: cls.clone(),
                        unmet_need: required - actual,
                        risk_of_no_treatment: 0.5,
                    })
                } else {
                    None
                }
            })
            .collect();
        let r: Vec<CoverageRedundancy> = req
            .iter()
            .filter_map(|(cls, &required)| {
                let actual = by_class.get(cls).map(|c| c.coverage).unwrap_or(0.0);
                if actual > required * 1.2 {
                    Some(CoverageRedundancy {
                        therapeutic_class: cls.clone(),
                        overallocation: actual - required,
                        consolidation_opportunity: 0.3,
                    })
                } else {
                    None
                }
            })
            .collect();
        (g, r)
    } else {
        (vec![], vec![])
    };
    TherapeuticCoverage {
        by_class,
        gaps,
        redundancies,
    }
}

fn design_rebalancing_strategy(
    _portfolio: &OptimalPortfolio,
    frequency: RebalancingFrequency,
) -> RebalancingStrategy {
    let triggers = vec![
        RebalancingTrigger {
            condition: "Allocation drift > 5%".into(),
            threshold: 0.05,
            action: "Rebalance to target".into(),
        },
        RebalancingTrigger {
            condition: "New safety signal".into(),
            threshold: 0.1,
            action: "Reduce allocation".into(),
        },
        RebalancingTrigger {
            condition: "Portfolio risk > tolerance".into(),
            threshold: 0.15,
            action: "Reduce high-risk".into(),
        },
    ];
    let freq = match frequency {
        RebalancingFrequency::Monthly => 12,
        RebalancingFrequency::Quarterly => 4,
        RebalancingFrequency::Annually => 1,
    };
    let timeline = vec![
        ImplementationPhase {
            phase: "Analysis".into(),
            duration: 7,
            critical_path: vec!["risk_assessment".into(), "approval".into()],
        },
        ImplementationPhase {
            phase: "Execution".into(),
            duration: 14,
            critical_path: vec!["orders".into(), "settlement".into()],
        },
    ];
    RebalancingStrategy {
        triggers,
        expected_rebalancing_frequency: freq,
        transaction_costs: 0.02,
        implementation_timeline: timeline,
    }
}

fn conduct_sensitivity_analysis(
    returns: &[f64],
    cov: &[Vec<f64>],
    input: &PortfolioInput,
) -> SensitivityAnalysis {
    let risk_impact: Vec<RiskToleranceImpact> = [0.05, 0.1, 0.15, 0.2, 0.25]
        .iter()
        .map(|&tol| {
            let weights = find_risk_adjusted(returns, cov, tol);
            let allocs: HashMap<String, f64> = input
                .drugs
                .iter()
                .zip(weights.iter())
                .map(|(d, &w)| (d.id.clone(), w))
                .collect();
            let outcome: f64 = weights.iter().zip(returns.iter()).map(|(w, r)| w * r).sum();
            RiskToleranceImpact {
                risk_tolerance: tol,
                optimal_allocation: allocs,
                expected_outcome: outcome,
            }
        })
        .collect();
    let reg_impact = vec![
        RegulatoryImpact {
            scenario: "FDA Warning".into(),
            affected_drugs: input
                .drugs
                .first()
                .map(|d| vec![d.id.clone()])
                .unwrap_or_default(),
            portfolio_impact: -0.15,
        },
        RegulatoryImpact {
            scenario: "Withdrawal".into(),
            affected_drugs: input
                .drugs
                .first()
                .map(|d| vec![d.id.clone()])
                .unwrap_or_default(),
            portfolio_impact: -0.30,
        },
    ];
    SensitivityAnalysis {
        risk_tolerance_impact: risk_impact,
        correlation_impact: CorrelationImpact {
            base_correlation: 0.3,
            stressed_correlation: 0.8,
            allocation_change: HashMap::new(),
        },
        regulatory_impact: reg_impact,
    }
}

fn create_implementation_plan(portfolio: &OptimalPortfolio) -> ImplementationPlan {
    let allocs: HashMap<String, f64> = portfolio
        .allocations
        .iter()
        .map(|a| (a.drug_id.clone(), a.weight))
        .collect();
    let phases = vec![
        PlanPhase {
            name: "Initial Construction".into(),
            duration: 90,
            allocations: allocs.clone(),
            risk_controls: vec!["Position limits".into(), "Stop-loss".into()],
            success_metrics: vec!["Targets achieved".into(), "Compliance".into()],
        },
        PlanPhase {
            name: "Ongoing Management".into(),
            duration: 365,
            allocations: allocs,
            risk_controls: vec!["Rebalancing".into(), "Monitoring".into()],
            success_metrics: vec!["Performance".into(), "Risk management".into()],
        },
    ];
    let risk_mgmt = RiskManagement {
        hedging_strategies: vec!["Diversification".into(), "Regulatory insurance".into()],
        monitoring_requirements: vec!["Daily risk".into(), "Weekly review".into()],
        contingency_plans: vec!["Emergency rebalance".into(), "Crisis comms".into()],
    };
    ImplementationPlan {
        phases,
        risk_management: risk_mgmt,
    }
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
fn normal_random(mean: f64, std: f64, rng: &mut SimpleRng) -> f64 {
    let u1 = rng.next_f64().max(1e-10);
    let u2 = rng.next_f64();
    (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos() * std + mean
}
fn normal_inverse(p: f64) -> f64 {
    if p <= 0.0 || p >= 1.0 {
        return 0.0;
    }
    let y = p - 0.5;
    if y.abs() < 0.42 {
        let r = y * y;
        y * (((-25.44106049637 * r + 41.39119773534) * r - 18.61500062529) * r + 2.50662823884)
            / ((((3.13082909833 * r - 21.06224101826) * r + 23.08336743743) * r - 8.47351093090)
                * r
                + 1.0)
    } else {
        let r = if p < 0.5 { p } else { 1.0 - p };
        let s = (-r.ln()).sqrt();
        let t = 2.30753 + s * 0.27061;
        if p < 0.5 { -t } else { t }
    }
}
fn normal_density(x: f64) -> f64 {
    (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn sample_drug() -> Drug {
        Drug {
            id: "D1".into(),
            name: "DrugA".into(),
            therapeutic_class: "ClassA".into(),
            safety_profile: SafetyProfile {
                expected_adverse_event_rate: 0.1,
                volatility: 0.2,
                severity_score: 5.0,
                frequency: 1.0,
            },
            efficacy_profile: EfficacyProfile {
                therapeutic_benefit: 8.0,
                treatment_success_rate: 0.9,
                quality_adjusted_life_years: 1.0,
            },
            market_data: MarketData {
                patient_exposure: 10000.0,
                market_share: 0.1,
                regulatory_status: RegulatoryStatus::Approved,
                cost_per_treatment: 1000.0,
            },
            constraints: None,
        }
    }
    fn sample_input() -> PortfolioInput {
        PortfolioInput {
            drugs: vec![
                sample_drug(),
                Drug {
                    id: "D2".into(),
                    name: "DrugB".into(),
                    therapeutic_class: "ClassB".into(),
                    ..sample_drug()
                },
            ],
            correlation_matrix: None,
            optimization_objective: OptimizationObjective::MaximizeBenefitRiskRatio,
            risk_tolerance: 0.1,
            population_constraints: None,
            optimization_method: OptimizationMethod::MeanVariance,
            rebalancing_frequency: RebalancingFrequency::Quarterly,
            forecast_horizon: 12,
            confidence_level: 0.95,
            monte_carlo_paths: 1000,
        }
    }
    #[test]
    fn test_optimize_portfolio() {
        let result = optimize_portfolio(&sample_input());
        assert!(!result.optimal_portfolio.allocations.is_empty());
        assert!(result.optimal_portfolio.portfolio_metrics.sharpe_ratio >= 0.0);
    }
    #[test]
    fn test_empty_input() {
        let input = PortfolioInput {
            drugs: vec![],
            ..sample_input()
        };
        let result = optimize_portfolio(&input);
        assert!(result.optimal_portfolio.allocations.is_empty());
    }
    #[test]
    fn test_batch_optimize() {
        let results = batch_optimize(&[sample_input(), sample_input()]);
        assert_eq!(results.len(), 2);
    }
    #[test]
    fn test_efficient_frontier() {
        let result = optimize_portfolio(&sample_input());
        assert!(!result.efficient_frontier.is_empty());
    }
}
