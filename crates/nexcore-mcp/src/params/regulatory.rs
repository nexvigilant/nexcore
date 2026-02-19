//! Regulatory & Compliance Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Commandment verification, PV pipeline orchestration, and regulatory primitives.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for verifying a single commandment
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CommandmentVerifyParams {
    /// Commandment name: TruthInGrounding, Falsifiability, Consensus, etc.
    pub commandment: String,
    /// Whether proof was provided for this commandment
    #[serde(default)]
    pub proof_provided: bool,
}

/// Parameters for getting commandment info
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CommandmentInfoParams {
    /// Commandment name or number (1-15)
    pub commandment: String,
}

/// Parameters for listing commandments by category
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CommandmentListParams {
    /// Category filter: Epistemic, Authority, Observability, Process, Integrity, or "all"
    #[serde(default = "default_commandment_category")]
    pub category: String,
}

fn default_commandment_category() -> String {
    "all".to_string()
}

/// Parameters for full commandment audit
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CommandmentAuditParams {
    /// Proof of grounding provided
    #[serde(default)]
    pub grounding_proof: bool,
    /// Owner identified for action
    #[serde(default)]
    pub owner_identified: bool,
    /// Audit trail exists
    #[serde(default)]
    pub audit_trail_exists: bool,
    /// Sensing is active
    #[serde(default)]
    pub sensing_active: bool,
    /// Correction mechanism exists
    #[serde(default)]
    pub correction_mechanism: bool,
    /// State is public
    #[serde(default)]
    pub state_public: bool,
    /// Persistence is guaranteed
    #[serde(default)]
    pub persistence_guaranteed: bool,
    /// Market is fair (no asymmetry abuse)
    #[serde(default)]
    pub fair_market: bool,
    /// Human override is available
    #[serde(default)]
    pub human_override_available: bool,
    /// Codex compliant
    #[serde(default)]
    pub codex_compliant: bool,
    /// Falsifiability test exists
    #[serde(default)]
    pub has_falsifiability_test: bool,
    /// Provenance chain exists
    #[serde(default)]
    pub has_provenance: bool,
    /// Oracle count - agreeing oracles for consensus
    #[serde(default)]
    pub oracle_agreeing: u32,
    /// Oracle count - total oracles for consensus
    #[serde(default)]
    pub oracle_total: u32,
    /// Precedent hash exists
    #[serde(default)]
    pub has_precedent: bool,
    /// Code compiles successfully
    #[serde(default)]
    pub compiled: bool,
    /// Code passes type-checking
    #[serde(default)]
    pub type_checked: bool,
    /// Effect annotations verified
    #[serde(default)]
    pub effects_verified: bool,
}

/// Parameters for end-to-end pharmacovigilance pipeline
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct PvPipelineParams {
    /// Drug name (generic or brand)
    pub drug_name: String,
    /// Adverse event (MedDRA Preferred Term)
    pub event_name: String,
    /// Signal detection threshold preset: "evans" (default), "strict", or "sensitive"
    #[serde(default = "default_threshold_preset")]
    pub threshold_preset: String,
}

fn default_threshold_preset() -> String {
    "evans".to_string()
}

/// Parameters for extracting regulatory primitives from FDA/ICH/CIOMS
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegulatoryExtractParams {
    /// Regulatory source identifier (e.g., "fda", "ich", "cioms", "21 CFR 314.80")
    pub source: String,
    /// Raw content to extract from (optional)
    #[serde(default)]
    pub content: String,
    /// Maximum tier to include: 1=T1 only, 2=T1+T2-P, 3=all (default: 3)
    #[serde(default = "default_max_tier")]
    pub max_tier: u8,
}

fn default_max_tier() -> u8 {
    3
}

/// Parameters for auditing FDA vs CIOMS/ICH consistency
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegulatoryAuditParams {
    /// FDA term to audit
    pub fda_term: String,
    /// Corresponding CIOMS/ICH term to compare
    pub cioms_term: String,
    /// Include component-level audit (default: true)
    #[serde(default = "default_include_components")]
    pub include_components: Option<bool>,
}

fn default_include_components() -> Option<bool> {
    Some(true)
}

/// Parameters for cross-domain primitive comparison
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct RegulatoryCompareParams {
    /// First domain
    pub domain1: String,
    /// Second domain
    pub domain2: String,
    /// Minimum transfer confidence threshold (default: 0.5)
    #[serde(default = "default_confidence_threshold")]
    pub confidence_threshold: f64,
}

fn default_confidence_threshold() -> f64 {
    0.5
}

/// Parameters for FDA effectiveness endpoint assessment
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EffectivenessAssessParams {
    /// Approval pathway
    pub pathway: String,
    /// Endpoint tier: "primary", "secondary", "exploratory"
    pub endpoint_tier: String,
    /// Endpoint type
    pub endpoint_type: String,
    /// Endpoint name
    pub endpoint_name: String,
    /// P-value from statistical analysis (optional)
    #[serde(default)]
    pub p_value: Option<f64>,
    /// Whether endpoint met success criterion (optional)
    #[serde(default)]
    pub success: Option<bool>,
    /// Alpha level (default: 0.05)
    #[serde(default = "default_alpha")]
    pub alpha: f64,
    /// Number of comparisons for multiplicity adjustment (default: 1)
    #[serde(default = "default_n_comparisons")]
    pub n_comparisons: usize,
    /// Multiplicity method: "bonferroni", "holm", "hochberg", "fixed_sequence"
    #[serde(default = "default_multiplicity_method")]
    pub multiplicity_method: String,
}

fn default_alpha() -> f64 {
    0.05
}

fn default_n_comparisons() -> usize {
    1
}

fn default_multiplicity_method() -> String {
    "bonferroni".to_string()
}

/// Parameters for QBRI benefit-risk computation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct QbriComputeParams {
    /// Benefit effect size
    pub benefit_effect: f64,
    /// Benefit p-value
    pub benefit_pvalue: f64,
    /// Unmet medical need [1-10]
    pub unmet_need: f64,
    /// Risk signal strength
    pub risk_signal: f64,
    /// Risk probability (causal likelihood 0-1)
    pub risk_probability: f64,
    /// Severity (Hartwig-Siegel 1-7)
    pub risk_severity: u8,
    /// Is the adverse event reversible?
    #[serde(default = "default_reversible")]
    pub reversible: bool,
}

fn default_reversible() -> bool {
    true
}

/// Parameters for deriving QBRI thresholds from historical data
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct QbriDeriveParams {
    /// Use synthetic test data
    #[serde(default = "default_use_synthetic")]
    pub use_synthetic: bool,
}

fn default_use_synthetic() -> bool {
    true
}
