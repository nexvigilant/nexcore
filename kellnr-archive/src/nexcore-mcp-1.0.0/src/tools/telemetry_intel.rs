//! Telemetry Intelligence MCP Tools
//!
//! Provides structured analysis, cross-reference, and intelligence
//! reporting for external telemetry sources.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::{
    TelemetryGovernanceCrossrefParams, TelemetryIntelReportParams, TelemetryRecentParams,
    TelemetrySnapshotEvolutionParams, TelemetrySourceAnalyzeParams, TelemetrySourcesListParams,
};

/// List all discovered telemetry sources (sessions).
///
/// Scans telemetry directories for session files and returns
/// metadata about each discovered source.
pub fn sources_list(_params: TelemetrySourcesListParams) -> Result<CallToolResult, McpError> {
    use nexcore_telemetry_core::parser::discover_sources;

    match discover_sources() {
        Ok(sources) => {
            let source_list: Vec<serde_json::Value> = sources
                .iter()
                .map(|s| {
                    json!({
                        "project_hash": s.project_hash.0,
                        "filename": s.filename,
                        "path": s.path.to_string_lossy()
                    })
                })
                .collect();
            let result = json!({
                "sources": source_list,
                "count": sources.len(),
                "status": "ok"
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
            )]))
        }
        Err(e) => {
            let result = json!({
                "sources": [],
                "count": 0,
                "status": "error",
                "error": format!("{}", e),
                "message": "Telemetry directory not found or empty"
            });
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
            )]))
        }
    }
}

/// Deep analysis of a specific source session.
///
/// Parses the session file and returns detailed operation counts,
/// file access patterns, and token usage.
pub fn source_analyze(params: TelemetrySourceAnalyzeParams) -> Result<CallToolResult, McpError> {
    use nexcore_telemetry_core::parser::{discover_sources_for_project, parse_source};
    use std::path::Path;

    // If session_path is provided, parse directly
    if let Some(ref path_str) = params.session_path {
        let path = Path::new(path_str);
        match parse_source(path) {
            Ok(source) => {
                let tokens = source.total_tokens();
                let ops = source.all_operations();
                let file_ops = source.file_operations();

                let result = json!({
                    "session_id": source.id,
                    "project_hash": source.project_hash,
                    "start_time": source.start_time.to_rfc3339(),
                    "last_updated": source.last_updated.to_rfc3339(),
                    "message_count": source.messages.len(),
                    "operation_count": ops.len(),
                    "file_operation_count": file_ops.len(),
                    "tokens": {
                        "input": tokens.input,
                        "output": tokens.output,
                        "cached": tokens.cached,
                        "thoughts": tokens.thoughts,
                        "tool": tokens.tool,
                        "total": tokens.total
                    },
                    "status": "ok"
                });
                return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
                )]));
            }
            Err(e) => {
                let result = json!({
                    "status": "error",
                    "error": format!("{}", e),
                    "path": path.to_string_lossy()
                });
                return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
                )]));
            }
        }
    }

    // If project_hash is provided, find and analyze the latest session
    if let Some(ref hash) = params.project_hash {
        match discover_sources_for_project(hash) {
            Ok(sources) => {
                // Get the most recent session (last in sorted list)
                let Some(latest) = sources.last() else {
                    let result = json!({
                        "status": "error",
                        "error": "No sessions found for project",
                        "project_hash": hash
                    });
                    return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
                    )]));
                };

                match parse_source(&latest.path) {
                    Ok(source) => {
                        let tokens = source.total_tokens();
                        let ops = source.all_operations();
                        let file_ops = source.file_operations();

                        let result = json!({
                            "session_id": source.id,
                            "project_hash": source.project_hash,
                            "start_time": source.start_time.to_rfc3339(),
                            "last_updated": source.last_updated.to_rfc3339(),
                            "message_count": source.messages.len(),
                            "operation_count": ops.len(),
                            "file_operation_count": file_ops.len(),
                            "tokens": {
                                "input": tokens.input,
                                "output": tokens.output,
                                "cached": tokens.cached,
                                "thoughts": tokens.thoughts,
                                "tool": tokens.tool,
                                "total": tokens.total
                            },
                            "status": "ok"
                        });
                        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                            serde_json::to_string_pretty(&result)
                                .unwrap_or_else(|_| "{}".to_string()),
                        )]));
                    }
                    Err(e) => {
                        let result = json!({
                            "status": "error",
                            "error": format!("{}", e),
                            "path": latest.path.to_string_lossy()
                        });
                        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                            serde_json::to_string_pretty(&result)
                                .unwrap_or_else(|_| "{}".to_string()),
                        )]));
                    }
                }
            }
            Err(e) => {
                let result = json!({
                    "status": "error",
                    "error": format!("{}", e),
                    "project_hash": hash
                });
                return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
                )]));
            }
        }
    }

    // Neither path nor hash provided
    let result = json!({
        "status": "error",
        "error": "Either session_path or project_hash must be provided"
    });
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Cross-reference telemetry with governance module changes.
///
/// Identifies file accesses to governance-related paths and tracks
/// modifications to primitives, governance rules, and capabilities.
pub fn governance_crossref(
    params: TelemetryGovernanceCrossrefParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_telemetry_core::analysis::{
        GovernanceCategory, governance_file_access, governance_summary,
    };
    use nexcore_telemetry_core::parser::{discover_sources, parse_source};

    // Discover and parse all sources
    let discovered = match discover_sources() {
        Ok(d) => d,
        Err(e) => {
            let result = json!({
                "status": "error",
                "error": format!("{}", e),
                "message": "Failed to discover telemetry sources"
            });
            return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
            )]));
        }
    };

    let mut sources = Vec::new();
    for d in &discovered {
        if let Ok(source) = parse_source(&d.path) {
            sources.push(source);
        }
    }

    // Get governance accesses
    let accesses = governance_file_access(&sources);
    let summary = governance_summary(&accesses);

    // Filter by category if specified
    let filtered: Vec<_> = if let Some(ref category) = params.category {
        let cat = match category.to_lowercase().as_str() {
            "primitives" => GovernanceCategory::Primitives,
            "governance" => GovernanceCategory::Governance,
            "capabilities" => GovernanceCategory::Capabilities,
            "constitutional" => GovernanceCategory::Constitutional,
            _ => GovernanceCategory::Unknown,
        };
        accesses.iter().filter(|a| a.category == cat).collect()
    } else {
        accesses.iter().collect()
    };

    let access_list: Vec<serde_json::Value> = filtered
        .iter()
        .map(|a| {
            json!({
                "path": a.path,
                "category": format!("{:?}", a.category),
                "was_modified": a.was_modified,
                "read_count": a.read_count,
                "write_count": a.write_count,
                "session_id": a.session_id,
                "last_accessed": a.last_accessed.to_rfc3339()
            })
        })
        .collect();

    let result = json!({
        "summary": {
            "total_files": summary.total_files,
            "files_modified": summary.files_modified,
            "primitives_accesses": summary.primitives_accesses,
            "governance_accesses": summary.governance_accesses,
            "capabilities_accesses": summary.capabilities_accesses,
            "constitutional_accesses": summary.constitutional_accesses
        },
        "accesses": access_list,
        "sources_analyzed": sources.len(),
        "category_filter": params.category,
        "status": "ok"
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Track snapshot/artifact version history.
///
/// Discovers brain sessions and their artifacts, showing
/// version evolution over time.
pub fn snapshot_evolution(
    params: TelemetrySnapshotEvolutionParams,
) -> Result<CallToolResult, McpError> {
    use nexcore_telemetry_core::parser::{discover_brain_sessions, parse_snapshots};

    // Discover brain sessions
    let sessions = match discover_brain_sessions() {
        Ok(s) => s,
        Err(e) => {
            let result = json!({
                "status": "error",
                "error": format!("{}", e),
                "message": "Brain directory not found"
            });
            return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
            )]));
        }
    };

    // If session_id is provided, get snapshots for that session only
    if let Some(ref session_id) = params.session_id {
        match parse_snapshots(session_id) {
            Ok(snapshots) => {
                let snap_list: Vec<serde_json::Value> = snapshots
                    .iter()
                    .map(|s| {
                        json!({
                            "name": s.name,
                            "snapshot_type": format!("{:?}", s.snapshot_type),
                            "versions": s.versions,
                            "latest_version": s.latest_version(),
                            "has_history": s.has_history(),
                            "path": s.path.to_string_lossy(),
                            "metadata": s.metadata.as_ref().map(|m| json!({
                                "artifact_type": m.artifact_type,
                                "summary": m.summary,
                                "updated_at": m.updated_at.to_rfc3339()
                            }))
                        })
                    })
                    .collect();
                let result = json!({
                    "session_id": session_id,
                    "snapshots": snap_list,
                    "status": "ok"
                });
                return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
                )]));
            }
            Err(e) => {
                let result = json!({
                    "status": "error",
                    "error": format!("{}", e),
                    "session_id": session_id
                });
                return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
                )]));
            }
        }
    }

    // No session_id - return overview of all sessions
    let limit = params.limit.unwrap_or(10);
    let session_summaries: Vec<serde_json::Value> = sessions
        .iter()
        .take(limit)
        .filter_map(|s| {
            parse_snapshots(&s.id).ok().map(|snapshots| {
                let snaps: Vec<serde_json::Value> = snapshots
                    .iter()
                    .map(|snap| {
                        json!({
                            "name": snap.name,
                            "type": format!("{:?}", snap.snapshot_type),
                            "versions": snap.versions.len(),
                            "latest": snap.latest_version()
                        })
                    })
                    .collect();
                json!({
                    "session_id": s.id,
                    "path": s.path.to_string_lossy(),
                    "snapshot_count": snapshots.len(),
                    "snapshots": snaps
                })
            })
        })
        .collect();

    let result = json!({
        "sessions": session_summaries,
        "total_sessions": sessions.len(),
        "shown": session_summaries.len(),
        "status": "ok"
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Generate full intelligence report.
///
/// Aggregates data from telemetry sources and brain snapshots
/// to produce a comprehensive intelligence report.
pub fn intel_report(params: TelemetryIntelReportParams) -> Result<CallToolResult, McpError> {
    use nexcore_telemetry_core::intel::generate_report;
    use nexcore_telemetry_core::parser::{
        discover_brain_sessions, discover_sources, parse_snapshots, parse_source,
    };

    // Discover and parse sources
    let discovered = discover_sources().unwrap_or_default();
    let mut sources = Vec::new();
    for d in &discovered {
        if let Ok(source) = parse_source(&d.path) {
            sources.push(source);
        }
    }

    // Discover and parse snapshots
    let sessions = discover_brain_sessions().unwrap_or_default();
    let mut snapshots = Vec::new();
    for session in &sessions {
        if let Ok(snaps) = parse_snapshots(&session.id) {
            snapshots.extend(snaps);
        }
    }

    // Generate report
    let report = generate_report(&sources, &snapshots);

    // Apply limit to recent activity
    let activity_limit = params.activity_limit.unwrap_or(20);
    let recent_activity: Vec<serde_json::Value> = report
        .recent_activity
        .iter()
        .take(activity_limit)
        .map(|a| {
            json!({
                "timestamp": a.timestamp.to_rfc3339(),
                "source_id": a.source_id,
                "operation_count": a.operation_count,
                "files_touched": a.files_touched
            })
        })
        .collect();

    // Apply limit to file access patterns
    let file_limit = params.file_limit.unwrap_or(50);
    let file_patterns: Vec<serde_json::Value> = report
        .file_access_patterns
        .iter()
        .take(file_limit)
        .map(|f| {
            json!({
                "path": f.path,
                "read_count": f.read_count,
                "write_count": f.write_count,
                "first_access": f.first_access.to_rfc3339(),
                "last_access": f.last_access.to_rfc3339(),
                "source_count": f.source_ids.len()
            })
        })
        .collect();

    let governance_list: Vec<serde_json::Value> = report
        .governance_access
        .iter()
        .map(|g| {
            json!({
                "category": g.category,
                "resource": g.resource,
                "access_count": g.access_count,
                "source_count": g.source_ids.len()
            })
        })
        .collect();

    let result = json!({
        "generated_at": report.generated_at.to_rfc3339(),
        "sources_analyzed": report.sources_analyzed,
        "tokens": {
            "input": report.total_tokens.input,
            "output": report.total_tokens.output,
            "cached": report.total_tokens.cached,
            "thoughts": report.total_tokens.thoughts,
            "tool": report.total_tokens.tool,
            "total": report.total_tokens.total
        },
        "file_access_patterns": file_patterns,
        "file_patterns_total": report.file_access_patterns.len(),
        "governance_access": governance_list,
        "recent_activity": recent_activity,
        "activity_total": report.recent_activity.len(),
        "snapshots_analyzed": snapshots.len(),
        "status": "ok"
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Real-time activity stream (last N operations).
///
/// Returns the most recent operations across all telemetry sources.
pub fn recent_activity(params: TelemetryRecentParams) -> Result<CallToolResult, McpError> {
    use chrono::Utc;
    use nexcore_telemetry_core::parser::{discover_sources, parse_source};

    let limit = params.count.unwrap_or(20);

    // Discover and parse sources
    let discovered = match discover_sources() {
        Ok(d) => d,
        Err(e) => {
            let result = json!({
                "status": "error",
                "error": format!("{}", e),
                "operations": [],
                "count": 0
            });
            return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
            )]));
        }
    };

    // Collect all operations with timestamps
    let mut all_ops: Vec<(chrono::DateTime<Utc>, String, serde_json::Value)> = Vec::new();

    for d in &discovered {
        if let Ok(source) = parse_source(&d.path) {
            for op in source.all_operations() {
                all_ops.push((
                    op.timestamp,
                    source.id.clone(),
                    json!({
                        "id": op.id,
                        "name": op.name,
                        "status": format!("{:?}", op.status),
                        "timestamp": op.timestamp.to_rfc3339(),
                        "session_id": source.id,
                        "file_path": op.file_path(),
                        "activity_type": format!("{:?}", op.activity_type())
                    }),
                ));
            }
        }
    }

    // Sort by timestamp descending and take limit
    all_ops.sort_by(|a, b| b.0.cmp(&a.0));
    let recent: Vec<serde_json::Value> = all_ops
        .iter()
        .take(limit)
        .map(|(_, _, v)| v.clone())
        .collect();

    let result = json!({
        "operations": recent,
        "count": recent.len(),
        "total_available": all_ops.len(),
        "status": "ok"
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sources_list_returns_valid_json() {
        let params = TelemetrySourcesListParams {};
        let result = sources_list(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_source_analyze_requires_params() {
        let params = TelemetrySourceAnalyzeParams {
            session_path: None,
            project_hash: None,
        };
        let result = source_analyze(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_governance_crossref_runs() {
        let params = TelemetryGovernanceCrossrefParams { category: None };
        let result = governance_crossref(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_snapshot_evolution_runs() {
        let params = TelemetrySnapshotEvolutionParams {
            session_id: None,
            limit: Some(5),
        };
        let result = snapshot_evolution(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_intel_report_runs() {
        let params = TelemetryIntelReportParams {
            activity_limit: Some(10),
            file_limit: Some(10),
        };
        let result = intel_report(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_recent_activity_runs() {
        let params = TelemetryRecentParams { count: Some(5) };
        let result = recent_activity(params);
        assert!(result.is_ok());
    }
}
