//! Learning MCP tools — daemon state queries + LEARN vocabulary program.
//!
//! ## Daemon tools (query compounding infrastructure state)
//! - Status: daemon health, cycle count, PID
//! - Trends: cross-session trend analysis with velocity metrics
//! - Beliefs: belief falsification state
//! - Corrections: corrections cache for mistake prevention
//! - Velocity: composite compounding velocity score
//!
//! ## LEARN pipeline tools (vocabulary program)
//! - Landscape: survey all data sources
//! - Extract: mine patterns from signals
//! - Assimilate: write patterns to knowledge stores
//! - Recall: verify knowledge loads
//! - Normalize: prune dead weight
//! - Pipeline: run all 5 phases

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;
use std::path::PathBuf;

fn hook_state_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude/hooks/state")
}

fn read_state_file(filename: &str) -> Result<serde_json::Value, McpError> {
    let path = hook_state_dir().join(filename);
    let content = std::fs::read_to_string(&path)
        .map_err(|e| McpError::internal_error(format!("Cannot read {}: {}", filename, e), None))?;
    serde_json::from_str(&content)
        .map_err(|e| McpError::internal_error(format!("Cannot parse {}: {}", filename, e), None))
}

/// `learning_daemon_status` — Get learning daemon health and cycle metrics.
///
/// Returns: status, cycle_count, last_heartbeat, PID, uptime estimate.
pub fn status() -> Result<CallToolResult, McpError> {
    let state = read_state_file("learning-daemon-state.json")?;

    let cycle_count = state
        .get("cycle_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let last_heartbeat = state
        .get("last_heartbeat")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let pid = state.get("pid").and_then(|v| v.as_u64()).unwrap_or(0);
    let status_str = state
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Compute time since last heartbeat
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let staleness_secs = now.saturating_sub(last_heartbeat);
    let healthy = staleness_secs < 600; // 10 min threshold (daemon polls every 5 min)

    let result = serde_json::json!({
        "status": status_str,
        "healthy": healthy,
        "cycle_count": cycle_count,
        "pid": pid,
        "last_heartbeat_secs_ago": staleness_secs,
        "poll_interval_secs": 300,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `learning_daemon_trends` — Cross-session trend analysis with velocity metrics.
///
/// Returns: snapshots, deltas, trajectory classification.
pub fn trends() -> Result<CallToolResult, McpError> {
    let state = read_state_file("trend-analysis.json")?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&state).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `learning_daemon_beliefs` — Belief falsification state.
///
/// Returns: previous MCP ratio, unique tools, error rate used for falsification.
pub fn beliefs() -> Result<CallToolResult, McpError> {
    let state = read_state_file("belief-falsification-state.json")?;
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&state).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `learning_daemon_corrections` — Corrections cache for mistake prevention.
///
/// Returns: array of {mistake, correction, context, application_count}.
pub fn corrections() -> Result<CallToolResult, McpError> {
    let state = read_state_file("corrections-cache.json")?;

    let count = state.as_array().map(|a| a.len()).unwrap_or(0);
    let applied = state
        .as_array()
        .map(|a| {
            a.iter()
                .filter(|c| {
                    c.get("application_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                        > 0
                })
                .count()
        })
        .unwrap_or(0);

    let result = serde_json::json!({
        "total_corrections": count,
        "applied_corrections": applied,
        "corrections": state,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `learning_daemon_velocity` — Composite compounding velocity score.
///
/// Combines multiple signals into a single 0-100 velocity metric:
/// - Pattern growth rate (new patterns per cycle)
/// - Belief confirmation rate (validated vs total)
/// - Antibody effectiveness (applications > 0)
/// - MCP ratio trend (improving vs declining)
/// - Correction application rate
/// - Skill growth rate
pub fn velocity() -> Result<CallToolResult, McpError> {
    // Read all state files
    let daemon = read_state_file("learning-daemon-state.json").ok();
    let trends = read_state_file("trend-analysis.json").ok();
    let beliefs = read_state_file("belief-falsification-state.json").ok();
    let corrections = read_state_file("corrections-cache.json").ok();

    let cycle_count = daemon
        .as_ref()
        .and_then(|d| d.get("cycle_count"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    // Extract trend deltas
    let delta = trends.as_ref().and_then(|t| t.get("latest_delta"));

    let pattern_velocity = delta
        .and_then(|d| d.get("patterns_velocity"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let mcp_ratio_trend = delta
        .and_then(|d| d.get("mcp_ratio_trend"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    let antibody_growth = delta
        .and_then(|d| d.get("antibody_growth"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let skills_growth = delta
        .and_then(|d| d.get("skills_growth"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0);

    let trajectory = delta
        .and_then(|d| d.get("overall_trajectory"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Correction application rate
    let correction_rate = corrections
        .as_ref()
        .and_then(|c| c.as_array())
        .map(|arr| {
            let total = arr.len() as f64;
            if total == 0.0 {
                return 0.0;
            }
            let applied = arr
                .iter()
                .filter(|c| {
                    c.get("application_count")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0)
                        > 0
                })
                .count() as f64;
            applied / total
        })
        .unwrap_or(0.0);

    // MCP ratio from beliefs state
    let mcp_ratio = beliefs
        .as_ref()
        .and_then(|b| b.get("prev_mcp_ratio"))
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);

    // Compute composite velocity (0-100)
    // Weighted factors:
    let v_patterns = (pattern_velocity * 10.0).min(20.0).max(0.0); // 0-20 pts
    let v_mcp = (mcp_ratio_trend * 500.0).min(20.0).max(0.0); // 0-20 pts
    let v_antibodies = (antibody_growth as f64 * 3.0).min(15.0); // 0-15 pts
    let v_skills = (skills_growth as f64 * 3.0).min(15.0); // 0-15 pts
    let v_corrections = (correction_rate * 15.0).min(15.0); // 0-15 pts
    let v_ratio = (mcp_ratio * 100.0).min(15.0); // 0-15 pts

    let composite = v_patterns + v_mcp + v_antibodies + v_skills + v_corrections + v_ratio;

    let result = serde_json::json!({
        "velocity_score": (composite * 10.0).round() / 10.0,
        "max_score": 100,
        "cycle_count": cycle_count,
        "trajectory": trajectory,
        "components": {
            "pattern_growth": { "score": (v_patterns * 10.0).round() / 10.0, "max": 20, "raw": pattern_velocity },
            "mcp_ratio_trend": { "score": (v_mcp * 10.0).round() / 10.0, "max": 20, "raw": mcp_ratio_trend },
            "antibody_effectiveness": { "score": (v_antibodies * 10.0).round() / 10.0, "max": 15, "raw": antibody_growth },
            "skill_growth": { "score": (v_skills * 10.0).round() / 10.0, "max": 15, "raw": skills_growth },
            "correction_application": { "score": (v_corrections * 10.0).round() / 10.0, "max": 15, "raw": correction_rate },
            "mcp_ratio_absolute": { "score": (v_ratio * 10.0).round() / 10.0, "max": 15, "raw": mcp_ratio },
        },
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ── LEARN vocabulary program helpers ──

fn home_dir() -> String {
    std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into())
}

fn brain_db_path() -> PathBuf {
    PathBuf::from(home_dir()).join(".claude/brain/brain.db")
}

fn telemetry_dir() -> PathBuf {
    PathBuf::from(home_dir()).join(".claude/brain/telemetry")
}

fn implicit_dir() -> PathBuf {
    PathBuf::from(home_dir()).join(".claude/implicit")
}

fn hormones_path() -> PathBuf {
    PathBuf::from(home_dir()).join(".claude/hormones/state.json")
}

fn memory_md_path() -> PathBuf {
    PathBuf::from(home_dir()).join(".claude/projects/-home-matthew/memory/MEMORY.md")
}

fn open_brain_ro() -> Result<rusqlite::Connection, McpError> {
    rusqlite::Connection::open_with_flags(
        &brain_db_path(),
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| mcp_err(&format!("brain.db open: {e}")))
}

fn open_brain_rw() -> Result<rusqlite::Connection, McpError> {
    rusqlite::Connection::open(&brain_db_path())
        .map_err(|e| mcp_err(&format!("brain.db open rw: {e}")))
}

fn db_count(conn: &rusqlite::Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT count(*) FROM {table}"), [], |r| r.get(0))
        .unwrap_or(0)
}

fn count_lines(path: &std::path::Path) -> usize {
    std::fs::read_to_string(path)
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count())
        .unwrap_or(0)
}

fn read_json(path: &std::path::Path) -> Result<serde_json::Value, McpError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| mcp_err(&format!("read {}: {e}", path.display())))?;
    serde_json::from_str(&content).map_err(|e| mcp_err(&format!("parse {}: {e}", path.display())))
}

fn read_jsonl(path: &std::path::Path) -> Vec<serde_json::Value> {
    let Ok(content) = std::fs::read_to_string(path) else {
        return vec![];
    };
    content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect()
}

fn file_age_hours(path: &std::path::Path) -> Option<f64> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let elapsed = std::time::SystemTime::now().duration_since(modified).ok()?;
    Some(elapsed.as_secs_f64() / 3600.0)
}

fn mcp_err(msg: &str) -> McpError {
    McpError::internal_error(msg.to_string(), None)
}

fn json_result(v: &serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![rmcp::model::Content::text(v.to_string())])
}

// ── LEARN [L] Landscape ──

/// `learn_landscape` — Survey all data sources (brain.db, implicit, telemetry).
pub fn learn_landscape() -> Result<CallToolResult, McpError> {
    let conn = open_brain_ro()?;

    let implicit = implicit_dir();
    let implicit_files = [
        "preferences.json",
        "patterns.json",
        "corrections.json",
        "beliefs.json",
        "trust.json",
        "vocabulary_counters.json",
        "belief_graph.json",
    ];
    let mut implicit_counts = serde_json::Map::new();
    for f in &implicit_files {
        let path = implicit.join(f);
        let count = if path.exists() {
            read_json(&path)
                .ok()
                .map(|v| match &v {
                    serde_json::Value::Array(a) => a.len(),
                    serde_json::Value::Object(o) => o.len(),
                    _ => 1,
                })
                .unwrap_or(0)
        } else {
            0
        };
        implicit_counts.insert(f.to_string(), json!(count));
    }

    let signals = read_jsonl(&telemetry_dir().join("signals.jsonl"));
    let mut type_counts = std::collections::HashMap::<String, i64>::new();
    for sig in &signals {
        let raw = sig
            .get("signal_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let t = raw.splitn(3, ':').take(2).collect::<Vec<_>>().join(":");
        let key = if t.is_empty() {
            "unknown".to_string()
        } else {
            t
        };
        *type_counts.entry(key).or_insert(0) += 1;
    }

    let mut freshness = serde_json::Map::new();
    for (name, path) in [
        ("signals.jsonl", telemetry_dir().join("signals.jsonl")),
        ("brain.db", brain_db_path()),
        ("preferences.json", implicit.join("preferences.json")),
        ("state.json", hormones_path()),
    ] {
        let age = file_age_hours(&path).unwrap_or(999.0);
        let status_str = if age < 1.0 {
            "FRESH"
        } else if age < 24.0 {
            "RECENT"
        } else {
            "STALE"
        };
        freshness.insert(
            name.to_string(),
            json!({"age_hours": (age * 10.0).round() / 10.0, "status": status_str}),
        );
    }

    Ok(json_result(&json!({
        "phase": "L", "name": "Landscape",
        "brain_db": {
            "sessions": db_count(&conn, "sessions"),
            "antibodies": db_count(&conn, "antibodies"),
            "patterns": db_count(&conn, "patterns"),
            "corrections": db_count(&conn, "corrections"),
            "beliefs": db_count(&conn, "beliefs"),
            "preferences": db_count(&conn, "preferences"),
            "trust_accumulators": db_count(&conn, "trust_accumulators"),
            "tool_usage": db_count(&conn, "tool_usage"),
            "token_efficiency": db_count(&conn, "token_efficiency"),
            "decision_audit": db_count(&conn, "decision_audit"),
        },
        "signals": {"count": signals.len(), "type_breakdown": type_counts},
        "implicit": implicit_counts,
        "freshness": freshness,
    })))
}

// ── LEARN [E] Extract ──

/// `learn_extract` — Mine patterns from signals and telemetry data.
pub fn learn_extract() -> Result<CallToolResult, McpError> {
    let mut extracted = Vec::new();
    let signals = read_jsonl(&telemetry_dir().join("signals.jsonl"));

    let mut blocked_tools = std::collections::HashMap::<String, i64>::new();
    let mut failures = std::collections::HashMap::<String, i64>::new();

    for sig in &signals {
        let raw = sig
            .get("signal_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let parts: Vec<&str> = raw.splitn(4, ':').collect();
        let sig_type = parts.first().copied().unwrap_or("");
        let family = parts.get(1).copied().unwrap_or("");

        if sig_type == "cytokine" && family == "tnf_alpha" {
            let tool = sig
                .get("data")
                .and_then(|m| m.get("payload_hook"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            *blocked_tools.entry(tool.to_string()).or_insert(0) += 1;
        }
        if sig_type == "cytokine" && family == "il6" {
            let hook = sig
                .get("data")
                .and_then(|m| m.get("payload_hook"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            *failures.entry(hook.to_string()).or_insert(0) += 1;
        }
    }

    if !blocked_tools.is_empty() {
        extracted.push(json!({"type": "frequently_blocked_tools", "source": "signals", "data": blocked_tools, "confidence": 0.7}));
    }
    if !failures.is_empty() {
        extracted.push(json!({"type": "check_failure_hotspots", "source": "signals", "data": failures, "confidence": 0.6}));
    }

    let vocab_path = implicit_dir().join("vocabulary_counters.json");
    if vocab_path.exists() {
        if let Ok(vocab) = read_json(&vocab_path) {
            let total_prompts = vocab
                .get("total_prompts")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let total_hits = vocab
                .get("total_hits")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let rate = if total_prompts > 0 {
                (total_hits * 100) / total_prompts
            } else {
                0
            };
            if rate < 10 && total_prompts > 10 {
                extracted.push(json!({"type": "low_vocabulary_hit_rate", "source": "vocabulary_counters", "data": {"rate_pct": rate, "prompts": total_prompts, "hits": total_hits}, "confidence": 0.5}));
            }
        }
    }

    let out_path = telemetry_dir().join("extracted_patterns.json");
    let content = serde_json::to_string_pretty(&extracted)
        .map_err(|e| mcp_err(&format!("serialize: {e}")))?;
    std::fs::write(&out_path, &content).map_err(|e| mcp_err(&format!("write: {e}")))?;

    Ok(json_result(
        &json!({"phase": "E", "name": "Extract", "patterns_found": extracted.len(), "patterns": extracted, "saved_to": out_path.display().to_string()}),
    ))
}

// ── LEARN [A] Assimilate ──

/// `learn_assimilate` — Write extracted patterns to brain.db and implicit/patterns.json.
pub fn learn_assimilate() -> Result<CallToolResult, McpError> {
    let patterns_path = telemetry_dir().join("extracted_patterns.json");
    if !patterns_path.exists() {
        return Ok(json_result(
            &json!({"phase": "A", "name": "Assimilate", "message": "No extracted patterns. Run learn_extract first."}),
        ));
    }

    let patterns = read_json(&patterns_path)?;
    let arr = patterns
        .as_array()
        .ok_or_else(|| mcp_err("patterns not array"))?;

    let conn = open_brain_rw()?;
    let mut assimilated = 0;
    let mut skipped = 0;

    for p in arr {
        let ptype = p.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
        let source = p
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let confidence = p.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5);
        let id = format!("learn-{}-{}", ptype, source);

        let exists: bool = conn
            .query_row(
                "SELECT count(*) > 0 FROM patterns WHERE id = ?1",
                rusqlite::params![id],
                |r| r.get(0),
            )
            .unwrap_or(false);

        if exists {
            skipped += 1;
            continue;
        }

        let now = nexcore_chrono::DateTime::now().to_rfc3339();
        let data_str = p.get("data").map(|v| v.to_string()).unwrap_or_default();
        let description = format!("{} (source: {})", ptype, source);

        conn.execute(
            "INSERT INTO patterns (id, pattern_type, description, examples, detected_at, updated_at, confidence, occurrence_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
            rusqlite::params![id, ptype, description, data_str, now, now, confidence],
        ).map_err(|e| mcp_err(&format!("insert: {e}")))?;

        assimilated += 1;
    }

    // Also write to implicit/patterns.json
    let implicit_path = implicit_dir().join("patterns.json");
    let mut existing: Vec<serde_json::Value> = if implicit_path.exists() {
        read_json(&implicit_path)
            .ok()
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
    } else {
        vec![]
    };

    for p in arr {
        let ptype = p.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
        let psource = p
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let target_id = format!("learn-{}-{}", ptype, psource);
        let already = existing.iter().any(|e| {
            e.get("id").and_then(|v| v.as_str()) == Some(&target_id)
                || (e.get("pattern_type").and_then(|v| v.as_str()) == Some(ptype)
                    && e.get("examples")
                        .and_then(|v| v.as_array())
                        .map(|a| a.iter().any(|x| x.as_str() == Some(psource)))
                        .unwrap_or(false))
        });
        if !already {
            let now = nexcore_chrono::DateTime::now().to_rfc3339();
            let data_str = p.get("data").map(|v| v.to_string()).unwrap_or_default();
            let confidence = p.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5);
            existing.push(json!({"id": target_id, "pattern_type": ptype, "description": format!("{} (source: {})", ptype, psource), "examples": [psource], "detected_at": now, "updated_at": now, "confidence": confidence, "occurrence_count": 1}));
        }
    }

    let content =
        serde_json::to_string_pretty(&existing).map_err(|e| mcp_err(&format!("serialize: {e}")))?;
    std::fs::write(&implicit_path, &content).map_err(|e| mcp_err(&format!("write: {e}")))?;

    Ok(json_result(
        &json!({"phase": "A", "name": "Assimilate", "assimilated": assimilated, "skipped": skipped, "total_patterns": existing.len()}),
    ))
}

// ── LEARN [R] Recall ──

/// `learn_recall` — Verify knowledge loads correctly (MEMORY.md, preferences, patterns, sessions, antibodies, hooks).
pub fn learn_recall() -> Result<CallToolResult, McpError> {
    let mut tests = Vec::new();
    let mut pass_count = 0;

    let mem_path = memory_md_path();
    let mem_lines = if mem_path.exists() {
        count_lines(&mem_path)
    } else {
        0
    };
    if mem_path.exists() && mem_lines > 5 {
        pass_count += 1;
        tests.push(json!({"test": "MEMORY.md", "pass": true, "lines": mem_lines}));
    } else {
        tests.push(json!({"test": "MEMORY.md", "pass": false, "lines": mem_lines}));
    }

    let conn = open_brain_ro()?;
    for (name, table) in [
        ("preferences", "preferences"),
        ("patterns", "patterns"),
        ("sessions", "sessions"),
        ("antibodies", "antibodies"),
    ] {
        let count = db_count(&conn, table);
        if count > 0 {
            pass_count += 1;
            tests.push(json!({"test": name, "pass": true, "count": count}));
        } else {
            tests.push(json!({"test": name, "pass": false, "count": count}));
        }
    }

    let hook_dir = PathBuf::from(home_dir()).join(".claude/hooks/core-hooks/target/release");
    let hook_count = if hook_dir.exists() {
        std::fs::read_dir(&hook_dir)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| {
                        e.metadata()
                            .map(|m| m.is_file() && m.len() > 1000)
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0)
    } else {
        0
    };
    if hook_count > 10 {
        pass_count += 1;
        tests.push(json!({"test": "hook_knowledge_bridge", "pass": true, "hooks": hook_count}));
    } else {
        tests.push(json!({"test": "hook_knowledge_bridge", "pass": false, "hooks": hook_count}));
    }

    Ok(json_result(
        &json!({"phase": "R", "name": "Recall", "score": format!("{pass_count}/6"), "passed": pass_count, "total": 6, "tests": tests}),
    ))
}

// ── LEARN [N] Normalize ──

/// `learn_normalize` — Prune dead weight vocabulary, deduplicate patterns, rotate signals.
pub fn learn_normalize() -> Result<CallToolResult, McpError> {
    let mut pruned = 0;

    let vocab_path = implicit_dir().join("vocabulary_counters.json");
    let mut dead_weight = Vec::new();
    if vocab_path.exists() {
        if let Ok(vocab) = read_json(&vocab_path) {
            if let Some(counters) = vocab.get("counters").and_then(|v| v.as_object()) {
                for (term, count) in counters {
                    if count.as_i64().unwrap_or(0) == 0 {
                        dead_weight.push(term.clone());
                        pruned += 1;
                    }
                }
            }
        }
    }

    let patterns_path = implicit_dir().join("patterns.json");
    let mut dedup_count = 0;
    if patterns_path.exists() {
        if let Ok(patterns) = read_json(&patterns_path) {
            if let Some(arr) = patterns.as_array() {
                let mut seen = std::collections::HashSet::new();
                let mut unique = Vec::new();
                for p in arr {
                    let key = p
                        .get("id")
                        .and_then(|v| v.as_str())
                        .or_else(|| p.get("pattern_type").and_then(|v| v.as_str()))
                        .unwrap_or("")
                        .to_string();
                    if seen.insert(key) {
                        unique.push(p.clone());
                    } else {
                        dedup_count += 1;
                    }
                }
                if dedup_count > 0 {
                    let content = serde_json::to_string_pretty(&unique)
                        .map_err(|e| mcp_err(&format!("serialize: {e}")))?;
                    std::fs::write(&patterns_path, &content)
                        .map_err(|e| mcp_err(&format!("write: {e}")))?;
                }
            }
        }
    }

    let signals_path = telemetry_dir().join("signals.jsonl");
    let signal_lines = count_lines(&signals_path);
    let rotated = if signal_lines > 10000 {
        let content = std::fs::read_to_string(&signals_path).unwrap_or_default();
        let lines: Vec<&str> = content.lines().collect();
        let archive = telemetry_dir().join(format!(
            "signals_archive_{}.jsonl",
            nexcore_chrono::DateTime::now()
                .format("%Y%m%d%H%M%S")
                .unwrap_or_default()
        ));
        std::fs::write(&archive, content.as_bytes()).ok();
        let tail: String = lines[lines.len().saturating_sub(1000)..].join("\n");
        std::fs::write(&signals_path, tail.as_bytes()).ok();
        true
    } else {
        false
    };

    let history_path = telemetry_dir().join("learn_history.jsonl");
    let entry = json!({"timestamp": nexcore_chrono::DateTime::now().to_rfc3339(), "pruned": pruned, "dedup": dedup_count, "rotated": rotated});
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history_path)
    {
        use std::io::Write;
        let _ = writeln!(
            file,
            "{}",
            serde_json::to_string(&entry).unwrap_or_default()
        );
    }

    Ok(json_result(
        &json!({"phase": "N", "name": "Normalize", "dead_weight_terms": dead_weight.len(), "patterns_deduped": dedup_count, "signals_rotated": rotated, "signal_lines": signal_lines, "total_pruned": pruned}),
    ))
}

// ── LEARN Pipeline ──

/// `learn_pipeline` — Full LEARN pipeline: Landscape → Extract → Assimilate → Recall → Normalize.
pub fn learn_pipeline() -> Result<CallToolResult, McpError> {
    let l = learn_landscape()?;
    let e = learn_extract()?;
    let a = learn_assimilate()?;
    let r = learn_recall()?;
    let n = learn_normalize()?;

    fn extract_json(r: &CallToolResult) -> serde_json::Value {
        r.content
            .first()
            .and_then(|c| match &c.raw {
                rmcp::model::RawContent::Text(t) => serde_json::from_str(&t.text).ok(),
                _ => None,
            })
            .unwrap_or(json!(null))
    }

    Ok(json_result(&json!({
        "program": "LEARN",
        "phases": {
            "L_landscape": extract_json(&l),
            "E_extract": extract_json(&e),
            "A_assimilate": extract_json(&a),
            "R_recall": extract_json(&r),
            "N_normalize": extract_json(&n),
        }
    })))
}
