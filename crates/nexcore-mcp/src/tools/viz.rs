//! STEM Visualization MCP tools
//!
//! Produces self-contained SVG diagrams for:
//! - STEM taxonomy sunburst (32 traits, 4 domains, 7 T1 groundings)
//! - Type composition (how any type decomposes to T1 primitives)
//! - Science/Chemistry/Math method loops
//! - Confidence propagation waterfall charts
//! - Bounded value number lines
//! - DAG topology with parallel execution levels

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::{
    VizBoundsParams, VizCompositionParams, VizConfidenceParams, VizDagParams, VizLoopParams,
    VizNodeConfidenceParams, VizTaxonomyParams,
};

/// Generate STEM taxonomy sunburst SVG.
pub fn taxonomy(params: VizTaxonomyParams) -> Result<CallToolResult, McpError> {
    let title = params
        .title
        .unwrap_or_else(|| "STEM Taxonomy — 32 Traits".to_string());
    let entries = nexcore_viz::taxonomy::standard_taxonomy();
    let svg = nexcore_viz::render_taxonomy(&entries, &title, &nexcore_viz::Theme::default());

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        svg.to_string(),
    )]))
}

/// Generate type composition diagram SVG.
pub fn composition(params: VizCompositionParams) -> Result<CallToolResult, McpError> {
    let primitives: Vec<nexcore_viz::PrimitiveNode> = params
        .primitives
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|name| nexcore_viz::PrimitiveNode {
            name: name.to_string(),
            symbol: primitive_symbol(name).to_string(),
            role: String::new(),
        })
        .collect();

    let comp = nexcore_viz::TypeComposition {
        type_name: params.type_name,
        tier: params.tier,
        primitives,
        dominant: params.dominant,
        confidence: params.confidence.unwrap_or(0.80),
    };

    let svg = nexcore_viz::render_composition(&comp, &nexcore_viz::Theme::default());
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        svg.to_string(),
    )]))
}

/// Generate science/chemistry/math loop SVG.
pub fn method_loop(params: VizLoopParams) -> Result<CallToolResult, McpError> {
    let (steps, name) = match params.domain.to_lowercase().as_str() {
        "science" => (nexcore_viz::science_loop::science_loop(), "SCIENCE"),
        "chemistry" | "chem" => (nexcore_viz::science_loop::chemistry_loop(), "CHEMISTRY"),
        "math" | "mathematics" => (nexcore_viz::science_loop::math_loop(), "MATHEMATICS"),
        other => {
            return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                format!("Unknown domain: {other}. Use: science, chemistry, or math"),
            )]));
        }
    };

    let svg = nexcore_viz::render_science_loop(&steps, name, &nexcore_viz::Theme::default());
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        svg.to_string(),
    )]))
}

/// Generate confidence propagation waterfall SVG.
pub fn confidence(params: VizConfidenceParams) -> Result<CallToolResult, McpError> {
    let title = params
        .title
        .unwrap_or_else(|| "Confidence Propagation".to_string());

    // Parse claims JSON
    let raw: Vec<serde_json::Value> = serde_json::from_str(&params.claims)
        .map_err(|e| McpError::invalid_params(format!("Invalid claims JSON: {e}"), None))?;

    let claims: Vec<nexcore_viz::Claim> = raw
        .iter()
        .map(|v| nexcore_viz::Claim {
            text: v["text"].as_str().unwrap_or("claim").to_string(),
            confidence: v["confidence"].as_f64().unwrap_or(0.5),
            proof_type: v["proof_type"].as_str().unwrap_or("derived").to_string(),
            parent: v["parent"].as_u64().map(|p| p as usize),
        })
        .collect();

    let svg = nexcore_viz::render_confidence_chain(&claims, &title, &nexcore_viz::Theme::default());
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        svg.to_string(),
    )]))
}

/// Generate bounds visualization SVG.
pub fn bounds(params: VizBoundsParams) -> Result<CallToolResult, McpError> {
    let bounded = nexcore_viz::BoundedValue {
        value: params.value,
        lower: params.lower,
        upper: params.upper,
        label: params.label.unwrap_or_else(|| "value".to_string()),
    };

    let svg = nexcore_viz::render_bounds(&bounded, &nexcore_viz::Theme::default());
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        svg.to_string(),
    )]))
}

/// Generate DAG topology visualization SVG.
pub fn dag(params: VizDagParams) -> Result<CallToolResult, McpError> {
    let title = params.title.unwrap_or_else(|| "DAG Topology".to_string());

    // Parse edges JSON
    let raw_edges: Vec<Vec<String>> = serde_json::from_str(&params.edges)
        .map_err(|e| McpError::invalid_params(format!("Invalid edges JSON: {e}"), None))?;

    // Collect unique node IDs
    let mut node_ids = std::collections::HashSet::new();
    let mut edges = Vec::new();

    for edge in &raw_edges {
        if edge.len() >= 2 {
            node_ids.insert(edge[0].clone());
            node_ids.insert(edge[1].clone());
            edges.push(nexcore_viz::DagEdge {
                from: edge[0].clone(),
                to: edge[1].clone(),
            });
        }
    }

    let nodes: Vec<nexcore_viz::DagNode> = node_ids
        .into_iter()
        .map(|id| nexcore_viz::DagNode {
            label: id.clone(),
            id,
            color: None,
        })
        .collect();

    let svg = nexcore_viz::render_dag(&nodes, &edges, &title, &nexcore_viz::Theme::default());
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        svg.to_string(),
    )]))
}

/// Compute node confidence for Observatory graph nodes.
///
/// Returns Measured<f64> — confidence score with calibration metadata.
// CALIBRATION: Matches TypeScript Observatory frontends exactly.
pub fn node_confidence(params: VizNodeConfidenceParams) -> Result<CallToolResult, McpError> {
    use nexcore_viz::node_confidence::{ConfidenceSource, compute_confidence};

    let source = match params.source.to_lowercase().as_str() {
        "chi_squared" => {
            let chi_sq = params.chi_squared.ok_or_else(|| {
                McpError::invalid_params("chi_squared field required for chi_squared source", None)
            })?;
            ConfidenceSource::ChiSquared {
                chi_squared: chi_sq,
                fdr_rejected: params.fdr_rejected.unwrap_or(true),
            }
        }
        "signal_strength" => {
            let prr = params.prr.ok_or_else(|| {
                McpError::invalid_params("prr field required for signal_strength source", None)
            })?;
            ConfidenceSource::SignalStrength {
                prr,
                signal_detected: params.signal_detected.unwrap_or(false),
            }
        }
        "severity" => {
            let sev = params.severity.ok_or_else(|| {
                McpError::invalid_params("severity field required for severity source", None)
            })?;
            ConfidenceSource::Severity { severity: sev }
        }
        "relevance_score" => {
            let sc = params.score.ok_or_else(|| {
                McpError::invalid_params("score field required for relevance_score source", None)
            })?;
            ConfidenceSource::RelevanceScore { score: sc }
        }
        "structural_certainty" => ConfidenceSource::StructuralCertainty {
            api_derived: params.api_derived.unwrap_or(false),
        },
        other => {
            return Err(McpError::invalid_params(
                format!(
                    "Unknown source: {other}. Use: chi_squared, signal_strength, severity, relevance_score, structural_certainty"
                ),
                None,
            ));
        }
    };

    let result = compute_confidence(&source);
    let json = serde_json::json!({
        "confidence": result.value,
        "confidence_meta": result.confidence.value(),
        "source": params.source,
        "measured": true,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        json.to_string(),
    )]))
}

/// Map primitive name to unicode symbol.
fn primitive_symbol(name: &str) -> &'static str {
    match name.to_lowercase().as_str() {
        "mapping" => "\u{03bc}",
        "sequence" => "\u{03c3}",
        "recursion" => "\u{03c1}",
        "state" => "\u{03c2}",
        "persistence" => "\u{03c0}",
        "boundary" => "\u{2202}",
        "sum" => "\u{03a3}",
        "void" => "\u{2205}",
        "frequency" => "\u{03bd}",
        "existence" => "\u{2203}",
        "causality" => "\u{2192}",
        "comparison" => "\u{03ba}",
        "quantity" => "N",
        "location" => "\u{03bb}",
        "irreversibility" => "\u{221d}",
        "product" => "\u{00d7}",
        _ => "?",
    }
}
