//! Digestive System MCP tools — data pipeline processing.
//!
//! Maps data ingestion to the GI tract:
//! - Mouth: initial parsing and quality assessment
//! - Stomach: decomposition into fragments
//! - Small intestine: nutrient triage (useful data extraction)
//! - Liver: transformation and detoxification
//!
//! ## T1 Primitive Grounding
//! - Pipeline: σ(Sequence) + μ(Mapping)
//! - Quality: κ(Comparison) + ∂(Boundary)
//! - Decomposition: Σ(Sum) + N(Quantity)

use crate::params::digestive::{
    DigestiveHealthParams, DigestiveProcessParams, DigestiveTasteParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Process data through the full digestive pipeline (mouth → stomach → intestine → liver).
pub fn process(params: DigestiveProcessParams) -> Result<CallToolResult, McpError> {
    let input = &params.input;
    let kind = params.kind.as_deref().unwrap_or("auto");

    // Mouth: fragment and assess
    let byte_len = input.len();
    let line_count = input.lines().count();
    let detected_kind = if input.starts_with('{') || input.starts_with('[') {
        "object"
    } else if input.parse::<f64>().is_ok() {
        "number"
    } else if input.contains('\n') && line_count > 3 {
        "text_block"
    } else {
        "text"
    };

    let effective_kind = if kind == "auto" { detected_kind } else { kind };

    // Stomach: decompose
    let fragments: Vec<&str> = if effective_kind == "object" {
        // JSON-like: split by top-level keys
        vec![input.as_str()]
    } else {
        input.lines().collect()
    };
    let fragment_count = fragments.len();

    // Small intestine: triage (extract useful content)
    let non_empty_count = fragments.iter().filter(|f| !f.trim().is_empty()).count();
    let useful_ratio = if fragment_count > 0 {
        non_empty_count as f64 / fragment_count as f64
    } else {
        0.0
    };

    // Liver: assess toxicity (suspicious patterns)
    let mut toxins = Vec::new();
    if input.contains("password") || input.contains("secret") || input.contains("token") {
        toxins.push("potential_secret_exposure");
    }
    if byte_len > 100_000 {
        toxins.push("oversized_payload");
    }
    if input.contains("<script") || input.contains("javascript:") {
        toxins.push("potential_injection");
    }

    let health = if toxins.is_empty() && useful_ratio > 0.5 {
        "fully_digested"
    } else if toxins.is_empty() {
        "partially_digested"
    } else {
        "toxic_detected"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "digestion": {
                "status": health,
                "input_bytes": byte_len,
                "detected_kind": effective_kind,
                "fragments": fragment_count,
                "useful_fragments": non_empty_count,
                "absorption_ratio": (useful_ratio * 100.0).round() / 100.0,
                "toxins_detected": toxins,
            },
            "stages": {
                "mouth": format!("{} bytes parsed, {} lines", byte_len, line_count),
                "stomach": format!("{} fragments produced", fragment_count),
                "intestine": format!("{} useful fragments absorbed ({}%)",
                    non_empty_count, (useful_ratio * 100.0).round()),
                "liver": if toxins.is_empty() { "clean".to_string() }
                    else { format!("{} toxins neutralized", toxins.len()) },
            },
        })
        .to_string(),
    )]))
}

/// Taste (quality assess) input data before full processing.
pub fn taste(params: DigestiveTasteParams) -> Result<CallToolResult, McpError> {
    let sample = &params.sample;

    let quality = if sample.is_empty() {
        "empty"
    } else if sample.len() < 5 {
        "poor"
    } else if serde_json::from_str::<serde_json::Value>(sample).is_ok() {
        "rich"
    } else {
        "normal"
    };

    let detected_kind = if sample.starts_with('{') || sample.starts_with('[') {
        "structured"
    } else if sample
        .chars()
        .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
    {
        "numeric"
    } else {
        "freetext"
    };

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "taste": {
                "quality": quality,
                "kind": detected_kind,
                "sample_size": sample.len(),
                "recommendation": match quality {
                    "empty" => "Nothing to digest — provide input data",
                    "poor" => "Minimal data — may not yield useful nutrients",
                    "rich" => "Structured data — optimal for processing",
                    _ => "Acceptable input — proceed with digestion",
                },
            },
            "analog": {
                "sweet": "Well-structured JSON/YAML (easy calories)",
                "salty": "Tabular data (mineral-rich)",
                "bitter": "Error messages (defensive response needed)",
                "umami": "Rich natural language (complex nutrients)",
                "sour": "Malformed or stale data (potential spoilage)",
            },
        })
        .to_string(),
    )]))
}

/// Get digestive system health overview.
pub fn health(_params: DigestiveHealthParams) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "digestive_health": {
                "status": "operational",
                "components": {
                    "mouth": "Skill trigger matching + YAML parse (94 skills = microbiome)",
                    "esophagus": "SKILL.md loading pipeline",
                    "stomach": "Parameter validation + decomposition",
                    "small_intestine": "Useful data extraction and routing",
                    "large_intestine": "Output formatting + response assembly",
                    "liver": "Detoxification (secret scanning, injection detection)",
                    "gallbladder": "Bile salts (error formatting enzymes)",
                    "pancreas": "Enzyme dispatch (tool routing)",
                },
                "microbiome": {
                    "description": "94 skills = gut bacteria aiding digestion",
                    "diversity": "11 vocabulary programs + 83 domain skills",
                },
            },
            "transit_time": {
                "fast": "<200ms (cached/myelinated paths)",
                "normal": "200-600ms (standard skill execution)",
                "slow": ">600ms (complex multi-tool pipelines)",
            },
        })
        .to_string(),
    )]))
}
