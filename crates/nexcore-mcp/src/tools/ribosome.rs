//! Ribosome MCP tools — schema contract registry + drift detection.
//!
//! # T1 Grounding
//! - κ (comparison): Schema drift scoring, type matching
//! - σ (sequence): Contract iteration, batch processing
//! - μ (function): Inference pipeline, generation pipeline
//! - ∂ (conditional): Drift threshold dispatch
//! - N (quantity): Drift score computation

use nexcore_ribosome::Ribosome;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use std::sync::OnceLock;

use crate::params::{
    RibosomeDriftParams, RibosomeGenerateParams, RibosomeStoreParams, RibosomeValidateParams,
};

/// Global ribosome instance (lazily initialized).
static RIBOSOME: OnceLock<parking_lot::Mutex<Ribosome>> = OnceLock::new();

fn get_ribosome() -> &'static parking_lot::Mutex<Ribosome> {
    RIBOSOME.get_or_init(|| parking_lot::Mutex::new(Ribosome::new()))
}

/// Store a baseline contract from JSON data.
pub fn ribosome_store(params: RibosomeStoreParams) -> Result<CallToolResult, McpError> {
    let json: serde_json::Value = serde_json::from_str(&params.json)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let schema = nexcore_transcriptase::infer(&json);
    let mut rb = get_ribosome().lock();
    let contract = rb
        .store_contract(params.contract_id, schema)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let result = serde_json::to_string_pretty(&contract)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

/// List all stored contracts.
pub fn ribosome_list() -> Result<CallToolResult, McpError> {
    let rb = get_ribosome().lock();
    let ids = rb.list_contracts();
    let count = rb.contract_count();

    let result = serde_json::json!({
        "count": count,
        "contracts": ids,
    });

    let json = serde_json::to_string_pretty(&result)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(json)]))
}

/// Validate data against a stored contract (drift detection).
pub fn ribosome_validate(params: RibosomeValidateParams) -> Result<CallToolResult, McpError> {
    let json: serde_json::Value = serde_json::from_str(&params.json)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    let rb = get_ribosome().lock();
    let drift_result = rb.validate(&params.contract_id, &json).ok_or_else(|| {
        McpError::invalid_params(format!("contract '{}' not found", params.contract_id), None)
    })?;

    let result = serde_json::to_string_pretty(&drift_result)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

/// Generate synthetic data from a stored contract's schema.
pub fn ribosome_generate(params: RibosomeGenerateParams) -> Result<CallToolResult, McpError> {
    let rb = get_ribosome().lock();
    let count = params.count.unwrap_or(1).max(1);

    let records = rb
        .generate_batch(&params.contract_id, count)
        .ok_or_else(|| {
            McpError::invalid_params(format!("contract '{}' not found", params.contract_id), None)
        })?;

    let result = if count == 1 {
        serde_json::to_string_pretty(&records[0])
    } else {
        serde_json::to_string_pretty(&records)
    }
    .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(result)]))
}

/// Batch drift detection across contracts.
pub fn ribosome_drift(params: RibosomeDriftParams) -> Result<CallToolResult, McpError> {
    // Parse all JSON strings into Values
    let mut data = std::collections::HashMap::new();
    for (contract_id, json_str) in &params.data {
        let value: serde_json::Value = serde_json::from_str(json_str)
            .map_err(|e| McpError::invalid_params(format!("{contract_id}: {e}"), None))?;
        data.insert(contract_id.clone(), value);
    }

    let rb = get_ribosome().lock();
    let signals = rb.detect_drift(&data);

    let result = serde_json::to_string_pretty(&signals)
        .map_err(|e| McpError::invalid_params(e.to_string(), None))?;

    Ok(CallToolResult::success(vec![Content::text(result)]))
}
