//! Urinary System MCP tools — waste management, telemetry pruning, retention.
//!
//! Maps Claude Code's cleanup infrastructure to the renal system:
//! - Glomerular filtration: separating useful data from waste
//! - Reabsorption: keeping valuable artifacts from being pruned
//! - Excretion: removing stale sessions, old telemetry, dead logs
//!
//! ## T1 Primitive Grounding
//! - Filtration: κ(Comparison) + ∂(Boundary)
//! - Retention: π(Persistence) + N(Quantity)
//! - Excretion: ∅(Void) + ∝(Irreversibility)

use crate::params::urinary::{
    UrinaryExpiryParams, UrinaryHealthParams, UrinaryPruningParams, UrinaryRetentionParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::Path;

/// Analyze telemetry pruning needs.
pub fn pruning(params: UrinaryPruningParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    let target = params
        .target_path
        .unwrap_or_else(|| format!("{}/.claude/hooks/state", home));
    let max_age_hours = params.max_age_hours.unwrap_or(168); // 7 days default

    let mut total_files = 0u64;
    let mut total_bytes = 0u64;
    let mut candidates = Vec::new();

    if let Ok(entries) = std::fs::read_dir(&target) {
        let now = std::time::SystemTime::now();
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total_files += 1;
                    total_bytes += meta.len();

                    let age_hours = meta
                        .modified()
                        .ok()
                        .and_then(|m| now.duration_since(m).ok())
                        .map(|d| d.as_secs() / 3600)
                        .unwrap_or(0);

                    if age_hours > max_age_hours {
                        candidates.push(json!({
                            "file": entry.file_name().to_string_lossy(),
                            "size_bytes": meta.len(),
                            "age_hours": age_hours,
                        }));
                    }
                }
            }
        }
    }

    let prunable_bytes: u64 = candidates
        .iter()
        .filter_map(|c| c["size_bytes"].as_u64())
        .sum();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "pruning_analysis": {
                "target": target,
                "max_age_hours": max_age_hours,
                "total_files": total_files,
                "total_bytes": total_bytes,
                "pruning_candidates": candidates.len(),
                "prunable_bytes": prunable_bytes,
                "savings_percent": if total_bytes > 0 {
                    (prunable_bytes as f64 / total_bytes as f64 * 100.0).round() / 100.0
                } else { 0.0 },
            },
            "candidates": if candidates.len() <= 10 { candidates.clone() } else { candidates[..10].to_vec() },
            "analog": {
                "glomerular_filtration": "Scanning files for age-based pruning",
                "reabsorption": "Keeping files newer than max_age threshold",
                "excretion": "Removing old/stale files",
            },
        })
        .to_string(),
    )]))
}

/// Check session expiry status.
pub fn expiry(params: UrinaryExpiryParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());
    let session_age = params.session_age_hours.unwrap_or(0.0);

    // Check brain sessions directory
    let sessions_dir = format!("{}/.claude/brain/sessions", home);
    let session_count = std::fs::read_dir(&sessions_dir)
        .map(|rd| rd.flatten().count())
        .unwrap_or(0);

    // Session expiry thresholds
    let fresh = session_age < 4.0;
    let active = session_age < 24.0;
    let stale = session_age >= 24.0 && session_age < 168.0;
    let expired = session_age >= 168.0;

    let status = if fresh {
        "fresh"
    } else if active {
        "active"
    } else if stale {
        "stale"
    } else {
        "expired"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "session_expiry": {
                "age_hours": session_age,
                "status": status,
                "is_fresh": fresh,
                "is_active": active,
                "is_stale": stale,
                "is_expired": expired,
            },
            "brain_sessions": session_count,
            "thresholds": {
                "fresh": "<4 hours",
                "active": "<24 hours",
                "stale": "24-168 hours (1-7 days)",
                "expired": ">168 hours (>7 days)",
            },
            "analog": {
                "bladder_capacity": "Session context window (200K tokens)",
                "voiding_reflex": "Session compaction or restart",
                "retention": "Keeping session alive for continuity",
            },
        })
        .to_string(),
    )]))
}

/// Evaluate retention policy compliance.
pub fn retention(params: UrinaryRetentionParams) -> Result<CallToolResult, McpError> {
    let category = &params.category;
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());

    let policy = match category.to_lowercase().as_str() {
        "telemetry" => {
            let path = format!("{}/.claude/brain/telemetry", home);
            let exists = Path::new(&path).exists();
            json!({
                "category": "telemetry",
                "path": path,
                "exists": exists,
                "policy": {
                    "retention_days": 30,
                    "max_size_mb": 100,
                    "rotation": "by_size_and_age",
                },
                "compliance": "check_manually",
            })
        }
        "artifacts" => {
            let path = format!("{}/.claude/brain/artifacts", home);
            let count = std::fs::read_dir(&path)
                .map(|rd| rd.flatten().count())
                .unwrap_or(0);
            json!({
                "category": "artifacts",
                "path": path,
                "count": count,
                "policy": {
                    "retention_days": "indefinite (resolved versions are immutable)",
                    "max_count": "no_limit",
                    "rotation": "manual_cleanup",
                },
                "compliance": "artifacts are persistent by design",
            })
        }
        "logs" | "hook_logs" => {
            let path = format!("{}/.claude/hooks/state/hook_executions.jsonl", home);
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            json!({
                "category": "logs",
                "path": path,
                "size_bytes": size,
                "size_mb": (size as f64 / 1_048_576.0 * 100.0).round() / 100.0,
                "policy": {
                    "retention_days": 7,
                    "max_size_mb": 50,
                    "rotation": "learning-consumer processes then prunes",
                },
                "compliance": if size > 50_000_000 { "over_limit" } else { "compliant" },
            })
        }
        "sessions" => {
            let path = format!("{}/.claude/brain/sessions", home);
            let count = std::fs::read_dir(&path)
                .map(|rd| rd.flatten().count())
                .unwrap_or(0);
            json!({
                "category": "sessions",
                "path": path,
                "count": count,
                "policy": {
                    "retention_days": 90,
                    "max_count": 100,
                    "rotation": "oldest_first",
                },
                "compliance": if count > 100 { "over_limit" } else { "compliant" },
            })
        }
        _ => json!({
            "category": category,
            "error": "Unknown category",
            "known_categories": ["telemetry", "artifacts", "logs", "sessions"],
        }),
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "retention_policy": policy,
            "analog": {
                "reabsorption": "Keeping valuable data (artifacts, resolved versions)",
                "excretion": "Removing stale data (old telemetry, expired sessions)",
                "filtration_rate": "How quickly waste is identified and separated",
            },
        })
        .to_string(),
    )]))
}

/// Get urinary system health overview.
pub fn health(_params: UrinaryHealthParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());

    let hook_log_size = std::fs::metadata(format!(
        "{}/.claude/hooks/state/hook_executions.jsonl",
        home
    ))
    .map(|m| m.len())
    .unwrap_or(0);

    let brain_dir_size = dir_size(&format!("{}/.claude/brain", home));

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "urinary_health": {
                "status": "operational",
                "hook_log_bytes": hook_log_size,
                "brain_dir_bytes": brain_dir_size,
                "waste_pressure": if hook_log_size > 10_000_000 || brain_dir_size > 100_000_000 {
                    "high"
                } else if hook_log_size > 1_000_000 || brain_dir_size > 50_000_000 {
                    "moderate"
                } else {
                    "low"
                },
            },
            "components": {
                "kidneys": "learning-consumer + waste-collector (Stop hook)",
                "bladder": "State files (accumulate between sessions)",
                "urethra": "File deletion / log rotation",
                "nephrons": "Individual pruning rules (age, size, category)",
            },
        })
        .to_string(),
    )]))
}

fn dir_size(path: &str) -> u64 {
    let mut total = 0u64;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Ok(meta) = entry.metadata() {
                if meta.is_file() {
                    total += meta.len();
                } else if meta.is_dir() {
                    total += dir_size(&entry.path().to_string_lossy());
                }
            }
        }
    }
    total
}
