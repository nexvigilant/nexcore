//! Causal reasoning MCP tool parameters.
//!
//! Typed parameter structs for causal DAG construction, inference, and
//! counterfactual evaluation.

use schemars::JsonSchema;
use serde::Deserialize;

/// A node in a causal DAG.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReasonNodeInput {
    /// Unique node identifier.
    pub id: String,
    /// Node label/description.
    pub label: String,
    /// Node type: "cause", "effect", "mediator", "confounder", "collider".
    pub node_type: String,
    /// Confidence in this node's existence [0.0, 1.0].
    pub confidence: Option<f64>,
}

/// A causal link between nodes.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReasonLinkInput {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Causal strength [0.0, 1.0].
    pub strength: Option<f64>,
    /// Evidence supporting this link.
    pub evidence: Option<String>,
}

/// Build a causal DAG and run inference to find causal chains and risk level.
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReasonInferParams {
    /// Nodes in the causal DAG.
    pub nodes: Vec<ReasonNodeInput>,
    /// Links (edges) in the causal DAG.
    pub links: Vec<ReasonLinkInput>,
}

/// Run counterfactual analysis: "what if we remove node X?"
#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReasonCounterfactualParams {
    /// Nodes in the causal DAG.
    pub nodes: Vec<ReasonNodeInput>,
    /// Links (edges) in the causal DAG.
    pub links: Vec<ReasonLinkInput>,
    /// Node ID to remove for counterfactual analysis.
    pub remove_node: String,
}
