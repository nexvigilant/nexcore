//! Processor MCP Tools
//!
//! Exposes the generic processor framework to AI agents.
//! Demonstrates ∂(σ(μ)) + {ς} via concrete signal detection example.
//! Optionally applies antibody patterns as entry boundaries (immunity bridge).
//!
//! T1 composition: μ(Mapping) + σ(Sequence) + ∂(Boundary) + κ(Comparison)

use nexcore_processor::{
    Bounded, FnProcessor, OpenBoundary, PredicateBoundary, Processor, ProcessorError,
    immunity::AntibodyBoundary, process_batch,
};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::ProcessorDemoParams;

/// Run a demonstration of the processor framework with a signal detection pipeline.
///
/// Takes a list of PRR values and runs them through a bounded classification pipeline:
/// 1. Antibody boundary (optional): reject values matching regex antipatterns
/// 2. Positive gate: reject PRR <= 0 (invalid input)
/// 3. Classification: map PRR to signal strength category
///
/// When `antibody_patterns` is provided, each PRR value's string representation
/// is checked against all patterns. Matching values are rejected before
/// the positive gate or classifier ever see them.
pub fn demo_pipeline(params: ProcessorDemoParams) -> Result<CallToolResult, McpError> {
    let threshold = params.threshold.unwrap_or(2.0);
    let values = params.values;
    let antibody_patterns = params.antibody_patterns;

    if values.is_empty() {
        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            r#"{"error": "No PRR values provided", "hint": "Pass values: [1.5, 3.2, 0.8, 5.1]"}"#,
        )]));
    }

    // Build antibody boundary if patterns provided
    let antibody = antibody_patterns.as_ref().map(|patterns| {
        let pairs: Vec<(String, String)> = patterns
            .iter()
            .map(|p| (p.description.clone(), p.regex.clone()))
            .collect();
        AntibodyBoundary::new("mcp-input-guard", pairs)
    });

    let antibody_count = antibody.as_ref().map_or(0, |ab| ab.pattern_count());

    // Pre-filter values through antibody boundary (operates on string repr)
    let mut antibody_rejections: Vec<serde_json::Value> = Vec::new();
    let mut surviving_values: Vec<(usize, f64)> = Vec::new();

    for (i, val) in values.iter().enumerate() {
        let val_str = format!("{val}");
        match &antibody {
            Some(ab) => match ab.check(&val_str) {
                Ok(()) => surviving_values.push((i, *val)),
                Err(e) => antibody_rejections.push(serde_json::json!({
                    "index": i,
                    "value": val,
                    "error": format!("{e}"),
                    "stage": "antibody"
                })),
            },
            None => surviving_values.push((i, *val)),
        }
    }

    // Stage 1: μ — classify PRR into signal strength
    let classifier = FnProcessor::new(
        "prr_classifier",
        |prr: f64| -> Result<serde_json::Value, ProcessorError> {
            let strength = if prr >= 4.0 {
                "strong"
            } else if prr >= 2.0 {
                "moderate"
            } else if prr >= 1.0 {
                "weak"
            } else {
                "inverse"
            };

            Ok(serde_json::json!({
                "prr": prr,
                "strength": strength,
                "signal": prr >= 2.0,
            }))
        },
    );

    // Stage 2: ∂ — entry boundary rejects non-positive PRR
    let positive_gate = PredicateBoundary::new("PRR must be positive", move |prr: &f64| *prr > 0.0);

    // Bounded processor: ∂(μ)
    let bounded_classifier = Bounded::new(classifier, positive_gate, OpenBoundary);

    // Process surviving values through the pipeline
    let surviving_vals: Vec<f64> = surviving_values.iter().map(|(_, v)| *v).collect();
    let surviving_indices: Vec<usize> = surviving_values.iter().map(|(i, _)| *i).collect();
    let batch_result = process_batch(&bounded_classifier, surviving_vals);

    let successes: Vec<serde_json::Value> = batch_result
        .successes
        .iter()
        .map(|(batch_i, v)| {
            let original_i = surviving_indices.get(*batch_i).copied().unwrap_or(*batch_i);
            serde_json::json!({
                "index": original_i,
                "result": v,
            })
        })
        .collect();

    let mut all_failures: Vec<serde_json::Value> = antibody_rejections;
    for (batch_i, e) in &batch_result.failures {
        let original_i = surviving_indices.get(*batch_i).copied().unwrap_or(*batch_i);
        all_failures.push(serde_json::json!({
            "index": original_i,
            "error": format!("{e}"),
            "stage": "pipeline"
        }));
    }

    // Count signals above threshold
    let signal_count = batch_result
        .successes
        .iter()
        .filter(|(_, v)| {
            v.get("prr")
                .and_then(|p| p.as_f64())
                .is_some_and(|p| p >= threshold)
        })
        .count();

    let mut pipeline_stages = vec!["positive_gate (∂)", "prr_classifier (μ)"];
    if antibody_count > 0 {
        pipeline_stages.insert(0, "antibody_guard (∂:immunity)");
    }

    let output = serde_json::json!({
        "framework": "nexcore-processor ∂(σ(μ)) + immunity",
        "pipeline": pipeline_stages,
        "threshold": threshold,
        "antibody_patterns": antibody_count,
        "input_count": values.len(),
        "success_count": successes.len(),
        "failure_count": all_failures.len(),
        "success_rate": if values.is_empty() { 0.0 } else { successes.len() as f64 / values.len() as f64 },
        "signal_count": signal_count,
        "results": successes,
        "failures": all_failures,
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
