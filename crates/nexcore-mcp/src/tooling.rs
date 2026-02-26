//! Shared tooling helpers for MCP server outputs, gating, and scan limits.

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::OnceLock;

use nexcore_vigilance::primitives::measurement::Confidence;
use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ============================================================================
// Tool Gating
// ============================================================================

/// Tool gate configuration (allow/deny lists).
///
/// Tier: T2-C (Cross-domain composite for tool access control)
/// Grounds to: T1 primitives (String, bool) via HashSet entries
/// Ord: N/A (composite configuration)
#[derive(Debug, Clone)]
pub struct ToolGate {
    allowlist: Option<HashSet<String>>,
    denylist: HashSet<String>,
}

/// Tool gate error (denied access).
///
/// Tier: T3 (Domain-specific MCP gating error)
/// Grounds to: T1 primitives (String)
/// Ord: N/A (error classification)
#[derive(Debug, Clone)]
pub struct ToolGateError {
    pub tool: String,
    pub reason: String,
}

static TOOL_GATE: OnceLock<ToolGate> = OnceLock::new();

/// Get the global tool gate configuration.
pub fn tool_gate() -> &'static ToolGate {
    TOOL_GATE.get_or_init(ToolGate::from_env)
}

impl ToolGate {
    /// Build tool gate config from environment.
    ///
    /// - `NEXCORE_MCP_TOOL_ALLOWLIST`: comma-separated tool names
    /// - `NEXCORE_MCP_TOOL_DENYLIST`: comma-separated tool names
    #[must_use]
    pub fn from_env() -> Self {
        let allowlist = parse_tool_list("NEXCORE_MCP_TOOL_ALLOWLIST");
        let denylist = parse_tool_list("NEXCORE_MCP_TOOL_DENYLIST").unwrap_or_default();
        Self {
            allowlist,
            denylist,
        }
    }

    /// Check whether a tool is permitted.
    pub fn check(&self, tool: &str) -> Result<(), ToolGateError> {
        if self.denylist.contains(tool) {
            return Err(ToolGateError {
                tool: tool.to_string(),
                reason: "tool explicitly denied by NEXCORE_MCP_TOOL_DENYLIST".to_string(),
            });
        }
        if let Some(allowlist) = &self.allowlist {
            if !allowlist.contains(tool) {
                return Err(ToolGateError {
                    tool: tool.to_string(),
                    reason: "tool not present in NEXCORE_MCP_TOOL_ALLOWLIST".to_string(),
                });
            }
        }
        Ok(())
    }
}

fn parse_tool_list(var: &str) -> Option<HashSet<String>> {
    let value = env::var(var).ok()?;
    let items: HashSet<String> = value
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| item.to_string())
        .collect();
    if items.is_empty() { None } else { Some(items) }
}

// ============================================================================
// Evidence & Violations
// ============================================================================

/// Violation code (quantified enum mapping).
///
/// Tier: T2-P (Cross-domain primitive code)
/// Grounds to: T1 primitive `u16`
/// Ord: Implemented (numeric code ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViolationCode(pub u16);

impl ViolationCode {
    /// Tool execution error.
    pub const TOOL_ERROR: Self = Self(1);
    /// Tool gated by allow/deny list.
    pub const TOOL_GATED: Self = Self(2);
}

/// Evidence snippet for tool output.
///
/// Tier: T2-C (Cross-domain composite evidence)
/// Grounds to: T1 primitives (String, bool, usize)
/// Ord: N/A (composite evidence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub text: String,
    pub truncated: bool,
    pub bytes: usize,
    pub lines: usize,
}

/// Evidence record with explicit confidence.
///
/// Tier: T2-C (Cross-domain composite evidence)
/// Grounds to: T1 primitives (String) + T2-P Confidence
/// Ord: N/A (composite evidence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub summary: String,
    pub confidence: Confidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<Snippet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// Tool execution violation record with explicit confidence.
///
/// Tier: T2-C (∂ + ∃ — boundary violation with evidence)
/// Grounds to: T1 primitives (String) + T2-P Confidence + ViolationCode
/// Ord: N/A (composite violation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolViolation {
    pub code: ViolationCode,
    pub summary: String,
    pub confidence: Confidence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
}

/// Structured tool report for parity and evidence tracking.
///
/// Tier: T2-C (Cross-domain composite report)
/// Grounds to: T1 primitives (String) + T2-P Confidence + Evidence/Violation
/// Ord: N/A (composite report)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolReport {
    pub tool: String,
    pub confidence: Confidence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub violations: Vec<ToolViolation>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notices: Vec<ScanLimitNotice>,
}

/// Backward-compatible alias.
#[deprecated(note = "use ToolViolation — F2 equivocation fix")]
pub type Violation = ToolViolation;

/// Wrap a tool result — grammar controller only, no ToolReport envelope.
///
/// The previous implementation wrapped every tool response in a `ToolReport`
/// with hardcoded `confidence: 0.85` and generic evidence. This added ~155
/// bytes per call with zero Shannon entropy. Now tools pass through cleanly.
pub fn wrap_result(mut result: CallToolResult) -> CallToolResult {
    // Apply Grammar Controller to text content
    let controller = crate::tools::grammar::PrimitiveLexicon::get();
    for i in 0..result.content.len() {
        if let Some(text_content) = result.content[i].as_text() {
            let new_text = controller.apply(&text_content.text);
            result.content[i] = Content::text(new_text);
        }
    }

    result
}

/// Attach machine-readable forensic metadata to a tool result.
///
/// Detection tools call this to surface their computed confidence
/// in `structured_content` for programmatic consumption.
pub fn attach_forensic_meta(
    result: &mut CallToolResult,
    confidence: f64,
    signal_detected: Option<bool>,
    category: &str,
) {
    result.structured_content = Some(json!({
        "forensic": {
            "confidence": confidence.clamp(0.0, 1.0),
            "signal_detected": signal_detected,
            "category": category,
        }
    }));
}

/// Create a gated tool result for denied tools.
pub fn gated_result(tool: &str, error: ToolGateError) -> CallToolResult {
    let violation = ToolViolation {
        code: ViolationCode::TOOL_GATED,
        summary: error.reason.clone(),
        confidence: Confidence::new(0.9),
        evidence: Vec::new(),
    };
    let report = ToolReport {
        tool: tool.to_string(),
        confidence: Confidence::new(0.9),
        evidence: Vec::new(),
        violations: vec![violation],
        notices: Vec::new(),
    };
    let mut result = CallToolResult::error(vec![Content::text(format!(
        "Tool '{tool}' denied: {}",
        error.reason
    ))]);
    attach_report(&mut result, &report, tool);
    result
}

fn attach_report(result: &mut CallToolResult, report: &ToolReport, tool: &str) {
    match serde_json::to_value(report) {
        Ok(value) => {
            result.structured_content = Some(value);
        }
        Err(e) => {
            result.structured_content = Some(json!({
                "tool": tool,
                "error": format!("Failed to serialize tool report: {e}")
            }));
        }
    }
}

// ============================================================================
// Scan Limits & Truncation
// ============================================================================

/// Scan limits for file reads and violation collection.
///
/// Tier: T2-C (Cross-domain composite limits)
/// Grounds to: T1 primitives (usize)
/// Ord: N/A (composite configuration)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScanLimits {
    pub max_bytes: usize,
    pub max_lines: usize,
    pub max_hits: usize,
    pub max_snippet_bytes: usize,
    pub max_snippet_lines: usize,
}

impl ScanLimits {
    /// Load scan limits from environment (with safe defaults).
    ///
    /// - `NEXCORE_MCP_SCAN_MAX_BYTES`
    /// - `NEXCORE_MCP_SCAN_MAX_LINES`
    /// - `NEXCORE_MCP_SCAN_MAX_HITS`
    /// - `NEXCORE_MCP_SNIPPET_MAX_BYTES`
    /// - `NEXCORE_MCP_SNIPPET_MAX_LINES`
    #[must_use]
    pub fn from_env() -> Self {
        Self {
            max_bytes: read_env_usize("NEXCORE_MCP_SCAN_MAX_BYTES", 1_000_000),
            max_lines: read_env_usize("NEXCORE_MCP_SCAN_MAX_LINES", 20_000),
            max_hits: read_env_usize("NEXCORE_MCP_SCAN_MAX_HITS", 500),
            max_snippet_bytes: read_env_usize("NEXCORE_MCP_SNIPPET_MAX_BYTES", 4_000),
            max_snippet_lines: read_env_usize("NEXCORE_MCP_SNIPPET_MAX_LINES", 120),
        }
    }
}

/// Scan limit notice with counters.
///
/// Tier: T2-C (Cross-domain composite notice)
/// Grounds to: T1 primitives (usize, bool)
/// Ord: N/A (composite notice)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanLimitNotice {
    pub max_bytes: usize,
    pub max_lines: usize,
    pub max_hits: usize,
    pub bytes_read: usize,
    pub lines_read: usize,
    pub hits: usize,
    pub byte_limit_reached: bool,
    pub line_limit_reached: bool,
    pub hit_limit_reached: bool,
}

/// Limited read outcome with optional notice.
///
/// Tier: T2-C (Cross-domain composite read outcome)
/// Grounds to: T1 primitives (String, Option)
/// Ord: N/A (composite outcome)
#[derive(Debug, Clone)]
pub struct ReadOutcome {
    pub content: String,
    pub notice: Option<ScanLimitNotice>,
}

/// Scan limiter for hit caps.
///
/// Tier: T2-C (Cross-domain composite limiter)
/// Grounds to: T1 primitives (usize, bool)
/// Ord: N/A (composite limiter)
#[derive(Debug, Clone)]
pub struct ScanLimiter {
    max_hits: usize,
    hits: usize,
    hit_limit_reached: bool,
}

impl ScanLimiter {
    #[must_use]
    pub fn new(max_hits: usize) -> Self {
        Self {
            max_hits,
            hits: 0,
            hit_limit_reached: false,
        }
    }

    /// Attempt to record a hit. Returns false if limit reached.
    pub fn allow(&mut self) -> bool {
        if self.hits >= self.max_hits {
            self.hit_limit_reached = true;
            return false;
        }
        self.hits += 1;
        true
    }

    #[must_use]
    pub fn hits(&self) -> usize {
        self.hits
    }

    #[must_use]
    pub fn notice(&self, limits: ScanLimits) -> Option<ScanLimitNotice> {
        if self.hit_limit_reached {
            Some(ScanLimitNotice {
                max_bytes: limits.max_bytes,
                max_lines: limits.max_lines,
                max_hits: limits.max_hits,
                bytes_read: 0,
                lines_read: 0,
                hits: self.hits,
                byte_limit_reached: false,
                line_limit_reached: false,
                hit_limit_reached: true,
            })
        } else {
            None
        }
    }
}

/// Read a file with byte/line caps.
pub fn read_limited_file(path: &Path, limits: ScanLimits) -> io::Result<ReadOutcome> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    let mut bytes_read = 0usize;
    let mut lines_read = 0usize;
    let mut byte_limit_reached = false;
    let mut line_limit_reached = false;
    let mut buf = String::new();

    loop {
        buf.clear();
        let read = reader.read_line(&mut buf)?;
        if read == 0 {
            break;
        }
        if lines_read >= limits.max_lines {
            line_limit_reached = true;
            break;
        }
        let remaining = limits.max_bytes.saturating_sub(bytes_read);
        if remaining == 0 {
            byte_limit_reached = true;
            break;
        }
        if buf.len() > remaining {
            let truncated = truncate_to_bytes(&buf, remaining);
            content.push_str(&truncated);
            bytes_read += truncated.len();
            lines_read += 1;
            byte_limit_reached = true;
            break;
        }
        content.push_str(&buf);
        bytes_read += buf.len();
        lines_read += 1;
    }

    let notice = if byte_limit_reached || line_limit_reached {
        Some(ScanLimitNotice {
            max_bytes: limits.max_bytes,
            max_lines: limits.max_lines,
            max_hits: limits.max_hits,
            bytes_read,
            lines_read,
            hits: 0,
            byte_limit_reached,
            line_limit_reached,
            hit_limit_reached: false,
        })
    } else {
        None
    };

    Ok(ReadOutcome { content, notice })
}

fn read_env_usize(var: &str, default: usize) -> usize {
    env::var(var)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn truncate_to_bytes(text: &str, max_bytes: usize) -> String {
    if text.len() <= max_bytes {
        return text.to_string();
    }
    let mut end = 0usize;
    for (idx, ch) in text.char_indices() {
        if idx >= max_bytes {
            break;
        }
        end = idx + ch.len_utf8();
    }
    text[..end].to_string()
}

fn truncate_snippet(text: &str, max_bytes: usize, max_lines: usize) -> Snippet {
    let mut truncated = false;
    let mut out = String::new();
    let mut lines = 0usize;

    for chunk in text.split_inclusive('\n') {
        if lines >= max_lines {
            truncated = true;
            break;
        }
        out.push_str(chunk);
        lines += 1;
    }

    if out.len() > max_bytes {
        out = truncate_to_bytes(&out, max_bytes);
        truncated = true;
    }

    Snippet {
        text: out.clone(),
        truncated,
        bytes: out.len(),
        lines: out.lines().count(),
    }
}

/// Create a standardized snippet using configured limits.
pub fn snippet_for(text: &str, limits: ScanLimits) -> Snippet {
    truncate_snippet(text, limits.max_snippet_bytes, limits.max_snippet_lines)
}
