//! Circulatory System MCP tools — data transport and routing.
//!
//! Maps data flow through Claude Code to blood circulation:
//! - Heart: MCP server pumping tool calls
//! - Arteries: outbound data flow (tool calls → resources)
//! - Veins: return data flow (results → context)
//! - Capillaries: fine-grained data exchange at tool level
//!
//! ## T1 Primitive Grounding
//! - Transport: σ(Sequence) + →(Causality)
//! - Routing: μ(Mapping) + κ(Comparison)
//! - Pressure: N(Quantity) + ∂(Boundary)

use crate::params::circulatory::{
    CirculatoryHealthParams, CirculatoryPressureParams, CirculatoryPumpParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Pump data through the circulatory system with routing.
pub fn pump(params: CirculatoryPumpParams) -> Result<CallToolResult, McpError> {
    let payload = &params.payload;
    let source = &params.source;
    let dest = params.destination.as_deref().unwrap_or("auto");

    // Route based on payload characteristics
    let routed_to = if dest != "auto" {
        dest.to_string()
    } else if payload.contains("error") || payload.contains("threat") {
        "immune".to_string()
    } else if payload.contains("config") || payload.contains("setting") {
        "nervous".to_string()
    } else if payload.len() > 10000 {
        "storage".to_string()
    } else {
        "digestive".to_string()
    };

    let cell_kind = if payload.starts_with('{') || payload.starts_with('[') {
        "red_cell"
    } else if payload.contains("alert") || payload.contains("warn") {
        "white_cell"
    } else {
        "platelet"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "pump_result": {
                "source": source,
                "destination": routed_to,
                "cell_kind": cell_kind,
                "payload_size": payload.len(),
                "route": format!("{} → heart → {} (arterial)", source, routed_to),
            },
            "analog": {
                "red_cell": "Data carrier (structured payloads)",
                "white_cell": "Defense signal (alerts, threats)",
                "platelet": "Repair data (patches, fixes)",
                "arterial": "Outbound from heart (dispatch to organs)",
                "venous": "Return to heart (results back to context)",
            },
        })
        .to_string(),
    )]))
}

/// Check blood pressure (queue depth vs capacity).
pub fn pressure(params: CirculatoryPressureParams) -> Result<CallToolResult, McpError> {
    let depth = params.queue_depth;
    let capacity = params.capacity;

    let ratio = if capacity > 0 {
        depth as f64 / capacity as f64
    } else {
        1.0
    };

    let classification = if ratio < 0.3 {
        "hypotension"
    } else if ratio < 0.7 {
        "normal"
    } else if ratio < 0.9 {
        "prehypertension"
    } else {
        "hypertension"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "blood_pressure": {
                "queue_depth": depth,
                "capacity": capacity,
                "pressure_ratio": (ratio * 100.0).round() / 100.0,
                "classification": classification,
            },
            "recommendations": match classification {
                "hypotension" => vec!["System underutilized — increase throughput"],
                "hypertension" => vec!["Queue pressure critical — add rate limiting or increase capacity"],
                "prehypertension" => vec!["Approaching capacity — monitor closely"],
                _ => vec!["Pressure within normal range"],
            },
        })
        .to_string(),
    )]))
}

/// Get circulatory system health overview.
pub fn health(_params: CirculatoryHealthParams) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "circulatory_health": {
                "status": "operational",
                "circuits": {
                    "pulmonary": "Context window ↔ Model (O2/CO2 exchange)",
                    "systemic": "MCP tools ↔ External resources (FAERS, GCloud, Wolfram)",
                    "portal": "Brain DB ↔ Implicit knowledge files",
                    "coronary": "Guardian loop (self-monitoring)",
                },
                "vessels": {
                    "aorta": "nexcore-mcp (main outflow — 394+ tools)",
                    "vena_cava": "Tool results returning to context",
                    "capillaries": "Individual tool-level data exchange",
                },
            },
            "hemodynamics": {
                "cardiac_output": "Tools dispatched per minute",
                "blood_pressure": "Queue depth / capacity ratio",
                "heart_rate": "Session-dependent dispatch frequency",
            },
        })
        .to_string(),
    )]))
}
