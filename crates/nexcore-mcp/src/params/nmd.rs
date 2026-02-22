// Copyright (c) 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! NMD (Nonsense-Mediated mRNA Decay) surveillance MCP parameters.
//!
//! ## T1 Primitive Grounding: ∂(Boundary) + ν(Frequency) + ς(State)

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for full NMD pipeline check (UPF → Thymic → SMG → Adaptive).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NmdCheckParams {
    /// Task category (e.g., "Explore", "Mutate", "Compute").
    pub category: String,
    /// Phase ID for the current execution checkpoint.
    pub phase_id: String,
    /// Tool categories observed in this phase (e.g., ["Read", "Grep"]).
    pub observed_categories: Vec<String>,
    /// Number of grounding signals (external validation events).
    #[serde(default)]
    pub grounding_signals: u32,
    /// Total tool calls in this phase.
    #[serde(default)]
    pub total_calls: u32,
    /// Checkpoint index in the execution sequence.
    #[serde(default)]
    pub checkpoint_index: usize,
}

/// Parameters for raw UPF evaluation only (no thymic/SMG/adaptive).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NmdUpfEvaluateParams {
    /// Phase ID for the current execution checkpoint.
    pub phase_id: String,
    /// Tool categories observed in this phase.
    pub observed_categories: Vec<String>,
    /// Number of grounding signals.
    #[serde(default)]
    pub grounding_signals: u32,
    /// Total tool calls in this phase.
    #[serde(default)]
    pub total_calls: u32,
    /// Checkpoint index in the execution sequence.
    #[serde(default)]
    pub checkpoint_index: usize,
}

/// Parameters for SMG verdict processing.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NmdSmgProcessParams {
    /// Verdict type: "continue", "stall", or "degrade".
    pub verdict: String,
    /// Anomaly descriptions (required for stall/degrade).
    #[serde(default)]
    pub anomalies: Vec<NmdAnomalyInput>,
}

/// Input format for an anomaly.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NmdAnomalyInput {
    /// UPF channel: "UPF1", "UPF2", or "UPF3".
    pub channel: String,
    /// Description of the anomaly.
    pub description: String,
    /// Severity score (0.0 to 1.0).
    #[serde(default = "default_severity")]
    pub severity: f32,
}

fn default_severity() -> f32 {
    0.5
}

/// Parameters for adaptive stats query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NmdAdaptiveStatsParams {
    /// Category to query. If omitted, returns all categories.
    #[serde(default)]
    pub category: Option<String>,
}

/// Parameters for thymic status query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct NmdThymicStatusParams {
    /// Category to check. If omitted, returns all categories.
    #[serde(default)]
    pub category: Option<String>,
}
