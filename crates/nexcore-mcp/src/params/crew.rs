//! Crew-Mode Orchestration Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Multi-agent task decomposition, assignment, and decision fusion.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameters for creating a crew task with role-based agent assignments.
///
/// Roles: analyzer, guardian, synthesizer, learner, executor.
/// Each role has predefined MCP tool permissions.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CrewAssignParams {
    /// Description of the task to decompose
    pub description: String,
    /// List of roles to assign (e.g., ["analyzer", "guardian", "synthesizer"])
    pub roles: Vec<String>,
}

/// Parameters for checking crew task status.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CrewTaskStatusParams {
    /// Task ID to check (omit to list all tasks)
    #[serde(default)]
    pub task_id: Option<String>,
}

/// Parameters for fusing multiple agent decisions into a unified verdict.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct CrewFuseDecisionsParams {
    /// Task ID to fuse decisions for
    pub task_id: String,
    /// Partial results from each agent: { "agent_id": { result_json } }
    pub agent_results: HashMap<String, serde_json::Value>,
    /// Fusion strategy: "majority_vote", "weighted_score", "guardian_veto", "unanimous"
    #[serde(default)]
    pub fusion_strategy: Option<String>,
}
