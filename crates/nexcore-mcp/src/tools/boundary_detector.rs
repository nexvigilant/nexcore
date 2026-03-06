//! Boundary Detector MCP Tools
//!
//! Operational ∂ primitive: scan values against named boundaries,
//! detect crossings, measure proximity, classify regimes.
//!
//! T1 composition: ∂(Boundary) + κ(Comparison) + ς(State) + →(Causality) + N(Quantity)

use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;

use crate::params::{
    BoundaryDef, BoundaryDetectProximityParams, BoundaryDetectScanParams,
    BoundaryDetectStreamParams,
};

/// Scan value(s) against multiple named boundaries.
/// Returns per-boundary classification, distance, and overall regime.
pub fn scan(params: BoundaryDetectScanParams) -> Result<CallToolResult, McpError> {
    let values = &params.values;
    let boundaries = &params.boundaries;

    if boundaries.is_empty() {
        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            r#"{"error": "No boundaries provided"}"#,
        )]));
    }

    let mut detections = Vec::new();
    let mut above_count = 0usize;
    let mut below_count = 0usize;
    let mut marginal_count = 0usize;
    let mut marginal_names = Vec::new();

    let scan_value: f64 = if values.len() == 1 {
        values[0]
    } else {
        // Multi-dimensional: weighted sum
        values.iter().sum::<f64>() / values.len() as f64
    };

    for b in boundaries {
        let distance = scan_value - b.threshold;
        let scale = b.threshold.abs().max(1.0);
        let proximity = (distance.abs() / scale).min(1.0);
        let margin = 0.05 * b.threshold.abs().max(0.01);

        let classification = if distance.abs() < margin {
            marginal_count += 1;
            marginal_names.push(b.name.clone());
            "MARGINAL"
        } else if distance > 0.0 {
            above_count += 1;
            "ABOVE"
        } else {
            below_count += 1;
            "BELOW"
        };

        detections.push(serde_json::json!({
            "boundary": b.name,
            "threshold": b.threshold,
            "value": scan_value,
            "classification": classification,
            "distance": distance,
            "proximity_pct": (proximity * 100.0 * 100.0).round() / 100.0,
            "weight": b.weight,
        }));
    }

    let regime = if marginal_count > 0 {
        serde_json::json!({
            "type": "WARNING",
            "marginal_boundaries": marginal_names,
        })
    } else if above_count == boundaries.len() {
        serde_json::json!({ "type": "ALL_ABOVE" })
    } else if below_count == boundaries.len() {
        serde_json::json!({ "type": "ALL_BELOW" })
    } else {
        serde_json::json!({
            "type": "MIXED",
            "above": above_count,
            "below": below_count,
        })
    };

    let result = serde_json::json!({
        "scan_value": scan_value,
        "detections": detections,
        "regime": regime,
        "boundary_count": boundaries.len(),
        "t1_grounding": "∂(Boundary) + κ(Comparison) + N(Quantity)",
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Scan a stream of values, detecting boundary crossings and regime transitions.
pub fn stream(params: BoundaryDetectStreamParams) -> Result<CallToolResult, McpError> {
    let values = &params.stream;
    let boundaries = &params.boundaries;

    if boundaries.is_empty() || values.is_empty() {
        return Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            r#"{"error": "Empty boundaries or stream"}"#,
        )]));
    }

    let mut crossings: Vec<serde_json::Value> = Vec::new();
    let mut crossing_counts: Vec<usize> = vec![0; boundaries.len()];
    let mut prev_class: Vec<Option<String>> = vec![None; boundaries.len()];
    let mut closest: Vec<(f64, usize)> = vec![(f64::MAX, 0); boundaries.len()];
    let mut above_time = 0usize;
    let mut below_time = 0usize;
    let mut mixed_time = 0usize;
    let mut warning_time = 0usize;

    for (idx, &value) in values.iter().enumerate() {
        let mut point_above = 0usize;
        let mut point_below = 0usize;
        let mut point_marginal = 0usize;

        for (bi, b) in boundaries.iter().enumerate() {
            let distance = value - b.threshold;
            let margin = 0.05 * b.threshold.abs().max(0.01);

            let class = if distance.abs() < margin {
                point_marginal += 1;
                "MARGINAL"
            } else if distance > 0.0 {
                point_above += 1;
                "ABOVE"
            } else {
                point_below += 1;
                "BELOW"
            };

            // Track closest approach
            if distance.abs() < closest[bi].0 {
                closest[bi] = (distance.abs(), idx);
            }

            // Detect crossing
            if let Some(ref prev) = prev_class[bi] {
                let crossed = prev != class && !(prev == "MARGINAL" || class == "MARGINAL");
                if crossed {
                    crossing_counts[bi] += 1;
                    let direction = if prev == "BELOW" && class == "ABOVE" {
                        "RISING"
                    } else {
                        "FALLING"
                    };
                    crossings.push(serde_json::json!({
                        "index": idx,
                        "value": value,
                        "boundary": b.name,
                        "direction": direction,
                        "from": prev,
                        "to": class,
                    }));
                }
            }

            prev_class[bi] = Some(class.to_string());
        }

        // Regime tracking
        if point_marginal > 0 {
            warning_time += 1;
        } else if point_above == boundaries.len() {
            above_time += 1;
        } else if point_below == boundaries.len() {
            below_time += 1;
        } else {
            mixed_time += 1;
        }
    }

    let total_crossings: usize = crossing_counts.iter().sum();

    let crossings_per_boundary: Vec<serde_json::Value> = boundaries
        .iter()
        .zip(crossing_counts.iter())
        .map(|(b, &c)| {
            serde_json::json!({
                "boundary": b.name,
                "crossings": c,
            })
        })
        .collect();

    let closest_approach: Vec<serde_json::Value> = boundaries
        .iter()
        .zip(closest.iter())
        .map(|(b, &(dist, idx))| {
            serde_json::json!({
                "boundary": b.name,
                "closest_distance": (dist * 10000.0).round() / 10000.0,
                "at_index": idx,
            })
        })
        .collect();

    let result = serde_json::json!({
        "points": values.len(),
        "total_crossings": total_crossings,
        "crossings_per_boundary": crossings_per_boundary,
        "crossing_events": crossings,
        "closest_approach": closest_approach,
        "regime_durations": {
            "above": above_time,
            "below": below_time,
            "mixed": mixed_time,
            "warning": warning_time,
        },
        "t1_grounding": "∂(Boundary) + κ(Comparison) + ς(State) + ν(Frequency)",
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

/// Quick proximity check: how close is a value to a single boundary?
pub fn proximity(params: BoundaryDetectProximityParams) -> Result<CallToolResult, McpError> {
    let distance = params.value - params.threshold;
    let scale = params.threshold.abs().max(1.0);
    let proximity_pct = ((distance.abs() / scale) * 100.0 * 100.0).round() / 100.0;
    let margin = 0.05 * params.threshold.abs().max(0.01);

    let classification = if distance.abs() < margin {
        "MARGINAL"
    } else if distance > 0.0 {
        "ABOVE"
    } else {
        "BELOW"
    };

    let urgency = if proximity_pct < 5.0 {
        "CRITICAL — at the boundary"
    } else if proximity_pct < 15.0 {
        "HIGH — approaching boundary"
    } else if proximity_pct < 30.0 {
        "MEDIUM — boundary visible"
    } else {
        "LOW — far from boundary"
    };

    let result = serde_json::json!({
        "boundary": params.name,
        "value": params.value,
        "threshold": params.threshold,
        "distance": distance,
        "proximity_pct": proximity_pct,
        "classification": classification,
        "urgency": urgency,
        "t1_grounding": "∂(Boundary) + κ(Comparison) + N(Quantity)",
    });

    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}
