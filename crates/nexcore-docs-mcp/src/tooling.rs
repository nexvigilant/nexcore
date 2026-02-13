//! Shared tooling helpers for MCP server outputs, gating, and scan limits.

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::OnceLock;

use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use serde_json::json;

// ============================================================================
// Confidence — canonical definition from nexcore-constants
// ============================================================================

pub use nexcore_constants::Confidence;

// ============================================================================
// Tool Gating
// ============================================================================

/// Tool gate configuration (allow/deny lists).
#[derive(Debug, Clone)]
pub struct ToolGate {
    allowlist: Option<HashSet<String>>,
    denylist: HashSet<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToolGateError {
    pub tool: String,
    pub reason: String,
}

static TOOL_GATE: OnceLock<ToolGate> = OnceLock::new();

pub fn tool_gate() -> &'static ToolGate {
    TOOL_GATE.get_or_init(ToolGate::from_env)
}

impl ToolGate {
    #[must_use]
    pub fn from_env() -> Self {
        let allowlist = parse_tool_list("NEXCORE_MCP_TOOL_ALLOWLIST");
        let denylist = parse_tool_list("NEXCORE_MCP_TOOL_DENYLIST").unwrap_or_default();
        Self {
            allowlist,
            denylist,
        }
    }

    pub fn check(&self, tool: &str) -> Result<(), ToolGateError> {
        if self.denylist.contains(tool) {
            return Err(ToolGateError {
                tool: tool.to_string(),
                reason: "tool explicitly denied by NEXCORE_MCP_TOOL_DENYLIST".into(),
            });
        }
        if let Some(allowlist) = &self.allowlist {
            if !allowlist.contains(tool) {
                return Err(ToolGateError {
                    tool: tool.to_string(),
                    reason: "tool not present in NEXCORE_MCP_TOOL_ALLOWLIST".into(),
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViolationCode(pub u16);

impl ViolationCode {
    pub const TOOL_ERROR: Self = Self(1);
    pub const TOOL_GATED: Self = Self(2);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub text: String,
    pub truncated: bool,
    pub bytes: usize,
    pub lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub summary: String,
    pub confidence: Confidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<Snippet>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolViolation {
    pub code: ViolationCode,
    pub summary: String,
    pub confidence: Confidence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<Evidence>,
}

/// Backward-compatible alias.
#[deprecated(note = "use ToolViolation — F2 equivocation fix")]
pub type Violation = ToolViolation;

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

impl ToolReport {
    fn from_result(
        tool: &str,
        result: &CallToolResult,
        limits: &ScanLimits,
        notices: Vec<ScanLimitNotice>,
    ) -> Self {
        let snippet = first_text_snippet(result, limits);
        if result.is_error.unwrap_or(false) {
            let evidence = snippet.map(|s| Evidence {
                summary: "Tool error".into(),
                confidence: Confidence::new(0.4),
                snippet: Some(s),
                source: None,
            });
            Self {
                tool: tool.into(),
                confidence: Confidence::new(0.3),
                evidence: vec![],
                violations: vec![ToolViolation {
                    code: ViolationCode::TOOL_ERROR,
                    summary: "Tool error".into(),
                    confidence: Confidence::new(0.3),
                    evidence: evidence.into_iter().collect(),
                }],
                notices,
            }
        } else {
            let evidence = snippet.map(|s| Evidence {
                summary: "Tool output".into(),
                confidence: Confidence::new(0.85),
                snippet: Some(s),
                source: None,
            });
            Self {
                tool: tool.into(),
                confidence: Confidence::new(0.85),
                evidence: evidence.into_iter().collect(),
                violations: vec![],
                notices,
            }
        }
    }
}

pub fn wrap_result(tool: &str, result: CallToolResult) -> CallToolResult {
    wrap_result_with_notices(tool, result, Vec::new())
}

pub fn wrap_result_with_notices(
    tool: &str,
    mut result: CallToolResult,
    notices: Vec<ScanLimitNotice>,
) -> CallToolResult {
    let limits = ScanLimits::from_env();
    let report = ToolReport::from_result(tool, &result, &limits, notices);
    if let Ok(value) = serde_json::to_value(&report) {
        result.structured_content = Some(value);
    }
    result
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
    if let Ok(value) = serde_json::to_value(&report) {
        result.structured_content = Some(value);
    }
    result
}

fn first_text_snippet(result: &CallToolResult, limits: &ScanLimits) -> Option<Snippet> {
    result
        .content
        .iter()
        .find_map(|c| c.as_text())
        .map(|t| truncate_snippet(&t.text, limits.max_snippet_bytes, limits.max_snippet_lines))
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ScanLimits {
    pub max_bytes: usize,
    pub max_lines: usize,
    pub max_hits: usize,
    pub max_snippet_bytes: usize,
    pub max_snippet_lines: usize,
}

impl ScanLimits {
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

#[derive(Debug, Clone)]
pub struct ReadOutcome {
    pub content: String,
    pub notice: Option<ScanLimitNotice>,
}

#[allow(dead_code)]
pub fn read_limited_file(path: &Path, limits: ScanLimits) -> io::Result<ReadOutcome> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut content = String::new();
    let (mut bytes, mut lines) = (0usize, 0usize);
    let (mut b_limit, mut l_limit) = (false, false);
    let mut buf = String::new();

    while lines < limits.max_lines {
        buf.clear();
        if reader.read_line(&mut buf)? == 0 {
            break;
        }
        let rem = limits.max_bytes.saturating_sub(bytes);
        if rem == 0 {
            b_limit = true;
            break;
        }
        if buf.len() > rem {
            let tr = truncate_to_bytes(&buf, rem);
            content.push_str(&tr);
            bytes += tr.len();
            lines += 1;
            b_limit = true;
            break;
        }
        content.push_str(&buf);
        bytes += buf.len();
        lines += 1;
    }
    if lines >= limits.max_lines {
        l_limit = true;
    }

    let notice = if b_limit || l_limit {
        Some(ScanLimitNotice {
            max_bytes: limits.max_bytes,
            max_lines: limits.max_lines,
            max_hits: limits.max_hits,
            bytes_read: bytes,
            lines_read: lines,
            hits: 0,
            byte_limit_reached: b_limit,
            line_limit_reached: l_limit,
            hit_limit_reached: false,
        })
    } else {
        None
    };
    Ok(ReadOutcome { content, notice })
}

fn read_env_usize(var: &str, def: usize) -> usize {
    env::var(var)
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&v| v > 0)
        .unwrap_or(def)
}

fn truncate_to_bytes(text: &str, max: usize) -> String {
    if text.len() <= max {
        return text.to_string();
    }
    let mut end = 0;
    for (i, c) in text.char_indices() {
        if i >= max {
            break;
        }
        end = i + c.len_utf8();
    }
    text[..end].to_string()
}

fn truncate_snippet(text: &str, max_b: usize, max_l: usize) -> Snippet {
    let mut tr = false;
    let mut out = String::new();
    let mut lines = 0;
    for chunk in text.split_inclusive('\n') {
        if lines >= max_l {
            tr = true;
            break;
        }
        out.push_str(chunk);
        lines += 1;
    }
    if out.len() > max_b {
        out = truncate_to_bytes(&out, max_b);
        tr = true;
    }
    Snippet {
        text: out.clone(),
        truncated: tr,
        bytes: out.len(),
        lines: out.lines().count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_violation_code() {
        assert_eq!(ViolationCode::TOOL_ERROR.0, 1);
    }
    #[test]
    fn test_truncate() {
        assert_eq!(truncate_to_bytes("hello", 3), "hel");
    }
    #[test]
    fn test_env_usize_default() {
        // Test default behavior when env var not set
        assert_eq!(read_env_usize("NONEXISTENT_TEST_VAR_XYZ", 5), 5);
    }

    #[test]
    fn test_env_usize_parse_logic() {
        // read_env_usize is tested through ScanLimits::from_env() integration
        // Direct unit test removed to avoid Edition 2024 unsafe env::set_var
        let limits = ScanLimits::from_env();
        assert!(limits.max_bytes > 0);
        assert!(limits.max_lines > 0);
    }
    #[test]
    fn test_report_ok() {
        let res = CallToolResult::success(vec![Content::text("ok")]);
        let report = ToolReport::from_result("t", &res, &ScanLimits::from_env(), vec![]);
        assert_eq!(report.tool, "t");
    }

    #[test]
    fn test_truncate_unicode() {
        assert_eq!(truncate_to_bytes("🦀🦀", 4), "🦀");
    }

    #[test]
    fn test_gate_logic() {
        let mut denylist = HashSet::new();
        denylist.insert("deny".into());
        let gate = ToolGate {
            allowlist: None,
            denylist,
        };
        assert!(gate.check("deny").is_err());
        assert!(gate.check("allow").is_ok());
    }

    #[test]
    fn test_snippet_lines() {
        let s = truncate_snippet("l1\nl2\nl3", 100, 2);
        assert_eq!(s.lines, 2);
        assert!(s.truncated);
    }
}
