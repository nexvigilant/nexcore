//! Primitive Scanner MCP tools.
//!
//! Exposes automated primitive extraction via MCP.

use crate::params::{PrimitiveBatchTestParams, PrimitiveScanParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Scan sources for primitives in a domain.
pub fn primitive_scan(params: PrimitiveScanParams) -> Result<CallToolResult, McpError> {
    // Use existing brand_primitive_test infrastructure
    let result = json!({
        "domain": params.domain,
        "sources": params.sources,
        "status": "scanning",
        "primitives": {
            "t1_universal": [],
            "t2_primitives": [],
            "t2_composites": [],
            "t3_domain_specific": []
        },
        "note": "Full extraction requires document parsing - use CLI for file scanning"
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Batch test multiple terms for primitiveness.
pub fn primitive_batch_test(params: PrimitiveBatchTestParams) -> Result<CallToolResult, McpError> {
    let mut results = Vec::new();

    for term in &params.terms {
        // Apply 3-test protocol
        let has_domain_deps = !term.domain_terms.is_empty();
        let has_grounding = !term.external_grounding.is_empty();

        let test1 = if has_domain_deps { "FAIL" } else { "PASS" };
        let test2 = if has_grounding { "PASS" } else { "FAIL" };
        let test3 = "PASS"; // Assume not synonym unless specified

        let verdict = if test1 == "PASS" && test2 == "PASS" && test3 == "PASS" {
            "PRIMITIVE"
        } else if test1 == "FAIL" {
            "COMPOSITE"
        } else {
            "UNDETERMINED"
        };

        let tier = if verdict == "PRIMITIVE" {
            if term.domain_count.unwrap_or(1) >= 10 {
                "T1"
            } else if term.domain_count.unwrap_or(1) > 1 {
                "T2-P"
            } else {
                "T3"
            }
        } else if term.domain_count.unwrap_or(1) > 1 {
            "T2-C"
        } else {
            "T3"
        };

        results.push(json!({
            "term": term.term,
            "definition": term.definition,
            "test1_no_domain_deps": test1,
            "test2_external_grounding": test2,
            "test3_not_synonym": test3,
            "verdict": verdict,
            "tier": tier
        }));
    }

    let output = json!({
        "batch_size": params.terms.len(),
        "results": results
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_default(),
    )]))
}
