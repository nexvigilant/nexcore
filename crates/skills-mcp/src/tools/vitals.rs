use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use super::common::*;

/// [V] Vigor — hormone state check.
pub fn vigor() -> Result<CallToolResult, McpError> {
    let path = hormones_path();
    if !path.exists() {
        return Ok(json_result(&json!({
            "status": "missing",
            "message": "No hormone state file found",
            "path": path.display().to_string()
        })));
    }

    let state = read_json_file(&path)?;
    let hormones = [
        "cortisol",
        "dopamine",
        "serotonin",
        "adrenaline",
        "oxytocin",
        "melatonin",
    ];
    let mut levels = serde_json::Map::new();
    let mut ok_count = 0;

    for h in &hormones {
        let val = state.get(h).and_then(|v| v.as_f64()).unwrap_or(-1.0);
        let status = if (0.0..=1.0).contains(&val) {
            ok_count += 1;
            "ok"
        } else {
            "out_of_range"
        };
        levels.insert(h.to_string(), json!({"value": val, "status": status}));
    }

    let age_hours = file_age_hours(&path).unwrap_or(999.0);
    let stale = age_hours > 24.0;

    Ok(json_result(&json!({
        "phase": "V",
        "name": "Vigor",
        "hormones": levels,
        "ok_count": ok_count,
        "total": hormones.len(),
        "age_hours": (age_hours * 10.0).round() / 10.0,
        "stale": stale,
        "status": if ok_count == hormones.len() && !stale { "healthy" } else { "degraded" }
    })))
}

/// [I] Immunity — antibody registry health.
pub fn immunity() -> Result<CallToolResult, McpError> {
    let conn = open_brain_db()?;
    let total = db_count(&conn, "antibodies");

    let mut stmt = conn
        .prepare("SELECT threat_type, count(*) FROM antibodies GROUP BY threat_type")
        .map_err(|e| mcp_err(&format!("query: {e}")))?;
    let classes: Vec<(String, i64)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| mcp_err(&format!("query: {e}")))?
        .filter_map(|r| r.ok())
        .collect();

    let coverage = match total {
        0 => "UNDEFENDED",
        1..=3 => "MINIMAL",
        4..=7 => "MODERATE",
        _ => "STRONG",
    };

    Ok(json_result(&json!({
        "phase": "I",
        "name": "Immunity",
        "total_antibodies": total,
        "classification_breakdown": classes.iter()
            .map(|(c, n)| json!({"type": c, "count": n}))
            .collect::<Vec<_>>(),
        "coverage": coverage
    })))
}

/// [T] Telemetry — signal processing health.
pub fn telemetry() -> Result<CallToolResult, McpError> {
    let signals_path = telemetry_dir().join("signals.jsonl");
    let signals = read_jsonl_file(&signals_path);
    let total = signals.len();

    let mut type_counts = std::collections::HashMap::new();
    for sig in &signals {
        // Signals use "signal_type" field with colon-delimited format: "cytokine:family:event:detail"
        let raw = sig
            .get("signal_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        // Extract family level (first two segments: "cytokine:il2")
        let t = raw.splitn(3, ':').take(2).collect::<Vec<_>>().join(":");
        let key = if t.is_empty() {
            "unknown".to_string()
        } else {
            t
        };
        *type_counts.entry(key).or_insert(0i64) += 1;
    }

    let receiver_exists =
        std::path::Path::new(&std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into()))
            .join(".claude/hooks/core-hooks/target/release/signal-receiver")
            .exists();

    Ok(json_result(&json!({
        "phase": "T",
        "name": "Telemetry",
        "signal_count": total,
        "type_breakdown": type_counts,
        "receiver_binary": receiver_exists,
        "signals_path": signals_path.display().to_string()
    })))
}

/// [A] Antibodies — detailed roster.
pub fn antibodies() -> Result<CallToolResult, McpError> {
    let conn = open_brain_db()?;
    let mut stmt = conn
        .prepare("SELECT name, threat_type, severity, confidence, detection, response FROM antibodies ORDER BY severity DESC")
        .map_err(|e| mcp_err(&format!("query: {e}")))?;

    let roster: Vec<serde_json::Value> = stmt
        .query_map([], |r| {
            let name: String = r.get(0)?;
            let class: String = r.get(1)?;
            let severity: String = r.get(2)?;
            let confidence: f64 = r.get(3)?;
            let detection: String = r.get(4)?;
            let response: String = r.get(5)?;
            Ok(json!({
                "name": name,
                "threat_type": class,
                "severity": severity,
                "confidence": confidence,
                "detection": serde_json::from_str::<serde_json::Value>(&detection).unwrap_or(json!(detection)),
                "response": serde_json::from_str::<serde_json::Value>(&response).unwrap_or(json!(response)),
            }))
        })
        .map_err(|e| mcp_err(&format!("query: {e}")))?
        .filter_map(|r| r.ok())
        .collect();

    // Coverage gap analysis
    let expected = ["unsafe", "panic", "injection", "ownership"];
    let names: Vec<String> = roster
        .iter()
        .filter_map(|r| {
            r.get("name")
                .and_then(|v| v.as_str())
                .map(|s| s.to_lowercase())
        })
        .collect();
    let gaps: Vec<&str> = expected
        .iter()
        .filter(|e| !names.iter().any(|n| n.contains(*e)))
        .copied()
        .collect();

    Ok(json_result(&json!({
        "phase": "A",
        "name": "Antibodies",
        "total": roster.len(),
        "roster": roster,
        "coverage_gaps": gaps
    })))
}

/// [L] Lifespan — brain session persistence.
pub fn lifespan() -> Result<CallToolResult, McpError> {
    let conn = open_brain_db()?;

    let sessions = db_count(&conn, "sessions");
    let artifacts = db_count(&conn, "artifacts");
    let patterns = db_count(&conn, "patterns");
    let preferences = db_count(&conn, "preferences");
    let beliefs = db_count(&conn, "beliefs");
    let corrections = db_count(&conn, "corrections");

    // Check implicit knowledge files
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

    Ok(json_result(&json!({
        "phase": "L",
        "name": "Lifespan",
        "brain_db": {
            "sessions": sessions,
            "artifacts": artifacts,
            "patterns": patterns,
            "preferences": preferences,
            "beliefs": beliefs,
            "corrections": corrections,
        },
        "implicit_files": implicit_counts,
    })))
}

/// [S] Synapse — overall health score.
pub fn synapse() -> Result<CallToolResult, McpError> {
    let mut score: i64 = 100;
    let mut deductions = Vec::new();

    // Check hormones
    let h_path = hormones_path();
    if !h_path.exists() {
        score -= 15;
        deductions.push("No hormone state file (-15)".to_string());
    } else if file_age_hours(&h_path).unwrap_or(999.0) > 24.0 {
        score -= 10;
        deductions.push("Hormone state stale >24h (-10)".to_string());
    }

    // Check brain.db
    let conn_result = open_brain_db();
    if let Ok(conn) = &conn_result {
        let antibodies = db_count(conn, "antibodies");
        if antibodies == 0 {
            score -= 20;
            deductions.push("No antibodies registered (-20)".to_string());
        }
        let sessions = db_count(conn, "sessions");
        if sessions == 0 {
            score -= 10;
            deductions.push("No brain sessions (-10)".to_string());
        }
        let patterns = db_count(conn, "patterns");
        if patterns == 0 {
            score -= 5;
            deductions.push("No learned patterns (-5)".to_string());
        }
    } else {
        score -= 30;
        deductions.push("brain.db unreachable (-30)".to_string());
    }

    // Check signals
    let signals_path = telemetry_dir().join("signals.jsonl");
    let signal_count = count_lines(&signals_path);
    if signal_count == 0 {
        score -= 10;
        deductions.push("No signals processed (-10)".to_string());
    }

    // Check MEMORY.md
    if !memory_md_path().exists() {
        score -= 10;
        deductions.push("MEMORY.md missing (-10)".to_string());
    }

    let score = score.max(0);
    let grade = match score {
        90..=100 => "A",
        80..=89 => "B",
        70..=79 => "C",
        60..=69 => "D",
        _ => "F",
    };

    Ok(json_result(&json!({
        "phase": "S",
        "name": "Synapse",
        "score": score,
        "grade": grade,
        "deductions": deductions,
        "max_score": 100
    })))
}

/// Full VITALS pipeline.
pub fn pipeline() -> Result<CallToolResult, McpError> {
    let v = vigor()?;
    let i = immunity()?;
    let t = telemetry()?;
    let a = antibodies()?;
    let l = lifespan()?;
    let s = synapse()?;

    // Extract the text content from each result
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
        "program": "VITALS",
        "phases": {
            "V_vigor": extract_json(&v),
            "I_immunity": extract_json(&i),
            "T_telemetry": extract_json(&t),
            "A_antibodies": extract_json(&a),
            "L_lifespan": extract_json(&l),
            "S_synapse": extract_json(&s),
        }
    })))
}
