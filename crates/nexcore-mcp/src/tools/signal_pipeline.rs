//! Signal detection pipeline MCP tools.
//!
//! Exposes nexcore-signal-pipeline: disproportionality metrics (PRR/ROR/IC/EBGM),
//! contingency tables, Evans thresholds, validation, reporting, relay chains, and
//! cross-domain transfer mappings.

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use signal::prelude::*;

use crate::params::signal_pipeline::{
    PipelineBatchComputeParams, PipelineComputeAllParams, PipelineDetectParams,
    PipelinePrimitivesParams, PipelineRelayChainParams, PipelineReportParams,
    PipelineThresholdsParams, PipelineTransferParams, PipelineValidateParams,
};

// ── Helpers ─────────────────────────────────────────────────────────────────

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

fn metrics_to_json(m: &signal::stats::SignalMetrics) -> serde_json::Value {
    serde_json::json!({
        "prr": m.prr.as_ref().map(|p| p.0),
        "ror": m.ror.as_ref().map(|r| r.0),
        "ic": m.ic.0,
        "ebgm": m.ebgm.0,
        "chi_square": m.chi_square.0,
        "strength": format!("{:?}", m.strength),
    })
}

// ── Tools ───────────────────────────────────────────────────────────────────

/// Compute all disproportionality metrics (PRR, ROR, IC, EBGM, Chi-square) from
/// a 2x2 contingency table.
pub fn pipeline_compute_all(p: PipelineComputeAllParams) -> Result<CallToolResult, McpError> {
    let table = ContingencyTable::new(p.a, p.b, p.c, p.d);
    let metrics = signal::stats::compute_all(&table);
    ok_json(serde_json::json!({
        "table": { "a": p.a, "b": p.b, "c": p.c, "d": p.d, "total": table.total() },
        "metrics": metrics_to_json(&metrics),
    }))
}

/// Batch compute disproportionality metrics for multiple drug-event pairs.
pub fn pipeline_batch_compute(p: PipelineBatchComputeParams) -> Result<CallToolResult, McpError> {
    let pairs_tables: Vec<(DrugEventPair, ContingencyTable)> = p
        .items
        .iter()
        .map(|item| {
            let pair = DrugEventPair::new(&item.drug, &item.event);
            let table = ContingencyTable::new(item.a, item.b, item.c, item.d);
            (pair, table)
        })
        .collect();

    let results = signal::stats::compute_batch(&pairs_tables);
    let mut signals_found = 0usize;
    let items: Vec<serde_json::Value> = results
        .iter()
        .map(|(pair, metrics)| {
            let is_signal = matches!(
                metrics.strength,
                SignalStrength::Moderate | SignalStrength::Strong | SignalStrength::Critical
            );
            if is_signal {
                signals_found += 1;
            }
            serde_json::json!({
                "drug": pair.drug,
                "event": pair.event,
                "metrics": metrics_to_json(metrics),
                "signal": is_signal,
            })
        })
        .collect();

    ok_json(serde_json::json!({
        "results": items,
        "total": items.len(),
        "signals_found": signals_found,
    }))
}

/// Detect a signal for a drug-event pair with configurable thresholds.
pub fn pipeline_detect(p: PipelineDetectParams) -> Result<CallToolResult, McpError> {
    let table = ContingencyTable::new(p.a, p.b, p.c, p.d);
    let metrics = signal::stats::compute_all(&table);

    let prr_min = p.prr_min.unwrap_or(2.0);
    let chi_square_min = p.chi_square_min.unwrap_or(3.841);
    let case_count_min = p.case_count_min.unwrap_or(3);

    let config = ThresholdConfig::with_mins(prr_min, chi_square_min, case_count_min);

    let threshold = signal::threshold::EvansThreshold::with_config(config);
    let pair = DrugEventPair::new(&p.drug, &p.event);
    let now = nexcore_chrono::DateTime::now();

    let detection = DetectionResult::new(
        pair,
        table,
        metrics.prr.clone(),
        metrics.ror.clone(),
        Some(metrics.ic),
        Some(metrics.ebgm),
        metrics.chi_square,
        metrics.strength,
        now,
    );

    let passes = <signal::threshold::EvansThreshold as signal::core::Threshold>::apply(
        &threshold, &detection,
    );

    ok_json(serde_json::json!({
        "drug": p.drug,
        "event": p.event,
        "signal_detected": passes,
        "metrics": metrics_to_json(&metrics),
        "thresholds_used": {
            "prr_min": prr_min,
            "chi_square_min": chi_square_min,
            "case_count_min": case_count_min,
        },
    }))
}

/// Validate a detection result against multiple quality checks.
pub fn pipeline_validate(p: PipelineValidateParams) -> Result<CallToolResult, McpError> {
    let table = ContingencyTable::new(p.a, p.b, p.c, p.d);
    let metrics = signal::stats::compute_all(&table);
    let pair = DrugEventPair::new(&p.drug, &p.event);

    let config = if p.strict.unwrap_or(false) {
        ThresholdConfig::strict()
    } else {
        ThresholdConfig::default()
    };

    let validator = signal::validate::StandardValidator::with_config(config);
    let detection = DetectionResult::new(
        pair.clone(),
        table,
        metrics.prr.clone(),
        metrics.ror.clone(),
        Some(metrics.ic),
        Some(metrics.ebgm),
        metrics.chi_square,
        metrics.strength,
        nexcore_chrono::DateTime::now(),
    );

    match <signal::validate::StandardValidator as signal::core::Validate>::validate(
        &validator, &detection,
    ) {
        Ok(report) => ok_json(serde_json::json!({
            "drug": p.drug,
            "event": p.event,
            "passed": report.passed,
            "checks": report.checks.iter().map(|c| serde_json::json!({
                "name": c.name,
                "passed": c.passed,
                "message": c.message,
            })).collect::<Vec<_>>(),
        })),
        Err(e) => err_result(&format!("Validation error: {e}")),
    }
}

/// Get threshold configurations (Evans, strict, sensitive).
pub fn pipeline_thresholds(p: PipelineThresholdsParams) -> Result<CallToolResult, McpError> {
    let format_config = |c: ThresholdConfig| {
        serde_json::json!({
            "prr_min": c.prr_min,
            "chi_square_min": c.chi_square_min,
            "case_count_min": c.case_count_min,
            "ror_lower_ci_min": c.ror_lower_ci_min,
            "ic025_min": c.ic025_min,
            "eb05_min": c.eb05_min,
        })
    };

    match p.config.as_deref() {
        Some("evans") => ok_json(format_config(ThresholdConfig::default())),
        Some("strict") => ok_json(format_config(ThresholdConfig::strict())),
        Some("sensitive") => ok_json(format_config(ThresholdConfig::sensitive())),
        Some(other) => err_result(&format!(
            "Unknown config: {other}. Use evans, strict, or sensitive."
        )),
        None => ok_json(serde_json::json!({
            "evans": format_config(ThresholdConfig::default()),
            "strict": format_config(ThresholdConfig::strict()),
            "sensitive": format_config(ThresholdConfig::sensitive()),
        })),
    }
}

/// Generate a signal detection report from batch data.
pub fn pipeline_report(p: PipelineReportParams) -> Result<CallToolResult, McpError> {
    let pairs_tables: Vec<(DrugEventPair, ContingencyTable)> = p
        .items
        .iter()
        .map(|item| {
            (
                DrugEventPair::new(&item.drug, &item.event),
                ContingencyTable::new(item.a, item.b, item.c, item.d),
            )
        })
        .collect();

    let results = signal::stats::compute_batch(&pairs_tables);
    let threshold = signal::threshold::EvansThreshold::new();

    let detections: Vec<DetectionResult> = pairs_tables
        .iter()
        .zip(results.iter())
        .map(|((pair, table), (_, metrics))| {
            DetectionResult::new(
                pair.clone(),
                *table,
                metrics.prr.clone(),
                metrics.ror.clone(),
                Some(metrics.ic),
                Some(metrics.ebgm),
                metrics.chi_square,
                metrics.strength,
                nexcore_chrono::DateTime::now(),
            )
        })
        .collect();

    let format = p.format.as_deref().unwrap_or("json");
    let reporter: Box<dyn signal::core::Report> = match format {
        "table" => Box::new(signal::report::TableReporter::new()),
        _ => Box::new(signal::report::JsonReporter::new()),
    };

    match reporter.report(&detections) {
        Ok(text) => ok_json(serde_json::json!({
            "format": format,
            "report": text,
            "signal_count": detections.iter().filter(|d| {
                <signal::threshold::EvansThreshold as signal::core::Threshold>::apply(&threshold, d)
            }).count(),
            "total_pairs": detections.len(),
        })),
        Err(e) => err_result(&format!("Report generation error: {e}")),
    }
}

/// Get the PV pipeline relay chain with fidelity per stage.
pub fn pipeline_relay_chain(p: PipelineRelayChainParams) -> Result<CallToolResult, McpError> {
    let chain_type = p.chain_type.as_deref().unwrap_or("full");
    let chain = match chain_type {
        "core" => core_detection_chain(),
        _ => pv_pipeline_chain(),
    };

    let verification = chain.verify();

    ok_json(serde_json::json!({
        "chain_type": chain_type,
        "hops": chain.hops().iter().map(|h| serde_json::json!({
            "stage": h.stage,
            "fidelity": h.fidelity.value(),
            "threshold": h.threshold,
            "activated": h.activated,
        })).collect::<Vec<_>>(),
        "total_fidelity": chain.total_fidelity().value(),
        "signal_loss": chain.signal_loss(),
        "preservation_verified": chain.verify_preservation(),
        "f_min": chain.f_min(),
        "active_hops": chain.active_hop_count(),
        "total_hops": chain.hop_count(),
        "axioms_passing": verification.axioms_passing(),
        "axiom_count": verification.axiom_count(),
        "is_valid": verification.is_valid(),
    }))
}

/// Look up cross-domain transfer mappings for signal-pipeline types.
pub fn pipeline_transfer(p: PipelineTransferParams) -> Result<CallToolResult, McpError> {
    let all_mappings = signal::transfer::transfer_mappings();

    let filtered: Vec<&signal::transfer::TransferMapping> = match (&p.source_type, &p.domain) {
        (Some(t), Some(d)) => all_mappings
            .iter()
            .filter(|m| m.source_type == t.as_str() && m.domain == d.as_str())
            .collect(),
        (Some(t), None) => signal::transfer::transfers_for_type(t),
        (None, Some(d)) => signal::transfer::transfers_for_domain(d),
        (None, None) => all_mappings.iter().collect(),
    };

    ok_json(serde_json::json!({
        "mappings": filtered.iter().map(|m| serde_json::json!({
            "source_type": m.source_type,
            "domain": m.domain,
            "analog": m.analog,
            "confidence": m.confidence,
        })).collect::<Vec<_>>(),
        "count": filtered.len(),
        "aggregate_confidence": signal::transfer::transfer_confidence("ContingencyTable"),
    }))
}

/// Get the crate's T1 primitive manifest.
pub fn pipeline_primitives(p: PipelinePrimitivesParams) -> Result<CallToolResult, McpError> {
    let manifest = signal::primitives::crate_manifest();

    if let Some(stage_name) = &p.stage {
        match signal::primitives::stage_primitive(stage_name) {
            Some(stage) => ok_json(serde_json::json!({
                "stage": stage.stage,
                "order": stage.order,
                "dominant_primitive": format!("{:?}", stage.dominant),
                "rationale": stage.rationale,
            })),
            None => err_result(&format!("Unknown stage: {stage_name}")),
        }
    } else {
        ok_json(serde_json::json!({
            "crate_name": manifest.crate_name,
            "primitive_count": manifest.primitive_count,
            "stages": manifest.stages.iter().map(|s| serde_json::json!({
                "stage": s.stage,
                "order": s.order,
                "dominant": format!("{:?}", s.dominant),
                "rationale": s.rationale,
            })).collect::<Vec<_>>(),
        }))
    }
}
