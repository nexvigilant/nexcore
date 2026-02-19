//! Network & Node Hunting Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Real-time node scanning, isolation, and behavioral fingerprinting.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for real-time node signal scanning
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NodeHuntScanParams {
    /// Behavioral signature to hunt for (e.g., 'LATENCY_SPIKE', 'ANOMALY')
    pub target_pattern: String,
    /// Optional: Filter to specific network partition
    #[serde(default)]
    pub partition: Option<String>,
}

/// Parameters for node isolation
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NodeHuntIsolateParams {
    /// Node ID to isolate
    pub node_id: String,
    /// Reason for isolation
    pub reason: String,
}
