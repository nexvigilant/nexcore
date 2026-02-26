//! Phenotype MCP tools — adversarial test generation via schema mutation.
//!
//! Uses nexcore-phenotype crate to generate mutated JSON for testing:
//! - 7 mutation types: TypeMismatch, AddField, RemoveField, RangeExpand,
//!   LengthChange, ArrayResize, StructureSwap
//! - Verification against drift thresholds
//!
//! ## T1 Primitive Grounding
//! - Mutation: μ(Mapping) + Σ(Sum)
//! - Verification: κ(Comparison) + ∂(Boundary)
//! - Generation: ∃(Existence) + N(Quantity)

use crate::params::phenotype::{PhenotypeMutateParams, PhenotypeVerifyParams};
use nexcore_phenotype::{Mutation, mutate, mutate_all, verify_with_threshold};
use nexcore_transcriptase::infer;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Generate mutations of a JSON input for adversarial testing.
pub fn phenotype_mutate(params: PhenotypeMutateParams) -> Result<CallToolResult, McpError> {
    let input: serde_json::Value = serde_json::from_str(&params.json_input)
        .map_err(|e| McpError::invalid_params(format!("Invalid JSON: {e}"), None))?;

    let schema = infer(&input);
    let mutation_filter = params.mutation.as_deref();

    if let Some(m_name) = mutation_filter {
        // Single mutation
        let mutation = match m_name.to_lowercase().as_str() {
            "type_mismatch" => Mutation::TypeMismatch,
            "add_field" => Mutation::AddField,
            "remove_field" => Mutation::RemoveField,
            "range_expand" => Mutation::RangeExpand,
            "length_change" => Mutation::LengthChange,
            "array_resize" => Mutation::ArrayResize,
            "structure_swap" => Mutation::StructureSwap,
            _ => {
                return Ok(CallToolResult::success(vec![Content::text(
                    json!({
                        "error": format!("Unknown mutation: {m_name}"),
                        "valid_mutations": ["type_mismatch", "add_field", "remove_field",
                            "range_expand", "length_change", "array_resize", "structure_swap"],
                    })
                    .to_string(),
                )]));
            }
        };

        let phenotype = mutate(&schema, mutation);
        Ok(CallToolResult::success(vec![Content::text(
            json!({
                "mutation": m_name,
                "original": input,
                "mutated": phenotype.data,
                "mutations_applied": phenotype.mutations_applied.iter().map(|m| m.to_string()).collect::<Vec<_>>(),
                "expected_drifts": phenotype.expected_drifts,
            })
            .to_string(),
        )]))
    } else {
        // All mutations
        let phenotypes = mutate_all(&schema);
        let results: Vec<serde_json::Value> = phenotypes
            .iter()
            .map(|p| {
                json!({
                    "mutations": p.mutations_applied.iter().map(|m| m.to_string()).collect::<Vec<_>>(),
                    "mutated": p.data,
                    "expected_drifts": p.expected_drifts,
                })
            })
            .collect();

        Ok(CallToolResult::success(vec![Content::text(
            json!({
                "original": input,
                "mutation_count": results.len(),
                "phenotypes": results,
            })
            .to_string(),
        )]))
    }
}

/// Verify schema compatibility between original and mutated JSON.
pub fn phenotype_verify(params: PhenotypeVerifyParams) -> Result<CallToolResult, McpError> {
    let original: serde_json::Value = serde_json::from_str(&params.original)
        .map_err(|e| McpError::invalid_params(format!("Invalid original JSON: {e}"), None))?;

    let mutated: serde_json::Value = serde_json::from_str(&params.mutated)
        .map_err(|e| McpError::invalid_params(format!("Invalid mutated JSON: {e}"), None))?;

    let schema = infer(&original);
    let threshold = params.threshold.unwrap_or(0.5);

    // Build a Phenotype from the mutated value for verification
    let phenotype = nexcore_phenotype::Phenotype {
        data: mutated.clone(),
        mutations_applied: vec![],
        expected_drifts: vec![],
    };

    let drift_detected = verify_with_threshold(&schema, &phenotype, threshold)
        .map_err(|e| McpError::internal_error(format!("Verification failed: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "verification": {
                "compatible": !drift_detected,
                "drift_detected": drift_detected,
                "threshold": threshold,
            },
        })
        .to_string(),
    )]))
}
