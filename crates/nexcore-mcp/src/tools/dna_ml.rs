//! DNA-ML MCP tools — DNA-encoded ML pipeline for PV signal detection.

use crate::params::dna_ml::{DnaMlEncodeParams, DnaMlPipelineRunParams, DnaMlSimilarityParams};
use nexcore_dna_ml::encode;
use nexcore_dna_ml::similarity;
use nexcore_ml_pipeline::types::*;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ---------------------------------------------------------------------------
// Tool: dna_ml_encode
// ---------------------------------------------------------------------------

/// Encode a PV feature vector as a DNA strand.
pub fn dna_ml_encode(params: DnaMlEncodeParams) -> Result<CallToolResult, McpError> {
    let dim = params.features.len();
    let mins = params.mins.unwrap_or_else(|| vec![0.0; dim]);
    let maxs = params.maxs.unwrap_or_else(|| vec![1.0; dim]);

    let strand = encode::encode_features(&params.features, &mins, &maxs);
    let bases: String = strand.bases.iter().map(|n| format!("{n:?}")).collect();

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!({
            "success": true,
            "strand_length": strand.bases.len(),
            "strand": bases,
            "feature_count": dim,
            "quantized_bytes": encode::quantize_features(&params.features, &mins, &maxs),
        }))
        .unwrap_or_default(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: dna_ml_similarity
// ---------------------------------------------------------------------------

/// Compute DNA similarity between two encoded feature vectors.
pub fn dna_ml_similarity(params: DnaMlSimilarityParams) -> Result<CallToolResult, McpError> {
    let dim = params.features_a.len().max(params.features_b.len());
    let mins = params.mins.unwrap_or_else(|| vec![0.0; dim]);
    let maxs = params.maxs.unwrap_or_else(|| vec![1.0; dim]);

    let strand_a = encode::encode_features(&params.features_a, &mins, &maxs);
    let strand_b = encode::encode_features(&params.features_b, &mins, &maxs);

    let sim = similarity::compute_similarity(&strand_a, &strand_b);

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!({
            "success": true,
            "hamming_distance": sim.hamming_distance,
            "gc_content_a": sim.gc_content_a,
            "gc_content_b": sim.gc_content_b,
            "gc_divergence": sim.gc_divergence,
            "lcs_ratio": sim.lcs_ratio,
        }))
        .unwrap_or_default(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: dna_ml_pipeline_run
// ---------------------------------------------------------------------------

/// Run the full DNA-ML pipeline: FAERS data → PV features → DNA encoding → augmented random forest.
pub fn dna_ml_pipeline_run(params: DnaMlPipelineRunParams) -> Result<CallToolResult, McpError> {
    let raw_data: Vec<RawPairData> = params
        .data
        .iter()
        .map(|e| RawPairData {
            contingency: ContingencyTable {
                drug: e.drug.clone(),
                event: e.event.clone(),
                a: e.a,
                b: e.b,
                c: e.c,
                d: e.d,
            },
            reporters: ReporterBreakdown {
                hcp: e.hcp_reports.unwrap_or(0),
                consumer: e.consumer_reports.unwrap_or(0),
                other: 0,
            },
            outcomes: OutcomeBreakdown {
                total: e.a,
                serious: e.serious_count.unwrap_or(0),
                death: e.death_count.unwrap_or(0),
                hospitalization: e.hospitalization_count.unwrap_or(0),
            },
            temporal: TemporalData {
                median_tto_days: e.median_tto_days,
                velocity: e.velocity.unwrap_or(0.0),
            },
        })
        .collect();

    let config = nexcore_dna_ml::pipeline::DnaMlConfig {
        n_trees: params.n_trees.unwrap_or(50),
        max_depth: params.max_depth.unwrap_or(8),
        use_dna_features: params.use_dna_features.unwrap_or(true),
        ..Default::default()
    };

    match nexcore_dna_ml::pipeline::run(&raw_data, &params.labels, &config) {
        Ok(result) => Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&json!({
                "success": true,
                "pv_feature_count": result.pv_feature_count,
                "dna_feature_count": result.dna_feature_count,
                "total_features": result.total_features,
                "n_samples": result.n_samples,
                "auc": result.metrics.auc,
                "precision": result.metrics.precision,
                "recall": result.metrics.recall,
                "f1": result.metrics.f1,
                "accuracy": result.metrics.accuracy,
                "feature_names": result.feature_names,
            }))
            .unwrap_or_default(),
        )])),
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "DNA-ML pipeline failed: {e}"
        ))])),
    }
}
