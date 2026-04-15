//! Core data types for workflow intelligence.

use serde::{Deserialize, Serialize};

/// A single tool call event extracted from session data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEvent {
    /// Tool name (e.g., "Read", "Bash", "mcp__nexcore__signal_detect").
    pub tool: String,
    /// Action or target (file path, command summary).
    pub action: String,
    /// Session ID this event belongs to.
    pub session_id: String,
    /// ISO timestamp.
    pub timestamp: String,
    /// Risk level: LOW, MEDIUM, HIGH.
    pub risk_level: String,
    /// Whether the action is reversible.
    pub reversible: bool,
}

/// A workflow — an ordered sequence of tool calls within a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    /// Session ID.
    pub session_id: String,
    /// Session description.
    pub description: String,
    /// Ordered tool events.
    pub events: Vec<ToolEvent>,
    /// Session verdict (if available).
    pub verdict: Option<String>,
    /// Total tool calls.
    pub tool_count: usize,
    /// Unique tools used.
    pub unique_tools: usize,
}

/// A tool transition — how often tool A is followed by tool B.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTransition {
    /// Source tool.
    pub from: String,
    /// Destination tool.
    pub to: String,
    /// Number of times this transition occurred.
    pub count: u64,
    /// Percentage of all transitions from `from`.
    pub pct: f64,
}

/// Workflow map — the DAG of tool transitions across sessions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMap {
    /// Time window analyzed.
    pub window: String,
    /// Total sessions analyzed.
    pub sessions_analyzed: u64,
    /// Total tool events.
    pub total_events: u64,
    /// Most common tool transitions.
    pub transitions: Vec<ToolTransition>,
    /// Most used tools (name, count).
    pub top_tools: Vec<(String, u64)>,
    /// Tool category breakdown (builtin vs MCP vs skill).
    pub category_breakdown: Vec<(String, u64)>,
}

/// A gap — a missing link or inefficiency in a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowGap {
    /// Gap type: "missing_automation", "manual_step", "error_prone", "underused_tool".
    pub gap_type: String,
    /// Human-readable description.
    pub description: String,
    /// Severity: 1 (minor) to 5 (critical).
    pub severity: u8,
    /// Evidence: session count, error rate, etc.
    pub evidence: String,
    /// Suggested fix.
    pub suggestion: String,
}

/// Gap analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapAnalysis {
    /// Time window analyzed.
    pub window: String,
    /// Gaps found, sorted by severity descending.
    pub gaps: Vec<WorkflowGap>,
    /// Overall workflow health score (0.0–1.0).
    pub health_score: f64,
    /// Summary statistics.
    pub stats: GapStats,
}

/// Summary statistics for gap analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapStats {
    /// Total sessions analyzed.
    pub sessions: u64,
    /// Sessions with errors.
    pub sessions_with_errors: u64,
    /// Average tools per session.
    pub avg_tools_per_session: f64,
    /// MCP tool usage percentage.
    pub mcp_usage_pct: f64,
    /// Skill invocation rate.
    pub skill_invocation_rate: f64,
    /// Fully demonstrated verdict rate.
    pub full_verdict_rate: f64,
}

/// A bottleneck — a tool or pattern that slows workflows.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bottleneck {
    /// Tool or pattern name.
    pub name: String,
    /// Bottleneck type: "high_failure", "high_frequency", "low_mcp", "repeated_reads".
    pub bottleneck_type: String,
    /// Impact score (0.0–1.0).
    pub impact: f64,
    /// Evidence.
    pub evidence: String,
    /// Recommendation.
    pub recommendation: String,
}

/// Live session intelligence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveIntel {
    /// Current session tool sequence so far.
    pub current_sequence: Vec<String>,
    /// Similar past workflows and their outcomes.
    pub similar_workflows: Vec<SimilarWorkflow>,
    /// Suggested next tools based on patterns.
    pub suggested_next: Vec<(String, f64)>,
    /// Warnings based on current pattern.
    pub warnings: Vec<String>,
}

/// A past workflow similar to the current one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarWorkflow {
    /// Session ID.
    pub session_id: String,
    /// Description.
    pub description: String,
    /// Verdict.
    pub verdict: String,
    /// Similarity score (0.0–1.0).
    pub similarity: f64,
}
