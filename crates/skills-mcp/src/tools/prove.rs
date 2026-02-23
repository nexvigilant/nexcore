use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use super::common::*;

/// [P] Prepare — environment check and baseline capture.
pub fn prepare() -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".into());
    let home = std::path::PathBuf::from(&home);

    // Claude CLI
    let claude_bin = home.join(".local/bin/claude");
    let claude_exists = claude_bin.exists();

    // MCP binary
    let mcp_bin = home.join("nexcore/crates/nexcore-mcp/target/release/nexcore-mcp");
    let mcp_exists = mcp_bin.exists();
    let mcp_size = if mcp_exists {
        std::fs::metadata(&mcp_bin).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    // Brain DB
    let db_path = brain_db_path();
    let db_exists = db_path.exists();
    let db_size = if db_exists {
        std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    // MCP servers from ~/.claude.json (NOT settings.json)
    let claude_json = claude_json_path();
    let mcp_servers: Vec<String> = if claude_json.exists() {
        read_json_file(&claude_json)
            .ok()
            .and_then(|v| {
                v.get("mcpServers")
                    .and_then(|s| s.as_object())
                    .map(|o| o.keys().cloned().collect())
            })
            .unwrap_or_default()
    } else {
        vec![]
    };

    // Capture baseline
    let conn = open_brain_db()?;
    let sessions = db_count(&conn, "sessions");
    let antibodies = db_count(&conn, "antibodies");
    let patterns = db_count(&conn, "patterns");
    let preferences = db_count(&conn, "preferences");
    let signals = count_lines(&telemetry_dir().join("signals.jsonl"));

    let baseline = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "sessions": sessions,
        "antibodies": antibodies,
        "patterns": patterns,
        "preferences": preferences,
        "signals": signals,
    });

    // Save baseline
    let baselines_dir = telemetry_dir().join("prove_baselines");
    std::fs::create_dir_all(&baselines_dir).map_err(|e| mcp_err(&format!("mkdir: {e}")))?;
    let baseline_file = baselines_dir.join(format!(
        "baseline_{}.json",
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    ));
    std::fs::write(
        &baseline_file,
        serde_json::to_string_pretty(&baseline).unwrap_or_default(),
    )
    .map_err(|e| mcp_err(&format!("write baseline: {e}")))?;

    let ready = claude_exists && db_exists && !mcp_servers.is_empty();

    Ok(json_result(&json!({
        "phase": "P",
        "name": "Prepare",
        "claude_cli": claude_exists,
        "mcp_binary": {"exists": mcp_exists, "size_bytes": mcp_size},
        "brain_db": {"exists": db_exists, "size_bytes": db_size},
        "mcp_servers": mcp_servers,
        "baseline": baseline,
        "baseline_saved": baseline_file.display().to_string(),
        "ready": ready,
    })))
}

/// [O] Observe — parse sub-Claude findings.
pub fn observe() -> Result<CallToolResult, McpError> {
    let results_dir = telemetry_dir().join("prove_results");
    if !results_dir.exists() {
        return Ok(json_result(&json!({
            "phase": "O", "name": "Observe",
            "message": "No prove_results directory. Run PROVE R.sh (sub-Claude) first."
        })));
    }

    // Find most recent run output
    let mut runs: Vec<_> = std::fs::read_dir(&results_dir)
        .map_err(|e| mcp_err(&format!("readdir: {e}")))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map(|n| n.starts_with("run_") && n.ends_with(".txt"))
                .unwrap_or(false)
        })
        .collect();
    runs.sort_by_key(|e| std::cmp::Reverse(e.file_name()));

    let latest = runs
        .first()
        .ok_or_else(|| mcp_err("No run output files found. Run PROVE R.sh first."))?;

    let content =
        std::fs::read_to_string(latest.path()).map_err(|e| mcp_err(&format!("read: {e}")))?;
    let content_lower = content.to_lowercase();

    // Extract findings
    let brain = if content_lower.contains("session") {
        if content_lower.contains("sessions:")
            || content_lower.contains("session") && content_lower.contains(char::is_numeric)
        {
            "operational"
        } else {
            "mentioned"
        }
    } else {
        "unknown"
    };

    let immunity = if content_lower.contains("antibod") || content_lower.contains("immun") {
        if content_lower.contains("antibodies:")
            || content_lower.contains("antibod") && content_lower.contains(char::is_numeric)
        {
            "operational"
        } else {
            "mentioned"
        }
    } else {
        "unknown"
    };

    let hormones = if content_lower.contains("hormon")
        || content_lower.contains("cortisol")
        || content_lower.contains("dopamine")
    {
        if content_lower.contains("hormones:")
            || content_lower.contains("cortisol") && content_lower.contains("0.")
        {
            "operational"
        } else {
            "mentioned"
        }
    } else {
        "unknown"
    };

    let error_count = content_lower.matches("error").count()
        + content_lower.matches("fail").count()
        + content_lower.matches("not found").count();

    let findings = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "source": latest.path().display().to_string(),
        "brain": brain,
        "immunity": immunity,
        "hormones": hormones,
        "error_indicators": error_count,
        "output_lines": content.lines().count(),
    });

    // Save findings
    let findings_path = results_dir.join("findings_latest.json");
    std::fs::write(
        &findings_path,
        serde_json::to_string_pretty(&findings).unwrap_or_default(),
    )
    .map_err(|e| mcp_err(&format!("write: {e}")))?;

    Ok(json_result(&json!({
        "phase": "O",
        "name": "Observe",
        "findings": findings,
        "saved_to": findings_path.display().to_string(),
    })))
}

/// [V] Validate — compare against baselines.
pub fn validate() -> Result<CallToolResult, McpError> {
    let results_dir = telemetry_dir().join("prove_results");
    let findings_path = results_dir.join("findings_latest.json");

    if !findings_path.exists() {
        return Ok(json_result(&json!({
            "phase": "V", "name": "Validate",
            "message": "No findings file. Run prove_observe first."
        })));
    }

    let findings = read_json_file(&findings_path)?;
    let mut pass = 0i64;
    let mut fail = 0i64;
    let mut total = 0i64;
    let mut checks = Vec::new();

    // Check 1: Brain sessions detected
    total += 1;
    let brain = findings
        .get("brain")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    if brain == "operational" || brain == "mentioned" {
        pass += 1;
        checks.push(json!({"check": "brain_sessions", "result": "PASS", "detail": brain}));
    } else {
        fail += 1;
        checks.push(json!({"check": "brain_sessions", "result": "FAIL", "detail": brain}));
    }

    // Check 2: Immunity detected
    total += 1;
    let immunity = findings
        .get("immunity")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    if immunity == "operational" || immunity == "mentioned" {
        pass += 1;
        checks.push(json!({"check": "immunity", "result": "PASS", "detail": immunity}));
    } else {
        fail += 1;
        checks.push(json!({"check": "immunity", "result": "FAIL", "detail": immunity}));
    }

    // Check 3: Hormones detected
    total += 1;
    let hormones = findings
        .get("hormones")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    if hormones == "operational" || hormones == "mentioned" {
        pass += 1;
        checks.push(json!({"check": "hormones", "result": "PASS", "detail": hormones}));
    } else {
        fail += 1;
        checks.push(json!({"check": "hormones", "result": "FAIL", "detail": hormones}));
    }

    // Check 4: Error rate
    total += 1;
    let errors = findings
        .get("error_indicators")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    if errors <= 2 {
        pass += 1;
        checks.push(json!({"check": "error_rate", "result": "PASS", "errors": errors}));
    } else {
        fail += 1;
        checks.push(json!({"check": "error_rate", "result": "FAIL", "errors": errors}));
    }

    // Check 5: Baseline comparison
    let baselines_dir = telemetry_dir().join("prove_baselines");
    if baselines_dir.exists() {
        let mut baselines: Vec<_> = std::fs::read_dir(&baselines_dir)
            .ok()
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| {
                        e.file_name()
                            .to_str()
                            .map(|n| n.starts_with("baseline_") && n.ends_with(".json"))
                            .unwrap_or(false)
                    })
                    .collect()
            })
            .unwrap_or_default();
        baselines.sort_by_key(|e| std::cmp::Reverse(e.file_name()));

        if let Some(latest) = baselines.first() {
            total += 1;
            if let Ok(baseline) = read_json_file(&latest.path()) {
                let baseline_ab = baseline
                    .get("antibodies")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let conn = open_brain_db()?;
                let current_ab = db_count(&conn, "antibodies");
                if current_ab >= baseline_ab {
                    pass += 1;
                    checks.push(json!({
                        "check": "baseline_antibodies", "result": "PASS",
                        "current": current_ab, "baseline": baseline_ab,
                    }));
                } else {
                    fail += 1;
                    checks.push(json!({
                        "check": "baseline_antibodies", "result": "FAIL",
                        "current": current_ab, "baseline": baseline_ab,
                    }));
                }
            }
        }
    }

    let rate = if total > 0 { (pass * 100) / total } else { 0 };

    // Save validation
    let validation = json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "passed": pass,
        "failed": fail,
        "total": total,
        "rate": rate,
    });

    let validation_path = results_dir.join("validation_latest.json");
    std::fs::write(
        &validation_path,
        serde_json::to_string_pretty(&validation).unwrap_or_default(),
    )
    .map_err(|e| mcp_err(&format!("write: {e}")))?;

    Ok(json_result(&json!({
        "phase": "V",
        "name": "Validate",
        "passed": pass,
        "failed": fail,
        "total": total,
        "rate_pct": rate,
        "checks": checks,
        "saved_to": validation_path.display().to_string(),
    })))
}

/// [E] Evaluate — track improvement over time.
pub fn evaluate() -> Result<CallToolResult, McpError> {
    let results_dir = telemetry_dir().join("prove_results");
    let validation_path = results_dir.join("validation_latest.json");
    let history_path = results_dir.join("prove_history.jsonl");

    // Append latest validation to history
    if validation_path.exists() {
        let validation = read_json_file(&validation_path)?;
        let compact = serde_json::to_string(&validation).unwrap_or_default();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&history_path)
            .map_err(|e| mcp_err(&format!("open history: {e}")))?;
        use std::io::Write;
        writeln!(file, "{compact}").map_err(|e| mcp_err(&format!("write history: {e}")))?;
    }

    // Read history
    let history = read_jsonl_file(&history_path);
    let total_runs = history.len();

    let runs: Vec<serde_json::Value> = history
        .iter()
        .map(|h| {
            json!({
                "timestamp": h.get("timestamp").and_then(|v| v.as_str()).unwrap_or("?"),
                "passed": h.get("passed").and_then(|v| v.as_i64()).unwrap_or(0),
                "failed": h.get("failed").and_then(|v| v.as_i64()).unwrap_or(0),
                "total": h.get("total").and_then(|v| v.as_i64()).unwrap_or(0),
                "rate": h.get("rate").and_then(|v| v.as_i64()).unwrap_or(0),
            })
        })
        .collect();

    let trend = if total_runs >= 2 {
        let first_rate = history
            .first()
            .and_then(|h| h.get("rate").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        let last_rate = history
            .last()
            .and_then(|h| h.get("rate").and_then(|v| v.as_i64()))
            .unwrap_or(0);
        let avg: i64 = history
            .iter()
            .filter_map(|h| h.get("rate").and_then(|v| v.as_i64()))
            .sum::<i64>()
            / total_runs as i64;

        let direction = match last_rate.cmp(&first_rate) {
            std::cmp::Ordering::Greater => "IMPROVING",
            std::cmp::Ordering::Less => "REGRESSING",
            std::cmp::Ordering::Equal => "STABLE",
        };

        json!({
            "first_rate": first_rate,
            "last_rate": last_rate,
            "average_rate": avg,
            "direction": direction,
        })
    } else {
        json!({"message": "Need 2+ runs to compute trend"})
    };

    Ok(json_result(&json!({
        "phase": "E",
        "name": "Evaluate",
        "total_runs": total_runs,
        "history": runs,
        "trend": trend,
    })))
}
