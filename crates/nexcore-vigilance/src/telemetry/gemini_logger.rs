//! Gemini API call telemetry logger.
//!
//! Appends JSONL entries to ~/.claude/logs/gemini_telemetry.jsonl for
//! unified monitoring via Watchtower MCP tools.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Gemini telemetry log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiLogEntry {
    /// ISO 8601 timestamp
    pub timestamp: DateTime<Utc>,
    /// Session identifier (e.g., "vigil-abc123")
    pub session_id: String,
    /// Source system (e.g., "vigil", "nexbet")
    pub source: String,
    /// Flow or function name
    pub flow_name: String,
    /// Model used (e.g., "gemini-2.5-flash")
    pub model: String,
    /// Request latency in milliseconds
    pub latency_ms: u64,
    /// Input token count
    pub input_tokens: u64,
    /// Output token count
    pub output_tokens: u64,
    /// Total token count
    pub total_tokens: u64,
    /// Call status
    pub status: CallStatus,
    /// Error message if status is Error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Call completion status.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CallStatus {
    Success,
    Error,
}

impl GeminiLogEntry {
    /// Create a new success entry.
    #[must_use]
    pub fn success(
        session_id: impl Into<String>,
        source: impl Into<String>,
        flow_name: impl Into<String>,
        model: impl Into<String>,
        latency_ms: u64,
        input_tokens: u64,
        output_tokens: u64,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            session_id: session_id.into(),
            source: source.into(),
            flow_name: flow_name.into(),
            model: model.into(),
            latency_ms,
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
            status: CallStatus::Success,
            error: None,
        }
    }

    /// Create a new error entry.
    #[must_use]
    pub fn error(
        session_id: impl Into<String>,
        source: impl Into<String>,
        flow_name: impl Into<String>,
        model: impl Into<String>,
        latency_ms: u64,
        error_msg: impl Into<String>,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            session_id: session_id.into(),
            source: source.into(),
            flow_name: flow_name.into(),
            model: model.into(),
            latency_ms,
            input_tokens: 0,
            output_tokens: 0,
            total_tokens: 0,
            status: CallStatus::Error,
            error: Some(error_msg.into()),
        }
    }
}

/// Get the default telemetry log path.
#[must_use]
pub fn get_log_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".claude/logs/gemini_telemetry.jsonl")
}

/// Append a telemetry entry to the log file.
///
/// # Errors
/// Returns an error if the log file cannot be opened or written to.
pub fn append_log(entry: &GeminiLogEntry) -> std::io::Result<()> {
    let path = get_log_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Append as JSONL
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

    let json = serde_json::to_string(entry).unwrap_or_else(|_| "{}".to_string());
    writeln!(file, "{}", json)?;

    Ok(())
}

/// Read recent telemetry entries (last N).
///
/// # Errors
/// Returns an error if the log file cannot be read.
pub fn read_recent(count: usize) -> std::io::Result<Vec<GeminiLogEntry>> {
    let path = get_log_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&path)?;
    let entries: Vec<GeminiLogEntry> = content
        .lines()
        .rev()
        .take(count)
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect();

    Ok(entries)
}

/// Aggregate statistics from telemetry log.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeminiStats {
    /// Total number of calls
    pub total_calls: usize,
    /// Successful calls
    pub success_count: usize,
    /// Failed calls
    pub error_count: usize,
    /// Total tokens used
    pub total_tokens: u64,
    /// Total input tokens
    pub input_tokens: u64,
    /// Total output tokens
    pub output_tokens: u64,
    /// Average latency in ms
    pub avg_latency_ms: f64,
    /// Calls by session
    pub by_session: std::collections::HashMap<String, usize>,
    /// Calls by flow
    pub by_flow: std::collections::HashMap<String, usize>,
}

/// Compute aggregated statistics from the log.
///
/// # Errors
/// Returns an error if the log file cannot be read.
pub fn compute_stats() -> std::io::Result<GeminiStats> {
    let path = get_log_path();
    if !path.exists() {
        return Ok(GeminiStats::default());
    }

    let content = fs::read_to_string(&path)?;
    let mut stats = GeminiStats::default();
    let mut total_latency: u64 = 0;

    for line in content.lines() {
        if let Ok(entry) = serde_json::from_str::<GeminiLogEntry>(line) {
            stats.total_calls += 1;
            total_latency += entry.latency_ms;
            stats.total_tokens += entry.total_tokens;
            stats.input_tokens += entry.input_tokens;
            stats.output_tokens += entry.output_tokens;

            if entry.status == CallStatus::Success {
                stats.success_count += 1;
            } else {
                stats.error_count += 1;
            }

            *stats.by_session.entry(entry.session_id).or_insert(0) += 1;
            *stats.by_flow.entry(entry.flow_name).or_insert(0) += 1;
        }
    }

    if stats.total_calls > 0 {
        stats.avg_latency_ms = total_latency as f64 / stats.total_calls as f64;
    }

    Ok(stats)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_success_entry() {
        let entry = GeminiLogEntry::success(
            "test-session",
            "vigil",
            "test_flow",
            "gemini-2.5-flash",
            150,
            100,
            50,
        );

        assert_eq!(entry.status, CallStatus::Success);
        assert_eq!(entry.total_tokens, 150);
        assert!(entry.error.is_none());
    }

    #[test]
    fn test_error_entry() {
        let entry = GeminiLogEntry::error(
            "test-session",
            "vigil",
            "test_flow",
            "gemini-2.5-flash",
            50,
            "API timeout",
        );

        assert_eq!(entry.status, CallStatus::Error);
        assert_eq!(entry.error, Some("API timeout".to_string()));
    }

    #[test]
    fn test_serialize_deserialize() {
        let entry = GeminiLogEntry::success(
            "sess-123",
            "nexbet",
            "analyzeSignal",
            "gemini-2.5-flash",
            200,
            500,
            200,
        );

        let json = serde_json::to_string(&entry).expect("serialize");
        let parsed: GeminiLogEntry = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.session_id, "sess-123");
        assert_eq!(parsed.total_tokens, 700);
    }
}
