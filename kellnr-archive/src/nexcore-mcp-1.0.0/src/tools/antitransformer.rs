//! Antitransformer MCP tools — AI text detection via statistical fingerprints.
//!
//! # T1 Grounding
//! - σ (Sequence): 7-stage analysis pipeline
//! - κ (Comparison): feature comparison against human baselines
//! - ∂ (Boundary): classification threshold

use antitransformer::pipeline::{self, AnalysisConfig};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{AntitransformerAnalyzeParams, AntitransformerBatchParams};

/// Analyze a single text for AI generation markers.
pub fn antitransformer_analyze(
    params: AntitransformerAnalyzeParams,
) -> Result<CallToolResult, McpError> {
    let config = AnalysisConfig {
        threshold: params.threshold.unwrap_or(0.5),
        window_size: params.window_size.unwrap_or(50),
    };

    let result = pipeline::analyze(&params.text, &config);

    let response = serde_json::json!({
        "verdict": result.verdict,
        "probability": result.probability,
        "confidence": result.confidence,
        "features": {
            "zipf_alpha": result.features.zipf_alpha,
            "zipf_deviation": result.features.zipf_deviation,
            "entropy_std": result.features.entropy_std,
            "burstiness": result.features.burstiness,
            "perplexity_var": result.features.perplexity_var,
            "ttr": result.features.ttr,
            "ttr_deviation": result.features.ttr_deviation,
            "normalized": result.features.normalized,
            "beer_lambert": result.features.beer_lambert,
            "composite": result.features.composite,
            "hill_score": result.features.hill_score,
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Analyze a batch of texts for AI generation markers.
pub fn antitransformer_batch(
    params: AntitransformerBatchParams,
) -> Result<CallToolResult, McpError> {
    let config = AnalysisConfig {
        threshold: params.threshold.unwrap_or(0.5),
        window_size: params.window_size.unwrap_or(50),
    };

    let samples: Vec<pipeline::InputSample> = params
        .texts
        .into_iter()
        .enumerate()
        .map(|(i, item)| pipeline::InputSample {
            id: item.id.unwrap_or_else(|| format!("s{i}")),
            text: item.text,
            label: None,
        })
        .collect();

    let (verdicts, stats) = pipeline::analyze_batch(&samples, &config);

    let results: Vec<serde_json::Value> = verdicts
        .iter()
        .map(|v| {
            serde_json::json!({
                "id": v.id,
                "verdict": v.verdict,
                "probability": v.probability,
                "confidence": v.confidence,
            })
        })
        .collect();

    let response = serde_json::json!({
        "results": results,
        "stats": {
            "records_processed": stats.records_processed,
            "human_count": stats.human_count,
            "generated_count": stats.generated_count,
            "duration_secs": stats.duration_secs,
        }
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}
