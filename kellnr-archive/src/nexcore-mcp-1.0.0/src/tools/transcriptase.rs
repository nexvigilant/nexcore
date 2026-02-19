//! Reverse Transcriptase MCP tools — schema inference + data generation.
//!
//! # T1 Grounding
//! - κ (comparison): Schema kind matching, range analysis
//! - σ (sequence): Record iteration, batch processing
//! - μ (function): Inference pipeline, generation pipeline
//! - ∂ (conditional): Schema kind dispatch, violation synthesis

use nexcore_transcriptase::{Config, Engine, infer, synthesize_violations};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{
    TranscriptaseGenerateParams, TranscriptaseInferParams, TranscriptaseProcessParams,
    TranscriptaseViolationsParams,
};

/// Full pipeline: infer + merge + violations + fidelity.
pub fn transcriptase_process(
    params: TranscriptaseProcessParams,
) -> Result<CallToolResult, McpError> {
    let config = Config {
        synthesize_violations: params.violations.unwrap_or(true),
        verify_fidelity: params.verify.unwrap_or(false),
        source_name: "mcp".to_string(),
    };

    let mut engine = Engine::with_config(config);
    let output = engine
        .process(&params.json)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let json = serde_json::to_string_pretty(&output)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Schema inference only — lightweight, no violations or fidelity.
pub fn transcriptase_infer(params: TranscriptaseInferParams) -> Result<CallToolResult, McpError> {
    let json: serde_json::Value = serde_json::from_str(&params.json)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let schema = infer(&json);

    let result = serde_json::to_string_pretty(&schema)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

/// Synthesize boundary violations from observed data.
pub fn transcriptase_violations(
    params: TranscriptaseViolationsParams,
) -> Result<CallToolResult, McpError> {
    let json: serde_json::Value = serde_json::from_str(&params.json)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let schema = infer(&json);
    let violations = synthesize_violations(&schema);

    let result = serde_json::to_string_pretty(&violations)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

/// Generate synthetic data from observed JSON.
pub fn transcriptase_generate(
    params: TranscriptaseGenerateParams,
) -> Result<CallToolResult, McpError> {
    let mut engine = Engine::new();
    engine
        .observe_str(&params.json)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let count = params.count.unwrap_or(1).max(1);

    let records = engine
        .generate_batch(count)
        .ok_or_else(|| McpError::invalid_params("no schema observed".to_string(), None))?;

    let result = if count == 1 {
        serde_json::to_string_pretty(&records[0])
    } else {
        serde_json::to_string_pretty(&records)
    }
    .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
