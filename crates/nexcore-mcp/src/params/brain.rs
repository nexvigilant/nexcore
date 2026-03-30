//! Brain Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Antigravity-style working memory, session persistence, and PROJECT GROUNDED beliefs.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for creating a new brain session
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainSessionCreateParams {
    /// Project name or path
    #[serde(default)]
    pub project: Option<String>,
    /// Git commit hash
    #[serde(default)]
    pub git_commit: Option<String>,
    /// Session description
    #[serde(default)]
    pub description: Option<String>,
}

/// Parameters for loading a brain session
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainSessionLoadParams {
    /// Session ID (UUID)
    pub session_id: String,
}

/// Parameters for listing brain sessions
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainSessionsListParams {
    /// Maximum number of sessions to return (default: 20)
    #[serde(default)]
    pub limit: Option<u32>,
}

/// Parameters for saving an artifact
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainArtifactSaveParams {
    /// Artifact name (e.g., "task.md", "plan.md")
    pub name: String,
    /// Artifact content
    pub content: String,
    /// Artifact type (task, plan, walkthrough, review, research, decision, custom)
    #[serde(default)]
    pub artifact_type: Option<String>,
    /// Session ID (defaults to latest session)
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Parameters for resolving an artifact
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainArtifactResolveParams {
    /// Artifact name
    pub name: String,
    /// Session ID (defaults to latest session)
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Parameters for getting an artifact
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainArtifactGetParams {
    /// Artifact name
    pub name: String,
    /// Specific version number (omit for current)
    #[serde(default)]
    pub version: Option<u32>,
    /// Session ID (defaults to latest session)
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Parameters for diffing artifact versions
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainArtifactDiffParams {
    /// Artifact name
    pub name: String,
    /// First version number
    pub v1: u32,
    /// Second version number
    pub v2: u32,
    /// Session ID (defaults to latest session)
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Parameters for tracking a file
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainCodeTrackerTrackParams {
    /// File path to track
    pub path: String,
    /// Project name (defaults to current directory)
    #[serde(default)]
    pub project: Option<String>,
}

/// Parameters for checking if a tracked file changed
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainCodeTrackerChangedParams {
    /// File path to check
    pub path: String,
    /// Project name
    #[serde(default)]
    pub project: Option<String>,
}

/// Parameters for getting original file content
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainCodeTrackerOriginalParams {
    /// File path
    pub path: String,
    /// Project name
    #[serde(default)]
    pub project: Option<String>,
}

/// Parameters for getting implicit knowledge
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainImplicitGetParams {
    /// Preference key
    pub key: String,
}

/// Parameters for setting implicit knowledge
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainImplicitSetParams {
    /// Preference key
    pub key: String,
    /// Preference value (JSON string)
    pub value: String,
}

/// Parameters for finding corrections by fuzzy matching
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainImplicitFindCorrectionsParams {
    /// Query string to fuzzy-match against corrections
    pub query: String,
    /// Minimum similarity threshold (0.0-1.0, default 0.3)
    pub threshold: Option<f64>,
}

/// Parameters for adding a correction with believability weighting
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainImplicitAddCorrectionParams {
    /// What was wrong (the mistake)
    pub mistake: String,
    /// What should have been done (the correction)
    pub correction: String,
    /// Context when this occurred
    pub context: Option<String>,
    /// Source of the correction: compiler, test, human, hook, model, training, incident
    pub source: Option<String>,
    /// Believability weight (0.0-1.0). Auto-detected from source if omitted.
    pub believability: Option<f64>,
}

/// Parameters for listing patterns by T1 grounding
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainImplicitPatternsByGroundingParams {
    /// T1 primitive to filter by: "sequence", "mapping", "recursion", "state", or "void"
    pub primitive: String,
}

/// Parameters for brain recovery repair
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainRecoveryRepairParams {
    /// Session ID to repair (defaults to latest)
    pub session_id: Option<String>,
}

/// Parameters for verifying engrams in the brain
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainVerifyParams {
    /// Placeholder for future expansion
    #[serde(default)]
    pub _placeholder: Option<bool>,
}

/// Parameters for saving a belief (PROJECT GROUNDED)
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainBeliefSaveParams {
    /// Unique belief identifier
    pub id: String,
    /// The proposition this belief represents
    pub proposition: String,
    /// Belief category (e.g., "capability", "behavior", "preference")
    pub category: String,
    /// If true, creates as hypothesis with lower initial confidence
    pub is_hypothesis: Option<bool>,
}

/// Parameters for getting a belief
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainBeliefGetParams {
    /// Belief ID to retrieve
    pub id: String,
}

/// Parameters for adding evidence to a belief
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainBeliefAddEvidenceParams {
    /// Belief ID to add evidence to
    pub belief_id: String,
    /// Unique evidence identifier
    pub evidence_id: String,
    /// Evidence type: observation, test_result, user_feedback, inference, authority, prior
    pub evidence_type: String,
    /// Description of what this evidence shows
    pub description: String,
    /// Weight (-1.0 to 1.0): positive supports, negative contradicts
    pub weight: f64,
    /// Source identifier
    pub source: String,
    /// Link to MCP execution that produced this evidence
    pub execution_id: Option<String>,
    /// Link to originating hypothesis
    pub hypothesis_id: Option<String>,
}

/// Parameters for validating a belief
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainBeliefValidateParams {
    /// Belief ID to validate
    pub belief_id: String,
    /// Whether validation passed
    pub passed: bool,
}

// ============================================================================
// Trust Parameters (PROJECT GROUNDED)
// ============================================================================

/// Parameters for recording a trust demonstration
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainTrustRecordParams {
    /// Domain to record trust for
    pub domain: String,
    /// Whether this was a success (true) or failure (false)
    pub success: bool,
}

/// Parameters for getting trust score
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainTrustGetParams {
    /// Domain to get trust for
    pub domain: String,
}

// ============================================================================
// Belief Graph Parameters (PROJECT GROUNDED)
// ============================================================================

/// Parameters for adding a belief implication
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainBeliefGraphAddImplicationParams {
    /// Source belief ID (antecedent)
    pub from: String,
    /// Target belief ID (consequent)
    pub to: String,
    /// Implication strength: "strong", "moderate", "weak"
    pub strength: String,
}

/// Parameters for querying the belief graph
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainBeliefGraphQueryParams {
    /// Belief ID to query
    pub belief_id: String,
}

// ============================================================================
// Brain Coordination Parameters (Multi-Agent Locking)
// ============================================================================

/// Parameters for acquiring a file lock
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainCoordinationAcquireParams {
    /// File path to lock
    pub path: String,
    /// Agent identifier
    pub agent_id: String,
    /// Lock TTL in seconds (default: 300)
    #[serde(default = "default_lock_ttl")]
    pub ttl: u64,
}

fn default_lock_ttl() -> u64 {
    300
}

/// Parameters for releasing a file lock
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainCoordinationReleaseParams {
    /// File path to unlock
    pub path: String,
    /// Agent identifier
    pub agent_id: String,
}

/// Parameters for checking lock status
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainCoordinationStatusParams {
    /// File path to check
    pub path: String,
}

// ============================================================================
// Brain Synapse Parameters (Amplitude Growth Learning)
// ============================================================================

/// Parameters for creating/getting a synapse
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SynapseGetOrCreateParams {
    /// Synapse ID
    pub id: String,
    /// Synapse type: "pattern", "preference", or "belief"
    #[serde(default = "default_synapse_type")]
    pub synapse_type: String,
}

fn default_synapse_type() -> String {
    "pattern".to_string()
}

/// Parameters for observing a learning signal
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SynapseObserveParams {
    /// Synapse ID
    pub id: String,
    /// Confidence in the observation (0.0 to 1.0)
    pub confidence: f64,
    /// Relevance to the learning target
    #[serde(default = "default_relevance")]
    pub relevance: f64,
}

fn default_relevance() -> f64 {
    1.0
}

/// Parameters for getting synapse info
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SynapseGetParams {
    /// Synapse ID
    pub id: String,
}

/// Parameters for listing synapses with optional filter
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct SynapseListParams {
    /// Filter by type prefix
    #[serde(default)]
    pub filter_type: Option<String>,
    /// Only show consolidated synapses
    #[serde(default)]
    pub consolidated_only: bool,
}

// ============================================================================
// Brain Health Metrics Parameters
// ============================================================================

/// Parameters for brain growth rate analysis
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainGrowthRateParams {
    /// Number of days to analyze
    #[serde(default = "default_growth_days")]
    pub days: u32,
}

fn default_growth_days() -> u32 {
    7
}

/// Parameters for retrieving largest artifacts
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct BrainLargestArtifactsParams {
    /// Number of artifacts to return
    #[serde(default = "default_largest_n")]
    pub n: usize,
}

fn default_largest_n() -> usize {
    10
}
