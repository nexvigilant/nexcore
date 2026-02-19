//! Graph Layout Parameters
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Force-directed graph layout convergence.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// A node for layout computation
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LayoutNode {
    /// Unique node identifier
    pub id: String,
    /// Optional group/cluster for coloring
    #[serde(default)]
    pub group: Option<String>,
    /// Optional value (affects node mass in force simulation)
    #[serde(default)]
    pub value: Option<f64>,
}

/// An edge for layout computation
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LayoutEdge {
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Optional edge weight (affects attraction strength)
    #[serde(default)]
    pub weight: Option<f64>,
}

/// Parameters for `graph_layout_converge`
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct GraphLayoutConvergeParams {
    /// Nodes to position
    pub nodes: Vec<LayoutNode>,
    /// Edges between nodes
    pub edges: Vec<LayoutEdge>,
    /// Number of dimensions: 2 or 3 (default 3)
    #[serde(default)]
    pub dimensions: Option<u8>,
    /// Maximum iterations (default 500)
    #[serde(default)]
    pub iterations: Option<u32>,
    /// Layout algorithm: "fruchterman-reingold" (default)
    #[serde(default)]
    pub algorithm: Option<String>,
}
