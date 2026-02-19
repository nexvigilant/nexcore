//! MCP Telemetry Collection Module
//!
//! Tracks per-tool-call metrics with minimal overhead:
//! - Tool name
//! - Duration (start to response)
//! - Success/failure
//! - Input/output byte estimates
//!
//! Data persisted asynchronously to `~/.claude/brain/telemetry/mcp_calls.jsonl`
//!
//! Tier: T2-C (Cross-domain composite telemetry infrastructure)
//! Grounds to: T1 primitives (String, bool, u64, Instant) via measurement types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;

// ============================================================================
// Types
// ============================================================================

/// Single telemetry record for an MCP tool call.
///
/// Tier: T2-C (Cross-domain composite telemetry record)
/// Grounds to: T1 primitives (String, bool, u64)
/// Ord: By timestamp (chronological ordering)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryRecord {
    /// ISO-8601 timestamp
    pub timestamp: DateTime<Utc>,
    /// Tool name (e.g., "pv_signal_prr")
    pub tool: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Whether the call succeeded
    pub success: bool,
    /// Input parameter size estimate (bytes)
    pub input_bytes: usize,
    /// Output response size estimate (bytes)
    pub output_bytes: usize,
}

/// Aggregated statistics for telemetry summary.
///
/// Tier: T2-C (Cross-domain composite statistics)
/// Grounds to: T1 primitives (u64, f64)
/// Ord: N/A (composite statistics)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySummary {
    /// Total number of calls
    pub total_calls: u64,
    /// Number of successful calls
    pub success_count: u64,
    /// Number of failed calls
    pub failure_count: u64,
    /// Total duration across all calls (ms)
    pub total_duration_ms: u64,
    /// Average duration per call (ms)
    pub avg_duration_ms: f64,
    /// Minimum duration (ms)
    pub min_duration_ms: u64,
    /// Maximum duration (ms)
    pub max_duration_ms: u64,
    /// Total input bytes processed
    pub total_input_bytes: u64,
    /// Total output bytes generated
    pub total_output_bytes: u64,
    /// Time range start
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_call: Option<DateTime<Utc>>,
    /// Time range end
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_call: Option<DateTime<Utc>>,
}

impl Default for TelemetrySummary {
    fn default() -> Self {
        Self {
            total_calls: 0,
            success_count: 0,
            failure_count: 0,
            total_duration_ms: 0,
            avg_duration_ms: 0.0,
            min_duration_ms: u64::MAX,
            max_duration_ms: 0,
            total_input_bytes: 0,
            total_output_bytes: 0,
            first_call: None,
            last_call: None,
        }
    }
}

/// Per-tool breakdown statistics.
///
/// Tier: T2-C (Cross-domain composite per-tool stats)
/// Grounds to: T1 primitives (String, u64, f64)
/// Ord: N/A (composite statistics)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStats {
    /// Tool name
    pub tool: String,
    /// Number of calls
    pub call_count: u64,
    /// Success rate (0.0-1.0)
    pub success_rate: f64,
    /// Average duration (ms)
    pub avg_duration_ms: f64,
    /// Minimum duration (ms)
    pub min_duration_ms: u64,
    /// Maximum duration (ms)
    pub max_duration_ms: u64,
    /// P50 duration estimate (ms)
    pub p50_duration_ms: u64,
    /// P95 duration estimate (ms)
    pub p95_duration_ms: u64,
    /// Total input bytes
    pub total_input_bytes: u64,
    /// Total output bytes
    pub total_output_bytes: u64,
}

/// A slow call record.
///
/// Tier: T2-C (Cross-domain composite slow call)
/// Grounds to: T1 primitives (String, u64, DateTime)
/// Ord: By duration (descending)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowCall {
    /// Timestamp of the call
    pub timestamp: DateTime<Utc>,
    /// Tool name
    pub tool: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Whether it succeeded
    pub success: bool,
}

/// Full audit record for an MCP tool call — includes actual params and response.
///
/// Separate from `TelemetryRecord` which only captures byte counts.
/// This captures the ground truth of what was computed.
///
/// Tier: T2-C (Cross-domain composite audit record)
/// Grounds to: T1 primitives (String, bool, u64, DateTime)
/// Ord: By timestamp (chronological ordering)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRecord {
    /// ISO-8601 timestamp
    pub timestamp: DateTime<Utc>,
    /// Tool name (e.g., "pv_signal_prr")
    pub tool: String,
    /// Serialized input parameters (JSON string, truncated at 4KB)
    pub input_json: String,
    /// Serialized response content (JSON string, truncated at 4KB)
    pub output_json: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Whether the call succeeded
    pub success: bool,
    /// Error message on failure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_msg: Option<String>,
}

/// Maximum bytes for input/output JSON in audit records (4KB).
const AUDIT_MAX_JSON_LEN: usize = 4096;

/// Truncate a string to max_len bytes, appending a truncation marker.
pub fn truncate_for_audit(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        let truncated = &s[..max_len.min(s.len())];
        format!("{}...(truncated, {} total)", truncated, s.len())
    }
}

// ============================================================================
// Writer Channel (Async, Non-blocking)
// ============================================================================

/// Global sender for telemetry records (fire-and-forget).
static TELEMETRY_SENDER: OnceLock<mpsc::UnboundedSender<TelemetryRecord>> = OnceLock::new();

/// Global sender for audit records (fire-and-forget).
static AUDIT_SENDER: OnceLock<mpsc::UnboundedSender<AuditRecord>> = OnceLock::new();

/// Initialize the telemetry and audit writer background tasks.
///
/// Safe to call multiple times - only initializes once.
pub fn init_telemetry_writer() {
    TELEMETRY_SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(telemetry_writer_task(rx));
        tx
    });
    AUDIT_SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(audit_writer_task(rx));
        tx
    });
}

/// Get the telemetry file path.
fn telemetry_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude/brain/telemetry/mcp_calls.jsonl")
}

/// Background task that writes telemetry records to disk.
async fn telemetry_writer_task(mut rx: mpsc::UnboundedReceiver<TelemetryRecord>) {
    let path = telemetry_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent).await;
    }

    while let Some(record) = rx.recv().await {
        // Serialize to JSON line
        if let Ok(mut line) = serde_json::to_string(&record) {
            line.push('\n');

            // Append to file (create if needed)
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await
            {
                let _ = file.write_all(line.as_bytes()).await;
            }
        }
    }
}

/// Record a telemetry event (non-blocking, fire-and-forget).
pub fn record(record: TelemetryRecord) {
    if let Some(tx) = TELEMETRY_SENDER.get() {
        // Ignore send errors (channel closed = telemetry disabled)
        let _ = tx.send(record);
    }
}

/// Get the audit log file path.
fn audit_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude/audit/mcp_server_audit.jsonl")
}

/// Background task that writes audit records to disk.
async fn audit_writer_task(mut rx: mpsc::UnboundedReceiver<AuditRecord>) {
    let path = audit_path();

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent).await;
    }

    while let Some(record) = rx.recv().await {
        // Serialize to JSON line
        if let Ok(mut line) = serde_json::to_string(&record) {
            line.push('\n');

            // Append to file (create if needed)
            if let Ok(mut file) = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .await
            {
                let _ = file.write_all(line.as_bytes()).await;
            }
        }
    }
}

/// Record an audit event (non-blocking, fire-and-forget).
pub fn record_audit(record: AuditRecord) {
    if let Some(tx) = AUDIT_SENDER.get() {
        let _ = tx.send(record);
    }
}

/// Create an AuditRecord from tool call data.
pub fn build_audit_record(
    tool: &str,
    input_json: &str,
    output_json: &str,
    duration: Duration,
    success: bool,
    error_msg: Option<String>,
) -> AuditRecord {
    AuditRecord {
        timestamp: Utc::now(),
        tool: tool.to_string(),
        input_json: truncate_for_audit(input_json, AUDIT_MAX_JSON_LEN),
        output_json: truncate_for_audit(output_json, AUDIT_MAX_JSON_LEN),
        duration_ms: duration.as_millis() as u64,
        success,
        error_msg,
    }
}

// ============================================================================
// Measurement Helper
// ============================================================================

/// RAII guard for measuring tool call duration.
///
/// Tier: T2-C (Cross-domain composite measurement guard)
/// Grounds to: T1 primitives (String, Instant, usize)
pub struct CallMeasurement {
    tool: String,
    start: Instant,
    input_bytes: usize,
}

impl CallMeasurement {
    /// Start measuring a tool call.
    #[must_use]
    pub fn start(tool: impl Into<String>, input_bytes: usize) -> Self {
        Self {
            tool: tool.into(),
            start: Instant::now(),
            input_bytes,
        }
    }

    /// Finish measurement and record telemetry.
    pub fn finish(self, success: bool, output_bytes: usize) {
        let duration = self.start.elapsed();
        let record = TelemetryRecord {
            timestamp: Utc::now(),
            tool: self.tool,
            duration_ms: duration.as_millis() as u64,
            success,
            input_bytes: self.input_bytes,
            output_bytes,
        };
        record::record(record);
    }
}

mod record {
    pub use super::record;
}

// ============================================================================
// Query Functions
// ============================================================================

/// Read all telemetry records from disk.
pub async fn read_all_records() -> Vec<TelemetryRecord> {
    let path = telemetry_path();
    let content = match fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

/// Compute aggregate summary statistics.
pub async fn summary() -> TelemetrySummary {
    let records = read_all_records().await;
    if records.is_empty() {
        return TelemetrySummary::default();
    }

    let mut summary = TelemetrySummary {
        total_calls: records.len() as u64,
        min_duration_ms: u64::MAX,
        ..Default::default()
    };

    for r in &records {
        if r.success {
            summary.success_count += 1;
        } else {
            summary.failure_count += 1;
        }
        summary.total_duration_ms += r.duration_ms;
        summary.min_duration_ms = summary.min_duration_ms.min(r.duration_ms);
        summary.max_duration_ms = summary.max_duration_ms.max(r.duration_ms);
        summary.total_input_bytes += r.input_bytes as u64;
        summary.total_output_bytes += r.output_bytes as u64;

        if summary.first_call.is_none() || r.timestamp < summary.first_call.unwrap_or(r.timestamp) {
            summary.first_call = Some(r.timestamp);
        }
        if summary.last_call.is_none() || r.timestamp > summary.last_call.unwrap_or(r.timestamp) {
            summary.last_call = Some(r.timestamp);
        }
    }

    if summary.total_calls > 0 {
        summary.avg_duration_ms = summary.total_duration_ms as f64 / summary.total_calls as f64;
    }
    if summary.min_duration_ms == u64::MAX {
        summary.min_duration_ms = 0;
    }

    summary
}

/// Get statistics for a specific tool.
pub async fn by_tool(tool_name: &str) -> Option<ToolStats> {
    let records: Vec<_> = read_all_records()
        .await
        .into_iter()
        .filter(|r| r.tool == tool_name)
        .collect();

    if records.is_empty() {
        return None;
    }

    let call_count = records.len() as u64;
    let success_count = records.iter().filter(|r| r.success).count() as u64;
    let success_rate = success_count as f64 / call_count as f64;

    let mut durations: Vec<u64> = records.iter().map(|r| r.duration_ms).collect();
    durations.sort_unstable();

    let total_duration: u64 = durations.iter().sum();
    let avg_duration_ms = total_duration as f64 / call_count as f64;
    let min_duration_ms = *durations.first().unwrap_or(&0);
    let max_duration_ms = *durations.last().unwrap_or(&0);

    // Percentile calculations
    let p50_idx = (durations.len() as f64 * 0.50) as usize;
    let p95_idx = (durations.len() as f64 * 0.95) as usize;
    let p50_duration_ms = durations
        .get(p50_idx.min(durations.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0);
    let p95_duration_ms = durations
        .get(p95_idx.min(durations.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0);

    let total_input_bytes: u64 = records.iter().map(|r| r.input_bytes as u64).sum();
    let total_output_bytes: u64 = records.iter().map(|r| r.output_bytes as u64).sum();

    Some(ToolStats {
        tool: tool_name.to_string(),
        call_count,
        success_rate,
        avg_duration_ms,
        min_duration_ms,
        max_duration_ms,
        p50_duration_ms,
        p95_duration_ms,
        total_input_bytes,
        total_output_bytes,
    })
}

/// Get calls exceeding a duration threshold.
pub async fn slow_calls(threshold_ms: u64) -> Vec<SlowCall> {
    read_all_records()
        .await
        .into_iter()
        .filter(|r| r.duration_ms > threshold_ms)
        .map(|r| SlowCall {
            timestamp: r.timestamp,
            tool: r.tool,
            duration_ms: r.duration_ms,
            success: r.success,
        })
        .collect()
}

// ============================================================================
// Audit Query Functions
// ============================================================================

/// Read all audit records from disk.
pub async fn read_all_audit_records() -> Vec<AuditRecord> {
    let path = audit_path();
    let content = match fs::read_to_string(&path).await {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

/// Query audit records with optional filters.
pub async fn query_audit_trail(
    tool_filter: Option<&str>,
    since: Option<DateTime<Utc>>,
    success_only: Option<bool>,
    limit: Option<usize>,
) -> Vec<AuditRecord> {
    let mut records = read_all_audit_records().await;

    // Apply filters
    if let Some(tool) = tool_filter {
        records.retain(|r| r.tool == tool);
    }
    if let Some(since_ts) = since {
        records.retain(|r| r.timestamp >= since_ts);
    }
    if let Some(success) = success_only {
        records.retain(|r| r.success == success);
    }

    // Sort by timestamp descending (most recent first)
    records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    // Apply limit
    if let Some(limit) = limit {
        records.truncate(limit);
    }

    records
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_record_serialization() {
        let record = TelemetryRecord {
            timestamp: Utc::now(),
            tool: "pv_signal_prr".to_string(),
            duration_ms: 12,
            success: true,
            input_bytes: 45,
            output_bytes: 230,
        };

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("pv_signal_prr"));
        assert!(json.contains("\"duration_ms\":12"));
    }

    #[test]
    fn test_summary_default() {
        let summary = TelemetrySummary::default();
        assert_eq!(summary.total_calls, 0);
        assert_eq!(summary.success_count, 0);
    }

    #[test]
    fn test_call_measurement() {
        let measurement = CallMeasurement::start("test_tool", 100);
        assert_eq!(measurement.tool, "test_tool");
        assert_eq!(measurement.input_bytes, 100);
        // Don't call finish() in test to avoid side effects
    }

    #[test]
    fn test_audit_record_serialization() {
        let record = AuditRecord {
            timestamp: Utc::now(),
            tool: "pv_signal_prr".to_string(),
            input_json: r#"{"a":15,"b":100}"#.to_string(),
            output_json: r#"{"prr":3.2}"#.to_string(),
            duration_ms: 5,
            success: true,
            error_msg: None,
        };

        let json = serde_json::to_string(&record);
        assert!(json.is_ok());
        let json_str = json.unwrap_or_default();
        assert!(json_str.contains("pv_signal_prr"));
        assert!(json_str.contains("input_json"));
        assert!(json_str.contains("output_json"));
        // error_msg should be skipped when None
        assert!(!json_str.contains("error_msg"));
    }

    #[test]
    fn test_truncate_for_audit_short() {
        let result = truncate_for_audit("short text", 4096);
        assert_eq!(result, "short text");
    }

    #[test]
    fn test_truncate_for_audit_long() {
        let long = "x".repeat(5000);
        let result = truncate_for_audit(&long, 100);
        assert!(result.contains("truncated"));
        assert!(result.contains("5000 total"));
    }

    #[test]
    fn test_build_audit_record() {
        let record = build_audit_record(
            "pv_signal_prr",
            r#"{"a":15}"#,
            r#"{"prr":3.2}"#,
            Duration::from_millis(10),
            true,
            None,
        );
        assert_eq!(record.tool, "pv_signal_prr");
        assert!(record.success);
        assert_eq!(record.duration_ms, 10);
        assert!(record.error_msg.is_none());
    }
}
