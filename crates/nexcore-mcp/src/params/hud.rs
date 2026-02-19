//! HUD (Heads-Up Display) Capability Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Small Business Act (allocation), Social Security Act (persistence), Federal Reserve Act (budget),
//! Securities Act (audit), and Communications Act (routing) parameters.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for Small Business Act agent allocation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SbaAllocateAgentParams {
    /// Task description for agent allocation
    pub task_description: String,
    /// Preferred model tier: economy, standard, premium, apex
    #[serde(default)]
    pub preferred_tier: Option<String>,
}

/// Parameters for Small Business Act chain next step
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SbaChainNextParams {
    /// Chain ID to query
    pub chain_id: String,
    /// Completed step name
    pub completed_step: String,
    /// Whether the completed step had errors
    #[serde(default)]
    pub had_errors: bool,
}

/// Parameters for Social Security Act state persistence
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SsaPersistStateParams {
    /// State identifier
    pub state_id: String,
    /// State content to persist
    pub content: String,
    /// Persistence level: ephemeral, session, project, global
    #[serde(default)]
    pub level: Option<String>,
}

/// Parameters for Social Security Act integrity verification
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SsaVerifyIntegrityParams {
    /// State identifier
    pub state_id: String,
    /// Expected hash (SHA-256 hex string)
    pub expected_hash: String,
    /// Content to verify
    pub content: String,
}

/// Parameters for Federal Reserve Act budget report
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FedBudgetReportParams {
    /// Current token usage
    #[serde(default)]
    pub current_tokens: Option<u64>,
    /// Budget limit
    #[serde(default)]
    pub budget_limit: Option<u64>,
}

/// Parameters for Federal Reserve Act model recommendation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FedRecommendModelParams {
    /// Task complexity: simple, moderate, complex, research
    #[serde(default)]
    pub task_complexity: Option<String>,
    /// Current budget utilization percentage (0-100)
    #[serde(default)]
    pub budget_utilization: Option<f64>,
    /// Requires high accuracy
    #[serde(default)]
    pub requires_accuracy: Option<bool>,
}

/// Parameters for Securities Act market audit
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SecAuditMarketParams {
    /// Market identifier to audit
    pub market_id: String,
    /// Trade volume for compliance check
    pub trade_volume: u64,
}

/// Parameters for Communications Act protocol recommendation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CommRecommendProtocolParams {
    /// Whether delivery guarantee is required
    #[serde(default)]
    pub needs_guarantee: bool,
    /// Whether low latency is required
    #[serde(default)]
    pub low_latency: bool,
    /// Whether message is broadcast
    #[serde(default)]
    pub is_broadcast: bool,
}

/// Parameters for Communications Act message routing
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CommRouteMessageParams {
    /// Sender agent ID
    pub from: String,
    /// Recipient agent ID
    pub to: String,
    /// Message payload (JSON string)
    pub payload: String,
    /// Protocol: mcp, jsonrpc, event, direct, rest
    #[serde(default)]
    pub protocol: Option<String>,
    /// Time-to-live in seconds
    #[serde(default)]
    pub ttl: Option<u32>,
}

/// Parameters for Exploration Act mission launch
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ExploreLaunchMissionParams {
    /// Target domain/path to explore
    pub target: String,
    /// Objective description
    pub objective: String,
    /// Scope: quick, medium, thorough
    #[serde(default)]
    pub scope: Option<String>,
    /// Search patterns
    #[serde(default)]
    pub patterns: Option<Vec<String>>,
}

/// Parameters for Exploration Act discovery recording
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ExploreRecordDiscoveryParams {
    /// What was found
    pub finding: String,
    /// File location
    #[serde(default)]
    pub location: Option<String>,
    /// Significance score 0.0-1.0
    #[serde(default)]
    pub significance: Option<f64>,
}

/// Parameters for Exploration Act frontier status
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ExploreGetFrontierParams {}

/// Parameters for CAP-014 Public Health Act: validate signal efficacy
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HealthValidateSignalParams {
    /// Signal identifier
    pub signal_id: String,
    /// Accuracy of signal against ground truth (0.0 - 1.0)
    pub accuracy: f64,
}

/// Parameters for CAP-014 Public Health Act: measure impact
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HealthMeasureImpactParams {
    /// Total signals analyzed
    pub total_signals: u32,
    /// Number of valid signals
    pub valid_signals: u32,
}

/// Parameters for CAP-018 Treasury Act: convert asymmetry
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TreasuryConvertAsymmetryParams {
    /// Signal identifier providing asymmetry
    pub signal_id: String,
    /// Asymmetry value
    pub asymmetry: f64,
    /// Market odds
    pub market_odds: f64,
}

/// Parameters for CAP-018 Treasury Act: audit treasury
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct TreasuryAuditParams {
    /// Compute quota in treasury
    pub compute_quota: u64,
    /// Memory quota in treasury
    pub memory_quota: u64,
}

/// Parameters for CAP-019 Transportation Act: dispatch manifest
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DotDispatchManifestParams {
    /// Origin domain
    pub origin: String,
    /// Destination domain
    pub destination: String,
    /// Number of signals in batch
    pub signal_count: u32,
    /// Transit priority (1-10)
    pub priority: u8,
}

/// Parameters for CAP-019 Transportation Act: verify highway safety
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DotVerifyHighwayParams {
    /// Route identifier to verify
    pub route_id: String,
}

/// Parameters for CAP-020 Homeland Security Act: verify boundary
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DhsVerifyBoundaryParams {
    /// Source identifier
    pub source_id: String,
    /// Payload hash for verification
    pub payload_hash: String,
}

/// Parameters for CAP-022 Education Act: train agent
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduTrainAgentParams {
    /// Subject area of training
    pub subject: String,
    /// Complexity level (1-10)
    pub level: u8,
    /// Current completion progress (0.0 - 1.0)
    pub completion: f64,
}

/// Parameters for CAP-022 Education Act: evaluate comprehension
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct EduEvaluateParams {
    /// List of comprehension scores
    pub scores: Vec<f64>,
}

/// Parameters for CAP-031 Science Foundation Act: fund research
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NsfFundResearchParams {
    /// Research project title
    pub project: String,
    /// Target capability ID being enhanced
    pub target_cap: String,
}

/// Parameters for CAP-037 General Services Act: procure resource
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsaProcureParams {
    /// Resource identifier
    pub resource_id: String,
    /// Quantity to procure
    pub quantity: u64,
    /// Priority (1-10)
    pub priority: u8,
}

/// Parameters for CAP-037 General Services Act: audit service value
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GsaAuditValueParams {
    /// Cost of service
    pub cost: f64,
    /// Benefit of service
    pub benefit: f64,
}
