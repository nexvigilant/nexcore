//! Cloud Intelligence param structs for 17 MCP tools.
//!
//! Phase 1: Query + Analysis (8 tools)
//! Phase 2: Infrastructure (4 tools)
//! Phase 3: Reasoning (5 tools)

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

// ============================================================================
// Phase 1: Core Query + Analysis (8 tools)
// ============================================================================

/// Get primitive composition of a cloud type.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudCompositionParams {
    /// Cloud type name (e.g. "VirtualMachine", "LoadBalancer", "Serverless")
    pub type_name: String,
}

/// Get transfer confidence from a cloud type to a target domain.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudTransferConfidenceParams {
    /// Cloud type name
    pub cloud_type: String,
    /// Target domain (PV, Biology, Economics)
    pub domain: String,
}

/// Classify a set of primitives into a tier.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudTierClassifyParams {
    /// List of primitive names or symbols
    pub primitives: Vec<String>,
}

/// Compare two cloud types by primitive overlap.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudCompareTypesParams {
    /// First cloud type name
    pub type_a: String,
    /// Second cloud type name
    pub type_b: String,
}

/// Reverse-synthesize: given primitives, find matching cloud types.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudReverseSynthesizeParams {
    /// List of primitive names or symbols
    pub primitives: Vec<String>,
    /// Minimum overlap confidence (0.0-1.0, default 0.0)
    #[serde(default)]
    pub min_confidence: Option<f64>,
}

/// List all cloud types, optionally filtered by tier.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudListTypesParams {
    /// Filter by tier: "T1", "T2-P", "T2-C", "T3" (omit for all)
    #[serde(default)]
    pub tier: Option<String>,
}

/// Get molecular weight (Shannon information bits) of a cloud type.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudMolecularWeightParams {
    /// Cloud type name
    pub type_name: String,
}

/// Detect dominant primitive shift when adding a primitive to a cloud type.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudDominantShiftParams {
    /// Cloud type name
    pub type_name: String,
    /// Primitive to add (name or symbol)
    pub added_primitive: String,
}

// ============================================================================
// Phase 2: Infrastructure Awareness (4 tools)
// ============================================================================

/// Get real-time GCE infrastructure status.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudInfraStatusParams {
    /// GCE project ID (optional, uses default if omitted)
    #[serde(default)]
    pub project: Option<String>,
}

/// Map a single GCE instance to its cloud model with composition overlay.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudInfraMapParams {
    /// Instance name
    pub instance: String,
    /// Zone (optional, uses default if omitted)
    #[serde(default)]
    pub zone: Option<String>,
}

/// Project resource capacity over time.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudCapacityProjectParams {
    /// Current utilization (0.0-1.0)
    pub current_utilization: f64,
    /// Total capacity (abstract units)
    pub total_capacity: f64,
    /// Daily growth rate (absolute units per day)
    pub daily_growth: f64,
    /// Number of days to project
    #[serde(default = "default_projection_days")]
    pub days: u32,
}

fn default_projection_days() -> u32 {
    30
}

/// Get nexcloud supervisor health mapped through cloud primitives.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudSupervisorHealthParams {
    /// Include primitive composition overlay (default true)
    #[serde(default = "default_true")]
    pub include_composition: bool,
}

fn default_true() -> bool {
    true
}

// ============================================================================
// Phase 3: Cross-Domain Reasoning (5 tools)
// ============================================================================

/// Reverse transfer: given a domain concept, find matching cloud type.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudReverseTransferParams {
    /// Target domain (e.g. "PV", "Biology", "Economics")
    pub domain: String,
    /// Keywords to match against domain analogs
    pub keywords: Vec<String>,
}

/// Find multi-hop transfer chains between a cloud type and a target domain.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudTransferChainParams {
    /// Starting cloud type name
    pub start_type: String,
    /// Target domain
    pub target_domain: String,
    /// Maximum hops (default 3, max 5)
    #[serde(default = "default_max_hops")]
    pub max_hops: usize,
}

fn default_max_hops() -> usize {
    3
}

/// Recommend cloud types based on required primitives and constraints.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudArchitectureAdvisorParams {
    /// Required primitive names or symbols
    pub required_primitives: Vec<String>,
    /// Optional: prefer specific tier ("T1", "T2-P", "T2-C", "T3")
    #[serde(default)]
    pub preferred_tier: Option<String>,
    /// Optional: maximum number of recommendations (default 5)
    #[serde(default = "default_top_n")]
    pub top_n: Option<usize>,
}

fn default_top_n() -> Option<usize> {
    Some(5)
}

/// Detect anomalies: compare a cloud type's expected primitives against observed.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudAnomalyDetectParams {
    /// Cloud type name
    pub type_name: String,
    /// Observed primitives (names or symbols)
    pub observed_primitives: Vec<String>,
}

/// Generate a full type×domain transfer confidence matrix.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CloudTransferMatrixParams {
    /// Filter by tier (optional)
    #[serde(default)]
    pub tier: Option<String>,
    /// Filter by domain (optional, shows all domains if omitted)
    #[serde(default)]
    pub domain: Option<String>,
}
