//! Learning DAG Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Learning progression DAG resolution with completion state.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parameters for `learning_dag_resolve`
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LearningDagResolveParams {
    /// Capability Pathway ID
    pub pathway_id: String,
    /// User ID for personalized completion state (None = generic view)
    #[serde(default)]
    pub user_id: Option<String>,
}
