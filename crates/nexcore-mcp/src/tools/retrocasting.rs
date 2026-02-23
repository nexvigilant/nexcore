//! Retrocasting MCP tools.
//!
//! Retrospective analysis linking FAERS safety signals to molecular structures,
//! structural clustering, alert correlation, and ML training data generation.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::retrocasting::{
    RetroClusterSignalsParams, RetroCorrelateAlertsParams, RetroDatasetStatsParams,
    RetroExtractFeaturesParams, RetroSignalSignificanceParams, RetroStructuralSimilarityParams,
};

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Compute structural similarity between two SMILES strings (bigram Jaccard).
pub fn retro_structural_similarity(
    p: RetroStructuralSimilarityParams,
) -> Result<CallToolResult, McpError> {
    let sim =
        nexcore_retrocasting::cluster::structural_similarity(Some(&p.smiles_a), Some(&p.smiles_b));
    ok_json(json!({
        "smiles_a": p.smiles_a,
        "smiles_b": p.smiles_b,
        "similarity": sim,
        "method": "smiles_bigram_jaccard",
    }))
}

/// Check if a pharmacovigilance signal meets standard significance thresholds.
pub fn retro_signal_significance(
    p: RetroSignalSignificanceParams,
) -> Result<CallToolResult, McpError> {
    let record = nexcore_retrocasting::SignalRecord {
        drug: String::new(),
        event: String::new(),
        prr: p.prr,
        ror: p.ror,
        case_count: p.case_count,
        ror_lci: p.ror_lci,
        prr_chi_sq: p.prr_chi_sq,
    };
    let significant = record.is_significant();
    let mut criteria_met = Vec::new();
    if p.prr >= 2.0 {
        if let Some(chi) = p.prr_chi_sq {
            if chi >= 3.841 {
                criteria_met.push("PRR >= 2.0 AND chi-sq >= 3.841");
            }
        }
    }
    if let Some(lci) = p.ror_lci {
        if lci > 1.0 {
            criteria_met.push("ROR lower CI > 1.0");
        }
    }
    if p.prr >= 2.0 {
        criteria_met.push("PRR >= 2.0");
    }

    ok_json(json!({
        "significant": significant,
        "prr": p.prr,
        "ror": p.ror,
        "case_count": p.case_count,
        "ror_lci": p.ror_lci,
        "prr_chi_sq": p.prr_chi_sq,
        "criteria_met": criteria_met,
    }))
}

/// Cluster structured signals by structural similarity.
pub fn retro_cluster_signals(p: RetroClusterSignalsParams) -> Result<CallToolResult, McpError> {
    // Parse the input JSON into StructuredSignals
    let signals: Vec<serde_json::Value> = match serde_json::from_str(&p.signals_json) {
        Ok(v) => v,
        Err(e) => return err_result(&["Failed to parse signals JSON: ", &e.to_string()].concat()),
    };

    // Build StructuredSignal objects from JSON
    let structured: Vec<nexcore_retrocasting::StructuredSignal> = signals
        .iter()
        .map(|v| {
            let smiles = v.get("smiles").and_then(|s| s.as_str()).map(String::from);
            nexcore_retrocasting::StructuredSignal {
                signal: nexcore_retrocasting::SignalRecord {
                    drug: v
                        .get("drug")
                        .and_then(|d| d.as_str())
                        .unwrap_or("")
                        .to_string(),
                    event: v
                        .get("event")
                        .and_then(|e| e.as_str())
                        .unwrap_or("")
                        .to_string(),
                    prr: v.get("prr").and_then(|p| p.as_f64()).unwrap_or(0.0),
                    ror: v.get("ror").and_then(|r| r.as_f64()).unwrap_or(0.0),
                    case_count: v.get("case_count").and_then(|c| c.as_u64()).unwrap_or(0),
                    ror_lci: v.get("ror_lci").and_then(|l| l.as_f64()),
                    prr_chi_sq: v.get("prr_chi_sq").and_then(|c| c.as_f64()),
                },
                compound: None,
                has_structure: smiles.is_some(),
            }
        })
        .collect();

    let threshold = p.threshold.unwrap_or(0.7);
    match nexcore_retrocasting::cluster::cluster_by_similarity(&structured, threshold) {
        Ok(clusters) => ok_json(json!({
            "cluster_count": clusters.len(),
            "threshold": threshold,
            "clusters": clusters.iter().map(|c| json!({
                "cluster_id": c.cluster_id,
                "members": c.members,
                "common_fragments": c.common_fragments,
                "similarity_threshold": c.similarity_threshold,
                "shared_events": c.shared_events,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&["Clustering error: ", &e.to_string()].concat()),
    }
}

/// Correlate structural clusters with adverse event patterns to find alert candidates.
pub fn retro_correlate_alerts(p: RetroCorrelateAlertsParams) -> Result<CallToolResult, McpError> {
    let clusters: Vec<nexcore_retrocasting::StructuralCluster> =
        match serde_json::from_str(&p.clusters_json) {
            Ok(c) => c,
            Err(e) => {
                return err_result(&["Failed to parse clusters JSON: ", &e.to_string()].concat());
            }
        };

    let signals_raw: Vec<serde_json::Value> = match serde_json::from_str(&p.signals_json) {
        Ok(v) => v,
        Err(e) => return err_result(&["Failed to parse signals JSON: ", &e.to_string()].concat()),
    };

    let structured: Vec<nexcore_retrocasting::StructuredSignal> = signals_raw
        .iter()
        .map(|v| nexcore_retrocasting::StructuredSignal {
            signal: nexcore_retrocasting::SignalRecord {
                drug: v
                    .get("drug")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string(),
                event: v
                    .get("event")
                    .and_then(|e| e.as_str())
                    .unwrap_or("")
                    .to_string(),
                prr: v.get("prr").and_then(|p| p.as_f64()).unwrap_or(0.0),
                ror: v.get("ror").and_then(|r| r.as_f64()).unwrap_or(0.0),
                case_count: v.get("case_count").and_then(|c| c.as_u64()).unwrap_or(0),
                ror_lci: v.get("ror_lci").and_then(|l| l.as_f64()),
                prr_chi_sq: v.get("prr_chi_sq").and_then(|c| c.as_f64()),
            },
            compound: None,
            has_structure: v.get("smiles").and_then(|s| s.as_str()).is_some(),
        })
        .collect();

    let min_confidence = p.min_confidence.unwrap_or(0.5);
    match nexcore_retrocasting::correlate::correlate_alerts(&clusters, &structured, min_confidence)
    {
        Ok(candidates) => ok_json(json!({
            "candidate_count": candidates.len(),
            "min_confidence": min_confidence,
            "candidates": candidates.iter().map(|c| json!({
                "fragment_smiles": c.fragment_smiles,
                "associated_events": c.associated_events,
                "confidence": c.confidence,
                "supporting_drugs": c.supporting_drugs,
                "mean_prr": c.mean_prr,
                "support_count": c.support_count,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&["Correlation error: ", &e.to_string()].concat()),
    }
}

/// Extract a 160-dimensional ML feature vector from a SMILES string.
pub fn retro_extract_features(p: RetroExtractFeaturesParams) -> Result<CallToolResult, McpError> {
    let features = nexcore_retrocasting::training::extract_features(Some(&p.smiles));
    ok_json(json!({
        "smiles": p.smiles,
        "feature_dim": features.len(),
        "features": features,
    }))
}

/// Compute summary statistics for a retrocasting training dataset.
pub fn retro_dataset_stats(p: RetroDatasetStatsParams) -> Result<CallToolResult, McpError> {
    let dataset: nexcore_retrocasting::TrainingDataset = match serde_json::from_str(&p.dataset_json)
    {
        Ok(d) => d,
        Err(e) => return err_result(&["Failed to parse dataset JSON: ", &e.to_string()].concat()),
    };

    let stats = nexcore_retrocasting::training::dataset_stats(&dataset);
    ok_json(json!({
        "total_records": stats.total_records,
        "positive_count": stats.positive_count,
        "negative_count": stats.negative_count,
        "class_balance": stats.class_balance,
        "mean_prr_positive": stats.mean_prr_positive,
        "feature_dim": stats.feature_dim,
        "cohort_years": stats.cohort_years,
    }))
}
