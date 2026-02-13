//! Laboratory MCP tools — virtual word/concept experiment engine.
//!
//! 4 tools:
//! - `lab_experiment`: Run a single word experiment (decompose → weigh → classify)
//! - `lab_compare`: Compare two concepts side-by-side
//! - `lab_react`: "React" two concepts — combine primitives, measure enthalpy
//! - `lab_batch`: Run batch experiments with statistical summary
//!
//! ## Tier: T2-C (μ + Σ + κ + × + σ)

use crate::params::{LabBatchParams, LabCompareParams, LabExperimentParams, LabReactParams};
use nexcore_laboratory::{Specimen, react, run_batch, run_experiment};
use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content, ErrorCode};
use serde_json::json;

// ============================================================================
// Helpers
// ============================================================================

fn parse_primitive(input: &str) -> Result<LexPrimitiva, McpError> {
    for p in LexPrimitiva::all() {
        if p.symbol() == input || p.name().eq_ignore_ascii_case(input) {
            return Ok(p);
        }
    }
    Err(McpError::new(
        ErrorCode(400),
        format!(
            "Unknown primitive '{}'. Use name (e.g. 'state', 'boundary') or symbol (e.g. 'ς', '∂')",
            input
        ),
        None,
    ))
}

fn parse_primitives(input: &[String]) -> Result<Vec<LexPrimitiva>, McpError> {
    input.iter().map(|s| parse_primitive(s.trim())).collect()
}

fn build_specimen(name: &str, primitives: &[String]) -> Result<Specimen, McpError> {
    let prims = parse_primitives(primitives)?;
    if prims.is_empty() {
        return Err(McpError::new(
            ErrorCode(400),
            "primitives array must not be empty",
            None,
        ));
    }
    Ok(Specimen::new(name, prims))
}

// ============================================================================
// lab_experiment
// ============================================================================

/// Run a complete experiment on a single word/concept.
///
/// Pipeline: Decompose → Weigh → Classify → Analyze → Report
pub fn lab_experiment(params: LabExperimentParams) -> Result<CallToolResult, McpError> {
    let name = params.name.as_deref().unwrap_or("unnamed");
    let specimen = build_specimen(name, &params.primitives)?;
    let result = run_experiment(&specimen);

    let response = serde_json::to_value(&result)
        .map_err(|e| McpError::new(ErrorCode(500), format!("Serialization error: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// lab_compare
// ============================================================================

/// Compare two concepts side-by-side.
///
/// Returns both experiment results plus comparative metrics:
/// weight delta, shared/unique primitives, Jaccard similarity.
pub fn lab_compare(params: LabCompareParams) -> Result<CallToolResult, McpError> {
    let name_a = params.name_a.as_deref().unwrap_or("concept_a");
    let name_b = params.name_b.as_deref().unwrap_or("concept_b");

    let specimen_a = build_specimen(name_a, &params.primitives_a)?;
    let specimen_b = build_specimen(name_b, &params.primitives_b)?;

    let result_a = run_experiment(&specimen_a);
    let result_b = run_experiment(&specimen_b);

    // Comparative metrics
    let set_a: std::collections::HashSet<&str> = result_a
        .spectrum
        .iter()
        .map(|s| s.symbol.as_str())
        .collect();
    let set_b: std::collections::HashSet<&str> = result_b
        .spectrum
        .iter()
        .map(|s| s.symbol.as_str())
        .collect();
    let shared: Vec<&str> = set_a.intersection(&set_b).copied().collect();
    let only_a: Vec<&str> = set_a.difference(&set_b).copied().collect();
    let only_b: Vec<&str> = set_b.difference(&set_a).copied().collect();
    let jaccard = if set_a.is_empty() && set_b.is_empty() {
        0.0
    } else {
        shared.len() as f64 / set_a.union(&set_b).count() as f64
    };

    let response = json!({
        "concept_a": serde_json::to_value(&result_a).ok(),
        "concept_b": serde_json::to_value(&result_b).ok(),
        "comparison": {
            "weight_delta": ((result_a.molecular_weight - result_b.molecular_weight).abs() * 1000.0).round() / 1000.0,
            "heavier": if result_a.molecular_weight > result_b.molecular_weight { name_a } else { name_b },
            "shared_primitives": shared,
            "only_in_a": only_a,
            "only_in_b": only_b,
            "jaccard_similarity": (jaccard * 1000.0).round() / 1000.0,
            "same_transfer_class": result_a.transfer_class == result_b.transfer_class,
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// lab_react
// ============================================================================

/// "React" two concepts — combine their primitive compositions.
///
/// Shared primitives are catalysts, unique primitives are reactants.
/// The product is the union of all primitives.
/// Enthalpy (ΔH) measures weight efficiency: negative = exothermic (more compact).
pub fn lab_react(params: LabReactParams) -> Result<CallToolResult, McpError> {
    let name_a = params.name_a.as_deref().unwrap_or("concept_a");
    let name_b = params.name_b.as_deref().unwrap_or("concept_b");

    let specimen_a = build_specimen(name_a, &params.primitives_a)?;
    let specimen_b = build_specimen(name_b, &params.primitives_b)?;

    let result = react(&specimen_a, &specimen_b);

    let response = serde_json::to_value(&result)
        .map_err(|e| McpError::new(ErrorCode(500), format!("Serialization error: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// lab_batch
// ============================================================================

/// Run experiments on a batch of specimens with statistical summary.
///
/// Returns individual results plus aggregate metrics:
/// lightest/heaviest, average weight, std dev, class distribution.
pub fn lab_batch(params: LabBatchParams) -> Result<CallToolResult, McpError> {
    let mut specimens = Vec::new();

    for spec in &params.specimens {
        let name = spec.name.as_deref().unwrap_or("unnamed");
        let prims = parse_primitives(&spec.primitives)?;
        if prims.is_empty() {
            return Err(McpError::new(
                ErrorCode(400),
                format!("Specimen '{}' has no primitives", name),
                None,
            ));
        }
        specimens.push(Specimen::new(name, prims));
    }

    if specimens.is_empty() {
        return Err(McpError::new(
            ErrorCode(400),
            "specimens array must not be empty",
            None,
        ));
    }

    let result = run_batch(&specimens);

    let response = serde_json::to_value(&result)
        .map_err(|e| McpError::new(ErrorCode(500), format!("Serialization error: {e}"), None))?;

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::{LabBatchSpecimen, LabExperimentParams};

    #[test]
    fn test_lab_experiment() {
        let params = LabExperimentParams {
            name: Some("Guardian".to_string()),
            primitives: vec![
                "state".to_string(),
                "boundary".to_string(),
                "comparison".to_string(),
            ],
        };
        let result = lab_experiment(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_lab_experiment_with_symbols() {
        let params = LabExperimentParams {
            name: Some("Symbolic".to_string()),
            primitives: vec!["ς".to_string(), "∂".to_string()],
        };
        assert!(lab_experiment(params).is_ok());
    }

    #[test]
    fn test_lab_experiment_empty_rejects() {
        let params = LabExperimentParams {
            name: Some("Empty".to_string()),
            primitives: vec![],
        };
        assert!(lab_experiment(params).is_err());
    }

    #[test]
    fn test_lab_compare() {
        let params = LabCompareParams {
            name_a: Some("Guardian".to_string()),
            primitives_a: vec!["state".to_string(), "boundary".to_string()],
            name_b: Some("Signal".to_string()),
            primitives_b: vec!["boundary".to_string(), "quantity".to_string()],
        };
        assert!(lab_compare(params).is_ok());
    }

    #[test]
    fn test_lab_react() {
        let params = LabReactParams {
            name_a: Some("Guardian".to_string()),
            primitives_a: vec![
                "state".to_string(),
                "boundary".to_string(),
                "comparison".to_string(),
            ],
            name_b: Some("Signal".to_string()),
            primitives_b: vec!["boundary".to_string(), "quantity".to_string()],
        };
        assert!(lab_react(params).is_ok());
    }

    #[test]
    fn test_lab_batch() {
        let params = LabBatchParams {
            specimens: vec![
                LabBatchSpecimen {
                    name: Some("Guardian".to_string()),
                    primitives: vec!["state".to_string(), "boundary".to_string()],
                },
                LabBatchSpecimen {
                    name: Some("Signal".to_string()),
                    primitives: vec!["boundary".to_string(), "quantity".to_string()],
                },
            ],
        };
        assert!(lab_batch(params).is_ok());
    }

    #[test]
    fn test_lab_batch_empty_rejects() {
        let params = LabBatchParams { specimens: vec![] };
        assert!(lab_batch(params).is_err());
    }

    #[test]
    fn test_unknown_primitive_rejects() {
        let params = LabExperimentParams {
            name: Some("Bad".to_string()),
            primitives: vec!["nonexistent".to_string()],
        };
        assert!(lab_experiment(params).is_err());
    }
}
