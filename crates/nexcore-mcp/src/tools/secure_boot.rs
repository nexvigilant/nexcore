//! Secure Boot Chain MCP tools — measured boot, PCR verification, attestation.
//!
//! # T1 Grounding
//! - σ (sequence): Boot chain stages execute in strict order
//! - → (causality): Each stage causes the next (hash extends)
//! - ∂ (boundary): Trust boundary between verified/unverified
//! - ∝ (irreversibility): PCR extends are one-way (append-only hash chain)
//! - κ (comparison): Expected vs actual measurement comparison

use nexcore_os::secure_boot::{BootPolicy, BootStage, Measurement, SecureBootChain};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{SecureBootQuoteParams, SecureBootStatusParams, SecureBootVerifyParams};

/// Parse a boot policy from a string.
fn parse_policy(s: &str) -> Result<BootPolicy, McpError> {
    match s.to_lowercase().as_str() {
        "strict" => Ok(BootPolicy::Strict),
        "degraded" => Ok(BootPolicy::Degraded),
        "permissive" => Ok(BootPolicy::Permissive),
        _ => Err(McpError::invalid_params(
            format!("Unknown policy: '{s}'. Valid: Strict, Degraded, Permissive"),
            None,
        )),
    }
}

/// Parse a boot stage from a string.
fn parse_stage(s: &str) -> Result<BootStage, McpError> {
    match s.to_lowercase().as_str() {
        "firmware" => Ok(BootStage::Firmware),
        "bootloader" => Ok(BootStage::Bootloader),
        "kernel" => Ok(BootStage::Kernel),
        "nexcoreos" | "nexcore-os" | "nexcore_os" => Ok(BootStage::NexCoreOs),
        "init" => Ok(BootStage::Init),
        "services" => Ok(BootStage::Services),
        "shell" => Ok(BootStage::Shell),
        "apps" => Ok(BootStage::Apps),
        _ => Err(McpError::invalid_params(
            format!(
                "Unknown stage: '{s}'. Valid: Firmware, Bootloader, Kernel, NexCoreOs, Init, Services, Shell, Apps"
            ),
            None,
        )),
    }
}

/// Get secure boot chain status — stages, policies, and capabilities.
pub fn secure_boot_status(params: SecureBootStatusParams) -> Result<CallToolResult, McpError> {
    let policy = parse_policy(&params.policy)?;

    let stages: Vec<serde_json::Value> = BootStage::all()
        .iter()
        .map(|s| {
            serde_json::json!({
                "stage": s.name(),
                "pcr_index": s.pcr_index(),
            })
        })
        .collect();

    let response = serde_json::json!({
        "policy": policy.to_string(),
        "allows_continue_on_failure": policy.allows_continue(),
        "pcr_count": 8,
        "stages": stages,
        "grounding": {
            "primitives": ["σ Sequence", "→ Causality", "∂ Boundary", "∝ Irreversibility", "κ Comparison", "∃ Existence"],
            "tier": "T3",
            "description": "TPM-style measured boot chain with PCR extend and verification",
        },
        "policies": {
            "Strict": "Halt boot on any verification failure",
            "Degraded": "Continue in reduced-privilege mode on failure",
            "Permissive": "Log warnings but continue (development mode)",
        },
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Verify a boot stage measurement — measure data, optionally compare against expected.
pub fn secure_boot_verify(params: SecureBootVerifyParams) -> Result<CallToolResult, McpError> {
    let policy = parse_policy(&params.policy)?;
    let stage = parse_stage(&params.stage)?;

    let mut chain = SecureBootChain::new(policy);

    // If expected hash provided, register it
    if let Some(ref hex) = params.expected_hex {
        let bytes = hex_to_bytes(hex)
            .map_err(|e| McpError::invalid_params(format!("Invalid expected_hex: {e}"), None))?;
        if bytes.len() != 32 {
            return Err(McpError::invalid_params(
                format!(
                    "expected_hex must be 64 hex chars (32 bytes), got {} bytes",
                    bytes.len()
                ),
                None,
            ));
        }
        let mut digest = [0u8; 32];
        digest.copy_from_slice(&bytes);
        chain.register_expected(stage, Measurement::from_digest(digest));
    }

    // Measure the data
    let result = chain.measure(
        stage,
        params.data.as_bytes(),
        format!("MCP verify: {}", stage.name()),
    );

    let pcr = chain.pcr(stage);

    let response = serde_json::json!({
        "stage": stage.name(),
        "pcr_index": stage.pcr_index(),
        "measurement_hex": Measurement::from_data(params.data.as_bytes()).hex(),
        "pcr_value_hex": pcr.hex(),
        "result": match &result {
            nexcore_os::secure_boot::VerifyResult::Ok => "ok",
            nexcore_os::secure_boot::VerifyResult::Mismatch { .. } => "mismatch",
            nexcore_os::secure_boot::VerifyResult::NoExpectation => "no_expectation",
            nexcore_os::secure_boot::VerifyResult::NotMeasured => "not_measured",
        },
        "is_ok": result.is_ok(),
        "is_violation": result.is_violation(),
        "can_proceed": chain.can_proceed(stage),
        "policy": policy.to_string(),
        "expected_provided": params.expected_hex.is_some(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Generate a boot quote — measure multiple stages and return PCR summary.
pub fn secure_boot_quote(params: SecureBootQuoteParams) -> Result<CallToolResult, McpError> {
    let policy = parse_policy(&params.policy)?;
    let mut chain = SecureBootChain::new(policy);

    let mut stage_results = Vec::new();

    for input in &params.stages {
        let stage = parse_stage(&input.stage)?;
        let desc = input
            .description
            .as_deref()
            .unwrap_or_else(|| input.stage.as_str());
        let result = chain.measure(stage, input.data.as_bytes(), desc);

        stage_results.push(serde_json::json!({
            "stage": stage.name(),
            "pcr_index": stage.pcr_index(),
            "measurement": Measurement::from_data(input.data.as_bytes()).short_hex(),
            "result": match &result {
                nexcore_os::secure_boot::VerifyResult::Ok => "ok",
                nexcore_os::secure_boot::VerifyResult::Mismatch { .. } => "mismatch",
                nexcore_os::secure_boot::VerifyResult::NoExpectation => "no_expectation",
                nexcore_os::secure_boot::VerifyResult::NotMeasured => "not_measured",
            },
            "verified": result.is_ok(),
        }));
    }

    let quote = chain.quote();
    let verification = chain.verify_chain();

    let pcrs: Vec<serde_json::Value> = quote
        .pcr_values
        .iter()
        .map(|(idx, m)| {
            serde_json::json!({
                "pcr_index": idx,
                "value": m.short_hex(),
                "full_hex": m.hex(),
            })
        })
        .collect();

    let response = serde_json::json!({
        "policy": policy.to_string(),
        "stages_measured": stage_results,
        "pcr_values": pcrs,
        "composite_hash": quote.composite.hex(),
        "composite_short": quote.composite.short_hex(),
        "degraded": quote.degraded,
        "failure_count": quote.failure_count,
        "all_verified": verification.all_verified,
        "should_proceed": verification.should_proceed(),
        "total_records": chain.record_count(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&response).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Decode a hex string to bytes.
fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
    if hex.len() % 2 != 0 {
        return Err("Hex string must have even length".to_string());
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|e| format!("Invalid hex at position {i}: {e}"))
        })
        .collect()
}
