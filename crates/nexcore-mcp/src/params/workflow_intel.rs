//! Parameter structs for workflow intelligence MCP tools.

use schemars::JsonSchema;
use serde::Deserialize;

/// Parameters for `workflow_map`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WorkflowMapParams {
    /// Number of days to analyze (default: 30).
    pub days: Option<u32>,
}

/// Parameters for `workflow_gaps` and `workflow_bottlenecks`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WorkflowGapsParams {
    /// Number of days to analyze (default: 30).
    pub days: Option<u32>,
}

/// Parameters for `workflow_live`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WorkflowLiveParams {
    /// Current session's tool call sequence (tool names in order).
    pub current_tools: Vec<String>,
}

/// Parameters for `workflow_suggest`.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct WorkflowSuggestParams {
    /// Number of days to analyze (default: 30).
    pub days: Option<u32>,
}
