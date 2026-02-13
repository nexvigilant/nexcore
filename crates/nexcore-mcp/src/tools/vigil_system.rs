//! # Vigil System Tools — π(∂·ν)|∝ Vigilance Engine
//!
//! MCP tools for the vigilance subsystem:
//! - vigil_sys_start: Start the vigilance daemon
//! - vigil_sys_stop: Stop the daemon gracefully
//! - vigil_sys_status: Daemon health + stats
//! - vigil_sys_boundaries: List configured boundaries
//! - vigil_sys_add_boundary: Add a boundary specification
//! - vigil_sys_ledger_query: Query ledger entries
//! - vigil_sys_ledger_verify: Verify hash chain integrity
//! - vigil_sys_stats: Runtime statistics
//!
//! ## Tier: T3 (π + ∂ + ν + ∝)

use crate::params::{VigilSysAddBoundaryParams, VigilSysLedgerQueryParams, VigilSysStartParams};
use nexcore_vigil::vigilance::{
    BoundarySpec, EscalationLevel, LogConsequence, ThresholdCheck, VigilConfig, VigilDaemon,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

/// Global daemon state — OnceLock<Mutex<Option<VigilDaemon>>>
static DAEMON: OnceLock<Mutex<Option<VigilDaemon>>> = OnceLock::new();

fn daemon_state() -> &'static Mutex<Option<VigilDaemon>> {
    DAEMON.get_or_init(|| Mutex::new(None))
}

/// Start the vigilance daemon.
pub fn vigil_sys_start(params: VigilSysStartParams) -> Result<CallToolResult, McpError> {
    let mut guard = daemon_state()
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;

    if guard.is_some() {
        return Ok(CallToolResult::success(vec![Content::text(
            json!({"status": "already_running", "message": "Vigil daemon is already running"})
                .to_string(),
        )]));
    }

    let config = if let Some(ref path) = params.config_path {
        VigilConfig::from_file(std::path::Path::new(path))
            .map_err(|e| McpError::internal_error(format!("Config load failed: {e}"), None))?
    } else {
        VigilConfig::default()
    };

    let mut daemon = VigilDaemon::new(config);

    // Add default timer source
    daemon.add_source(Box::new(
        nexcore_vigil::vigilance::sources::TimerSource::new("heartbeat", Duration::from_secs(60)),
    ));

    // Add default log consequence
    daemon.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Observe)));

    daemon
        .start()
        .map_err(|e| McpError::internal_error(format!("Start failed: {e}"), None))?;

    let health = daemon.health();
    *guard = Some(daemon);

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!({
            "status": "started",
            "health": health,
            "formula": "π(∂·ν)|∝",
        }))
        .unwrap_or_else(|_| "daemon started".to_string()),
    )]))
}

/// Stop the vigilance daemon gracefully.
pub fn vigil_sys_stop() -> Result<CallToolResult, McpError> {
    let mut guard = daemon_state()
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;

    match guard.as_mut() {
        Some(daemon) => {
            let stats = daemon.stats();
            daemon
                .stop()
                .map_err(|e| McpError::internal_error(format!("Stop failed: {e}"), None))?;
            *guard = None;

            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({
                    "status": "stopped",
                    "final_stats": stats,
                }))
                .unwrap_or_else(|_| "daemon stopped".to_string()),
            )]))
        }
        None => Ok(CallToolResult::success(vec![Content::text(
            json!({"status": "not_running", "message": "No daemon to stop"}).to_string(),
        )])),
    }
}

/// Get daemon health and status.
pub fn vigil_sys_status() -> Result<CallToolResult, McpError> {
    let guard = daemon_state()
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;

    match guard.as_ref() {
        Some(daemon) => {
            let health = daemon.health();
            let stats = daemon.stats();

            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({
                    "status": "running",
                    "health": health,
                    "stats": stats,
                    "formula": "π(∂·ν)|∝",
                }))
                .unwrap_or_else(|_| "daemon running".to_string()),
            )]))
        }
        None => Ok(CallToolResult::success(vec![Content::text(
            json!({"status": "not_running"}).to_string(),
        )])),
    }
}

/// List configured boundary specifications.
pub fn vigil_sys_boundaries() -> Result<CallToolResult, McpError> {
    let guard = daemon_state()
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;

    match guard.as_ref() {
        Some(daemon) => {
            let names = daemon.boundary_names();
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({
                    "boundaries": names,
                    "count": names.len(),
                }))
                .unwrap_or_else(|_| "boundaries listed".to_string()),
            )]))
        }
        None => Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "daemon not running"}).to_string(),
        )])),
    }
}

/// Add a boundary specification to the running daemon.
pub fn vigil_sys_add_boundary(
    params: VigilSysAddBoundaryParams,
) -> Result<CallToolResult, McpError> {
    let mut guard = daemon_state()
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;

    match guard.as_mut() {
        Some(daemon) => {
            let threshold = match params.threshold_type.as_str() {
                "always" => ThresholdCheck::Always,
                "severity" => {
                    let level = match params.severity.as_deref() {
                        Some("info") => nexcore_vigil::vigilance::EventSeverity::Info,
                        Some("low") => nexcore_vigil::vigilance::EventSeverity::Low,
                        Some("medium") => nexcore_vigil::vigilance::EventSeverity::Medium,
                        Some("high") => nexcore_vigil::vigilance::EventSeverity::High,
                        Some("critical") => nexcore_vigil::vigilance::EventSeverity::Critical,
                        _ => nexcore_vigil::vigilance::EventSeverity::High,
                    };
                    ThresholdCheck::SeverityAtLeast(level)
                }
                "count" => ThresholdCheck::CountExceeds {
                    count: params.count.unwrap_or(5),
                    window: Duration::from_millis(params.window_ms.unwrap_or(60_000)),
                },
                _ => ThresholdCheck::Always,
            };

            let spec = BoundarySpec {
                name: params.name.clone(),
                source_filter: params.source_filter,
                kind_filter: None,
                threshold,
                cooldown: Duration::from_millis(params.cooldown_ms.unwrap_or(5000)),
            };

            daemon.add_boundary(spec);

            Ok(CallToolResult::success(vec![Content::text(
                json!({
                    "status": "added",
                    "boundary": params.name,
                    "boundaries": daemon.boundary_names(),
                })
                .to_string(),
            )]))
        }
        None => Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "daemon not running"}).to_string(),
        )])),
    }
}

/// Query ledger entries.
pub fn vigil_sys_ledger_query(
    params: VigilSysLedgerQueryParams,
) -> Result<CallToolResult, McpError> {
    let guard = daemon_state()
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;

    match guard.as_ref() {
        Some(daemon) => {
            let ledger = daemon
                .ledger()
                .lock()
                .map_err(|e| McpError::internal_error(format!("Ledger lock failed: {e}"), None))?;

            let entry_type = params.entry_type.as_deref().and_then(|t| match t {
                "event_observed" => {
                    Some(nexcore_vigil::vigilance::ledger::LedgerEntryType::EventObserved)
                }
                "boundary_violation" => {
                    Some(nexcore_vigil::vigilance::ledger::LedgerEntryType::BoundaryViolation)
                }
                "consequence_scheduled" => {
                    Some(nexcore_vigil::vigilance::ledger::LedgerEntryType::ConsequenceScheduled)
                }
                "consequence_executed" => {
                    Some(nexcore_vigil::vigilance::ledger::LedgerEntryType::ConsequenceExecuted)
                }
                "consequence_failed" => {
                    Some(nexcore_vigil::vigilance::ledger::LedgerEntryType::ConsequenceFailed)
                }
                "daemon_started" => {
                    Some(nexcore_vigil::vigilance::ledger::LedgerEntryType::DaemonStarted)
                }
                "daemon_stopped" => {
                    Some(nexcore_vigil::vigilance::ledger::LedgerEntryType::DaemonStopped)
                }
                _ => None,
            });

            let query = nexcore_vigil::vigilance::ledger::LedgerQuery {
                entry_type,
                since: params.since,
                limit: params.limit.map(|l| l as usize),
            };

            let results = ledger.query(&query);
            let entries: Vec<serde_json::Value> = results
                .iter()
                .map(|e| {
                    json!({
                        "sequence": e.sequence,
                        "timestamp": e.timestamp,
                        "entry_type": format!("{}", e.entry_type),
                        "payload": e.payload,
                        "hash": e.hash.iter().map(|b| format!("{b:02x}")).collect::<String>(),
                    })
                })
                .collect();

            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({
                    "entries": entries,
                    "count": entries.len(),
                    "total_ledger_size": ledger.len(),
                }))
                .unwrap_or_else(|_| "query complete".to_string()),
            )]))
        }
        None => Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "daemon not running"}).to_string(),
        )])),
    }
}

/// Verify hash chain integrity.
pub fn vigil_sys_ledger_verify() -> Result<CallToolResult, McpError> {
    let guard = daemon_state()
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;

    match guard.as_ref() {
        Some(daemon) => {
            let ledger = daemon
                .ledger()
                .lock()
                .map_err(|e| McpError::internal_error(format!("Ledger lock failed: {e}"), None))?;

            let verified = ledger
                .verify_chain()
                .map_err(|e| McpError::internal_error(format!("Verify failed: {e}"), None))?;

            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({
                    "chain_verified": verified,
                    "entries": ledger.len(),
                    "head_hash": ledger.head_hash().iter().map(|b| format!("{b:02x}")).collect::<String>(),
                    "integrity": if verified { "INTACT" } else { "COMPROMISED" },
                }))
                .unwrap_or_else(|_| "verify complete".to_string()),
            )]))
        }
        None => Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "daemon not running"}).to_string(),
        )])),
    }
}

/// Get runtime statistics.
pub fn vigil_sys_stats() -> Result<CallToolResult, McpError> {
    let guard = daemon_state()
        .lock()
        .map_err(|e| McpError::internal_error(format!("Lock failed: {e}"), None))?;

    match guard.as_ref() {
        Some(daemon) => {
            let stats = daemon.stats();
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({
                    "stats": stats,
                    "formula": "π(∂·ν)|∝",
                }))
                .unwrap_or_else(|_| "stats complete".to_string()),
            )]))
        }
        None => Ok(CallToolResult::success(vec![Content::text(
            json!({"error": "daemon not running"}).to_string(),
        )])),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_when_not_running() {
        let result = vigil_sys_status();
        assert!(result.is_ok());
    }

    #[test]
    fn stop_when_not_running() {
        let result = vigil_sys_stop();
        assert!(result.is_ok());
    }

    #[test]
    fn boundaries_when_not_running() {
        let result = vigil_sys_boundaries();
        assert!(result.is_ok());
    }

    #[test]
    fn verify_when_not_running() {
        let result = vigil_sys_ledger_verify();
        assert!(result.is_ok());
    }
}
