//! Foundry MCP tool parameters.
//!
//! Typed parameter structs for The Foundry's assembly-line pipeline tools:
//! artifact validation, cascade checking, report rendering, and VDAG ordering.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::Deserialize;

/// Parameters for `foundry_validate_artifact` — validate a deliverable
/// against The Foundry's B3 quality gate.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FoundryValidateArtifactParams {
    /// Whether `cargo build` passed.
    pub build_pass: bool,
    /// Total number of discovered tests.
    pub test_count: u32,
    /// Number of tests that passed.
    pub tests_passed: u32,
    /// Whether `cargo clippy -- -D warnings` passed.
    pub lint_pass: bool,
    /// Line-coverage percentage (0.0–100.0).
    pub coverage_percent: f64,
    /// Human-readable failure descriptions (empty if none).
    #[serde(default)]
    pub failures: Vec<String>,
}

/// Parameters for `foundry_cascade_validate` — check alignment of a
/// SMART goal cascade (Strategic → Team → Operational).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FoundryCascadeValidateParams {
    /// Total number of operational goals in the plan.
    pub total_operational: u32,
    /// Number of operational goals traced to at least one team goal.
    pub traced_to_team: u32,
    /// Number of operational goals traced to a strategic goal.
    pub traced_to_strategic: u32,
}

/// Parameters for `foundry_render_intelligence` — render an A3
/// intelligence report as markdown.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FoundryRenderIntelligenceParams {
    /// Key findings from the analyst pipeline.
    pub findings: Vec<String>,
    /// Prioritized actionable recommendations.
    pub recommendations: Vec<String>,
    /// Risk level: "low", "moderate", "high", or "critical".
    pub risk_level: String,
    /// Confidence score in [0.0, 1.0].
    pub confidence: f64,
}

/// Parameters for `foundry_vdag_order` — return the VDAG pipeline
/// ordering. Accepts an optional pipeline variant.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FoundryVdagOrderParams {
    /// Pipeline variant: "full" (14 stations), "builder" (3), or "analyst" (3).
    /// Defaults to "full".
    #[serde(default = "default_pipeline_variant")]
    pub variant: String,
}

fn default_pipeline_variant() -> String {
    "full".to_string()
}

/// Parameters for `foundry_infer` — run causal inference over a DAG
/// described as nodes and links.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FoundryInferParams {
    /// Causal nodes as `[{"id": "...", "label": "...", "node_type": "metric|pattern|module|risk|recommendation"}]`.
    pub nodes: Vec<FoundryNodeInput>,
    /// Causal links as `[{"from": "...", "to": "...", "strength": 0.0-1.0, "evidence": "..."}]`.
    pub links: Vec<FoundryLinkInput>,
}

/// A causal node for the `foundry_infer` tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FoundryNodeInput {
    /// Node identifier.
    pub id: String,
    /// Human-readable label.
    pub label: String,
    /// Node type: "metric", "pattern", "module", "risk", or "recommendation".
    pub node_type: String,
}

/// A causal link for the `foundry_infer` tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct FoundryLinkInput {
    /// Source node ID.
    pub from: String,
    /// Target node ID.
    pub to: String,
    /// Causal strength in [0.0, 1.0].
    pub strength: f64,
    /// Free-text evidence description.
    #[serde(default)]
    pub evidence: String,
}
