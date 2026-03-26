//! Aggregate MCP Tools — Σ (Sum) + ρ (Recursion) + κ (Comparison)
//!
//! Exposes nexcore-aggregate operations as MCP tools:
//! - fold_all: Single-pass statistical aggregation
//! - tree_fold: Recursive tree aggregation
//! - rank: Comparison-based ranking
//! - percentile: Percentile computation
//! - outliers: IQR-based outlier detection
//!
//! ## Tier: T2-C (Σ + ρ + κ + σ + N)
//!
//! ## Lifecycle
//! - **begins**: Tool invocation starts with input data
//! - **exists**: Computation proceeds through fold/traverse/compare
//! - **changes**: Accumulator state transforms with each element
//! - **persists**: Results returned as JSON
//! - **ends**: Tool completes with aggregated result

use crate::params::{
    AggregateFoldParams, AggregateOutliersParams, AggregatePercentileParams, AggregateRankParams,
    AggregateTreeFoldParams,
};
use nexcore_aggregate::prelude::*;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

// ---------------------------------------------------------------------------
// aggregate_fold_all: Single-pass aggregation
// ---------------------------------------------------------------------------

/// Run all standard folds (sum, count, min, max, mean, variance) in one pass.
pub fn aggregate_fold_all(params: AggregateFoldParams) -> Result<CallToolResult, McpError> {
    let results = fold_all(&params.values);
    let output = json!({
        "sum": results.sum,
        "count": results.count,
        "min": results.min,
        "max": results.max,
        "mean": results.mean,
        "variance": results.variance,
        "std_dev": results.variance.map(|v| v.sqrt()),
        "primitives": ["Σ", "σ", "N", "κ", "∝"],
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "fold complete".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// aggregate_tree_fold: Recursive tree aggregation
// ---------------------------------------------------------------------------

/// Parse a JSON tree and perform recursive fold.
pub fn aggregate_tree_fold(params: AggregateTreeFoldParams) -> Result<CallToolResult, McpError> {
    // Parse tree from JSON
    let tree = match parse_tree_node(&params.tree) {
        Ok(t) => t,
        Err(e) => {
            return Ok(CallToolResult::success(vec![Content::text(
                json!({"error": e.to_string()}).to_string(),
            )]));
        }
    };

    // Select combine function
    let combine_fn: Box<dyn Fn(f64, &[f64]) -> f64> = match params.combine.as_str() {
        "max" => Box::new(combine_max),
        "mean" => Box::new(combine_mean),
        _ => Box::new(combine_sum), // default
    };

    let config = TraversalConfig::default();
    match tree_fold(&tree, &*combine_fn, &config) {
        Ok(result) => {
            let depth = tree_depth(&tree).unwrap_or(0);
            let count = tree_count(&tree).unwrap_or(0);
            let output = json!({
                "result": result,
                "combine": params.combine,
                "depth": depth,
                "node_count": count,
                "primitives": ["ρ", "Σ", "κ"],
            });
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&output)
                    .unwrap_or_else(|_| "tree fold complete".to_string()),
            )]))
        }
        Err(e) => Ok(CallToolResult::success(vec![Content::text(
            json!({"error": e.to_string()}).to_string(),
        )])),
    }
}

/// Parse a JSON value into a SimpleNode tree.
fn parse_tree_node(value: &serde_json::Value) -> Result<SimpleNode, nexcore_error::NexError> {
    let id = value
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| nexcore_error::nexerror!("missing 'id' field"))?;
    let node_value = value
        .get("value")
        .and_then(|v| v.as_f64())
        .ok_or_else(|| nexcore_error::nexerror!("missing 'value' field on node '{id}'"))?;

    let children = if let Some(children_arr) = value.get("children").and_then(|v| v.as_array()) {
        children_arr
            .iter()
            .map(parse_tree_node)
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };

    Ok(SimpleNode::branch(id, node_value, children))
}

// ---------------------------------------------------------------------------
// aggregate_rank: Comparison-based ranking
// ---------------------------------------------------------------------------

/// Rank items by value, optionally returning only top N.
pub fn aggregate_rank(params: AggregateRankParams) -> Result<CallToolResult, McpError> {
    let items: Vec<(&str, f64)> = params
        .items
        .iter()
        .map(|(name, value)| (name.as_str(), *value))
        .collect();

    let ranked = if params.top_n > 0 {
        top_n(&items, params.top_n)
    } else {
        rank(&items)
    };

    let output = json!({
        "ranked": ranked.iter().map(|r| json!({
            "rank": r.rank,
            "name": r.name,
            "value": r.value,
        })).collect::<Vec<_>>(),
        "total": params.items.len(),
        "returned": ranked.len(),
        "primitives": ["κ", "σ", "N"],
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "rank complete".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// aggregate_percentile: Percentile computation
// ---------------------------------------------------------------------------

/// Compute a specific percentile from values.
pub fn aggregate_percentile(params: AggregatePercentileParams) -> Result<CallToolResult, McpError> {
    let result = percentile(&params.values, params.percentile);
    let q = quartiles(&params.values);

    let output = json!({
        "percentile": params.percentile,
        "value": result,
        "quartiles": q.map(|(q1, q2, q3)| json!({
            "q1": q1,
            "median": q2,
            "q3": q3,
        })),
        "iqr": iqr(&params.values),
        "count": params.values.len(),
        "primitives": ["κ", "∝", "N"],
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output).unwrap_or_else(|_| "percentile complete".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// aggregate_outliers: IQR-based outlier detection
// ---------------------------------------------------------------------------

/// Detect outliers using the IQR method.
pub fn aggregate_outliers(params: AggregateOutliersParams) -> Result<CallToolResult, McpError> {
    let items: Vec<(&str, f64)> = params
        .items
        .iter()
        .map(|(name, value)| (name.as_str(), *value))
        .collect();

    let outliers = detect_outliers(&items);
    let values: Vec<f64> = params.items.iter().map(|(_, v)| *v).collect();
    let q = quartiles(&values);

    let output = json!({
        "outliers": outliers.iter().map(|(name, value, direction)| json!({
            "name": name,
            "value": value,
            "direction": format!("{direction:?}"),
        })).collect::<Vec<_>>(),
        "outlier_count": outliers.len(),
        "total_items": params.items.len(),
        "quartiles": q.map(|(q1, q2, q3)| json!({
            "q1": q1,
            "median": q2,
            "q3": q3,
        })),
        "iqr": iqr(&values),
        "primitives": ["κ", "∂", "N"],
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&output)
            .unwrap_or_else(|_| "outlier detection complete".to_string()),
    )]))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregate_fold_all() {
        let params = AggregateFoldParams {
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        };
        let result = aggregate_fold_all(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_tree_fold() {
        let params = AggregateTreeFoldParams {
            tree: json!({
                "id": "root",
                "value": 1.0,
                "children": [
                    {"id": "a", "value": 2.0},
                    {"id": "b", "value": 3.0}
                ]
            }),
            combine: "sum".to_string(),
        };
        let result = aggregate_tree_fold(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_rank() {
        let params = AggregateRankParams {
            items: vec![
                ("alpha".to_string(), 10.0),
                ("beta".to_string(), 30.0),
                ("gamma".to_string(), 20.0),
            ],
            top_n: 2,
        };
        let result = aggregate_rank(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_percentile() {
        let params = AggregatePercentileParams {
            values: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            percentile: 0.5,
        };
        let result = aggregate_percentile(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_aggregate_outliers() {
        let params = AggregateOutliersParams {
            items: vec![
                ("a".to_string(), 5.0),
                ("b".to_string(), 6.0),
                ("c".to_string(), 100.0),
            ],
        };
        let result = aggregate_outliers(params);
        assert!(result.is_ok());
    }
}
