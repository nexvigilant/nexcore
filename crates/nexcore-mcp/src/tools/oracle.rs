//! Oracle MCP tools — Bayesian event prediction engine
//!
//! Learns transition probabilities from event sequences, predicts what
//! happens next, and self-tracks accuracy over time.
//!
//! State persists to `~/.claude/oracle/state.json` between calls.
//!
//! ## T1 Primitive Grounding
//!
//! - σ (Sequence) → event sequences, transition chains
//! - → (Causality) → P(next | current) predictions
//! - ν (Frequency) → transition counts, observation frequency
//! - κ (Comparison) → accuracy tracking, calibration
//! - N (Quantity) → confidence scores, entropy, Brier score
//! - π (Persistence) → file-backed Oracle state

use nexcore_oracle::{EventSequence, Oracle};
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use std::path::PathBuf;

use crate::params::{
    OracleIngestParams, OracleObserveParams, OraclePredictParams, OracleResetParams,
    OracleStatusParams, OracleTopPredictionsParams,
};

// ============================================================================
// Persistence
// ============================================================================

fn oracle_path() -> PathBuf {
    let dir = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".claude")
        .join("oracle");
    dir
}

fn state_file() -> PathBuf {
    oracle_path().join("state.json")
}

fn load_oracle() -> Result<Oracle, McpError> {
    let path = state_file();
    if !path.exists() {
        return Ok(Oracle::new());
    }
    let data = std::fs::read_to_string(&path)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    serde_json::from_str(&data).map_err(|e| McpError::internal_error(e.to_string(), None))
}

fn save_oracle(oracle: &Oracle) -> Result<(), McpError> {
    let dir = oracle_path();
    std::fs::create_dir_all(&dir).map_err(|e| McpError::internal_error(e.to_string(), None))?;
    let data = serde_json::to_string_pretty(oracle)
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;
    std::fs::write(state_file(), data).map_err(|e| McpError::internal_error(e.to_string(), None))
}

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value)
            .map_err(|e| McpError::internal_error(e.to_string(), None))?,
    )]))
}

// ============================================================================
// Tools
// ============================================================================

/// Ingest an event sequence to teach the Oracle transition patterns.
pub fn oracle_ingest(params: OracleIngestParams) -> Result<CallToolResult, McpError> {
    let mut oracle = load_oracle()?;

    let mut seq = EventSequence::new();
    for kind in &params.events {
        seq.push_kind(kind);
    }

    let event_count = seq.len();
    oracle.ingest(&seq);
    save_oracle(&oracle)?;

    ok_json(serde_json::json!({
        "success": true,
        "events_ingested": event_count,
        "total_events": oracle.total_events(),
        "vocabulary_size": oracle.vocabulary_size(),
        "predictability": format!("{:.3}", oracle.predictability()),
    }))
}

/// Predict the next event given the current state.
pub fn oracle_predict(params: OraclePredictParams) -> Result<CallToolResult, McpError> {
    let oracle = load_oracle()?;

    let pred = if let Some(prev) = &params.previous {
        oracle.predict_with_context(&params.current, prev)
    } else {
        oracle.predict(&params.current)
    };

    let alternatives: Vec<serde_json::Value> = pred
        .alternatives
        .iter()
        .take(5)
        .map(|(name, conf)| {
            serde_json::json!({
                "event": name,
                "confidence": format!("{:.4}", conf),
            })
        })
        .collect();

    ok_json(serde_json::json!({
        "predicted_event": pred.event,
        "confidence": format!("{:.4}", pred.confidence),
        "entropy": format!("{:.4}", pred.entropy),
        "evidence_count": pred.evidence_count,
        "alternatives": alternatives,
        "context": {
            "current": params.current,
            "previous": params.previous,
        },
    }))
}

/// Record what actually happened after a prediction (feeds accuracy tracker).
pub fn oracle_observe(params: OracleObserveParams) -> Result<CallToolResult, McpError> {
    let mut oracle = load_oracle()?;

    // Get the prediction first to report it
    let pred = oracle.predict(&params.predicted_from);
    let was_correct = pred.event == params.actual_next;

    oracle.observe(&params.predicted_from, &params.actual_next);
    save_oracle(&oracle)?;

    let report = oracle.accuracy_report();

    ok_json(serde_json::json!({
        "success": true,
        "predicted": pred.event,
        "actual": params.actual_next,
        "correct": was_correct,
        "confidence_was": format!("{:.4}", pred.confidence),
        "accuracy": {
            "recent": format!("{:.3}", report.recent_accuracy),
            "lifetime": format!("{:.3}", report.lifetime_accuracy),
            "lifetime_count": report.lifetime_count,
            "brier_score": format!("{:.4}", report.brier_score),
        },
    }))
}

/// Get the Oracle's accuracy report and calibration status.
pub fn oracle_report(_params: OracleStatusParams) -> Result<CallToolResult, McpError> {
    let oracle = load_oracle()?;
    let report = oracle.accuracy_report();

    ok_json(serde_json::json!({
        "accuracy": {
            "recent": format!("{:.3}", report.recent_accuracy),
            "lifetime": format!("{:.3}", report.lifetime_accuracy),
            "window_count": report.window_count,
            "lifetime_count": report.lifetime_count,
        },
        "calibration": {
            "is_calibrated": oracle.is_calibrated(),
            "avg_confidence_correct": format!("{:.4}", report.avg_confidence_correct),
            "avg_confidence_incorrect": format!("{:.4}", report.avg_confidence_incorrect),
            "brier_score": format!("{:.4}", report.brier_score),
        },
        "system": {
            "total_events": oracle.total_events(),
            "vocabulary_size": oracle.vocabulary_size(),
            "predictability": format!("{:.3}", oracle.predictability()),
        },
    }))
}

/// Get Oracle status (quick overview).
pub fn oracle_status(_params: OracleStatusParams) -> Result<CallToolResult, McpError> {
    let oracle = load_oracle()?;

    ok_json(serde_json::json!({
        "total_events": oracle.total_events(),
        "vocabulary_size": oracle.vocabulary_size(),
        "predictability": format!("{:.3}", oracle.predictability()),
        "lifetime_predictions": oracle.accuracy_report().lifetime_count,
        "is_calibrated": oracle.is_calibrated(),
        "state_file": state_file().display().to_string(),
    }))
}

/// Reset the Oracle (start fresh with optional config).
pub fn oracle_reset(params: OracleResetParams) -> Result<CallToolResult, McpError> {
    let oracle = Oracle::with_config(params.alpha, params.accuracy_window);
    save_oracle(&oracle)?;

    ok_json(serde_json::json!({
        "success": true,
        "alpha": params.alpha,
        "accuracy_window": params.accuracy_window,
        "state_file": state_file().display().to_string(),
    }))
}

/// Get top-N predictions from a state (with optional 2nd-order context).
pub fn oracle_top_predictions(
    params: OracleTopPredictionsParams,
) -> Result<CallToolResult, McpError> {
    let oracle = load_oracle()?;

    let pred = if let Some(prev) = &params.previous {
        oracle.predict_with_context(&params.current, prev)
    } else {
        oracle.predict(&params.current)
    };

    let mut all_predictions = vec![serde_json::json!({
        "rank": 1,
        "event": pred.event,
        "confidence": format!("{:.4}", pred.confidence),
    })];

    for (i, (name, conf)) in pred.alternatives.iter().enumerate() {
        if i + 1 >= params.top_n {
            break;
        }
        all_predictions.push(serde_json::json!({
            "rank": i + 2,
            "event": name,
            "confidence": format!("{:.4}", conf),
        }));
    }

    ok_json(serde_json::json!({
        "current": params.current,
        "previous": params.previous,
        "predictions": all_predictions,
        "entropy": format!("{:.4}", pred.entropy),
        "evidence_count": pred.evidence_count,
    }))
}
