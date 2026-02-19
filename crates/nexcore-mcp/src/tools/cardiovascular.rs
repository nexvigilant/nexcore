//! Cardiovascular System MCP tools — data transport, pressure, flow diagnostics.
//!
//! Maps MCP server architecture to circulatory system:
//! - Heart: nexcore-mcp binary pumping tool calls
//! - Blood pressure: throughput × resistance
//! - Blood cells: data carriers (red), defense (white), repair (platelets)
//!
//! ## T1 Primitive Grounding
//! - Pressure: N(Quantity) + ×(Product)
//! - Flow: σ(Sequence) + ν(Frequency)
//! - Diagnosis: κ(Comparison) + ∂(Boundary)

use crate::params::cardiovascular::{
    CardioBloodHealthParams, CardioBloodPressureParams, CardioDiagnoseParams, CardioVitalsParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Compute blood pressure (data throughput pressure).
pub fn blood_pressure(params: CardioBloodPressureParams) -> Result<CallToolResult, McpError> {
    let co = params.cardiac_output; // tools/min
    let pr = params.peripheral_resistance; // latency factor

    // MAP = CO × PR (Mean Arterial Pressure analog)
    let map = co * pr;

    // Systolic/Diastolic estimates (peak vs resting)
    let systolic = map * 1.3; // peak load
    let diastolic = map * 0.8; // idle

    let classification = if map < 50.0 {
        "hypotension"
    } else if map < 150.0 {
        "normal"
    } else if map < 300.0 {
        "prehypertension"
    } else {
        "hypertension"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "blood_pressure": {
                "systolic": (systolic * 100.0).round() / 100.0,
                "diastolic": (diastolic * 100.0).round() / 100.0,
                "mean_arterial": (map * 100.0).round() / 100.0,
            },
            "inputs": {
                "cardiac_output_tools_per_min": co,
                "peripheral_resistance": pr,
            },
            "classification": classification,
            "analog": {
                "systolic": "Peak tool throughput under load",
                "diastolic": "Idle throughput (background hooks only)",
                "map": "Average data transport pressure",
            },
            "recommendations": match classification {
                "hypotension" => vec!["Increase tool call frequency", "Check MCP server responsiveness"],
                "hypertension" => vec!["Add rate limiting", "Check for tool call storms", "Consider batch operations"],
                _ => vec!["System operating within normal parameters"],
            },
        })
        .to_string(),
    )]))
}

/// Assess blood health (data quality across transport).
pub fn blood_health(params: CardioBloodHealthParams) -> Result<CallToolResult, McpError> {
    let red = params.red_cells;
    let white = params.white_cells;
    let platelets = params.platelets;
    let total = red + white + platelets;

    // Normal ratios (biological reference)
    let red_ratio = if total > 0 {
        red as f64 / total as f64
    } else {
        0.0
    };
    let white_ratio = if total > 0 {
        white as f64 / total as f64
    } else {
        0.0
    };
    let platelet_ratio = if total > 0 {
        platelets as f64 / total as f64
    } else {
        0.0
    };

    // Health assessment
    let mut conditions = Vec::new();
    if red_ratio < 0.5 {
        conditions.push(json!({"condition": "anemia", "description": "Too few data carriers (MCP tools)", "severity": "warning"}));
    }
    if white_ratio > 0.3 {
        conditions.push(json!({"condition": "leukocytosis", "description": "Excessive defense hooks — potential autoimmune overhead", "severity": "info"}));
    }
    if white_ratio < 0.05 {
        conditions.push(json!({"condition": "leukopenia", "description": "Insufficient defense hooks — vulnerable to antipatterns", "severity": "warning"}));
    }
    if platelet_ratio < 0.02 {
        conditions.push(json!({"condition": "thrombocytopenia", "description": "Few error handlers — slow recovery from failures", "severity": "warning"}));
    }

    let health = if conditions.is_empty() {
        "healthy"
    } else if conditions.iter().any(|c| c["severity"] == "warning") {
        "at_risk"
    } else {
        "monitoring"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "blood_count": {
                "red_cells": red,
                "white_cells": white,
                "platelets": platelets,
                "total": total,
            },
            "ratios": {
                "red": (red_ratio * 100.0).round() / 100.0,
                "white": (white_ratio * 100.0).round() / 100.0,
                "platelets": (platelet_ratio * 100.0).round() / 100.0,
            },
            "analog": {
                "red_cells": "Data carriers — active MCP tools moving information",
                "white_cells": "Defense — hooks, validators, immunity system",
                "platelets": "Repair — error handlers, fallback mechanisms",
            },
            "conditions": conditions,
            "health": health,
        })
        .to_string(),
    )]))
}

/// Diagnose cardiovascular pathology from symptoms.
pub fn diagnose(params: CardioDiagnoseParams) -> Result<CallToolResult, McpError> {
    let symptoms = &params.symptoms;

    let mut diagnoses = Vec::new();

    for symptom in symptoms {
        let s = symptom.to_lowercase();
        let diagnosis = match s.as_str() {
            "high_latency" | "slow" => json!({
                "symptom": symptom,
                "diagnosis": "peripheral_resistance_elevated",
                "analog": "Vasoconstriction — tool dispatch bottleneck",
                "treatment": "Profile dispatch path, check for blocking hooks",
            }),
            "data_loss" | "dropped" => json!({
                "symptom": symptom,
                "diagnosis": "hemorrhage",
                "analog": "Data leak in transport pipeline",
                "treatment": "Add result validation, check error propagation",
            }),
            "backpressure" | "queue_full" => json!({
                "symptom": symptom,
                "diagnosis": "congestive_heart_failure",
                "analog": "MCP server overwhelmed — tool calls backing up",
                "treatment": "Rate limiting, batch operations, async processing",
            }),
            "intermittent" | "flaky" => json!({
                "symptom": symptom,
                "diagnosis": "arrhythmia",
                "analog": "Irregular tool dispatch timing",
                "treatment": "Check timeout settings, retry logic, connection stability",
            }),
            "timeout" => json!({
                "symptom": symptom,
                "diagnosis": "ischemia",
                "analog": "Tool calls not reaching target in time",
                "treatment": "Increase timeout, check network, reduce payload size",
            }),
            _ => json!({
                "symptom": symptom,
                "diagnosis": "unclassified",
                "analog": "Unknown cardiovascular symptom",
                "treatment": "Run full vitals check (cardio_vitals)",
            }),
        };
        diagnoses.push(diagnosis);
    }

    let severity = if diagnoses
        .iter()
        .any(|d| d["diagnosis"] == "hemorrhage" || d["diagnosis"] == "congestive_heart_failure")
    {
        "critical"
    } else if diagnoses.iter().any(|d| d["diagnosis"] == "ischemia") {
        "high"
    } else if diagnoses.iter().any(|d| d["diagnosis"] != "unclassified") {
        "moderate"
    } else {
        "low"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "diagnoses": diagnoses,
            "symptom_count": symptoms.len(),
            "severity": severity,
        })
        .to_string(),
    )]))
}

/// Get cardiac vitals overview.
pub fn vitals(_params: CardioVitalsParams) -> Result<CallToolResult, McpError> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/matthew".to_string());

    // Count MCP tools (red cells)
    let tools_count = 415u64; // Known tool count from help_catalog

    // Count hooks (white cells)
    let hooks_dir = format!("{}/.claude/hooks/core-hooks/target/release", home);
    let hook_count = std::fs::read_dir(&hooks_dir)
        .map(|rd| {
            rd.flatten()
                .filter(|e| {
                    e.file_type().is_ok_and(|ft| ft.is_file())
                        && !e.file_name().to_string_lossy().contains('.')
                        && !e.file_name().to_string_lossy().starts_with("lib")
                })
                .count() as u64
        })
        .unwrap_or(0);

    // Count error handlers (platelets) — PostToolUseFailure hooks
    let platelet_count = 5u64; // Known from hook audit

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "cardiac_vitals": {
                "heart_rate": "Session-dependent (tools/min)",
                "blood_pressure": "Use cardio_blood_pressure to compute",
                "blood_count": {
                    "red_cells_mcp_tools": tools_count,
                    "white_cells_hooks": hook_count,
                    "platelets_error_handlers": platelet_count,
                },
            },
            "circulation": {
                "pulmonary": "Context window ↔ Model (respiratory system)",
                "systemic": "MCP tools ↔ External resources (FAERS, GCloud, Wolfram)",
                "portal": "Brain DB ↔ Implicit knowledge files",
            },
            "status": "operational",
        })
        .to_string(),
    )]))
}
