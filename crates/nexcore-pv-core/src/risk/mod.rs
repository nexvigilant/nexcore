//! # Risk Analytics Module
//!
//! Comprehensive suite of financial risk management algorithms
//! adapted for pharmacovigilance and patient safety applications.
//!
//! ## Algorithms
//!
//! | Algorithm | Purpose | Status |
//! |-----------|---------|--------|
//! | Safety-at-Risk (SaR) | VaR for safety | ✅ |
//! | Expected Shortfall | CVaR/tail risk | ✅ |
//! | Monte Carlo | Stochastic simulation | ✅ |
//! | Credit Scoring | Patient risk scoring | ✅ |
//! | GARCH | Volatility modeling | ✅ |
//! | Extreme Value Theory | Black swan events | ✅ |
//! | Loss Distribution | Operational risk | ✅ |
//! | Copula Models | Dependency modeling | ✅ |
//! | Network Analysis | Systemic risk | ✅ |
//! | Portfolio Optimization | Drug portfolio | ✅ |

pub mod copula;
pub mod credit;
pub mod evt;
pub mod expected_shortfall;
pub mod garch;
pub mod lda;
pub mod monte_carlo;
pub mod network;
pub mod portfolio;
pub mod sar;

// Re-exports - P1
pub use expected_shortfall::{
    ConfidenceInterval, DistributionType, EsBreakdown, EsInput, EsMethod, EsResult, EsValidation,
    TailRiskMetrics, calculate_expected_shortfall, calculate_portfolio_es,
};
pub use monte_carlo::{
    AdverseEventRates, McDistribution, McInput, McResult, McRiskMetrics, McScenarios,
    McSignalDetection, McSummary, MonteCarloEngine, PatientExposure, SignalThresholds, TimeHorizon,
    run_monte_carlo,
};
pub use sar::{RiskComponents, SarBacktest, SarInput, SarMethod, SarResult, calculate_sar};

// Re-exports - P2 Credit Scoring
pub use credit::{
    Benchmarks, ComponentScore, CreditComponents, CreditInput, CreditResult, DataSourceMix,
    ExperienceProfile, ExposureProfile, Impact, PregnancyCategory, RecentActivity,
    RiskFactorAnalysis, RiskFactors, RiskTier, SafetyHistory, ScoreChangePoint, ScoreHistory,
    ScoreTrend, batch_safety_credit_scoring, calculate_safety_credit_score,
};

// Re-exports - P2 GARCH
pub use garch::{
    ClusterSeverity, ClusteringAnalysis, ConfidenceInterval as GarchCi, GarchDiagnostics,
    GarchInput, GarchModel, GarchParams, GarchResult, ResidualDiagnostics,
    TestResult as GarchTestResult, VolatilityCluster, VolatilityForecast, estimate_garch,
};

// Re-exports - P2 EVT
pub use evt::{
    AggregateRisk, BlackSwanIndicators, DomainOfAttraction, EvtDiagnostics, EvtForecast, EvtInput,
    EvtMethod, EvtParams, EvtResult, EvtValidation, ExtremeRiskRating, MeanExcessPoint,
    NextExtremeForecast, ReturnLevel, RiskAssessment, StatTest,
    TailRiskMetrics as EvtTailRiskMetrics, ThresholdMethod, WorstCaseScenario,
    analyze_extreme_values,
};

// Re-exports - P3 LDA
pub use lda::{
    AggregateLoss, BackTesting, DependenceModel, DistributionParams, DiversificationBenefit,
    EventType, EventTypeAggregateLoss, EventTypeAnalysis, FittedDistribution, FrequencyAnalysis,
    FrequencyDistribution, GoodnessOfFit, LdaInput, LdaResult, LossEvent, ModelValidation,
    Percentiles, RiskContribution, ScenarioAnalysis, SeverityAnalysis, SeverityDistribution,
    StatisticalTest, StressScenario, batch_lda, calculate_loss_distribution,
};

// Re-exports - P3 Copula
pub use copula::{
    AggregateRisk as CopulaAggregateRisk, BacktestingResults, ConditionalRisk, CopulaInput,
    CopulaParameters, CopulaResult, CopulaType, DependenceAnalysis, DependenceMetrics,
    DependenceStructure, GoodnessOfFit as CopulaGoodnessOfFit, MarginalDistribution,
    MarginalImpact, MarginalResult, ModelValidation as CopulaModelValidation, PairwiseDependency,
    RankCorrelation, RiskAggregationMethod, RiskContributionFactor, RiskFactor, RiskPercentiles,
    StressScenarioResult, StressTestScenario, StressTesting, SystemicRiskIndicators,
    TailDependence, TailDependenceCoeff, batch_copula, estimate_copula_model,
};

// Re-exports - P3 Network
pub use network::{
    AnalysisType, CentralityMetrics, Community, CommunityStructure, ContagionParameters,
    ContagionSimulation, CriticalNode, CriticalPath, Criticality, Edge, EdgeDirection, EdgeType,
    InterventionType, Neighborhoods, NetworkInput, NetworkMetrics, NetworkRedesign,
    NetworkResilience, NetworkResult, Node, NodeMetrics, NodeRiskMetrics, NodeType, PeakRisk,
    RecoveryPattern, RiskMitigation, StressScenario as NetworkStressScenario, StressTestResult,
    StressTesting as NetworkStressTesting, SystemicRiskAnalysis, TargetedIntervention,
    TimelineEntry, VulnerabilityAssessment, WorstCaseScenario as NetworkWorstCase, analyze_network,
    batch_network_analysis,
};

// Re-exports - P3 Portfolio
pub use portfolio::{
    BaseCase, ClassCoverage, CorrelationImpact, CoverageGap, CoverageRedundancy, Drug,
    DrugAllocation, DrugConstraints, DrugRiskContribution, EfficacyProfile, EfficientFrontierPoint,
    ImplementationPhase, ImplementationPlan, MarketData, MonteCarloResult, OptimalPortfolio,
    OptimizationMethod, OptimizationObjective, PlanPhase, PopulationConstraints, PortfolioInput,
    PortfolioMetrics, PortfolioResult, PortfolioRiskMetrics, RebalancingFrequency,
    RebalancingStrategy, RebalancingTrigger, Recovery, RegulatoryImpact, RegulatoryStatus,
    RiskManagement, RiskToleranceImpact, SafetyProfile,
    ScenarioAnalysis as PortfolioScenarioAnalysis, SensitivityAnalysis,
    StressScenarioResult as PortfolioStressScenarioResult, TherapeuticCoverage,
    WorstCase as PortfolioWorstCase, batch_optimize, optimize_portfolio,
};
