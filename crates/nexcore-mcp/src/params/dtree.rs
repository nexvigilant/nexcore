//! Decision Tree Parameters (CART Engine)
//! Tier: T3 (Domain-specific MCP tool parameters)
//!
//! Training, prediction, importance, pruning, and export for Decision Trees.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for training a decision tree.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DtreeTrainParams {
    /// Feature matrix: each row is a sample, each column a feature (numeric).
    pub features: Vec<Vec<f64>>,
    /// Class labels (one per row).
    pub labels: Vec<String>,
    /// Splitting criterion: "gini", "entropy", "gain_ratio", "mse"
    #[serde(default)]
    pub criterion: Option<String>,
    /// Maximum tree depth
    #[serde(default)]
    pub max_depth: Option<usize>,
    /// Minimum samples to attempt a split
    #[serde(default)]
    pub min_samples_split: Option<usize>,
    /// Minimum samples per leaf
    #[serde(default)]
    pub min_samples_leaf: Option<usize>,
    /// Feature names for explainability
    #[serde(default)]
    pub feature_names: Option<Vec<String>>,
}

/// Parameters for predicting with a trained decision tree.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DtreePredictParams {
    /// Tree ID returned from dtree_train
    pub tree_id: String,
    /// Feature values for prediction
    pub features: Vec<f64>,
}

/// Parameters for getting feature importance from a trained tree.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DtreeImportanceParams {
    /// Tree ID returned from dtree_train
    pub tree_id: String,
}

/// Parameters for pruning a trained decision tree.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DtreePruneParams {
    /// Tree ID returned from dtree_train
    pub tree_id: String,
    /// Cost-complexity pruning alpha parameter
    pub alpha: f64,
}

/// Parameters for exporting a trained decision tree.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DtreeExportParams {
    /// Tree ID returned from dtree_train
    pub tree_id: String,
    /// Export format: "json", "rules", "summary"
    #[serde(default)]
    pub format: Option<String>,
}

/// Parameters for getting info about a trained decision tree.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct DtreeInfoParams {
    /// Tree ID returned from dtree_train
    pub tree_id: String,
}
