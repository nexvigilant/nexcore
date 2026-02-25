//! Watchtower MCP Tools - Session monitoring and primitive extraction
//!
//! Provides real-time Claude Code session monitoring, primitive extraction,
//! and hook telemetry analysis.

use nexcore_fs::dirs;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::tooling::{ReadOutcome, ScanLimiter, ScanLimits, read_limited_file, snippet_for};

/// Session log entry parsed from commands.log
///
/// Tier: T3 (Domain-specific session log record)
/// Grounds to T1 Concepts via String fields
/// Ord: N/A (composite record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub session_id: String,
    pub cwd: String,
    pub tool: String,
    pub details: String,
}

/// Extracted primitive from session analysis
///
/// Tier: T3 (Domain-specific session primitive)
/// Grounds to T1 Concepts via String fields
/// Ord: N/A (composite record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Primitive {
    pub name: String,
    pub tier: String,
    pub description: String,
}

/// Anti-pattern detected in session
///
/// Tier: T3 (Domain-specific session anti-pattern)
/// Grounds to T1 Concepts via String fields
/// Ord: N/A (composite record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiPattern {
    pub name: String,
    pub description: String,
}

/// Session statistics
///
/// Tier: T3 (Domain-specific session statistics)
/// Grounds to T1 Concepts via usize fields
/// Ord: N/A (composite record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionStats {
    pub total_operations: usize,
    pub bash_commands: usize,
    pub file_reads: usize,
    pub file_writes: usize,
    pub file_edits: usize,
    pub task_spawns: usize,
    pub mcp_calls: usize,
}

/// Transfer confidence scores
///
/// Tier: T3 (Domain-specific transfer confidence)
/// Grounds to T1 Concepts via f64 fields
/// Ord: N/A (composite record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferConfidence {
    pub devops: f64,
    pub data_migration: f64,
    pub code_review: f64,
}

/// Complete session analysis result
///
/// Tier: T3 (Domain-specific session analysis)
/// Grounds to T1 Concepts via String/Vec and nested structs
/// Ord: N/A (composite record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionAnalysis {
    pub session_file: String,
    pub statistics: SessionStats,
    pub primitives: Vec<Primitive>,
    pub anti_patterns: Vec<AntiPattern>,
    pub transfer_confidence: TransferConfidence,
}

/// Get default log paths
fn get_log_paths() -> (PathBuf, PathBuf, PathBuf) {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp"));
    let commands_log = home.join(".claude/logs/commands.log");
    let sessions_dir = home.join(".claude/logs/sessions");
    let telemetry_log = home.join(".claude/logs/hook_telemetry.jsonl");
    (commands_log, sessions_dir, telemetry_log)
}

/// List available sessions
pub fn watchtower_sessions_list() -> Value {
    let (_, sessions_dir, _) = get_log_paths();
    let limits = ScanLimits::from_env();
    let mut limiter = ScanLimiter::new(limits.max_hits);

    if !sessions_dir.exists() {
        return json!({
            "sessions": [],
            "count": 0,
            "message": "No sessions directory found"
        });
    }

    let mut sessions: Vec<Value> = Vec::new();

    if let Ok(entries) = fs::read_dir(&sessions_dir) {
        for entry in entries.flatten() {
            if !limiter.allow() {
                break;
            }
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "log") {
                let filename = path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                let metadata = fs::metadata(&path).ok();
                let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
                let modified = metadata
                    .and_then(|m| m.modified().ok())
                    .map(|t| {
                        nexcore_chrono::DateTime::from(t)
                            .format("%Y-%m-%d %H:%M:%S")
                            .unwrap_or_default()
                    })
                    .unwrap_or_default();

                sessions.push(json!({
                    "filename": filename,
                    "path": path.to_string_lossy(),
                    "size_bytes": size,
                    "modified": modified
                }));
            }
        }
    }

    // Sort by modified time (newest first)
    sessions.sort_by(|a, b| b["modified"].as_str().cmp(&a["modified"].as_str()));

    let count = sessions.len();
    let mut result = json!({
        "sessions": sessions,
        "count": count,
        "sessions_dir": sessions_dir.to_string_lossy()
    });

    if let Some(notice) = limiter.notice(limits) {
        if let Ok(value) = serde_json::to_value(notice) {
            result["scan_notice"] = value;
        }
    }

    result
}

/// Get active sessions from recent log entries
pub fn watchtower_active_sessions() -> Value {
    let (commands_log, _, _) = get_log_paths();
    let limits = ScanLimits::from_env();

    if !commands_log.exists() {
        return json!({
            "active_sessions": [],
            "message": "No commands log found"
        });
    }

    let read_outcome = read_limited_file(&commands_log, limits).unwrap_or(ReadOutcome {
        content: String::new(),
        notice: None,
    });
    let content = read_outcome.content;
    let mut notices = Vec::new();
    if let Some(notice) = read_outcome.notice {
        notices.push(notice);
    }
    let max_hits = limits.max_hits.min(100);
    let mut limiter = ScanLimiter::new(max_hits);
    let mut sessions: HashMap<String, String> = HashMap::new();

    // Parse last 100 lines for session IDs
    for line in content.lines().rev() {
        if !limiter.allow() {
            break;
        }
        // Format: [HH:MM:SS] [session_id] [cwd] ...
        if let Some(start) = line.find('[') {
            if let Some(end) = line[start + 1..].find(']') {
                let after_time = &line[start + end + 3..];
                if let Some(s_start) = after_time.find('[') {
                    if let Some(s_end) = after_time[s_start + 1..].find(']') {
                        let session_id = &after_time[s_start + 1..s_start + 1 + s_end];
                        // Get cwd
                        let after_session = &after_time[s_start + s_end + 3..];
                        if let Some(c_start) = after_session.find('[') {
                            if let Some(c_end) = after_session[c_start + 1..].find(']') {
                                let cwd = &after_session[c_start + 1..c_start + 1 + c_end];
                                sessions.insert(session_id.to_string(), cwd.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    let active: Vec<Value> = sessions
        .into_iter()
        .map(|(id, cwd)| json!({"session_id": id, "cwd": cwd}))
        .collect();

    if let Some(notice) = limiter.notice(limits) {
        notices.push(notice);
    }

    let mut result = json!({
        "active_sessions": active,
        "count": active.len()
    });

    if !notices.is_empty() {
        if let Ok(value) = serde_json::to_value(notices) {
            result["scan_notice"] = value;
        }
    }

    result
}

/// Analyze a session log file and extract primitives
pub fn watchtower_analyze(session_path: Option<&str>) -> Value {
    let (_, sessions_dir, _) = get_log_paths();
    let limits = ScanLimits::from_env();

    // Determine which file to analyze
    let file_path = if let Some(path) = session_path {
        PathBuf::from(path)
    } else {
        // Get most recent session
        let mut latest: Option<(PathBuf, std::time::SystemTime)> = None;
        if let Ok(entries) = fs::read_dir(&sessions_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "log") {
                    if let Ok(meta) = fs::metadata(&path) {
                        if let Ok(modified) = meta.modified() {
                            if latest.as_ref().map_or(true, |(_, t)| modified > *t) {
                                latest = Some((path, modified));
                            }
                        }
                    }
                }
            }
        }
        match latest {
            Some((path, _)) => path,
            None => {
                return json!({
                    "error": "No session files found",
                    "sessions_dir": sessions_dir.to_string_lossy()
                });
            }
        }
    };

    if !file_path.exists() {
        return json!({
            "error": "Session file not found",
            "path": file_path.to_string_lossy()
        });
    }

    let read_outcome = read_limited_file(&file_path, limits).unwrap_or(ReadOutcome {
        content: String::new(),
        notice: None,
    });
    let content = read_outcome.content;
    let mut notices = Vec::new();
    if let Some(notice) = read_outcome.notice {
        notices.push(notice);
    }

    // Count operations
    let bash_count = content.matches("Bash:").count();
    let read_count = content.matches("Read:").count();
    let write_count = content.matches("Write:").count();
    let edit_count = content.matches("Edit:").count();
    let task_count = content.matches("Task").count();
    let mcp_count = content.matches("mcp__").count();
    let total = bash_count + read_count + write_count + edit_count + task_count + mcp_count;

    let stats = SessionStats {
        total_operations: total,
        bash_commands: bash_count,
        file_reads: read_count,
        file_writes: write_count,
        file_edits: edit_count,
        task_spawns: task_count,
        mcp_calls: mcp_count,
    };

    // Extract primitives
    let mut primitives = Vec::new();

    // Lower threshold to 3 for better detection in smaller sessions
    if total > 2 {
        primitives.push(Primitive {
            name: "SEQUENCE".to_string(),
            tier: "T1".to_string(),
            description: format!("Ordered operations detected ({} ops)", total),
        });
    }

    if write_count > 0 || edit_count > 0 {
        primitives.push(Primitive {
            name: "MAPPING".to_string(),
            tier: "T1".to_string(),
            description: format!(
                "State transformations via Write/Edit ({} transforms)",
                write_count + edit_count
            ),
        });
    }

    if read_count > 0 && (write_count + edit_count) > 0 {
        let ratio = read_count as f64 / (write_count + edit_count + 1) as f64;
        if ratio > 0.5 {
            primitives.push(Primitive {
                name: "VERIFY_BEFORE_MODIFY".to_string(),
                tier: "T2P".to_string(),
                description: format!("Read-before-write pattern (ratio: {:.2})", ratio),
            });
        }
    }

    if bash_count > 3 {
        primitives.push(Primitive {
            name: "BATCH_OPERATIONS".to_string(),
            tier: "T2P".to_string(),
            description: format!("Multiple shell commands batched ({} commands)", bash_count),
        });
    }

    if read_count > write_count + edit_count + 3 {
        primitives.push(Primitive {
            name: "EXPLORATION".to_string(),
            tier: "T2C".to_string(),
            description: "Read-heavy session (exploring codebase)".to_string(),
        });
    }

    if write_count + edit_count > read_count {
        primitives.push(Primitive {
            name: "CONSTRUCTION".to_string(),
            tier: "T2C".to_string(),
            description: "Write-heavy session (building/modifying)".to_string(),
        });
    }

    if mcp_count > 0 {
        primitives.push(Primitive {
            name: "MCP_DELEGATION".to_string(),
            tier: "T3".to_string(),
            description: format!("Delegated to MCP tools ({} calls)", mcp_count),
        });
    }

    if task_count > 0 {
        primitives.push(Primitive {
            name: "SUBAGENT_ORCHESTRATION".to_string(),
            tier: "T3".to_string(),
            description: format!("Spawned subagents ({} tasks)", task_count),
        });
    }

    // Detect anti-patterns
    let mut anti_patterns = Vec::new();

    if bash_count > 20 {
        anti_patterns.push(AntiPattern {
            name: "EXCESSIVE_BASH".to_string(),
            description: format!(
                "High shell command count ({}) - consider MCP tools",
                bash_count
            ),
        });
    }

    // Calculate transfer confidence
    let total_f = (total + 1) as f64;
    let transfer = TransferConfidence {
        devops: 0.7 + (bash_count as f64 / total_f) * 0.25,
        data_migration: 0.6 + ((write_count + edit_count) as f64 / total_f) * 0.35,
        code_review: 0.5 + (read_count as f64 / total_f) * 0.45,
    };

    let analysis = SessionAnalysis {
        session_file: file_path.to_string_lossy().to_string(),
        statistics: stats,
        primitives,
        anti_patterns,
        transfer_confidence: transfer,
    };

    let mut result =
        serde_json::to_value(analysis).unwrap_or_else(|_| json!({"error": "Serialization failed"}));
    if !notices.is_empty() {
        if let Some(obj) = result.as_object_mut() {
            if let Ok(value) = serde_json::to_value(notices) {
                obj.insert("scan_notice".to_string(), value);
            }
        }
    }
    result
}

/// Get hook telemetry statistics
pub fn watchtower_telemetry_stats() -> Value {
    let (_, _, telemetry_log) = get_log_paths();
    let limits = ScanLimits::from_env();

    if !telemetry_log.exists() {
        return json!({
            "error": "No telemetry log found",
            "path": telemetry_log.to_string_lossy()
        });
    }

    let read_outcome = read_limited_file(&telemetry_log, limits).unwrap_or(ReadOutcome {
        content: String::new(),
        notice: None,
    });
    let content = read_outcome.content;
    let mut notices = Vec::new();
    if let Some(notice) = read_outcome.notice {
        notices.push(notice);
    }
    let mut limiter = ScanLimiter::new(limits.max_hits);

    let mut tool_counts: HashMap<String, usize> = HashMap::new();
    let mut hook_timings: HashMap<String, Vec<u64>> = HashMap::new();
    let mut sessions: HashMap<String, usize> = HashMap::new();
    let mut entries_processed = 0usize;

    for line in content.lines() {
        if !limiter.allow() {
            break;
        }
        entries_processed += 1;
        if let Ok(entry) = serde_json::from_str::<Value>(line) {
            // Count tools
            if let Some(tool) = entry.get("tool").and_then(|t| t.as_str()) {
                *tool_counts.entry(tool.to_string()).or_insert(0) += 1;
            }

            // Collect hook timings
            if let (Some(hook), Some(duration)) = (
                entry.get("hook").and_then(|h| h.as_str()),
                entry.get("duration_ms").and_then(|d| d.as_u64()),
            ) {
                hook_timings
                    .entry(hook.to_string())
                    .or_default()
                    .push(duration);
            }

            // Count sessions
            if let Some(session) = entry.get("session").and_then(|s| s.as_str()) {
                *sessions.entry(session.to_string()).or_insert(0) += 1;
            }
        }
    }

    // Calculate average timings
    let avg_timings: HashMap<String, f64> = hook_timings
        .into_iter()
        .map(|(hook, times)| {
            let avg = times.iter().sum::<u64>() as f64 / times.len() as f64;
            (hook, avg)
        })
        .collect();

    if let Some(notice) = limiter.notice(limits) {
        notices.push(notice);
    }

    let mut result = json!({
        "tool_distribution": tool_counts,
        "hook_avg_timing_ms": avg_timings,
        "sessions": sessions,
        "total_entries": entries_processed
    });

    if !notices.is_empty() {
        if let Ok(value) = serde_json::to_value(notices) {
            result["scan_notice"] = value;
        }
    }

    result
}

/// Read recent log entries (last N lines)
pub fn watchtower_recent(count: Option<usize>, session_filter: Option<&str>) -> Value {
    let (commands_log, _, _) = get_log_paths();
    let limits = ScanLimits::from_env();

    if !commands_log.exists() {
        return json!({
            "entries": [],
            "message": "No commands log found"
        });
    }

    let read_outcome = read_limited_file(&commands_log, limits).unwrap_or(ReadOutcome {
        content: String::new(),
        notice: None,
    });
    let content = read_outcome.content;
    let mut notices = Vec::new();
    if let Some(notice) = read_outcome.notice {
        notices.push(notice);
    }
    let limit = count.unwrap_or(20).min(limits.max_hits);
    let mut hit_limit_reached = false;
    let mut entries: Vec<&str> = Vec::new();

    for line in content.lines().rev().filter(|line| {
        if let Some(filter) = session_filter {
            line.contains(&format!("[{}]", filter))
        } else {
            true
        }
    }) {
        if entries.len() >= limit {
            hit_limit_reached = true;
            break;
        }
        entries.push(line);
    }

    let count = entries.len();
    if hit_limit_reached {
        notices.push(crate::tooling::ScanLimitNotice {
            max_bytes: limits.max_bytes,
            max_lines: limits.max_lines,
            max_hits: limits.max_hits,
            bytes_read: 0,
            lines_read: 0,
            hits: entries.len(),
            byte_limit_reached: false,
            line_limit_reached: false,
            hit_limit_reached: true,
        });
    }

    let mut result = json!({
        "entries": entries.into_iter().rev().collect::<Vec<_>>(),
        "count": count,
        "filter": session_filter
    });

    if !notices.is_empty() {
        if let Ok(value) = serde_json::to_value(notices) {
            result["scan_notice"] = value;
        }
    }

    result
}

/// Symbol usage context
///
/// Tier: T3 (Domain-specific symbol usage)
/// Grounds to T1 Concepts via String and usize fields
/// Ord: N/A (composite record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolUsage {
    pub symbol: String,
    pub file: String,
    pub line: usize,
    pub context: String, // "definition", "equation", "reference", "code"
    pub snippet: String,
}

/// Symbol collision detection result
///
/// Tier: T3 (Domain-specific symbol collision)
/// Grounds to T1 Concepts via String fields and Vec
/// Ord: N/A (composite record)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolCollision {
    pub symbol: String,
    pub usages: Vec<SymbolUsage>,
    pub severity: String, // "high", "medium", "low"
    pub description: String,
}

/// Audit symbols in files for potential collisions
pub fn watchtower_symbol_audit(path: &str) -> Value {
    use regex::Regex;
    use std::collections::HashSet;

    let limits = ScanLimits::from_env();
    let mut scan_notices: Vec<crate::tooling::ScanLimitNotice> = Vec::new();
    let mut hit_limiter = ScanLimiter::new(limits.max_hits);
    let mut hit_limit_reached = false;

    let target = PathBuf::from(path);
    if !target.exists() {
        return json!({"error": "Path not found", "path": path});
    }

    // Patterns for symbol detection
    let definition_re = Regex::new(r"(?:let|where|define|denote)\s+([A-Z])\s*(?:=|:=|be)").ok();
    let equation_re = Regex::new(r"([A-Z])\s*=\s*[^=]").ok();
    let subscript_re = Regex::new(r"([A-Z])_\{?(\w+)\}?").ok();
    let _single_letter_re = Regex::new(r"\b([A-Z])\b").ok();

    let mut all_usages: HashMap<String, Vec<SymbolUsage>> = HashMap::new();

    // Collect files to scan
    let files: Vec<PathBuf> = if target.is_file() {
        vec![target.clone()]
    } else {
        nexcore_fs::walk::WalkDir::new(&target)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "md" || ext == "rs" || ext == "txt")
            })
            .map(|e| e.path().to_path_buf())
            .collect()
    };

    let files_count = files.len();

    'files_loop: for file_path in &files {
        let read_outcome = match read_limited_file(file_path, limits) {
            Ok(outcome) => outcome,
            Err(_) => continue,
        };
        if let Some(notice) = read_outcome.notice {
            scan_notices.push(notice);
        }
        let content = read_outcome.content;

        for (line_num, line) in content.lines().enumerate() {
            // Skip comment lines in Rust
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                continue;
            }

            // Check for definitions
            if let Some(ref re) = definition_re {
                for cap in re.captures_iter(line) {
                    if let Some(m) = cap.get(1) {
                        let symbol = m.as_str().to_string();
                        if !hit_limiter.allow() {
                            hit_limit_reached = true;
                            break 'files_loop;
                        }
                        let snippet = snippet_for(line, limits).text;
                        all_usages
                            .entry(symbol.clone())
                            .or_default()
                            .push(SymbolUsage {
                                symbol,
                                file: file_path.to_string_lossy().to_string(),
                                line: line_num + 1,
                                context: "definition".to_string(),
                                snippet,
                            });
                    }
                }
            }

            // Check for equations
            if let Some(ref re) = equation_re {
                for cap in re.captures_iter(line) {
                    if let Some(m) = cap.get(1) {
                        let symbol = m.as_str().to_string();
                        // Avoid duplicating definitions
                        let existing = all_usages.entry(symbol.clone()).or_default();
                        if !existing
                            .iter()
                            .any(|u| u.line == line_num + 1 && u.context == "definition")
                        {
                            if !hit_limiter.allow() {
                                hit_limit_reached = true;
                                break 'files_loop;
                            }
                            let snippet = snippet_for(line, limits).text;
                            existing.push(SymbolUsage {
                                symbol,
                                file: file_path.to_string_lossy().to_string(),
                                line: line_num + 1,
                                context: "equation".to_string(),
                                snippet,
                            });
                        }
                    }
                }
            }

            // Check for subscripted symbols
            if let Some(ref re) = subscript_re {
                for cap in re.captures_iter(line) {
                    if let Some(m) = cap.get(1) {
                        let symbol = m.as_str().to_string();
                        let subscript = cap.get(2).map(|s| s.as_str()).unwrap_or("");
                        let full_symbol = format!("{}_{}", symbol, subscript);
                        if !hit_limiter.allow() {
                            hit_limit_reached = true;
                            break 'files_loop;
                        }
                        let snippet = snippet_for(line, limits).text;
                        all_usages
                            .entry(full_symbol.clone())
                            .or_default()
                            .push(SymbolUsage {
                                symbol: full_symbol,
                                file: file_path.to_string_lossy().to_string(),
                                line: line_num + 1,
                                context: "subscript".to_string(),
                                snippet,
                            });
                    }
                }
            }
        }
    }

    // Detect collisions (same symbol, different contexts/meanings)
    let mut collisions: Vec<SymbolCollision> = Vec::new();
    let mut flagged: HashSet<String> = HashSet::new();

    for (symbol, usages) in &all_usages {
        if usages.len() < 2 || flagged.contains(symbol) {
            continue;
        }

        // Check for potential collision: multiple definitions or mixed contexts
        let definition_count = usages.iter().filter(|u| u.context == "definition").count();
        let unique_files: HashSet<_> = usages.iter().map(|u| &u.file).collect();

        let severity = if definition_count > 1 {
            "high"
        } else if unique_files.len() > 1 && usages.len() > 5 {
            "medium"
        } else {
            continue; // Skip low-severity
        };

        if definition_count > 1 || (unique_files.len() > 1 && usages.len() > 3) {
            collisions.push(SymbolCollision {
                symbol: symbol.clone(),
                usages: usages.clone(),
                severity: severity.to_string(),
                description: format!(
                    "Symbol '{}' has {} definitions across {} files ({} total usages)",
                    symbol,
                    definition_count,
                    unique_files.len(),
                    usages.len()
                ),
            });
            flagged.insert(symbol.clone());
        }
    }

    // Sort by severity
    collisions.sort_by(|a, b| {
        let order = |s: &str| match s {
            "high" => 0,
            "medium" => 1,
            _ => 2,
        };
        order(&a.severity).cmp(&order(&b.severity))
    });

    if hit_limit_reached {
        scan_notices.push(crate::tooling::ScanLimitNotice {
            max_bytes: limits.max_bytes,
            max_lines: limits.max_lines,
            max_hits: limits.max_hits,
            bytes_read: 0,
            lines_read: 0,
            hits: hit_limiter.hits(),
            byte_limit_reached: false,
            line_limit_reached: false,
            hit_limit_reached: true,
        });
    }

    let mut result = json!({
        "path": path,
        "files_scanned": files_count,
        "unique_symbols": all_usages.len(),
        "collisions": collisions,
        "collision_count": collisions.len()
    });

    if !scan_notices.is_empty() {
        if let Ok(value) = serde_json::to_value(scan_notices) {
            result["scan_notice"] = value;
        }
    }

    result
}

// ============================================================================
// Gemini Telemetry Tools
// ============================================================================

/// Get Gemini telemetry statistics
pub fn watchtower_gemini_stats() -> Value {
    use nexcore_vigilance::telemetry::{compute_stats, get_log_path};

    match compute_stats() {
        Ok(stats) => {
            json!({
                "total_calls": stats.total_calls,
                "success_count": stats.success_count,
                "error_count": stats.error_count,
                "total_tokens": stats.total_tokens,
                "input_tokens": stats.input_tokens,
                "output_tokens": stats.output_tokens,
                "avg_latency_ms": stats.avg_latency_ms,
                "by_session": stats.by_session,
                "by_flow": stats.by_flow,
                "log_path": get_log_path().to_string_lossy()
            })
        }
        Err(e) => json!({
            "error": format!("Failed to compute stats: {}", e),
            "log_path": get_log_path().to_string_lossy()
        }),
    }
}

/// Get recent Gemini calls
pub fn watchtower_gemini_recent(count: usize) -> Value {
    use nexcore_vigilance::telemetry::{get_log_path, read_recent};

    match read_recent(count) {
        Ok(entries) => {
            let count = entries.len();
            json!({
                "entries": entries,
                "count": count,
                "log_path": get_log_path().to_string_lossy()
            })
        }
        Err(e) => json!({
            "error": format!("Failed to read entries: {}", e),
            "entries": [],
            "count": 0
        }),
    }
}

/// Unified Claude + Gemini telemetry view
pub fn watchtower_unified(include_claude: bool, include_gemini: bool) -> Value {
    let mut result = json!({});

    if include_claude {
        let claude_stats = watchtower_telemetry_stats();
        result["claude"] = claude_stats;
    }

    if include_gemini {
        let gemini_stats = watchtower_gemini_stats();
        result["gemini"] = gemini_stats;
    }

    // Add combined totals if both are included
    if include_claude && include_gemini {
        let claude_total = result["claude"]["total_entries"].as_u64().unwrap_or(0);
        let gemini_total = result["gemini"]["total_calls"].as_u64().unwrap_or(0);

        result["combined"] = json!({
            "total_operations": claude_total + gemini_total,
            "sources": ["claude", "gemini"]
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sessions_list() {
        let result = watchtower_sessions_list();
        assert!(result.get("sessions").is_some());
        assert!(result.get("count").is_some());
    }

    #[test]
    fn test_active_sessions() {
        let result = watchtower_active_sessions();
        assert!(result.get("active_sessions").is_some());
    }

    #[test]
    fn test_telemetry_stats() {
        let result = watchtower_telemetry_stats();
        // May return error if no telemetry log exists
        assert!(result.get("error").is_some() || result.get("tool_distribution").is_some());
    }

    #[test]
    fn test_gemini_stats() {
        let result = watchtower_gemini_stats();
        // May return error if log doesn't exist, or stats if it does
        assert!(result.get("error").is_some() || result.get("total_calls").is_some());
    }

    #[test]
    fn test_gemini_recent() {
        let result = watchtower_gemini_recent(5);
        assert!(result.get("entries").is_some());
        assert!(result.get("count").is_some());
    }

    #[test]
    fn test_unified() {
        let result = watchtower_unified(true, true);
        // Should have both claude and gemini sections
        assert!(result.get("claude").is_some() || result.get("gemini").is_some());
    }
}
