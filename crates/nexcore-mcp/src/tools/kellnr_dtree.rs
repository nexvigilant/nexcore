//! Kellnr Decision Tree computation tools (3).
//! Consolidated from kellnr-mcp/src/dtree.rs.

use crate::params::kellnr::{
    KellnrDtreeFeatureImportanceParams, KellnrDtreePruneParams, KellnrDtreeToRulesParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::HashMap;

fn json_result(value: serde_json::Value) -> CallToolResult {
    CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into()),
    )])
}

fn gini_impurity(labels: &[u32]) -> f64 {
    if labels.is_empty() {
        return 0.0;
    }
    let n = labels.len() as f64;
    let mut counts = HashMap::new();
    for &l in labels {
        *counts.entry(l).or_insert(0u64) += 1;
    }
    1.0 - counts
        .values()
        .map(|&c| (c as f64 / n).powi(2))
        .sum::<f64>()
}

/// Feature importance via Gini impurity decrease.
pub fn compute_dtree_feature_importance(
    params: KellnrDtreeFeatureImportanceParams,
) -> Result<CallToolResult, McpError> {
    let features = &params.features;
    let labels = &params.labels;
    if features.is_empty() || labels.is_empty() || features.len() != labels.len() {
        return Ok(json_result(
            json!({"success": false, "error": "features and labels must have matching non-zero length"}),
        ));
    }
    let n_features = features[0].len();
    let n_samples = labels.len() as f64;
    let base_gini = gini_impurity(labels);

    let mut importances = vec![0.0f64; n_features];
    for f_idx in 0..n_features {
        let mut vals: Vec<(f64, u32)> = features
            .iter()
            .map(|row| row[f_idx])
            .zip(labels.iter().copied())
            .collect();
        vals.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

        let mut best_gain = 0.0f64;
        for split_at in 1..vals.len() {
            if (vals[split_at].0 - vals[split_at - 1].0).abs() < 1e-15 {
                continue;
            }
            let left: Vec<u32> = vals[..split_at].iter().map(|v| v.1).collect();
            let right: Vec<u32> = vals[split_at..].iter().map(|v| v.1).collect();
            let w_left = left.len() as f64 / n_samples;
            let w_right = right.len() as f64 / n_samples;
            let gain = base_gini - w_left * gini_impurity(&left) - w_right * gini_impurity(&right);
            if gain > best_gain {
                best_gain = gain;
            }
        }
        importances[f_idx] = best_gain;
    }

    let total: f64 = importances.iter().sum();
    let normalized: Vec<f64> = if total > 0.0 {
        importances.iter().map(|&i| i / total).collect()
    } else {
        importances.clone()
    };

    Ok(json_result(json!({
        "success": true,
        "importances": importances,
        "normalized": normalized,
        "base_gini": base_gini,
        "n_features": n_features,
        "n_samples": labels.len()
    })))
}

/// Cost-complexity pruning analysis.
pub fn compute_dtree_prune(params: KellnrDtreePruneParams) -> Result<CallToolResult, McpError> {
    let alpha = params.alpha.unwrap_or(0.01);
    let cost_complexity = params.training_error + alpha * params.tree_size as f64;
    let alpha_max = if params.tree_size > 1 {
        params.training_error / (params.tree_size as f64 - 1.0)
    } else {
        0.0
    };
    Ok(json_result(json!({
        "success": true,
        "tree_size": params.tree_size,
        "training_error": params.training_error,
        "alpha": alpha,
        "cost_complexity": cost_complexity,
        "alpha_max": alpha_max,
        "should_prune": alpha > alpha_max,
        "effective_cost_per_leaf": if params.tree_size > 0 { cost_complexity / params.tree_size as f64 } else { 0.0 }
    })))
}

/// Convert splits to interpretable rules.
pub fn compute_dtree_to_rules(
    params: KellnrDtreeToRulesParams,
) -> Result<CallToolResult, McpError> {
    let rules: Vec<serde_json::Value> = params
        .splits
        .iter()
        .enumerate()
        .map(|(i, s)| {
            json!({
                "rule_id": i + 1,
                "condition": format!("feature[{}] <= {:.4}", s.feature_index, s.threshold),
                "feature_index": s.feature_index,
                "threshold": s.threshold,
                "label": s.label,
            })
        })
        .collect();
    let rule_count = rules.len();
    Ok(json_result(json!({
        "success": true,
        "rules": rules,
        "rule_count": rule_count
    })))
}
