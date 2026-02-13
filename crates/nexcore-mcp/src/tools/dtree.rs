//! Decision tree MCP tools
//!
//! Exposes the nexcore-dtree CART engine as MCP tools:
//! - Train classification/regression trees
//! - Predict with confidence and explainable paths
//! - Feature importance
//! - Cost-complexity pruning
//! - Export to JSON/rules/summary

use crate::params::{
    DtreeExportParams, DtreeImportanceParams, DtreeInfoParams, DtreePredictParams,
    DtreePruneParams, DtreeTrainParams,
};
use nexcore_dtree::prelude::*;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

// ---------------------------------------------------------------------------
// In-memory tree store (session-scoped)
// ---------------------------------------------------------------------------

static TREE_STORE: OnceLock<Mutex<HashMap<String, DecisionTree>>> = OnceLock::new();

fn store() -> &'static Mutex<HashMap<String, DecisionTree>> {
    TREE_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    format!("dtree_{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

fn get_tree(
    tree_id: &str,
) -> Result<std::sync::MutexGuard<'static, HashMap<String, DecisionTree>>, McpError> {
    store()
        .lock()
        .map_err(|_| McpError::internal_error("Store lock failed", None))
}

// ---------------------------------------------------------------------------
// Helpers for dtree_train (keeps function under 50 lines)
// ---------------------------------------------------------------------------

fn parse_criterion(input: Option<&str>) -> Result<CriterionType, CallToolResult> {
    match input {
        Some("entropy") => Ok(CriterionType::Entropy),
        Some("gain_ratio") => Ok(CriterionType::GainRatio),
        Some("mse") => Ok(CriterionType::Mse),
        Some("gini") | None => Ok(CriterionType::Gini),
        Some(other) => Err(CallToolResult::error(vec![Content::text(format!(
            "Unknown criterion: {other}. Use: gini, entropy, gain_ratio, mse"
        ))])),
    }
}

fn build_config(params: &DtreeTrainParams, criterion: CriterionType) -> TreeConfig {
    TreeConfig {
        max_depth: params.max_depth,
        min_samples_split: params.min_samples_split.unwrap_or(2),
        min_samples_leaf: params.min_samples_leaf.unwrap_or(1),
        criterion,
        ..TreeConfig::default()
    }
}

fn features_to_matrix(features: &[Vec<f64>]) -> Vec<Vec<Feature>> {
    features
        .iter()
        .map(|row| row.iter().map(|&v| Feature::Continuous(v)).collect())
        .collect()
}

fn train_tree(
    data: &[Vec<Feature>],
    labels: &[String],
    config: TreeConfig,
) -> Result<DecisionTree, McpError> {
    let is_regression = config.criterion == CriterionType::Mse;

    if is_regression {
        let targets: Result<Vec<f64>, _> = labels.iter().map(|s| s.parse::<f64>()).collect();
        let targets = targets.map_err(|e| {
            McpError::invalid_params(format!("MSE requires numeric labels: {e}"), None)
        })?;
        fit_regression(data, &targets, config)
            .map_err(|e| McpError::invalid_params(format!("Training failed: {e}"), None))
    } else {
        fit(data, labels, config)
            .map_err(|e| McpError::invalid_params(format!("Training failed: {e}"), None))
    }
}

fn train_response(tree_id: &str, tree: &DecisionTree, is_regression: bool) -> serde_json::Value {
    let stats = tree.stats();
    json!({
        "tree_id": tree_id,
        "depth": stats.as_ref().map(|s| s.depth),
        "n_leaves": stats.as_ref().map(|s| s.n_leaves),
        "n_splits": stats.as_ref().map(|s| s.n_splits),
        "n_features": stats.as_ref().map(|s| s.n_features),
        "n_classes": stats.as_ref().map(|s| s.n_classes),
        "n_samples": stats.as_ref().map(|s| s.n_samples),
        "criterion": format!("{:?}", stats.as_ref().map(|s| s.criterion)),
        "is_regression": is_regression,
    })
}

// ---------------------------------------------------------------------------
// Tool: dtree_train
// ---------------------------------------------------------------------------

/// Train a decision tree on the provided data.
pub fn dtree_train(params: DtreeTrainParams) -> Result<CallToolResult, McpError> {
    if params.features.is_empty() || params.features.len() != params.labels.len() {
        return Ok(CallToolResult::error(vec![Content::text(
            "Feature rows must be non-empty and match label count",
        )]));
    }

    let criterion = match parse_criterion(params.criterion.as_deref()) {
        Ok(c) => c,
        Err(err_result) => return Ok(err_result),
    };

    let config = build_config(&params, criterion);
    let data = features_to_matrix(&params.features);
    let is_regression = criterion == CriterionType::Mse;

    let mut tree = train_tree(&data, &params.labels, config)?;
    if let Some(names) = params.feature_names {
        tree.set_feature_names(names);
    }

    let tree_id = next_id();
    let response = train_response(&tree_id, &tree, is_regression);

    if let Ok(mut s) = store().lock() {
        s.insert(tree_id, tree);
    }

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: dtree_predict
// ---------------------------------------------------------------------------

/// Predict class/value for a single sample.
pub fn dtree_predict(params: DtreePredictParams) -> Result<CallToolResult, McpError> {
    let s = get_tree(&params.tree_id)?;
    let tree = s.get(&params.tree_id).ok_or_else(|| {
        McpError::invalid_params(format!("Tree not found: {}", params.tree_id), None)
    })?;

    let features: Vec<Feature> = params
        .features
        .iter()
        .map(|&v| Feature::Continuous(v))
        .collect();

    let result = predict(tree, &features)
        .map_err(|e| McpError::invalid_params(format!("Prediction failed: {e}"), None))?;

    let path: Vec<_> = result.path.iter().map(|step| format!("{step}")).collect();

    let response = json!({
        "prediction": result.prediction,
        "confidence": result.confidence.value(),
        "class_distribution": result.class_distribution,
        "leaf_samples": result.leaf_samples,
        "depth": result.depth,
        "path": path,
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: dtree_importance
// ---------------------------------------------------------------------------

/// Get feature importance scores.
pub fn dtree_importance(params: DtreeImportanceParams) -> Result<CallToolResult, McpError> {
    let s = get_tree(&params.tree_id)?;
    let tree = s.get(&params.tree_id).ok_or_else(|| {
        McpError::invalid_params(format!("Tree not found: {}", params.tree_id), None)
    })?;

    let imp = feature_importance(tree);
    let features: Vec<_> = imp
        .iter()
        .map(|fi| json!({"index": fi.index, "name": fi.name, "importance": fi.importance}))
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({"tree_id": params.tree_id, "features": features}).to_string(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: dtree_prune
// ---------------------------------------------------------------------------

/// Prune a tree using cost-complexity pruning.
pub fn dtree_prune(params: DtreePruneParams) -> Result<CallToolResult, McpError> {
    let mut s = store()
        .lock()
        .map_err(|_| McpError::internal_error("Store lock failed", None))?;
    let tree = s.get_mut(&params.tree_id).ok_or_else(|| {
        McpError::invalid_params(format!("Tree not found: {}", params.tree_id), None)
    })?;

    let before = tree.stats();
    cost_complexity_prune(tree, params.alpha);
    let after = tree.stats();

    let response = json!({
        "tree_id": params.tree_id,
        "alpha": params.alpha,
        "before": {"depth": before.as_ref().map(|s| s.depth), "n_leaves": before.as_ref().map(|s| s.n_leaves)},
        "after": {"depth": after.as_ref().map(|s| s.depth), "n_leaves": after.as_ref().map(|s| s.n_leaves)},
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: dtree_export
// ---------------------------------------------------------------------------

/// Export a tree in the requested format.
pub fn dtree_export(params: DtreeExportParams) -> Result<CallToolResult, McpError> {
    let s = get_tree(&params.tree_id)?;
    let tree = s.get(&params.tree_id).ok_or_else(|| {
        McpError::invalid_params(format!("Tree not found: {}", params.tree_id), None)
    })?;

    let format = params.format.as_deref().unwrap_or("json");
    let output = match format {
        "json" => nexcore_dtree::serialize::to_json(tree),
        "rules" => nexcore_dtree::serialize::to_rules(tree),
        "summary" => nexcore_dtree::serialize::to_summary(tree),
        other => {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Unknown format: {other}. Use: json, rules, summary"
            ))]));
        }
    };

    let text = output.map_err(|e| McpError::internal_error(format!("Export failed: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(text)]))
}

// ---------------------------------------------------------------------------
// Tool: dtree_info
// ---------------------------------------------------------------------------

/// Get tree statistics and metadata.
pub fn dtree_info(params: DtreeInfoParams) -> Result<CallToolResult, McpError> {
    let s = get_tree(&params.tree_id)?;
    let tree = s.get(&params.tree_id).ok_or_else(|| {
        McpError::invalid_params(format!("Tree not found: {}", params.tree_id), None)
    })?;

    let stats = tree
        .stats()
        .ok_or_else(|| McpError::internal_error("Tree not fitted", None))?;

    let response = json!({
        "tree_id": params.tree_id,
        "depth": stats.depth,
        "n_leaves": stats.n_leaves,
        "n_splits": stats.n_splits,
        "n_nodes": stats.n_nodes,
        "n_features": stats.n_features,
        "n_classes": stats.n_classes,
        "n_samples": stats.n_samples,
        "criterion": format!("{:?}", stats.criterion),
        "feature_names": tree.feature_names(),
        "is_fitted": tree.is_fitted(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}
