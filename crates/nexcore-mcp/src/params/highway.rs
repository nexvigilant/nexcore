//! Digital Highway Parameters (Infrastructure Acceleration)
//! Source: Chatburn, "Highways and Highway Transportation" (1923)
//! T1 Grounding: σ(Sequence) + →(Causality) + ∂(Boundary) + N(Quantity) + κ(Comparison)

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Classify a tool into its Digital Highway tier (I-IV).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HighwayClassifyParams {
    /// Tool name to classify
    pub tool_name: String,
    /// Number of internal crate dependencies (0 = foundation)
    pub internal_deps: u32,
    /// Average response time in milliseconds
    pub avg_response_ms: f64,
    /// Whether the tool calls external APIs
    #[serde(default)]
    pub calls_external: bool,
    /// Whether the tool maintains state across calls
    #[serde(default)]
    pub stateful: bool,
}

/// Assess a tool against the 7 Ideal Tool Qualities (Chatburn Ch.8).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HighwayQualityParams {
    /// Tool name
    pub tool_name: String,
    /// Lines of implementation code
    pub impl_lines: u32,
    /// Number of parameters
    pub param_count: u32,
    /// Has typed error handling (not string errors)
    #[serde(default)]
    pub typed_errors: bool,
    /// Has input validation on all params
    #[serde(default)]
    pub validates_input: bool,
    /// Average calls per session (adoption metric)
    #[serde(default)]
    pub calls_per_session: f64,
    /// Versions without breaking changes
    #[serde(default)]
    pub stable_versions: u32,
}

/// Run a traffic census on tool usage patterns.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HighwayTrafficCensusParams {
    /// Tool category to census (or "all")
    #[serde(default = "default_all")]
    pub category: String,
}

fn default_all() -> String {
    "all".into()
}

/// Compute destructive factor score for a tool.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HighwayDestructiveParams {
    /// Tool name
    pub tool_name: String,
    /// Calls per hour (density)
    pub calls_per_hour: f64,
    /// Average payload size in bytes (weight)
    pub avg_payload_bytes: u64,
    /// Average response time in ms (speed factor)
    pub avg_response_ms: f64,
    /// Error rate 0.0-1.0
    pub error_rate: f64,
}

/// Check if a tool is being used in its legitimate field.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HighwayLegitimateFieldParams {
    /// Tool name
    pub tool_name: String,
    /// What the tool is being used for
    pub use_case: String,
    /// Current highway class of the tool (1-4)
    pub highway_class: u32,
}

/// Plan parallel lanes for a batch of proposed tool calls.
/// Applies grade separation: groups tools by highway class, assigns lane types.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HighwayParallelPlanParams {
    /// List of tool calls to plan. Each entry: { "tool": "name", "highway_class": 1-4, "estimated_ms": 50.0 }
    pub tools: Vec<ToolCallSpec>,
    /// Minimum confidence gate for the composed result (default: 0.8)
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f64,
}

/// Specification for a single tool call in a parallel plan.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ToolCallSpec {
    /// Tool name
    pub tool: String,
    /// Highway class (1=Interstate/Foundation, 2=State/Domain, 3=County/Orchestration, 4=Township/Service)
    pub highway_class: u32,
    /// Estimated response time in ms
    #[serde(default = "default_estimated_ms")]
    pub estimated_ms: f64,
    /// Confidence in this tool's output (0.0-1.0)
    #[serde(default = "default_tool_confidence")]
    pub confidence: f64,
    /// Whether this tool depends on another tool's output
    #[serde(default)]
    pub depends_on: Option<String>,
}

fn default_min_confidence() -> f64 {
    0.8
}

fn default_estimated_ms() -> f64 {
    100.0
}

fn default_tool_confidence() -> f64 {
    0.9
}

/// Merge N parallel tool results through grounded confidence composition.
/// The Interchange Pattern: parallel lanes → controlled merging → confidence gate.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HighwayInterchangeParams {
    /// Results from parallel tool calls. Each: { "tool": "name", "value": <any>, "confidence": 0.9 }
    pub results: Vec<LaneResult>,
    /// Merge strategy: "multiplicative" (default), "minimum", "average", "weighted"
    #[serde(default = "default_merge_strategy")]
    pub strategy: String,
    /// Minimum confidence gate for the merged result
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f64,
    /// Optional label for the merged result
    #[serde(default)]
    pub label: Option<String>,
}

/// Result from a single parallel lane.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct LaneResult {
    /// Tool name that produced this result
    pub tool: String,
    /// The value returned
    pub value: serde_json::Value,
    /// Confidence in this result (0.0-1.0)
    pub confidence: f64,
    /// Optional weight for weighted merge (default: 1.0)
    #[serde(default = "default_weight")]
    pub weight: f64,
}

fn default_merge_strategy() -> String {
    "multiplicative".into()
}

fn default_weight() -> f64 {
    1.0
}

/// Sort tools into grade-separated batches by highway class.
/// Returns ordered batches that can be executed sequentially (fast first, slow last).
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct HighwayGradeSeparateParams {
    /// Tool names with their highway classes
    pub tools: Vec<ToolCallSpec>,
}
