//! Human-in-the-Loop Pipeline Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Decision approval queue, review workflow, and feedback tracking.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for submitting a decision for human review.
///
/// Creates a pending approval entry in the HITL queue.
/// Decisions above the risk threshold are automatically flagged.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HitlSubmitParams {
    /// Source tool that generated the decision (e.g., "guardian_evaluate_pv")
    pub tool: String,
    /// Recommended action (e.g., "investigate", "restrict_access", "emergency_withdrawal")
    pub recommendation: String,
    /// Target entity (e.g., drug name, signal ID)
    pub target: String,
    /// Risk level: "low", "medium", "high", "critical"
    pub risk_level: String,
    /// Risk score (0.0 - 10.0)
    pub risk_score: f64,
    /// Supporting evidence as JSON
    #[serde(default)]
    pub evidence: Option<serde_json::Value>,
    /// Assign to specific reviewer
    #[serde(default)]
    pub assign_to: Option<String>,
    /// Expiration in hours (default: 72)
    #[serde(default)]
    pub expires_hours: Option<u64>,
}

/// Parameters for viewing the HITL approval queue.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HitlQueueParams {
    /// Filter by status: "pending", "approved", "rejected", "expired", "all" (default: "pending")
    #[serde(default)]
    pub status: Option<String>,
    /// Filter by risk level
    #[serde(default)]
    pub risk_level: Option<String>,
    /// Maximum entries to return (default: 20)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Filter by assigned reviewer
    #[serde(default)]
    pub assigned_to: Option<String>,
}

/// Parameters for reviewing a pending decision.
///
/// Approves, rejects, or modifies a pending decision in the HITL queue.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HitlReviewParams {
    /// Decision ID to review
    pub decision_id: String,
    /// Review action: "approve", "reject", "modify"
    pub action: String,
    /// Reviewer identifier
    pub reviewer: String,
    /// Review comments / rationale
    #[serde(default)]
    pub comments: Option<String>,
    /// Modified action (required if action is "modify")
    #[serde(default)]
    pub modified_action: Option<String>,
}

/// Parameters for HITL pipeline statistics.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HitlStatsParams {
    /// Include per-tool breakdown (default: true)
    #[serde(default)]
    pub include_tool_breakdown: Option<bool>,
    /// Include reviewer activity (default: true)
    #[serde(default)]
    pub include_reviewer_activity: Option<bool>,
}
