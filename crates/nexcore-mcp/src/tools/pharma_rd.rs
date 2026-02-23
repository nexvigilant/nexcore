//! Pharma R&D taxonomy MCP tools.
//!
//! Predictive pharmaceutical R&D taxonomy — 62 concepts across 4 tiers,
//! 9-stage pipeline with Chomsky classification, and cross-domain
//! transfer confidence analysis.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::pharma_rd::{
    PharmaClassifyGeneratorsParams, PharmaLookupTransferParams, PharmaPipelineStageParams,
    PharmaStrongestTransfersParams, PharmaSymbolCoverageParams, PharmaTaxonomySummaryParams,
    PharmaTransferMatrixParams, PharmaWeakestTransfersParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

fn parse_primitive(s: &str) -> Option<nexcore_pharma_rd::PharmaPrimitive> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
}

fn parse_domain(s: &str) -> Option<nexcore_pharma_rd::TransferDomain> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
}

fn parse_stage(s: &str) -> Option<nexcore_pharma_rd::PipelineStage> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
}

fn parse_generator(s: &str) -> Option<nexcore_pharma_rd::Generator> {
    serde_json::from_value(serde_json::Value::String(s.to_string())).ok()
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Get the full pharma R&D taxonomy summary (concept counts by tier).
pub fn pharma_taxonomy_summary(
    _p: PharmaTaxonomySummaryParams,
) -> Result<CallToolResult, McpError> {
    let summary = nexcore_pharma_rd::taxonomy_summary();
    ok_json(serde_json::to_value(&summary).unwrap_or_default())
}

/// Look up transfer confidence for a pharma primitive to a target domain.
pub fn pharma_lookup_transfer(p: PharmaLookupTransferParams) -> Result<CallToolResult, McpError> {
    let primitive = match parse_primitive(&p.primitive) {
        Some(v) => v,
        None => return err_result(&["Unknown primitive: '", &p.primitive, "'"].concat()),
    };
    let domain = match parse_domain(&p.domain) {
        Some(v) => v,
        None => return err_result(&["Unknown domain: '", &p.domain, "'"].concat()),
    };
    let conf = nexcore_pharma_rd::lookup_transfer(primitive, domain);
    ok_json(json!({
        "primitive": p.primitive,
        "domain": p.domain,
        "structural": conf.structural,
        "functional": conf.functional,
        "contextual": conf.contextual,
        "source_modifier": conf.source_modifier,
        "raw_score": conf.raw_score(),
        "final_score": conf.final_score(),
        "label": conf.label(),
        "limiting_factor": conf.limiting_factor(),
    }))
}

/// Get the full transfer confidence matrix (all primitives x all domains).
pub fn pharma_transfer_matrix(_p: PharmaTransferMatrixParams) -> Result<CallToolResult, McpError> {
    let matrix = nexcore_pharma_rd::transfer_matrix();
    let items: Vec<serde_json::Value> = matrix
        .iter()
        .map(|(prim, dom, conf)| {
            json!({
                "primitive": format!("{prim:?}"),
                "domain": format!("{dom:?}"),
                "final_score": conf.final_score(),
                "label": conf.label(),
            })
        })
        .collect();
    ok_json(json!({
        "total": items.len(),
        "matrix": items,
    }))
}

/// Get the top N strongest cross-domain transfer corridors.
pub fn pharma_strongest_transfers(
    p: PharmaStrongestTransfersParams,
) -> Result<CallToolResult, McpError> {
    let n = p.top_n.unwrap_or(10);
    let results = nexcore_pharma_rd::strongest_transfers(n);
    let items: Vec<serde_json::Value> = results
        .iter()
        .map(|(prim, dom, score)| {
            json!({
                "primitive": format!("{prim:?}"),
                "domain": format!("{dom:?}"),
                "score": score,
            })
        })
        .collect();
    ok_json(json!({
        "top_n": n,
        "results": items,
    }))
}

/// Get the bottom N weakest cross-domain transfer corridors.
pub fn pharma_weakest_transfers(
    p: PharmaWeakestTransfersParams,
) -> Result<CallToolResult, McpError> {
    let n = p.bottom_n.unwrap_or(10);
    let results = nexcore_pharma_rd::weakest_transfers(n);
    let items: Vec<serde_json::Value> = results
        .iter()
        .map(|(prim, dom, score)| {
            json!({
                "primitive": format!("{prim:?}"),
                "domain": format!("{dom:?}"),
                "score": score,
            })
        })
        .collect();
    ok_json(json!({
        "bottom_n": n,
        "results": items,
    }))
}

/// Get Lex Primitiva symbol coverage across the R&D pipeline.
pub fn pharma_symbol_coverage(_p: PharmaSymbolCoverageParams) -> Result<CallToolResult, McpError> {
    let coverage = nexcore_pharma_rd::pipeline::symbol_coverage();
    let items: Vec<serde_json::Value> = coverage
        .iter()
        .map(|&(ref sym, count)| {
            json!({
                "symbol": sym.glyph(),
                "stage_count": count,
            })
        })
        .collect();
    ok_json(json!({
        "total_symbols": items.len(),
        "coverage": items,
    }))
}

/// Get info about a specific R&D pipeline stage.
pub fn pharma_pipeline_stage(p: PharmaPipelineStageParams) -> Result<CallToolResult, McpError> {
    let stage = match parse_stage(&p.stage) {
        Some(v) => v,
        None => return err_result(&["Unknown pipeline stage: '", &p.stage, "'"].concat()),
    };
    let primitives: Vec<&str> = stage
        .active_primitives()
        .iter()
        .map(|p| p.description())
        .collect();
    let composites: Vec<&str> = stage
        .active_composites()
        .iter()
        .map(|c| c.description())
        .collect();
    let symbols: Vec<&str> = stage.dominant_symbols().iter().map(|s| s.glyph()).collect();

    ok_json(json!({
        "stage": p.stage,
        "order": stage.order(),
        "description": stage.description(),
        "chomsky_level": format!("{:?}", stage.chomsky_level()),
        "is_cross_cutting": stage.is_cross_cutting(),
        "dominant_symbols": symbols,
        "active_primitives": primitives,
        "active_composites": composites,
    }))
}

/// Classify a set of generators into a Chomsky hierarchy level.
pub fn pharma_classify_generators(
    p: PharmaClassifyGeneratorsParams,
) -> Result<CallToolResult, McpError> {
    let mut generators = Vec::with_capacity(p.generators.len());
    for name in &p.generators {
        match parse_generator(name) {
            Some(g) => generators.push(g),
            None => return err_result(&["Unknown generator: '", name, "'"].concat()),
        }
    }
    let level = nexcore_pharma_rd::classify_generators(&generators);
    ok_json(json!({
        "generators": p.generators,
        "chomsky_level": format!("{level:?}"),
        "automaton": level.automaton(),
        "architecture": level.architecture(),
        "required_generators": level.generator_count(),
    }))
}
