use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use super::common::*;

/// [L] Landscape — survey all data sources.
pub fn landscape() -> Result<CallToolResult, McpError> {
    let conn = open_brain_db()?;

    let signals_count = count_lines(&telemetry_dir().join("signals.jsonl"));
    let sessions = db_count(&conn, "sessions");
    let antibodies = db_count(&conn, "antibodies");
    let patterns = db_count(&conn, "patterns");
    let corrections = db_count(&conn, "corrections");
    let beliefs = db_count(&conn, "beliefs");
    let preferences = db_count(&conn, "preferences");
    let trust = db_count(&conn, "trust_accumulators");
    let tool_usage = db_count(&conn, "tool_usage");
    let token_eff = db_count(&conn, "token_efficiency");
    let decision_audit = db_count(&conn, "decision_audit");

    // Implicit files
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
            read_json_file(&path)
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

    // Signal type breakdown — signals use "signal_type" field with colon-delimited format
    let signals = read_jsonl_file(&telemetry_dir().join("signals.jsonl"));
    let mut type_counts = std::collections::HashMap::new();
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
        *type_counts.entry(key).or_insert(0i64) += 1;
    }

    // Freshness
    let mut freshness = serde_json::Map::new();
    for (name, path) in [
        ("signals.jsonl", telemetry_dir().join("signals.jsonl")),
        ("brain.db", brain_db_path()),
        ("preferences.json", implicit.join("preferences.json")),
        ("state.json", hormones_path()),
    ] {
        let age = file_age_hours(&path).unwrap_or(999.0);
        let status = if age < 1.0 {
            "FRESH"
        } else if age < 24.0 {
            "RECENT"
        } else {
            "STALE"
        };
        freshness.insert(
            name.to_string(),
            json!({"age_hours": (age * 10.0).round() / 10.0, "status": status}),
        );
    }

    Ok(json_result(&json!({
        "phase": "L",
        "name": "Landscape",
        "brain_db": {
            "sessions": sessions, "antibodies": antibodies, "patterns": patterns,
            "corrections": corrections, "beliefs": beliefs, "preferences": preferences,
            "trust_accumulators": trust, "tool_usage": tool_usage,
            "token_efficiency": token_eff, "decision_audit": decision_audit,
        },
        "signals": {"count": signals_count, "type_breakdown": type_counts},
        "implicit": implicit_counts,
        "freshness": freshness,
    })))
}

/// [E] Extract — mine patterns from signals and data.
pub fn extract() -> Result<CallToolResult, McpError> {
    let mut extracted = Vec::new();

    // Mine signals
    let signals = read_jsonl_file(&telemetry_dir().join("signals.jsonl"));
    let mut blocked_tools: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
    let mut failures: std::collections::HashMap<String, i64> = std::collections::HashMap::new();

    for sig in &signals {
        // Parse colon-delimited signal_type: "cytokine:family:event:detail"
        let raw = sig
            .get("signal_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let parts: Vec<&str> = raw.splitn(4, ':').collect();
        let sig_type = parts.first().copied().unwrap_or("");
        let family = parts.get(1).copied().unwrap_or("");

        // TNF-alpha signals indicate blocked tools
        if sig_type == "cytokine" && family == "tnf_alpha" {
            let tool = sig
                .get("data")
                .and_then(|m| m.get("payload_hook"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            *blocked_tools.entry(tool.to_string()).or_insert(0) += 1;
        }
        // IL-6 signals indicate failures/errors
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
        extracted.push(json!({
            "type": "frequently_blocked_tools",
            "source": "signals",
            "data": blocked_tools,
            "confidence": 0.7,
        }));
    }

    if !failures.is_empty() {
        extracted.push(json!({
            "type": "check_failure_hotspots",
            "source": "signals",
            "data": failures,
            "confidence": 0.6,
        }));
    }

    // Mine vocabulary counters
    let vocab_path = implicit_dir().join("vocabulary_counters.json");
    if vocab_path.exists() {
        if let Ok(vocab) = read_json_file(&vocab_path) {
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
                extracted.push(json!({
                    "type": "low_vocabulary_hit_rate",
                    "source": "vocabulary_counters",
                    "data": {"rate_pct": rate, "prompts": total_prompts, "hits": total_hits},
                    "confidence": 0.5,
                }));
            }
        }
    }

    // Save extracted patterns
    let out_path = telemetry_dir().join("extracted_patterns.json");
    let content = serde_json::to_string_pretty(&extracted)
        .map_err(|e| mcp_err(&format!("serialize: {e}")))?;
    std::fs::write(&out_path, &content).map_err(|e| mcp_err(&format!("write: {e}")))?;

    Ok(json_result(&json!({
        "phase": "E",
        "name": "Extract",
        "patterns_found": extracted.len(),
        "patterns": extracted,
        "saved_to": out_path.display().to_string(),
    })))
}

/// [A] Assimilate — write patterns to knowledge stores.
pub fn assimilate() -> Result<CallToolResult, McpError> {
    let patterns_path = telemetry_dir().join("extracted_patterns.json");
    if !patterns_path.exists() {
        return Ok(json_result(&json!({
            "phase": "A", "name": "Assimilate",
            "message": "No extracted patterns. Run learn_extract first."
        })));
    }

    let patterns = read_json_file(&patterns_path)?;
    let arr = patterns
        .as_array()
        .ok_or_else(|| mcp_err("patterns not array"))?;

    let conn = open_brain_db_rw()?;
    let mut assimilated = 0;
    let mut skipped = 0;

    for p in arr {
        let ptype = p.get("type").and_then(|v| v.as_str()).unwrap_or("unknown");
        let source = p
            .get("source")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        let confidence = p.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5);

        // Build a unique ID from type+source
        let id = format!("learn-{}-{}", ptype, source);

        // Check for duplicates by id
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

        let now = chrono::Utc::now().to_rfc3339();
        let data_str = p.get("data").map(|v| v.to_string()).unwrap_or_default();
        let description = format!("{} (source: {})", ptype, source);

        conn.execute(
            "INSERT INTO patterns (id, pattern_type, description, examples, detected_at, updated_at, confidence, occurrence_count) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 1)",
            rusqlite::params![id, ptype, description, data_str, now, now, confidence],
        )
        .map_err(|e| mcp_err(&format!("insert: {e}")))?;

        assimilated += 1;
    }

    // Also write to implicit/patterns.json
    let implicit_path = implicit_dir().join("patterns.json");
    let mut existing: Vec<serde_json::Value> = if implicit_path.exists() {
        read_json_file(&implicit_path)
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
        // Check by id (matches brain.db schema) or by type+source (matches extracted format)
        let already = existing.iter().any(|e| {
            e.get("id").and_then(|v| v.as_str()) == Some(&target_id)
                || (e.get("pattern_type").and_then(|v| v.as_str()) == Some(ptype)
                    && e.get("examples")
                        .and_then(|v| v.as_array())
                        .map(|a| a.iter().any(|x| x.as_str() == Some(psource)))
                        .unwrap_or(false))
        });
        if !already {
            // Write in brain.db-compatible schema
            let now = chrono::Utc::now().to_rfc3339();
            let data_str = p.get("data").map(|v| v.to_string()).unwrap_or_default();
            let confidence = p.get("confidence").and_then(|v| v.as_f64()).unwrap_or(0.5);
            existing.push(json!({
                "id": target_id,
                "pattern_type": ptype,
                "description": format!("{} (source: {})", ptype, psource),
                "examples": [psource],
                "detected_at": now,
                "updated_at": now,
                "confidence": confidence,
                "occurrence_count": 1
            }));
        }
    }

    let content =
        serde_json::to_string_pretty(&existing).map_err(|e| mcp_err(&format!("serialize: {e}")))?;
    std::fs::write(&implicit_path, &content).map_err(|e| mcp_err(&format!("write: {e}")))?;

    Ok(json_result(&json!({
        "phase": "A",
        "name": "Assimilate",
        "assimilated": assimilated,
        "skipped": skipped,
        "total_patterns": existing.len(),
    })))
}

/// [R] Recall — verify knowledge loads.
pub fn recall() -> Result<CallToolResult, McpError> {
    let mut tests = Vec::new();
    let mut pass_count = 0;

    // Test 1: MEMORY.md
    let mem_path = memory_md_path();
    let mem_exists = mem_path.exists();
    let mem_lines = if mem_exists {
        count_lines(&mem_path)
    } else {
        0
    };
    if mem_exists && mem_lines > 5 {
        pass_count += 1;
        tests.push(json!({"test": "MEMORY.md", "pass": true, "lines": mem_lines}));
    } else {
        tests.push(json!({"test": "MEMORY.md", "pass": false, "lines": mem_lines}));
    }

    // Test 2: Preferences
    let conn = open_brain_db()?;
    let prefs = db_count(&conn, "preferences");
    if prefs > 0 {
        pass_count += 1;
        tests.push(json!({"test": "preferences", "pass": true, "count": prefs}));
    } else {
        tests.push(json!({"test": "preferences", "pass": false, "count": 0}));
    }

    // Test 3: Patterns
    let patterns = db_count(&conn, "patterns");
    if patterns > 0 {
        pass_count += 1;
        tests.push(json!({"test": "patterns", "pass": true, "count": patterns}));
    } else {
        tests.push(json!({"test": "patterns", "pass": false, "count": 0}));
    }

    // Test 4: Sessions
    let sessions = db_count(&conn, "sessions");
    if sessions > 0 {
        pass_count += 1;
        tests.push(json!({"test": "sessions", "pass": true, "count": sessions}));
    } else {
        tests.push(json!({"test": "sessions", "pass": false, "count": 0}));
    }

    // Test 5: Antibodies
    let antibodies = db_count(&conn, "antibodies");
    if antibodies > 0 {
        pass_count += 1;
        tests.push(json!({"test": "antibodies", "pass": true, "count": antibodies}));
    } else {
        tests.push(json!({"test": "antibodies", "pass": false, "count": 0}));
    }

    // Test 6: Hook-knowledge bridge (check hook binaries exist)
    let hook_dir =
        std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into()))
            .join(".claude/hooks/core-hooks/target/release");
    let hooks_exist = hook_dir.exists();
    let hook_count = if hooks_exist {
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

    Ok(json_result(&json!({
        "phase": "R",
        "name": "Recall",
        "score": format!("{pass_count}/6"),
        "passed": pass_count,
        "total": 6,
        "tests": tests,
    })))
}

/// [N] Normalize — prune dead weight.
pub fn normalize() -> Result<CallToolResult, McpError> {
    let mut pruned = 0;

    // Flag dead weight vocabulary terms
    let vocab_path = implicit_dir().join("vocabulary_counters.json");
    let mut dead_weight = Vec::new();
    if vocab_path.exists() {
        if let Ok(vocab) = read_json_file(&vocab_path) {
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

    // Deduplicate patterns
    let patterns_path = implicit_dir().join("patterns.json");
    let mut dedup_count = 0;
    if patterns_path.exists() {
        if let Ok(patterns) = read_json_file(&patterns_path) {
            if let Some(arr) = patterns.as_array() {
                let mut seen = std::collections::HashSet::new();
                let mut unique = Vec::new();
                for p in arr {
                    // Use "id" as primary dedup key, fall back to "pattern_type"
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

    // Check signal file rotation
    let signals_path = telemetry_dir().join("signals.jsonl");
    let signal_lines = count_lines(&signals_path);
    let rotated = if signal_lines > 10000 {
        // Archive and keep last 1000
        let content = std::fs::read_to_string(&signals_path).unwrap_or_default();
        let lines: Vec<&str> = content.lines().collect();
        let archive_path = telemetry_dir().join(format!(
            "signals_archive_{}.jsonl",
            chrono::Utc::now().format("%Y%m%d%H%M%S")
        ));
        std::fs::write(&archive_path, content.as_bytes()).ok();
        let tail: String = lines[lines.len().saturating_sub(1000)..].join("\n");
        std::fs::write(&signals_path, tail.as_bytes()).ok();
        true
    } else {
        false
    };

    // Record learn run
    let history_path = telemetry_dir().join("learn_history.jsonl");
    let entry = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "pruned": pruned,
        "dedup": dedup_count,
        "rotated": rotated,
    });
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&history_path)
        .map_err(|e| mcp_err(&format!("open history: {e}")))?;
    use std::io::Write;
    writeln!(
        file,
        "{}",
        serde_json::to_string(&entry).unwrap_or_default()
    )
    .map_err(|e| mcp_err(&format!("write history: {e}")))?;

    Ok(json_result(&json!({
        "phase": "N",
        "name": "Normalize",
        "dead_weight_terms": dead_weight.len(),
        "patterns_deduped": dedup_count,
        "signals_rotated": rotated,
        "signal_lines": signal_lines,
        "total_pruned": pruned,
    })))
}

/// Full LEARN pipeline.
pub fn pipeline() -> Result<CallToolResult, McpError> {
    let l = landscape()?;
    let e = extract()?;
    let a = assimilate()?;
    let r = recall()?;
    let n = normalize()?;

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
