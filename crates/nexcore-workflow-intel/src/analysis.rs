//! Workflow analysis — map, gap, bottleneck, and live intel.

use crate::db::{self, AutopsyRow};
use crate::error::{Result, WorkflowError};
use crate::types::*;
use rusqlite::Connection;
use std::collections::HashMap;

/// Build a workflow map from recent sessions.
pub fn build_workflow_map(conn: &Connection, days: u32) -> Result<WorkflowMap> {
    let events = db::query_tool_events(conn, days)?;
    if events.is_empty() {
        return Err(WorkflowError::NoData(format!(
            "no tool events in the last {days} days"
        )));
    }

    let transitions = db::compute_transitions(&events);

    // Top tools by frequency
    let mut tool_counts: HashMap<String, u64> = HashMap::new();
    for event in &events {
        *tool_counts.entry(event.tool.clone()).or_insert(0) += 1;
    }
    let mut top_tools: Vec<(String, u64)> = tool_counts.into_iter().collect();
    top_tools.sort_by(|a, b| b.1.cmp(&a.1));
    top_tools.truncate(20);

    // Category breakdown
    let mut categories: HashMap<&str, u64> = HashMap::new();
    for event in &events {
        *categories
            .entry(db::classify_tool(&event.tool))
            .or_insert(0) += 1;
    }
    let category_breakdown: Vec<(String, u64)> = categories
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

    // Count unique sessions
    let unique_sessions: std::collections::HashSet<&str> =
        events.iter().map(|e| e.session_id.as_str()).collect();

    Ok(WorkflowMap {
        window: format!("last {days} days"),
        sessions_analyzed: unique_sessions.len() as u64,
        total_events: events.len() as u64,
        transitions: transitions.into_iter().take(30).collect(),
        top_tools,
        category_breakdown,
    })
}

/// Analyze workflows for gaps and inefficiencies.
pub fn analyze_gaps(conn: &Connection, days: u32) -> Result<GapAnalysis> {
    let autopsies = db::query_autopsy_records(conn, days)?;
    let tool_usage = db::query_tool_usage(conn)?;
    let skill_invocations = db::query_skill_invocations(conn, days)?;

    if autopsies.is_empty() {
        return Err(WorkflowError::NoData(format!(
            "no sessions in the last {days} days"
        )));
    }

    let stats = compute_stats(&autopsies, &tool_usage, &skill_invocations);
    let mut gaps = Vec::new();

    // Gap 1: Low MCP utilization
    if stats.mcp_usage_pct < 10.0 {
        gaps.push(WorkflowGap {
            gap_type: "underused_tool".to_string(),
            description: "MCP tool utilization is critically low".to_string(),
            severity: 4,
            evidence: format!(
                "{:.1}% of tool calls use MCP (target: >15%)",
                stats.mcp_usage_pct
            ),
            suggestion: "Route more operations through MCP tools instead of bash/manual"
                .to_string(),
        });
    }

    // Gap 2: High-failure tools
    for (name, total, _success, failures) in &tool_usage {
        if *total > 5 {
            let failure_rate = *failures as f64 / *total as f64;
            if failure_rate > 0.15 {
                gaps.push(WorkflowGap {
                    gap_type: "error_prone".to_string(),
                    description: format!("Tool '{name}' has high failure rate"),
                    severity: if failure_rate > 0.3 { 5 } else { 3 },
                    evidence: format!("{failures}/{total} failures ({:.0}%)", failure_rate * 100.0),
                    suggestion: format!(
                        "Investigate root cause of {name} failures; consider wrapper or alternative"
                    ),
                });
            }
        }
    }

    // Gap 3: Low verdict rate
    if stats.full_verdict_rate < 0.5 {
        gaps.push(WorkflowGap {
            gap_type: "missing_automation".to_string(),
            description: "Less than half of sessions reach 'fully demonstrated'".to_string(),
            severity: 3,
            evidence: format!(
                "{:.0}% fully demonstrated rate",
                stats.full_verdict_rate * 100.0
            ),
            suggestion: "Improve session exhale discipline; review partial sessions for patterns"
                .to_string(),
        });
    }

    // Gap 4: Skill underutilization
    if stats.skill_invocation_rate < 0.3 {
        gaps.push(WorkflowGap {
            gap_type: "underused_tool".to_string(),
            description: "Skills are invoked in fewer than 30% of sessions".to_string(),
            severity: 2,
            evidence: format!(
                "{:.0}% skill invocation rate across {} sessions",
                stats.skill_invocation_rate * 100.0,
                stats.sessions
            ),
            suggestion: "Use skill-advisor at session start to surface relevant skills".to_string(),
        });
    }

    // Gap 5: Sessions with zero files modified (potential vapor)
    let zero_file_sessions = autopsies
        .iter()
        .filter(|a| a.files_modified == 0 && a.tool_calls > 5)
        .count();
    if zero_file_sessions > 0 {
        let pct = zero_file_sessions as f64 / autopsies.len() as f64 * 100.0;
        if pct > 20.0 {
            gaps.push(WorkflowGap {
                gap_type: "manual_step".to_string(),
                description: "Many sessions have high tool usage but zero file changes".to_string(),
                severity: 3,
                evidence: format!(
                    "{zero_file_sessions} sessions ({pct:.0}%) with >5 tools but 0 files modified"
                ),
                suggestion: "These may be research/audit sessions — verify they produce artifacts"
                    .to_string(),
            });
        }
    }

    // Gap 6: Unused high-value tools
    let known_high_value = [
        "signal_detect",
        "guardian_homeostasis_tick",
        "brain_artifact_save",
    ];
    for tool_name in &known_high_value {
        let used = tool_usage
            .iter()
            .any(|(name, total, _, _)| name.contains(tool_name) && *total > 0);
        if !used {
            gaps.push(WorkflowGap {
                gap_type: "underused_tool".to_string(),
                description: format!("High-value tool '{tool_name}' has zero recorded usage"),
                severity: 2,
                evidence: "Not found in tool_usage table".to_string(),
                suggestion: format!("Wire {tool_name} into standard workflows"),
            });
        }
    }

    gaps.sort_by(|a, b| b.severity.cmp(&a.severity));

    let health_score = compute_health_score(&stats, &gaps);

    Ok(GapAnalysis {
        window: format!("last {days} days"),
        gaps,
        health_score,
        stats,
    })
}

/// Find workflow bottlenecks.
pub fn find_bottlenecks(conn: &Connection, days: u32) -> Result<Vec<Bottleneck>> {
    let tool_usage = db::query_tool_usage(conn)?;
    let events = db::query_tool_events(conn, days)?;
    let mut bottlenecks = Vec::new();

    // Bottleneck 1: High-failure tools
    for (name, total, _success, failures) in &tool_usage {
        if *total > 10 && *failures as f64 / *total as f64 > 0.1 {
            bottlenecks.push(Bottleneck {
                name: name.clone(),
                bottleneck_type: "high_failure".to_string(),
                impact: *failures as f64 / *total as f64,
                evidence: format!("{failures}/{total} failures"),
                recommendation: format!("Audit {name} failure causes; add retry or fallback"),
            });
        }
    }

    // Bottleneck 2: Repeated reads of same file (within sessions)
    let mut file_read_counts: HashMap<String, HashMap<String, u32>> = HashMap::new();
    for event in &events {
        if event.tool == "Read" && !event.action.is_empty() {
            *file_read_counts
                .entry(event.session_id.clone())
                .or_default()
                .entry(event.action.clone())
                .or_insert(0) += 1;
        }
    }
    let repeated_reads: usize = file_read_counts
        .values()
        .flat_map(|files| files.values())
        .filter(|&&count| count > 3)
        .count();
    if repeated_reads > 5 {
        bottlenecks.push(Bottleneck {
            name: "Read (repeated)".to_string(),
            bottleneck_type: "repeated_reads".to_string(),
            impact: 0.3,
            evidence: format!("{repeated_reads} file×session pairs read >3 times"),
            recommendation: "Cache file contents or use targeted line reads".to_string(),
        });
    }

    // Bottleneck 3: High bash usage (potential MCP replacement)
    let bash_count = events.iter().filter(|e| e.tool == "Bash").count() as f64;
    let total = events.len() as f64;
    if total > 0.0 && bash_count / total > 0.3 {
        bottlenecks.push(Bottleneck {
            name: "Bash overuse".to_string(),
            bottleneck_type: "low_mcp".to_string(),
            impact: bash_count / total,
            evidence: format!(
                "{:.0}% of tool calls are Bash ({:.0}/{:.0})",
                bash_count / total * 100.0,
                bash_count,
                total
            ),
            recommendation: "Migrate common bash patterns to MCP tools or skills".to_string(),
        });
    }

    bottlenecks.sort_by(|a, b| {
        b.impact
            .partial_cmp(&a.impact)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    Ok(bottlenecks)
}

/// Generate live intel for the current session based on tool sequence.
pub fn live_intel(conn: &Connection, current_tools: &[String]) -> Result<LiveIntel> {
    if current_tools.is_empty() {
        return Ok(LiveIntel {
            current_sequence: Vec::new(),
            similar_workflows: Vec::new(),
            suggested_next: Vec::new(),
            warnings: vec!["No tool calls yet — sequence empty".to_string()],
        });
    }

    let events = db::query_tool_events(conn, 30)?;
    let autopsies = db::query_autopsy_records(conn, 30)?;

    // Build session tool sequences for comparison
    let mut session_sequences: HashMap<String, Vec<String>> = HashMap::new();
    for event in &events {
        session_sequences
            .entry(event.session_id.clone())
            .or_default()
            .push(event.tool.clone());
    }

    // Find similar past workflows by prefix match
    let mut similar: Vec<SimilarWorkflow> = Vec::new();
    for (sid, sequence) in &session_sequences {
        let sim = prefix_similarity(current_tools, sequence);
        if sim > 0.3 {
            let autopsy = autopsies.iter().find(|a| &a.session_id == sid);
            similar.push(SimilarWorkflow {
                session_id: sid.clone(),
                description: autopsy.map(|a| a.description.clone()).unwrap_or_default(),
                verdict: autopsy
                    .and_then(|a| a.verdict.clone())
                    .unwrap_or_else(|| "unknown".to_string()),
                similarity: sim,
            });
        }
    }
    similar.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    similar.truncate(5);

    // Suggest next tools based on what follows this prefix in history
    let last_tool = current_tools.last().map(|s| s.as_str()).unwrap_or("");
    let mut next_counts: HashMap<String, u64> = HashMap::new();
    for sequence in session_sequences.values() {
        for window in sequence.windows(2) {
            if window[0] == last_tool {
                *next_counts.entry(window[1].clone()).or_insert(0) += 1;
            }
        }
    }
    let mut suggested_next: Vec<(String, f64)> = next_counts
        .into_iter()
        .map(|(tool, count)| {
            let total_from_last: f64 = session_sequences
                .values()
                .flat_map(|s| s.windows(2))
                .filter(|w| w[0] == last_tool)
                .count() as f64;
            let confidence = if total_from_last > 0.0 {
                count as f64 / total_from_last
            } else {
                0.0
            };
            (tool, confidence)
        })
        .collect();
    suggested_next.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    suggested_next.truncate(5);

    // Warnings
    let mut warnings = Vec::new();
    if current_tools.len() > 20 {
        warnings
            .push("High tool count — consider whether you're exploring vs executing".to_string());
    }
    let read_count = current_tools.iter().filter(|t| *t == "Read").count();
    if read_count > 5 {
        warnings.push(format!(
            "{read_count} Read calls — velocity protocol caps at 5"
        ));
    }

    Ok(LiveIntel {
        current_sequence: current_tools.to_vec(),
        similar_workflows: similar,
        suggested_next,
        warnings,
    })
}

/// Compute prefix similarity between two tool sequences.
fn prefix_similarity(current: &[String], historical: &[String]) -> f64 {
    if current.is_empty() || historical.is_empty() {
        return 0.0;
    }
    let max_compare = current.len().min(historical.len());
    let matches = current
        .iter()
        .zip(historical.iter())
        .take(max_compare)
        .filter(|(a, b)| a == b)
        .count();
    matches as f64 / current.len() as f64
}

/// Compute aggregate stats from autopsy records.
fn compute_stats(
    autopsies: &[AutopsyRow],
    _tool_usage: &[(String, u64, u64, u64)],
    skill_invocations: &[(String, u64)],
) -> GapStats {
    let sessions = autopsies.len() as u64;
    let sessions_with_errors = autopsies
        .iter()
        .filter(|a| a.verdict.as_deref() == Some("not_demonstrated"))
        .count() as u64;

    let total_tools: u64 = autopsies.iter().map(|a| a.tool_calls).sum();
    let avg_tools = if sessions > 0 {
        total_tools as f64 / sessions as f64
    } else {
        0.0
    };

    let total_mcp: u64 = autopsies.iter().map(|a| a.mcp_calls).sum();
    let mcp_pct = if total_tools > 0 {
        total_mcp as f64 / total_tools as f64 * 100.0
    } else {
        0.0
    };

    let sessions_with_skills = autopsies
        .iter()
        .filter(|_| !skill_invocations.is_empty())
        .count();
    let skill_rate = if sessions > 0 {
        sessions_with_skills as f64 / sessions as f64
    } else {
        0.0
    };

    let fully = autopsies
        .iter()
        .filter(|a| a.verdict.as_deref() == Some("fully_demonstrated"))
        .count();
    let full_rate = if sessions > 0 {
        fully as f64 / sessions as f64
    } else {
        0.0
    };

    GapStats {
        sessions,
        sessions_with_errors,
        avg_tools_per_session: avg_tools,
        mcp_usage_pct: mcp_pct,
        skill_invocation_rate: skill_rate,
        full_verdict_rate: full_rate,
    }
}

/// Compute overall workflow health score.
///
/// Uses a weighted composite of 5 dimensions rather than per-gap penalty.
/// Each dimension is 0.0–1.0, weighted to produce a final 0.0–1.0 score.
///
/// Dimensions:
/// - Tool reliability (30%): fraction of tools with <15% failure rate
/// - MCP utilization (15%): pct of tool calls through MCP (target: 15%+)
/// - Verdict quality (20%): fully_demonstrated rate
/// - Critical gap count (25%): inverse of severity-5 gap density (capped at 20)
/// - Skill usage (10%): skill invocation rate
fn compute_health_score(stats: &GapStats, gaps: &[WorkflowGap]) -> f64 {
    // Dimension 1: Tool reliability — what fraction of high-usage tools are healthy?
    let total_gap_tools = gaps.iter().filter(|g| g.gap_type == "error_prone").count();
    // Cap denominator at a reasonable tool count to avoid near-zero scores
    let reliability = 1.0 - (total_gap_tools as f64 / 80.0).min(1.0);

    // Dimension 2: MCP utilization — 0 at 0%, 1.0 at 15%+
    let mcp_util = (stats.mcp_usage_pct / 15.0).min(1.0);

    // Dimension 3: Verdict quality — fully_demonstrated rate
    let verdict_quality = stats.full_verdict_rate;

    // Dimension 4: Critical gap density — severity 5 gaps, with diminishing impact
    let crit_gaps = gaps.iter().filter(|g| g.severity == 5).count();
    // 0 critical = 1.0, 5 = 0.75, 10 = 0.50, 20+ = 0.0
    let crit_score = 1.0 - (crit_gaps as f64 / 20.0).min(1.0);

    // Dimension 5: Skill usage rate
    let skill_score = stats.skill_invocation_rate.min(1.0);

    // Weighted composite
    let score = reliability * 0.30
        + mcp_util * 0.15
        + verdict_quality * 0.20
        + crit_score * 0.25
        + skill_score * 0.10;

    score.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_similarity_identical() {
        let a = vec!["Read".to_string(), "Edit".to_string(), "Bash".to_string()];
        let b = vec!["Read".to_string(), "Edit".to_string(), "Bash".to_string()];
        assert!((prefix_similarity(&a, &b) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_prefix_similarity_partial() {
        let a = vec!["Read".to_string(), "Edit".to_string(), "Bash".to_string()];
        let b = vec!["Read".to_string(), "Write".to_string(), "Bash".to_string()];
        let sim = prefix_similarity(&a, &b);
        assert!((sim - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_prefix_similarity_empty() {
        let a: Vec<String> = vec![];
        let b = vec!["Read".to_string()];
        assert!((prefix_similarity(&a, &b)).abs() < f64::EPSILON);
    }

    #[test]
    fn test_health_score_no_gaps() {
        let stats = GapStats {
            sessions: 100,
            sessions_with_errors: 5,
            avg_tools_per_session: 15.0,
            mcp_usage_pct: 20.0,
            skill_invocation_rate: 0.5,
            full_verdict_rate: 0.8,
        };
        let score = compute_health_score(&stats, &[]);
        assert!(score > 0.9);
    }

    #[test]
    fn test_health_score_many_gaps() {
        let stats = GapStats {
            sessions: 10,
            sessions_with_errors: 5,
            avg_tools_per_session: 5.0,
            mcp_usage_pct: 3.0,
            skill_invocation_rate: 0.1,
            full_verdict_rate: 0.2,
        };
        let gaps = vec![
            WorkflowGap {
                gap_type: "error_prone".to_string(),
                description: "test".to_string(),
                severity: 5,
                evidence: "test".to_string(),
                suggestion: "test".to_string(),
            },
            WorkflowGap {
                gap_type: "underused_tool".to_string(),
                description: "test".to_string(),
                severity: 4,
                evidence: "test".to_string(),
                suggestion: "test".to_string(),
            },
        ];
        let score = compute_health_score(&stats, &gaps);
        assert!(score < 0.8);
    }

    #[test]
    fn test_health_score_real_world_62_gaps() {
        // Simulates the actual production scenario: 62 gaps, 40 at severity 5
        let stats = GapStats {
            sessions: 259,
            sessions_with_errors: 0,
            avg_tools_per_session: 155.0,
            mcp_usage_pct: 4.3,
            skill_invocation_rate: 1.0,
            full_verdict_rate: 0.61,
        };
        let mut gaps = Vec::new();
        for _ in 0..40 {
            gaps.push(WorkflowGap {
                gap_type: "error_prone".to_string(),
                description: "test".to_string(),
                severity: 5,
                evidence: "test".to_string(),
                suggestion: "test".to_string(),
            });
        }
        for _ in 0..22 {
            gaps.push(WorkflowGap {
                gap_type: "error_prone".to_string(),
                description: "test".to_string(),
                severity: 3,
                evidence: "test".to_string(),
                suggestion: "test".to_string(),
            });
        }
        let score = compute_health_score(&stats, &gaps);
        // Must NOT floor to 0.00 — the whole point of the calibration
        assert!(score > 0.15, "Score {score} floored — calibration failed");
        // But should reflect poor health
        assert!(score < 0.60, "Score {score} too generous for 62 gaps");
    }
}
