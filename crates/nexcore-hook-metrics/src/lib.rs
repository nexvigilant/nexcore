//! Hook execution metrics collection and analysis.
//!
//! # Codex Compliance
//!
//! - **Tier**: T2-C (Cross-Domain Composite)
//! - **Grounding**: Types ground to T1 (String, u64, u8, bool) via T2-P newtypes.
//! - **Quantification**: Exit codes enumerated, durations measured.
//!
//! # Usage
//!
//! ```rust,ignore
//! use hook_metrics_lib::{record_execution, HookEvent};
//! use std::time::Instant;
//!
//! let start = Instant::now();
//! // ... hook logic ...
//! let duration_ms = start.elapsed().as_millis() as u64;
//! record_execution("my-hook", HookEvent::PreToolUse, duration_ms, 0);
//! ```

use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

// ============================================================================
// T2-P Newtypes (Primitives)
// ============================================================================

/// Hook name identifier.
///
/// # Tier: T2-P
/// Grounds to: T1(String).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct HookName(pub String);

impl From<&str> for HookName {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for HookName {
    fn from(s: String) -> Self {
        Self(s)
    }
}

/// Duration in milliseconds.
///
/// # Tier: T2-P
/// Grounds to: T1(u64).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DurationMs(pub u64);

impl From<u64> for DurationMs {
    fn from(ms: u64) -> Self {
        Self(ms)
    }
}

/// Exit code from hook execution.
///
/// # Tier: T2-P
/// Grounds to: T1(u8).
/// Values: 0=pass, 1=warn, 2=block.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ExitCode {
    Pass = 0,
    Warn = 1,
    Block = 2,
}

impl From<i32> for ExitCode {
    fn from(code: i32) -> Self {
        match code {
            0 => Self::Pass,
            1 => Self::Warn,
            _ => Self::Block, // Treat 2+ and negatives as block
        }
    }
}

impl From<ExitCode> for u8 {
    fn from(code: ExitCode) -> Self {
        code as u8
    }
}

// ============================================================================
// T2-P Hook Event Types
// ============================================================================

/// Hook lifecycle event type.
///
/// # Tier: T2-P
/// Grounds to: T1(String) via serde.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HookEvent {
    SessionStart,
    SessionEnd,
    UserPromptSubmit,
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PermissionRequest,
    PreCompact,
    Stop,
    Setup,
    SubagentStart,
    SubagentStop,
    Notification,
}

impl From<&str> for HookEvent {
    fn from(s: &str) -> Self {
        match s {
            "SessionStart" => Self::SessionStart,
            "SessionEnd" => Self::SessionEnd,
            "UserPromptSubmit" => Self::UserPromptSubmit,
            "PreToolUse" => Self::PreToolUse,
            "PostToolUse" => Self::PostToolUse,
            "PostToolUseFailure" => Self::PostToolUseFailure,
            "PermissionRequest" => Self::PermissionRequest,
            "PreCompact" => Self::PreCompact,
            "Stop" => Self::Stop,
            "Setup" => Self::Setup,
            "SubagentStart" => Self::SubagentStart,
            "SubagentStop" => Self::SubagentStop,
            "Notification" => Self::Notification,
            _ => Self::PreToolUse, // Default fallback
        }
    }
}

// ============================================================================
// T2-C Composite Types
// ============================================================================

/// Single hook execution record.
///
/// # Tier: T2-C
/// Grounds to: T2-P(HookName, DurationMs, ExitCode, HookEvent) -> T1(String, u64, u8).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// ISO-8601 timestamp
    pub timestamp: String,
    /// Hook name
    pub hook: HookName,
    /// Event type that triggered the hook
    pub event: HookEvent,
    /// Execution duration in milliseconds
    pub duration_ms: DurationMs,
    /// Exit code (0=pass, 1=warn, 2=block)
    pub exit_code: u8,
    /// Whether the hook blocked the action
    pub blocked: bool,
}

impl ExecutionRecord {
    /// Create a new execution record.
    pub fn new(
        hook: impl Into<HookName>,
        event: HookEvent,
        duration_ms: impl Into<DurationMs>,
        exit_code: ExitCode,
    ) -> Self {
        let now = chrono_lite_now();
        let exit_u8: u8 = exit_code.into();
        Self {
            timestamp: now,
            hook: hook.into(),
            event,
            duration_ms: duration_ms.into(),
            exit_code: exit_u8,
            blocked: exit_code == ExitCode::Block,
        }
    }
}

// ============================================================================
// Timestamp Generation (chrono-free)
// ============================================================================

/// Get current Unix timestamp in seconds.
fn unix_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Check if year is a leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// Days in each month (non-leap year). Used by `month_day_from_days`.
#[allow(dead_code)]
const MONTH_DAYS: [i32; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Calculate year from days since epoch.
fn year_from_days(mut days: i32) -> (i32, i32) {
    let mut year = 1970;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            return (year, days);
        }
        days -= days_in_year;
        year += 1;
    }
}

/// Calculate month and day from remaining days in year.
fn month_day_from_days(mut days: i32, leap: bool) -> (u32, i32) {
    let feb_days = if leap { 29 } else { 28 };
    let month_lens = [31, feb_days, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    for (i, &len) in month_lens.iter().enumerate() {
        if days < len {
            return ((i + 1) as u32, days + 1);
        }
        days -= len;
    }
    (12, 31) // Fallback
}

/// Generate ISO-8601 timestamp without chrono dependency.
fn chrono_lite_now() -> String {
    let secs = unix_secs();
    let days = (secs / 86400) as i32;
    let time_secs = secs % 86400;

    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;

    let (year, rem_days) = year_from_days(days);
    let (month, day) = month_day_from_days(rem_days, is_leap_year(year));

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

// ============================================================================
// Metrics File Operations
// ============================================================================

/// Default telemetry file path.
pub fn telemetry_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".claude")
        .join("brain")
        .join("telemetry")
        .join("hook_executions.jsonl")
}

/// Record a hook execution to the telemetry file.
///
/// This is the primary API for hooks to report their execution metrics.
pub fn record_execution(hook_name: &str, event: HookEvent, duration_ms: u64, exit_code: i32) {
    let record = ExecutionRecord::new(hook_name, event, duration_ms, ExitCode::from(exit_code));

    if let Err(e) = append_record(&record) {
        eprintln!("[hook-metrics] Failed to record: {}", e);
    }
}

/// Append a record to the telemetry file.
fn append_record(record: &ExecutionRecord) -> std::io::Result<()> {
    let path = telemetry_path();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

    let line = serde_json::to_string(record)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;

    writeln!(file, "{}", line)
}

// ============================================================================
// Analysis Types
// ============================================================================

/// Percentile statistics for hook execution times.
///
/// # Tier: T2-C
/// Grounds to: T1(f64).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PercentileStats {
    pub p50: f64,
    pub p95: f64,
    pub p99: f64,
    pub min: f64,
    pub max: f64,
    pub count: usize,
}

impl PercentileStats {
    /// Calculate percentile stats from a sorted slice of durations.
    pub fn from_sorted(values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        let count = values.len();
        Some(Self {
            p50: percentile_value(values, 0.50),
            p95: percentile_value(values, 0.95),
            p99: percentile_value(values, 0.99),
            min: values.first().copied().unwrap_or(0.0),
            max: values.last().copied().unwrap_or(0.0),
            count,
        })
    }
}

/// Get value at percentile from sorted slice.
fn percentile_value(sorted: &[f64], p: f64) -> f64 {
    let idx = ((sorted.len() as f64) * p).floor() as usize;
    sorted
        .get(idx.min(sorted.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0.0)
}

/// Per-hook metrics summary.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMetricsSummary {
    pub hook: String,
    pub execution_count: usize,
    pub block_count: usize,
    pub warn_count: usize,
    pub block_rate: f64,
    pub timing: PercentileStats,
}

/// Overall metrics summary.
///
/// # Tier: T2-C
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub total_executions: usize,
    pub total_blocks: usize,
    pub total_warns: usize,
    pub unique_hooks: usize,
    pub by_hook: Vec<HookMetricsSummary>,
    pub by_event: std::collections::HashMap<String, usize>,
    pub slowest_hooks: Vec<(String, f64)>,
}

/// Load and parse all execution records.
pub fn load_records() -> Vec<ExecutionRecord> {
    let path = telemetry_path();
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

// ============================================================================
// Summary Generation Primitives
// ============================================================================

use std::collections::HashMap;

/// Group records by hook name.
fn group_by_hook(records: &[ExecutionRecord]) -> HashMap<String, Vec<&ExecutionRecord>> {
    let mut map: HashMap<String, Vec<&ExecutionRecord>> = HashMap::new();
    for record in records {
        map.entry(record.hook.0.clone()).or_default().push(record);
    }
    map
}

/// Count records by event type.
fn count_by_event(records: &[ExecutionRecord]) -> HashMap<String, usize> {
    let mut map: HashMap<String, usize> = HashMap::new();
    for record in records {
        *map.entry(format!("{:?}", record.event)).or_insert(0) += 1;
    }
    map
}

/// Calculate hook summary from its records.
fn summarize_hook(hook_name: &str, records: &[&ExecutionRecord]) -> Option<HookMetricsSummary> {
    let mut durations: Vec<f64> = records.iter().map(|r| r.duration_ms.0 as f64).collect();
    durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let timing = PercentileStats::from_sorted(&durations)?;
    let block_count = records.iter().filter(|r| r.blocked).count();
    let warn_count = records.iter().filter(|r| r.exit_code == 1).count();
    let execution_count = records.len();

    Some(HookMetricsSummary {
        hook: hook_name.to_string(),
        execution_count,
        block_count,
        warn_count,
        block_rate: if execution_count > 0 {
            block_count as f64 / execution_count as f64
        } else {
            0.0
        },
        timing,
    })
}

/// Build list of hook summaries and totals.
fn build_summaries(
    by_hook: &HashMap<String, Vec<&ExecutionRecord>>,
) -> (Vec<HookMetricsSummary>, usize, usize) {
    let mut summaries = Vec::new();
    let mut total_blocks = 0usize;
    let mut total_warns = 0usize;

    for (hook_name, hook_records) in by_hook {
        if let Some(summary) = summarize_hook(hook_name, hook_records) {
            total_blocks += summary.block_count;
            total_warns += summary.warn_count;
            summaries.push(summary);
        }
    }

    summaries.sort_by(|a, b| {
        b.timing
            .p99
            .partial_cmp(&a.timing.p99)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    (summaries, total_blocks, total_warns)
}

/// Extract slowest hooks from sorted summaries.
fn extract_slowest(summaries: &[HookMetricsSummary], limit: usize) -> Vec<(String, f64)> {
    summaries
        .iter()
        .take(limit)
        .map(|s| (s.hook.clone(), s.timing.p99))
        .collect()
}

/// Generate metrics summary from all records.
pub fn generate_summary() -> MetricsSummary {
    let records = load_records();
    let by_hook = group_by_hook(&records);
    let by_event = count_by_event(&records);
    let (summaries, total_blocks, total_warns) = build_summaries(&by_hook);
    let slowest = extract_slowest(&summaries, 10);

    MetricsSummary {
        total_executions: records.len(),
        total_blocks,
        total_warns,
        unique_hooks: by_hook.len(),
        by_hook: summaries,
        by_event,
        slowest_hooks: slowest,
    }
}

/// Generate summary filtered by event type.
pub fn generate_summary_by_event(event: HookEvent) -> MetricsSummary {
    let records: Vec<ExecutionRecord> = load_records()
        .into_iter()
        .filter(|r| r.event == event)
        .collect();

    let by_hook = group_by_hook(&records);
    let by_event = count_by_event(&records);
    let (summaries, total_blocks, total_warns) = build_summaries(&by_hook);
    let slowest = extract_slowest(&summaries, 10);

    MetricsSummary {
        total_executions: records.len(),
        total_blocks,
        total_warns,
        unique_hooks: by_hook.len(),
        by_hook: summaries,
        by_event,
        slowest_hooks: slowest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_from_i32() {
        assert_eq!(ExitCode::from(0), ExitCode::Pass);
        assert_eq!(ExitCode::from(1), ExitCode::Warn);
        assert_eq!(ExitCode::from(2), ExitCode::Block);
        assert_eq!(ExitCode::from(99), ExitCode::Block);
    }

    #[test]
    fn test_hook_event_from_str() {
        assert_eq!(HookEvent::from("PreToolUse"), HookEvent::PreToolUse);
        assert_eq!(HookEvent::from("SessionStart"), HookEvent::SessionStart);
        assert_eq!(HookEvent::from("Stop"), HookEvent::Stop);
    }

    #[test]
    fn test_execution_record_creation() {
        let record = ExecutionRecord::new("test-hook", HookEvent::PreToolUse, 5u64, ExitCode::Pass);
        assert_eq!(record.hook.0, "test-hook");
        assert_eq!(record.exit_code, 0);
        assert!(!record.blocked);
    }

    #[test]
    fn test_percentile_stats() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let stats = PercentileStats::from_sorted(&values).unwrap();
        assert_eq!(stats.min, 1.0);
        assert_eq!(stats.max, 10.0);
        assert_eq!(stats.count, 10);
        // p50 at index 5 (floor of 10 * 0.5) = 6.0
        assert_eq!(stats.p50, 6.0);
    }

    #[test]
    fn test_chrono_lite_now() {
        let timestamp = chrono_lite_now();
        assert!(timestamp.contains("T"));
        assert!(timestamp.ends_with("Z"));
        assert_eq!(timestamp.len(), 20);
    }

    #[test]
    fn test_empty_percentile_stats() {
        let values: Vec<f64> = vec![];
        assert!(PercentileStats::from_sorted(&values).is_none());
    }

    #[test]
    fn test_is_leap_year() {
        assert!(is_leap_year(2000));
        assert!(is_leap_year(2024));
        assert!(!is_leap_year(1900));
        assert!(!is_leap_year(2023));
    }
}
