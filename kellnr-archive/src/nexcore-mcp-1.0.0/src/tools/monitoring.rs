//! Monitoring MCP tools — queryable alerting from telemetry files.
//!
//! Computes anomaly scores from hook telemetry, code health metrics,
//! and biological signal data. Makes monitoring actionable through
//! Claude Code's MCP query mechanism.
//!
//! # Tier: T3 (Domain-Specific MCP tools)
//! # Grounding: κ (Comparison) + ∂ (Boundary) + Σ (Sum)

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde::Deserialize;
use std::collections::HashMap;

use crate::params::{
    MonitoringAlertsParams, MonitoringHookHealthParams, MonitoringSignalDigestParams,
};

// ============================================================================
// Anomaly Thresholds
// ============================================================================

/// Hook block rate warning threshold.
const HOOK_BLOCK_WARN: f64 = 0.40;
/// Hook block rate critical threshold.
const HOOK_BLOCK_CRITICAL: f64 = 0.60;
/// Hook execution slow threshold in ms.
const HOOK_SLOW_MS: u64 = 5000;
/// Signal file size info threshold in bytes (5MB).
const SIGNAL_SIZE_INFO: u64 = 5 * 1024 * 1024;
/// Max hook telemetry age before WARN (seconds).
const HOOK_STALE_SECS: u64 = 3600;

// ============================================================================
// Data structures
// ============================================================================

#[derive(Debug, Clone, Deserialize)]
struct HookRecord {
    #[serde(default)]
    hook: String,
    #[serde(default)]
    event: String,
    #[serde(default)]
    duration_ms: u64,
    #[serde(default)]
    blocked: bool,
    #[serde(default)]
    timestamp: String,
}

#[derive(Debug, Clone, Deserialize)]
struct SignalRecord {
    #[serde(default)]
    signal_type: String,
    #[serde(default)]
    timestamp_ms: u128,
    #[serde(default)]
    data: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
struct Capabilities {
    #[serde(default)]
    overall_score: Option<f64>,
    #[serde(default)]
    test_count: Option<u64>,
    #[serde(default)]
    previous_score: Option<f64>,
    #[serde(default)]
    previous_test_count: Option<u64>,
}

// ============================================================================
// Alert types
// ============================================================================

#[derive(Debug, Clone, serde::Serialize)]
struct Alert {
    severity: &'static str,
    source: &'static str,
    message: String,
    details: HashMap<String, String>,
}

impl Alert {
    fn new(severity: &'static str, source: &'static str, message: impl Into<String>) -> Self {
        Self {
            severity,
            source,
            message: message.into(),
            details: HashMap::new(),
        }
    }

    fn with_detail(mut self, key: &str, value: impl Into<String>) -> Self {
        self.details.insert(key.to_string(), value.into());
        self
    }
}

// ============================================================================
// Helper: load telemetry files
// ============================================================================

fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
}

fn hook_telemetry_path() -> String {
    format!(
        "{}/.claude/brain/telemetry/hook_executions.jsonl",
        home_dir()
    )
}

fn signals_path() -> String {
    format!("{}/.claude/brain/telemetry/signals.jsonl", home_dir())
}

fn capabilities_path() -> String {
    format!("{}/.claude/metrics/capabilities.json", home_dir())
}

fn load_hook_records(limit: usize) -> Vec<HookRecord> {
    let path = hook_telemetry_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(limit);
    lines[start..]
        .iter()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

fn load_signal_records(limit: usize) -> Vec<SignalRecord> {
    let path = signals_path();
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let lines: Vec<&str> = content.lines().collect();
    let start = lines.len().saturating_sub(limit);
    lines[start..]
        .iter()
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

fn load_capabilities() -> Option<Capabilities> {
    let content = std::fs::read_to_string(capabilities_path()).ok()?;
    serde_json::from_str(&content).ok()
}

// ============================================================================
// Anomaly computation
// ============================================================================

fn compute_hook_alerts(records: &[HookRecord]) -> Vec<Alert> {
    let mut alerts = Vec::new();

    // Per-hook block rates
    let mut stats: HashMap<String, (usize, usize)> = HashMap::new();
    for r in records {
        let entry = stats.entry(r.hook.clone()).or_insert((0, 0));
        entry.0 += 1;
        if r.blocked {
            entry.1 += 1;
        }
    }

    for (hook, (total, blocked)) in &stats {
        if *total < 5 {
            continue;
        }
        let rate = *blocked as f64 / *total as f64;
        if rate > HOOK_BLOCK_CRITICAL {
            alerts.push(
                Alert::new(
                    "CRITICAL",
                    "hook-telemetry",
                    format!("Hook '{}' block rate {:.0}%", hook, rate * 100.0),
                )
                .with_detail("hook", hook.as_str())
                .with_detail("block_rate", format!("{:.2}", rate))
                .with_detail("total", total.to_string())
                .with_detail("blocked", blocked.to_string()),
            );
        } else if rate > HOOK_BLOCK_WARN {
            alerts.push(
                Alert::new(
                    "WARN",
                    "hook-telemetry",
                    format!("Hook '{}' block rate {:.0}%", hook, rate * 100.0),
                )
                .with_detail("hook", hook.as_str())
                .with_detail("block_rate", format!("{:.2}", rate))
                .with_detail("total", total.to_string())
                .with_detail("blocked", blocked.to_string()),
            );
        }
    }

    // Slow hooks
    let mut slow: HashMap<String, u64> = HashMap::new();
    for r in records {
        if r.duration_ms > HOOK_SLOW_MS {
            let max = slow.entry(r.hook.clone()).or_insert(0);
            if r.duration_ms > *max {
                *max = r.duration_ms;
            }
        }
    }
    for (hook, max_ms) in &slow {
        alerts.push(
            Alert::new(
                "WARN",
                "hook-telemetry",
                format!("Hook '{}' slow: {}ms", hook, max_ms),
            )
            .with_detail("hook", hook.as_str())
            .with_detail("max_duration_ms", max_ms.to_string()),
        );
    }

    // Check staleness
    let path = hook_telemetry_path();
    if let Ok(metadata) = std::fs::metadata(&path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(age) = std::time::SystemTime::now().duration_since(modified) {
                if age.as_secs() > HOOK_STALE_SECS {
                    alerts.push(
                        Alert::new(
                            "WARN",
                            "hook-telemetry",
                            format!("Hook telemetry stale: {}h old", age.as_secs() / 3600),
                        )
                        .with_detail("age_hours", (age.as_secs() / 3600).to_string()),
                    );
                }
            }
        }
    } else {
        alerts.push(Alert::new(
            "WARN",
            "hook-telemetry",
            "Hook telemetry file missing",
        ));
    }

    alerts
}

fn compute_code_health_alerts() -> Vec<Alert> {
    let mut alerts = Vec::new();
    let caps = match load_capabilities() {
        Some(c) => c,
        None => {
            alerts.push(Alert::new(
                "INFO",
                "code-health",
                "Capabilities metrics file missing",
            ));
            return alerts;
        }
    };

    // Score degradation
    if let (Some(current), Some(previous)) = (caps.overall_score, caps.previous_score) {
        if previous > 0.0 {
            let delta = (previous - current) / previous;
            if delta > 0.40 {
                alerts.push(
                    Alert::new(
                        "CRITICAL",
                        "code-health",
                        format!("Score degraded {:.1}%", delta * 100.0),
                    )
                    .with_detail("current", format!("{:.2}", current))
                    .with_detail("previous", format!("{:.2}", previous)),
                );
            } else if delta > 0.20 {
                alerts.push(
                    Alert::new(
                        "WARN",
                        "code-health",
                        format!("Score degraded {:.1}%", delta * 100.0),
                    )
                    .with_detail("current", format!("{:.2}", current))
                    .with_detail("previous", format!("{:.2}", previous)),
                );
            }
        }
    }

    // Test regression
    if let (Some(current), Some(previous)) = (caps.test_count, caps.previous_test_count) {
        if current < previous {
            let lost = previous - current;
            let sev = if lost > 100 {
                "CRITICAL"
            } else if lost > 20 {
                "WARN"
            } else {
                "INFO"
            };
            alerts.push(
                Alert::new(
                    sev,
                    "code-health",
                    format!("{} tests lost ({} → {})", lost, previous, current),
                )
                .with_detail("current_tests", current.to_string())
                .with_detail("previous_tests", previous.to_string()),
            );
        }
    }

    alerts
}

fn compute_signal_alerts(records: &[SignalRecord]) -> Vec<Alert> {
    let mut alerts = Vec::new();

    // File size check
    let path = signals_path();
    if let Ok(metadata) = std::fs::metadata(&path) {
        let size = metadata.len();
        if size > SIGNAL_SIZE_INFO {
            let size_mb = size as f64 / (1024.0 * 1024.0);
            alerts.push(
                Alert::new(
                    "INFO",
                    "signal-health",
                    format!("Signal file {:.1}MB", size_mb),
                )
                .with_detail("size_bytes", size.to_string()),
            );
        }
    }

    // Pro-inflammatory burst detection
    let pro_inflam: Vec<&SignalRecord> = records
        .iter()
        .filter(|r| {
            r.signal_type.starts_with("cytokine:")
                && ["il1", "il6", "tnf_alpha", "ifn_gamma"]
                    .iter()
                    .any(|f| r.signal_type.contains(f))
        })
        .collect();

    if pro_inflam.len() > 5 {
        // Check if recent 5 are within 60 seconds
        let recent = &pro_inflam[pro_inflam.len().saturating_sub(5)..];
        if recent.len() >= 5 {
            let first_ts = recent.first().map(|r| r.timestamp_ms).unwrap_or(0);
            let last_ts = recent.last().map(|r| r.timestamp_ms).unwrap_or(0);
            let window = last_ts.saturating_sub(first_ts);
            if window <= 60_000 {
                alerts.push(
                    Alert::new(
                        "HIGH",
                        "signal-health",
                        format!("Cytokine burst: {} in {}s", recent.len(), window / 1000),
                    )
                    .with_detail("count", recent.len().to_string())
                    .with_detail("window_ms", window.to_string()),
                );
            }
        }
    }

    // Circuit breaker open detection
    let mut cb_open: Vec<String> = Vec::new();
    for r in records {
        if r.signal_type.contains("circuit_breaker")
            && r.data.get("state").map_or(false, |s| s == "open")
        {
            let sub = r
                .data
                .get("subsystem")
                .cloned()
                .unwrap_or_else(|| "unknown".to_string());
            if !cb_open.contains(&sub) {
                cb_open.push(sub);
            }
        }
    }
    for sub in &cb_open {
        alerts.push(
            Alert::new(
                "HIGH",
                "signal-health",
                format!("Circuit breaker OPEN: {}", sub),
            )
            .with_detail("subsystem", sub.as_str()),
        );
    }

    alerts
}

// ============================================================================
// Tool Functions
// ============================================================================

/// `monitoring_health_check` — overall system health status.
pub fn health_check() -> Result<CallToolResult, McpError> {
    let hook_records = load_hook_records(100);
    let signal_records = load_signal_records(500);

    let mut all_alerts = Vec::new();
    all_alerts.extend(compute_hook_alerts(&hook_records));
    all_alerts.extend(compute_code_health_alerts());
    all_alerts.extend(compute_signal_alerts(&signal_records));

    let critical_count = all_alerts
        .iter()
        .filter(|a| a.severity == "CRITICAL")
        .count();
    let high_count = all_alerts.iter().filter(|a| a.severity == "HIGH").count();
    let warn_count = all_alerts.iter().filter(|a| a.severity == "WARN").count();
    let info_count = all_alerts.iter().filter(|a| a.severity == "INFO").count();

    let status = if critical_count > 0 {
        "RED"
    } else if high_count > 0 || warn_count > 0 {
        "YELLOW"
    } else {
        "GREEN"
    };

    let result = serde_json::json!({
        "status": status,
        "summary": {
            "critical": critical_count,
            "high": high_count,
            "warn": warn_count,
            "info": info_count,
            "total": all_alerts.len(),
        },
        "hook_records_analyzed": hook_records.len(),
        "signal_records_analyzed": signal_records.len(),
        "alerts": all_alerts,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `monitoring_alerts` — filtered alert list by severity.
pub fn alerts(params: MonitoringAlertsParams) -> Result<CallToolResult, McpError> {
    let hook_records = load_hook_records(100);
    let signal_records = load_signal_records(500);

    let mut all_alerts = Vec::new();
    all_alerts.extend(compute_hook_alerts(&hook_records));
    all_alerts.extend(compute_code_health_alerts());
    all_alerts.extend(compute_signal_alerts(&signal_records));

    // Filter by severity if provided
    if let Some(ref filter) = params.severity_filter {
        let filter_upper = filter.to_uppercase();
        all_alerts.retain(|a| a.severity == filter_upper.as_str());
    }

    // Apply limit
    let limit = params.limit.unwrap_or(20).min(100);
    all_alerts.truncate(limit);

    let result = serde_json::json!({
        "count": all_alerts.len(),
        "severity_filter": params.severity_filter,
        "alerts": all_alerts,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `monitoring_hook_health` — deep analysis for a specific hook or all hooks.
pub fn hook_health(params: MonitoringHookHealthParams) -> Result<CallToolResult, McpError> {
    let records = load_hook_records(200);

    // Group by hook name
    let mut by_hook: HashMap<String, Vec<&HookRecord>> = HashMap::new();
    for r in &records {
        by_hook.entry(r.hook.clone()).or_default().push(r);
    }

    // Filter to specific hook if requested
    if let Some(ref name) = params.hook_name {
        by_hook.retain(|k, _| k == name);
    }

    let mut hook_stats: Vec<serde_json::Value> = Vec::new();
    for (hook, recs) in &by_hook {
        let total = recs.len();
        let blocked = recs.iter().filter(|r| r.blocked).count();
        let block_rate = if total > 0 {
            blocked as f64 / total as f64
        } else {
            0.0
        };
        let max_ms = recs.iter().map(|r| r.duration_ms).max().unwrap_or(0);
        let avg_ms = if total > 0 {
            recs.iter().map(|r| r.duration_ms).sum::<u64>() / total as u64
        } else {
            0
        };
        let events: Vec<String> = recs
            .iter()
            .map(|r| r.event.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let health = if block_rate > HOOK_BLOCK_CRITICAL {
            "CRITICAL"
        } else if block_rate > HOOK_BLOCK_WARN || max_ms > HOOK_SLOW_MS {
            "DEGRADED"
        } else {
            "HEALTHY"
        };

        hook_stats.push(serde_json::json!({
            "hook": hook,
            "health": health,
            "total_executions": total,
            "blocked": blocked,
            "block_rate": format!("{:.1}%", block_rate * 100.0),
            "max_duration_ms": max_ms,
            "avg_duration_ms": avg_ms,
            "events": events,
        }));
    }

    // Sort by block rate descending (unhealthiest first)
    hook_stats.sort_by(|a, b| {
        let rate_a = a["block_rate"]
            .as_str()
            .unwrap_or("0")
            .trim_end_matches('%')
            .parse::<f64>()
            .unwrap_or(0.0);
        let rate_b = b["block_rate"]
            .as_str()
            .unwrap_or("0")
            .trim_end_matches('%')
            .parse::<f64>()
            .unwrap_or(0.0);
        rate_b
            .partial_cmp(&rate_a)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let result = serde_json::json!({
        "total_hooks": hook_stats.len(),
        "total_records": records.len(),
        "hooks": hook_stats,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `monitoring_signal_digest` — signal groups by type/priority.
pub fn signal_digest(params: MonitoringSignalDigestParams) -> Result<CallToolResult, McpError> {
    let window_minutes = params.window_minutes.unwrap_or(60);
    let records = load_signal_records(1000);

    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let cutoff_ms = now_ms.saturating_sub((window_minutes as u128) * 60_000);

    // Filter to window
    let windowed: Vec<&SignalRecord> = records
        .iter()
        .filter(|r| r.timestamp_ms >= cutoff_ms)
        .collect();

    // Group by signal_type prefix (before first ':')
    let mut groups: HashMap<String, Vec<&SignalRecord>> = HashMap::new();
    for r in &windowed {
        let prefix = r
            .signal_type
            .split(':')
            .next()
            .unwrap_or("unknown")
            .to_string();
        groups.entry(prefix).or_default().push(r);
    }

    let mut group_summaries: Vec<serde_json::Value> = groups
        .iter()
        .map(|(prefix, recs)| {
            serde_json::json!({
                "type": prefix,
                "count": recs.len(),
                "latest_timestamp_ms": recs.iter().map(|r| r.timestamp_ms).max().unwrap_or(0),
            })
        })
        .collect();

    // Sort by count descending
    group_summaries.sort_by(|a, b| {
        let ca = a["count"].as_u64().unwrap_or(0);
        let cb = b["count"].as_u64().unwrap_or(0);
        cb.cmp(&ca)
    });

    let result = serde_json::json!({
        "window_minutes": window_minutes,
        "total_signals_in_window": windowed.len(),
        "total_signals_on_file": records.len(),
        "groups": group_summaries,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
