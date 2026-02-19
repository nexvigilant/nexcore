//! FDA AI Credibility Assessment Parameters
//! Tier: T3 (Domain-specific)
use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parameters for defining Context of Use (Step 2).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaDefineCouParams {
    /// Decision question
    pub question: String,
    /// Input domain
    pub input_domain: String,
    /// Output domain
    pub output_domain: String,
    /// Purpose description
    pub purpose_description: String,
    /// Evidence integration approach
    pub integration: String,
    /// Confirmatory sources
    #[serde(default)]
    pub confirmatory_sources: Option<Vec<String>>,
    /// Regulatory context
    pub regulatory_context: String,
}

/// Parameters for assessing risk (Step 3).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaAssessRiskParams {
    /// Model influence level
    pub influence: String,
    /// Decision consequence level
    pub consequence: String,
}

/// Parameters for creating a credibility plan (Step 4).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaCreatePlanParams {
    /// Decision question
    pub question: String,
    /// Input domain
    pub input_domain: String,
    /// Output domain
    pub output_domain: String,
    /// Model influence level
    pub influence: String,
    /// Decision consequence level
    pub consequence: String,
    /// Regulatory context
    pub regulatory_context: String,
}

/// Parameters for validating evidence (Step 5-6).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaValidateEvidenceParams {
    /// Evidence type
    pub evidence_type: String,
    /// Evidence quality
    pub quality: String,
    /// Description
    pub description: String,
    /// Relevant to COU
    pub relevant: bool,
    /// Reliable methodology
    pub reliable: bool,
    /// Representative data
    pub representative: bool,
}

/// Parameters for deciding adequacy (Step 7).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaDecideAdequacyParams {
    /// Risk level
    pub risk_level: String,
    /// Count of high-quality evidence items
    pub high_quality_evidence_count: usize,
    /// Whether fit-for-use check passed
    pub fit_for_use_passed: bool,
    /// Whether critical drift was detected
    pub critical_drift_detected: bool,
}

/// Parameters for calculating credibility score.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaCalculateScoreParams {
    /// Evidence quality score (0.0-1.0)
    pub evidence_quality: f64,
    /// Fit for use score (0.0-1.0)
    pub fit_for_use: f64,
    /// Risk mitigation score (0.0-1.0)
    pub risk_mitigation: f64,
    /// Documentation score (0.0-1.0)
    pub documentation: f64,
}

/// Parameters for metrics summary.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaMetricsSummaryParams {
    /// Assessments started
    pub started: usize,
    /// Assessments completed
    pub completed: usize,
    /// Assessments approved
    pub approved: usize,
    /// Assessments rejected
    pub rejected: usize,
    /// Assessments in revision
    pub revision: usize,
    /// Drift alerts
    pub drift_alerts: usize,
}

/// Parameters for evidence distribution analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaEvidenceDistributionParams {
    /// Evidence items as (type, quality) pairs
    pub evidence_items: Vec<(String, String)>,
}

/// Parameters for risk distribution analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaRiskDistributionParams {
    /// Risk levels
    pub risk_levels: Vec<String>,
}

/// Parameters for drift trend analysis.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FdaDriftTrendParams {
    /// Drift measurements as (timestamp, value, source, description) tuples
    pub measurements: Vec<(u64, f64, String, String)>,
    /// Trend threshold
    pub trend_threshold: f64,
}
