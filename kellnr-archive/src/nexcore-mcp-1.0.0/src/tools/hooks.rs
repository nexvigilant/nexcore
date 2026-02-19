//! Hooks MCP tools - Hook Registry API
//!
//! Provides structured APIs for querying the hook catalog and execution metrics.
//! Includes compound hook (HookMolecule) support for nested hook architectures.

use nexcore_config::{HookEvent, HookRegistry, HookTier};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

use crate::params::{
    HookListNestedParams, HookMetricsByEventParams, HookMetricsSummaryParams, HooksForEventParams,
    HooksForTierParams,
};
use crate::tooling::{ReadOutcome, ScanLimits, read_limited_file};

// ============================================================================
// Hook Registry Queries
// ============================================================================

/// Get hook registry statistics
pub fn stats() -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").map_err(|e| McpError::internal_error(e.to_string(), None))?;
    let catalog_path = format!("{}/nexcore/crates/nexcore-hooks/hooks-catalog.json", home);

    let registry = HookRegistry::from_file(&catalog_path)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let all_hooks = registry.all_hooks();
    let by_tier = registry.count_by_tier();

    // Count by event
    let mut by_event = std::collections::HashMap::new();
    for hook in &all_hooks {
        *by_event.entry(format!("{:?}", hook.event)).or_insert(0) += 1;
    }

    let stats = serde_json::json!({
        "total_hooks": all_hooks.len(),
        "by_tier": {
            "dev": by_tier.get(&HookTier::Dev).unwrap_or(&0),
            "review": by_tier.get(&HookTier::Review).unwrap_or(&0),
            "deploy": by_tier.get(&HookTier::Deploy).unwrap_or(&0),
        },
        "by_event": by_event,
        "event_types": 13, // Total event types in system
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        stats.to_string(),
    )]))
}

/// Get hooks for a specific event type
pub fn for_event(params: HooksForEventParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").map_err(|e| McpError::internal_error(e.to_string(), None))?;
    let catalog_path = format!("{}/nexcore/crates/nexcore-hooks/hooks-catalog.json", home);

    let registry = HookRegistry::from_file(&catalog_path)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Parse event from string
    let event = match params.event.as_str() {
        "SessionStart" => HookEvent::SessionStart,
        "SessionEnd" => HookEvent::SessionEnd,
        "UserPromptSubmit" => HookEvent::UserPromptSubmit,
        "PreToolUse:Bash" => HookEvent::PreToolUseBash,
        "PreToolUse:Edit|Write" => HookEvent::PreToolUseEditWrite,
        "PreToolUse:Task" => HookEvent::PreToolUseTask,
        "PostToolUse" => HookEvent::PostToolUse,
        "PostToolUseFailure" => HookEvent::PostToolUseFailure,
        "PreCompact" => HookEvent::PreCompact,
        "PermissionRequest" => HookEvent::PermissionRequest,
        "Stop" => HookEvent::Stop,
        "Setup" => HookEvent::Setup,
        "SubagentStart" => HookEvent::SubagentStart,
        _ => {
            return Err(McpError::invalid_params(
                format!(
                    "Unknown event type: {}. Valid events: SessionStart, SessionEnd, UserPromptSubmit, PreToolUse:Bash, PreToolUse:Edit|Write, PreToolUse:Task, PostToolUse, PostToolUseFailure, PreCompact, PermissionRequest, Stop, Setup, SubagentStart",
                    params.event
                ),
                None,
            ));
        }
    };

    let hooks = registry.get_event_hooks(&event);

    let result = serde_json::json!({
        "event": params.event,
        "hook_count": hooks.len(),
        "hooks": hooks.iter().map(|h| serde_json::json!({
            "name": h.name,
            "tiers": h.tiers,
            "timeout": h.timeout,
            "description": h.description,
        })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

/// Get hooks for a specific deployment tier
pub fn for_tier(params: HooksForTierParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").map_err(|e| McpError::internal_error(e.to_string(), None))?;
    let catalog_path = format!("{}/nexcore/crates/nexcore-hooks/hooks-catalog.json", home);

    let registry = HookRegistry::from_file(&catalog_path)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Parse tier from string
    let tier = match params.tier.to_lowercase().as_str() {
        "dev" => HookTier::Dev,
        "review" => HookTier::Review,
        "deploy" => HookTier::Deploy,
        _ => {
            return Err(McpError::invalid_params(
                format!(
                    "Unknown tier: {}. Valid tiers: dev, review, deploy",
                    params.tier
                ),
                None,
            ));
        }
    };

    let hooks = registry.filter_by_tier(tier);

    let result = serde_json::json!({
        "tier": params.tier,
        "hook_count": hooks.len(),
        "hooks": hooks.iter().map(|h| serde_json::json!({
            "name": h.name,
            "event": format!("{:?}", h.event),
            "timeout": h.timeout,
            "description": h.description,
        })).collect::<Vec<_>>(),
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Compound Hook (HookMolecule) Support
// ============================================================================

/// Simplified macromolecule for deserialization from registry
///
/// Tier: T3 (Domain-specific hook macromolecule)
/// Grounds to T1 Concepts via String/bool/u8 and Vec fields
/// Ord: N/A (composite record)
#[derive(Deserialize)]
struct MacromoleculeEntry {
    nucleus: String,
    formula: String,
    stable: bool,
    #[serde(default)]
    declared_chain: Vec<String>,
    #[serde(default)]
    polymer_units: Vec<PolymerUnitEntry>,
    #[serde(default)]
    broken_bonds: Vec<String>,
    nesting_depth: u8,
}

/// Nested polymer unit entry
///
/// Tier: T3 (Domain-specific hook polymer unit)
/// Grounds to T1 Concepts via String/u8 fields
/// Ord: N/A (composite record)
#[derive(Deserialize)]
struct PolymerUnitEntry {
    name: String,
    path: String,
    bond_strength: u8,
}

/// Macromolecule registry root
///
/// Tier: T3 (Domain-specific registry)
/// Grounds to T1 Concepts via HashMap/Vec/usize
/// Ord: N/A (composite record)
#[derive(Deserialize, Default)]
struct MacromoleculeRegistry {
    #[serde(default)]
    macromolecules: HashMap<String, MacromoleculeEntry>,
    #[serde(default)]
    _formulas: Vec<String>,
    #[serde(default)]
    _total_units: usize,
}

/// List nested hooks for a compound hook (hook molecule)
///
/// Reads the macromolecule registry to find nested hooks for a parent.
/// Returns molecular formula, nested hooks, and bond information.
pub fn list_nested(params: HookListNestedParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Load macromolecule registry (created by sessionstart_nested_skill_resolver)
    let registry_path = format!("{}/.claude/brain/macromolecule_registry.json", home);

    let limits = ScanLimits::from_env();
    let read_outcome =
        read_limited_file(Path::new(&registry_path), limits).unwrap_or(ReadOutcome {
            content: String::new(),
            notice: None,
        });
    let scan_notice = read_outcome
        .notice
        .and_then(|notice| serde_json::to_value(vec![notice]).ok());
    let registry: MacromoleculeRegistry =
        serde_json::from_str(&read_outcome.content).unwrap_or_default();

    // Find the requested parent
    let Some(mol) = registry.macromolecules.get(&params.parent) else {
        // Check if any macromolecule contains this as a nested hook
        let containing = registry
            .macromolecules
            .values()
            .find(|m| m.polymer_units.iter().any(|u| u.name == params.parent));

        if let Some(parent_mol) = containing {
            let mut result = serde_json::json!({
                "error": "not_a_parent",
                "message": format!("'{}' is a nested hook, not a parent", params.parent),
                "parent": parent_mol.nucleus,
                "parent_formula": parent_mol.formula,
            });
            if let Some(value) = scan_notice.clone() {
                result["scan_notice"] = value;
            }
            return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                result.to_string(),
            )]));
        }

        let mut result = serde_json::json!({
            "error": "not_found",
            "message": format!("No compound hook found for parent '{}'", params.parent),
            "available_compounds": registry.macromolecules.keys().collect::<Vec<_>>(),
        });
        if let Some(value) = scan_notice.clone() {
            result["scan_notice"] = value;
        }
        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            result.to_string(),
        )]));
    };

    let mut result = serde_json::json!({
        "parent": mol.nucleus,
        "formula": mol.formula,
        "stable": mol.stable,
        "nesting_depth": mol.nesting_depth,
        "declared": mol.declared_chain,
        "nested_hooks": mol.polymer_units.iter().map(|u| serde_json::json!({
            "name": u.name,
            "path": u.path,
            "bond_strength": u.bond_strength,
        })).collect::<Vec<_>>(),
        "broken_bonds": mol.broken_bonds,
        "count": mol.polymer_units.len(),
    });
    if let Some(value) = scan_notice {
        result["scan_notice"] = value;
    }

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        result.to_string(),
    )]))
}

// ============================================================================
// Hook Execution Metrics
// ============================================================================

/// Hook execution metrics record (loaded from telemetry JSONL)
#[derive(Debug, Clone, Deserialize)]
struct MetricsRecord {
    #[allow(dead_code)]
    timestamp: String,
    hook: String,
    event: String,
    duration_ms: u64,
    exit_code: u8,
    blocked: bool,
}

/// Percentile statistics
#[derive(Debug, Clone, serde::Serialize)]
struct PercentileStats {
    p50: f64,
    p95: f64,
    p99: f64,
    min: f64,
    max: f64,
    count: usize,
}

/// Load metrics records from telemetry file.
fn load_metrics_records() -> Vec<MetricsRecord> {
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => return Vec::new(),
    };
    let path = format!("{}/.claude/brain/telemetry/hook_executions.jsonl", home);
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str(line).ok())
        .collect()
}

/// Calculate percentile from sorted values.
fn percentile_at(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() as f64) * p).floor() as usize;
    sorted
        .get(idx.min(sorted.len().saturating_sub(1)))
        .copied()
        .unwrap_or(0.0)
}

/// Calculate percentile stats from durations.
fn calc_percentiles(durations: &mut [f64]) -> Option<PercentileStats> {
    if durations.is_empty() {
        return None;
    }
    durations.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(PercentileStats {
        p50: percentile_at(durations, 0.50),
        p95: percentile_at(durations, 0.95),
        p99: percentile_at(durations, 0.99),
        min: durations.first().copied().unwrap_or(0.0),
        max: durations.last().copied().unwrap_or(0.0),
        count: durations.len(),
    })
}

/// Build hook summary JSON from records.
fn build_hook_summary(hook_name: &str, records: &[&MetricsRecord]) -> Option<serde_json::Value> {
    let mut durations: Vec<f64> = records.iter().map(|r| r.duration_ms as f64).collect();
    let timing = calc_percentiles(&mut durations)?;
    let block_count = records.iter().filter(|r| r.blocked).count();
    let warn_count = records.iter().filter(|r| r.exit_code == 1).count();
    let rate = if records.is_empty() {
        0.0
    } else {
        block_count as f64 / records.len() as f64
    };
    Some(serde_json::json!({
        "hook": hook_name,
        "executions": records.len(),
        "blocks": block_count,
        "warns": warn_count,
        "block_rate": rate,
        "timing_ms": timing,
    }))
}

/// Group records by hook name.
fn group_records_by_hook(records: &[MetricsRecord]) -> HashMap<String, Vec<&MetricsRecord>> {
    let mut map: HashMap<String, Vec<&MetricsRecord>> = HashMap::new();
    for record in records {
        map.entry(record.hook.clone()).or_default().push(record);
    }
    map
}

/// Count records by event.
fn count_by_event(records: &[MetricsRecord]) -> HashMap<String, usize> {
    let mut map: HashMap<String, usize> = HashMap::new();
    for record in records {
        *map.entry(record.event.clone()).or_insert(0) += 1;
    }
    map
}

/// Sort summaries by p99 descending.
fn sort_by_p99(summaries: &mut [serde_json::Value]) {
    summaries.sort_by(|a, b| {
        let a_p99 = a["timing_ms"]["p99"].as_f64().unwrap_or(0.0);
        let b_p99 = b["timing_ms"]["p99"].as_f64().unwrap_or(0.0);
        b_p99
            .partial_cmp(&a_p99)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

/// Get aggregate hook execution metrics summary.
#[allow(unused_variables)]
pub fn metrics_summary(_params: HookMetricsSummaryParams) -> Result<CallToolResult, McpError> {
    let records = load_metrics_records();
    if records.is_empty() {
        let result = serde_json::json!({
            "status": "no_data",
            "message": "No hook execution metrics found.",
            "telemetry_path": "~/.claude/brain/telemetry/hook_executions.jsonl"
        });
        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            result.to_string(),
        )]));
    }

    let by_hook = group_records_by_hook(&records);
    let by_event = count_by_event(&records);
    let (summaries, total_blocks, total_warns) = build_all_summaries(&by_hook);
    let slowest = extract_slowest(&summaries, 10);

    let metrics_result = serde_json::json!({
        "total_executions": records.len(),
        "total_blocks": total_blocks,
        "total_warns": total_warns,
        "unique_hooks": by_hook.len(),
        "block_rate": if records.is_empty() { 0.0 } else { total_blocks as f64 / records.len() as f64 },
        "by_event": by_event,
        "by_hook": summaries,
        "slowest_hooks": slowest,
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        metrics_result.to_string(),
    )]))
}

/// Build all hook summaries and count totals.
fn build_all_summaries(
    by_hook: &HashMap<String, Vec<&MetricsRecord>>,
) -> (Vec<serde_json::Value>, usize, usize) {
    let mut summaries = Vec::new();
    let mut total_blocks = 0usize;
    let mut total_warns = 0usize;
    for (name, recs) in by_hook {
        if let Some(summary) = build_hook_summary(name, recs) {
            total_blocks += summary["blocks"].as_u64().unwrap_or(0) as usize;
            total_warns += summary["warns"].as_u64().unwrap_or(0) as usize;
            summaries.push(summary);
        }
    }
    sort_by_p99(&mut summaries);
    (summaries, total_blocks, total_warns)
}

/// Extract slowest hooks from sorted summaries.
fn extract_slowest(summaries: &[serde_json::Value], limit: usize) -> Vec<serde_json::Value> {
    summaries
        .iter()
        .take(limit)
        .map(|s| serde_json::json!({ "hook": s["hook"], "p99_ms": s["timing_ms"]["p99"] }))
        .collect()
}

/// Get hook execution metrics filtered by event type.
pub fn metrics_by_event(params: HookMetricsByEventParams) -> Result<CallToolResult, McpError> {
    let all_records = load_metrics_records();
    let filtered: Vec<MetricsRecord> = all_records
        .into_iter()
        .filter(|r| r.event == params.event)
        .collect();

    if filtered.is_empty() {
        let no_data = serde_json::json!({
            "status": "no_data",
            "event": params.event,
            "message": format!("No metrics found for event '{}'", params.event),
        });
        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            no_data.to_string(),
        )]));
    }

    let by_hook = group_records_by_hook(&filtered);
    let (summaries, total_blocks, total_warns) = build_all_summaries(&by_hook);

    let event_result = serde_json::json!({
        "event": params.event,
        "total_executions": filtered.len(),
        "total_blocks": total_blocks,
        "total_warns": total_warns,
        "unique_hooks": by_hook.len(),
        "by_hook": summaries,
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        event_result.to_string(),
    )]))
}
