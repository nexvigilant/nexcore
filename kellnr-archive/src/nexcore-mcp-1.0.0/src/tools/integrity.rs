//! Integrity Assessment MCP tools
//!
//! AI text detection for KSB assessment integrity.
//! 3 tools: analyze (full pipeline), assess_ksb (convenience), calibration (profile lookup).

use crate::params::{IntegrityAnalyzeParams, IntegrityAssessKsbParams, IntegrityCalibrationParams};
use nexcore_integrity::{
    AssessmentContext, BloomThresholds, CalibrationProfile, assess_ksb_response, assess_text,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

/// Full integrity analysis with optional Bloom/domain/threshold configuration.
pub fn integrity_analyze(params: IntegrityAnalyzeParams) -> Result<CallToolResult, McpError> {
    let bloom_level = params.bloom_level.unwrap_or(3);

    let context = match AssessmentContext::new(bloom_level) {
        Ok(ctx) => ctx,
        Err(e) => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Invalid context: {e}"
            ))]));
        }
    };

    // Apply optional configuration
    let context = if let Some(ref domain_id) = params.domain_id {
        context.with_domain(domain_id)
    } else {
        context
    };
    let context = if let Some(threshold) = params.threshold {
        context.with_threshold(threshold)
    } else {
        context
    };
    let context = context.with_strict(params.strict_mode);

    match assess_text(&params.text, &context) {
        Ok(assessment) => {
            let result = json!({
                "verdict": format!("{:?}", assessment.classification.verdict),
                "probability": assessment.classification.probability,
                "confidence": assessment.classification.confidence,
                "threshold": assessment.threshold,
                "bloom_level": assessment.bloom_level,
                "bloom_name": assessment.bloom_name,
                "domain_id": assessment.domain_id,
                "token_count": assessment.token_count,
                "features": {
                    "zipf_deviation": assessment.features.zipf_deviation,
                    "zipf_alpha": assessment.features.zipf_alpha,
                    "zipf_r_squared": assessment.features.zipf_r_squared,
                    "entropy_std": assessment.features.entropy_std,
                    "entropy_mean": assessment.features.entropy_mean,
                    "burstiness": assessment.features.burstiness,
                    "perplexity_var": assessment.features.perplexity_var,
                    "ttr": assessment.features.ttr,
                    "ttr_deviation": assessment.features.ttr_deviation,
                    "composite_score": assessment.features.composite_score,
                    "hill_score": assessment.features.hill_score,
                }
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Assessment error: {e}"
        ))])),
    }
}

/// Convenience KSB response assessment with required Bloom level.
pub fn integrity_assess_ksb(params: IntegrityAssessKsbParams) -> Result<CallToolResult, McpError> {
    let domain_id = params.domain_id.as_deref();

    match assess_ksb_response(&params.text, params.bloom_level, domain_id) {
        Ok(assessment) => {
            let result = json!({
                "verdict": format!("{:?}", assessment.classification.verdict),
                "probability": assessment.classification.probability,
                "confidence": assessment.classification.confidence,
                "threshold": assessment.threshold,
                "bloom_level": assessment.bloom_level,
                "bloom_name": assessment.bloom_name,
                "domain_id": assessment.domain_id,
                "token_count": assessment.token_count,
                "hill_score": assessment.features.hill_score,
                "composite_score": assessment.features.composite_score,
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "KSB assessment error: {e}"
        ))])),
    }
}

/// Get domain calibration profile with baseline feature expectations.
pub fn integrity_calibration(
    params: IntegrityCalibrationParams,
) -> Result<CallToolResult, McpError> {
    match nexcore_integrity::profile::get_profile(&params.domain_id) {
        Ok(profile) => {
            let result = json!({
                "domain_id": profile.domain_id,
                "domain_name": profile.domain_name,
                "baselines": {
                    "zipf_alpha": profile.zipf_alpha_baseline,
                    "entropy_std": profile.entropy_std_baseline,
                    "burstiness": profile.burstiness_baseline,
                    "perplexity_var": profile.perplexity_var_baseline,
                    "ttr": profile.ttr_baseline,
                },
                "bloom_thresholds": {
                    "pv_education": bloom_thresholds_json(&BloomThresholds::pv_education()),
                    "strict": bloom_thresholds_json(&BloomThresholds::strict()),
                    "lenient": bloom_thresholds_json(&BloomThresholds::lenient()),
                },
                "available_domains": nexcore_integrity::profile::list_profiles()
                    .iter()
                    .map(|p| json!({"id": p.domain_id, "name": p.domain_name}))
                    .collect::<Vec<_>>(),
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Calibration error: {e}"
        ))])),
    }
}

/// Helper to serialize bloom thresholds for all 7 levels.
fn bloom_thresholds_json(preset: &BloomThresholds) -> serde_json::Value {
    let mut levels = serde_json::Map::new();
    for level in 1..=7u8 {
        if let Ok(threshold) = preset.threshold_for_level(level) {
            let name = BloomThresholds::level_name(level).unwrap_or("Unknown");
            levels.insert(format!("L{level}_{name}"), json!(threshold));
        }
    }
    serde_json::Value::Object(levels)
}
