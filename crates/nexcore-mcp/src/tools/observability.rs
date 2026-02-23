//! AI Observability Metrics — inference latency, confidence, data freshness.
//!
//! Inspired by AI Engineering Bible Section 37 (Orchestration & Observability):
//! provides AI-specific observability beyond traditional monitoring, enabling
//! root-cause analysis of model performance degradation.
//!
//! # Key Metrics
//!
//! 1. **Inference Latency**: Per-endpoint latency tracking with percentiles (p50, p95, p99)
//! 2. **Data Freshness**: SLA-based tracking of data source staleness
//! 3. **Throughput & Error Rates**: Request counting with success/failure breakdown
//!
//! # T1 Grounding: ν(Frequency) + N(Quantity) + ∂(Boundary) + π(Persistence)
//! - ν: Request rate and throughput tracking
//! - N: Latency measurements and percentile computation
//! - ∂: SLA boundaries for freshness and latency
//! - π: Persistent metric storage

use crate::params::{
    ObservabilityFreshnessParams, ObservabilityQueryParams, ObservabilityRecordLatencyParams,
};
use parking_lot::RwLock;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::LazyLock;

// ============================================================================
// State
// ============================================================================

fn now_secs() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LatencyRecord {
    timestamp: f64,
    latency_ms: f64,
    success: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct EndpointMetrics {
    records: Vec<LatencyRecord>,
    tags: Vec<String>,
    created_at: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct FreshnessRecord {
    source_id: String,
    last_updated: f64,
    max_age_secs: u64,
    recorded_at: f64,
}

#[derive(Debug, Default)]
struct ObservabilityState {
    endpoints: HashMap<String, EndpointMetrics>,
    freshness: HashMap<String, FreshnessRecord>,
}

static STATE: LazyLock<RwLock<ObservabilityState>> =
    LazyLock::new(|| RwLock::new(ObservabilityState::default()));

/// Compute percentile from sorted values.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }
    let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

// ============================================================================
// MCP Tools
// ============================================================================

/// `observability_record_latency` — Record an inference/tool latency measurement.
///
/// Stores latency with timestamp for percentile computation.
/// Track per-model, per-tool, or per-endpoint latencies.
pub fn observability_record_latency(
    params: ObservabilityRecordLatencyParams,
) -> Result<CallToolResult, McpError> {
    let success = params.success.unwrap_or(true);
    let now = now_secs();

    let mut state = STATE.write();
    let metrics = state
        .endpoints
        .entry(params.endpoint_id.clone())
        .or_insert_with(|| EndpointMetrics {
            records: Vec::new(),
            tags: Vec::new(),
            created_at: now,
        });

    metrics.records.push(LatencyRecord {
        timestamp: now,
        latency_ms: params.latency_ms,
        success,
    });

    if let Some(tags) = params.tags {
        for tag in tags {
            if !metrics.tags.contains(&tag) {
                metrics.tags.push(tag);
            }
        }
    }

    // Prune old records (keep last 10000)
    if metrics.records.len() > 10000 {
        let drain = metrics.records.len() - 10000;
        metrics.records.drain(..drain);
    }

    let total = metrics.records.len();
    let success_count = metrics.records.iter().filter(|r| r.success).count();

    let result = json!({
        "endpoint_id": params.endpoint_id,
        "recorded": {
            "latency_ms": params.latency_ms,
            "success": success,
            "timestamp": now,
        },
        "endpoint_stats": {
            "total_records": total,
            "success_count": success_count,
            "error_count": total - success_count,
            "error_rate": if total > 0 {
                ((total - success_count) as f64 / total as f64 * 1000.0).round() / 1000.0
            } else {
                0.0
            },
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `observability_query` — Query observability metrics with percentiles.
///
/// Returns p50, p95, p99 latency, error rates, throughput, and trend analysis
/// for a specific endpoint or all endpoints.
pub fn observability_query(params: ObservabilityQueryParams) -> Result<CallToolResult, McpError> {
    let window_secs = params.window_secs.unwrap_or(300) as f64;
    let cutoff = now_secs() - window_secs;
    let state = STATE.read();

    let endpoints_to_query: Vec<(&String, &EndpointMetrics)> = match params.endpoint_id {
        Some(ref id) => match state.endpoints.get(id) {
            Some(m) => vec![(id, m)],
            None => {
                return Err(McpError::invalid_params(
                    format!("Endpoint '{}' not found", id),
                    None,
                ));
            }
        },
        None => state.endpoints.iter().collect(),
    };

    let mut endpoint_results: Vec<serde_json::Value> = Vec::new();

    for (id, metrics) in &endpoints_to_query {
        let window_records: Vec<&LatencyRecord> = metrics
            .records
            .iter()
            .filter(|r| r.timestamp >= cutoff)
            .collect();

        let total = window_records.len();
        let successes = window_records.iter().filter(|r| r.success).count();
        let errors = total - successes;

        let mut latencies: Vec<f64> = window_records.iter().map(|r| r.latency_ms).collect();
        latencies.sort_by(|a, b| a.total_cmp(b));

        let p50 = percentile(&latencies, 50.0);
        let p95 = percentile(&latencies, 95.0);
        let p99 = percentile(&latencies, 99.0);
        let mean = if latencies.is_empty() {
            0.0
        } else {
            latencies.iter().sum::<f64>() / latencies.len() as f64
        };
        let min_lat = latencies.first().copied().unwrap_or(0.0);
        let max_lat = latencies.last().copied().unwrap_or(0.0);

        let throughput = if window_secs > 0.0 {
            total as f64 / window_secs
        } else {
            0.0
        };

        // Latency health assessment
        let latency_health = if p99 < 100.0 {
            "excellent"
        } else if p99 < 500.0 {
            "good"
        } else if p99 < 2000.0 {
            "degraded"
        } else {
            "critical"
        };

        endpoint_results.push(json!({
            "endpoint_id": id,
            "tags": metrics.tags,
            "window_secs": window_secs,
            "latency": {
                "p50_ms": (p50 * 100.0).round() / 100.0,
                "p95_ms": (p95 * 100.0).round() / 100.0,
                "p99_ms": (p99 * 100.0).round() / 100.0,
                "mean_ms": (mean * 100.0).round() / 100.0,
                "min_ms": (min_lat * 100.0).round() / 100.0,
                "max_ms": (max_lat * 100.0).round() / 100.0,
                "health": latency_health,
            },
            "throughput": {
                "total_requests": total,
                "requests_per_sec": (throughput * 1000.0).round() / 1000.0,
            },
            "errors": {
                "count": errors,
                "rate": if total > 0 { (errors as f64 / total as f64 * 1000.0).round() / 1000.0 } else { 0.0 },
                "successes": successes,
            },
            "total_all_time": metrics.records.len(),
        }));
    }

    // Data freshness summary
    let freshness_entries: Vec<serde_json::Value> = state
        .freshness
        .values()
        .map(|f| {
            let age_secs = now_secs() - f.last_updated;
            let stale = age_secs > f.max_age_secs as f64;
            json!({
                "source_id": f.source_id,
                "age_secs": age_secs.round(),
                "max_age_secs": f.max_age_secs,
                "stale": stale,
                "status": if stale { "STALE" } else { "FRESH" },
            })
        })
        .collect();

    let result = json!({
        "endpoints": endpoint_results,
        "endpoint_count": endpoint_results.len(),
        "data_freshness": freshness_entries,
        "freshness_count": freshness_entries.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `observability_freshness` — Track data source freshness against SLAs.
///
/// Records when a data source was last updated and checks against
/// a maximum acceptable age (SLA). Flags stale sources.
pub fn observability_freshness(
    params: ObservabilityFreshnessParams,
) -> Result<CallToolResult, McpError> {
    let now = now_secs();
    let last_updated = params.last_updated.unwrap_or(now);
    let max_age_secs = params.max_age_secs.unwrap_or(3600);

    let mut state = STATE.write();

    let record = FreshnessRecord {
        source_id: params.source_id.clone(),
        last_updated,
        max_age_secs,
        recorded_at: now,
    };

    state.freshness.insert(params.source_id.clone(), record);

    let age_secs = now - last_updated;
    let stale = age_secs > max_age_secs as f64;
    let remaining_secs = (max_age_secs as f64 - age_secs).max(0.0);
    let freshness_pct = if max_age_secs > 0 {
        ((1.0 - age_secs / max_age_secs as f64) * 100.0).clamp(0.0, 100.0)
    } else {
        0.0
    };

    let result = json!({
        "source_id": params.source_id,
        "last_updated": last_updated,
        "age_secs": age_secs.round(),
        "max_age_secs": max_age_secs,
        "remaining_secs": remaining_secs.round(),
        "freshness_pct": (freshness_pct * 10.0).round() / 10.0,
        "stale": stale,
        "status": if stale { "STALE" } else { "FRESH" },
        "recommendation": if stale {
            "Data source has exceeded its freshness SLA. Trigger refresh or investigate pipeline."
        } else if freshness_pct < 25.0 {
            "Data source is approaching staleness. Schedule proactive refresh."
        } else {
            "Data source is within freshness SLA."
        },
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
