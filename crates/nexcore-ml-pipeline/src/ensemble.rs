//! Random forest ensemble built on nexcore-dtree.
//!
//! Implements bagging: bootstrap sampling with feature subsampling per tree.
//! Aggregates via majority vote (classification) or mean (regression).
//!
//! ## Primitive Foundation
//! - T1: Recursion (ρ) — each tree is recursive; forest is ensemble recursion
//! - T1: Quantity (N) — bootstrap sampling, vote counting
//! - T1: State (ς) — accumulated trees, OOB predictions

use crate::types::{Dataset, ForestConfig, Metrics};
use nexcore_dtree::prelude::*;
use serde::{Deserialize, Serialize};

/// Errors during ensemble training.
#[derive(Debug, nexcore_error::Error)]
pub enum EnsembleError {
    /// Training data is empty or invalid.
    #[error("invalid training data: {0}")]
    InvalidData(String),
    /// Individual tree training failed.
    #[error("tree training failed: {0}")]
    TreeFailed(String),
}

/// A trained random forest model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RandomForest {
    /// Trained decision trees.
    trees: Vec<DecisionTree>,
    /// Configuration used.
    pub config: ForestConfig,
    /// Feature names for interpretability.
    pub feature_names: Vec<String>,
    /// Number of classes seen during training.
    pub n_classes: usize,
    /// Class labels in sorted order.
    pub class_labels: Vec<String>,
}

impl RandomForest {
    /// Train a random forest on the given dataset.
    ///
    /// # Errors
    /// Returns `EnsembleError` if data is empty or tree training fails.
    pub fn train(dataset: &Dataset, config: ForestConfig) -> Result<Self, EnsembleError> {
        let (data, labels) = dataset.to_dtree_data();
        if data.is_empty() {
            return Err(EnsembleError::InvalidData("no labeled samples".into()));
        }

        let n_features = data[0].len();
        let _ = n_features; // used for validation; max_features disabled (see tree loop)

        // Collect class labels
        let mut class_labels: Vec<String> = labels.clone();
        class_labels.sort();
        class_labels.dedup();
        let n_classes = class_labels.len();

        let n_samples = data.len();
        let mut trees = Vec::with_capacity(config.n_trees);

        for tree_idx in 0..config.n_trees {
            // Bootstrap sample (deterministic from seed + tree_idx)
            let bootstrap_indices = bootstrap_indices(n_samples, config.seed, tree_idx as u64);

            let boot_data: Vec<Vec<Feature>> =
                bootstrap_indices.iter().map(|&i| data[i].clone()).collect();
            let boot_labels: Vec<String> = bootstrap_indices
                .iter()
                .map(|&i| labels[i].clone())
                .collect();

            // With only 12 features, use all features at each split.
            // Bootstrap sampling provides sufficient tree diversity.
            // max_features subsampling in nexcore-dtree evaluates features
            // 0..max_f sequentially (not randomly), so capping would always
            // exclude later features like velocity and TTO.
            let tree_config = TreeConfig {
                max_depth: config.max_depth,
                max_features: None, // all features — fixes feature selection bias
                min_samples_split: config.min_samples_split,
                min_samples_leaf: config.min_samples_leaf,
                criterion: CriterionType::Gini,
                ..TreeConfig::default()
            };

            let tree = fit(&boot_data, &boot_labels, tree_config)
                .map_err(|e| EnsembleError::TreeFailed(format!("tree {tree_idx}: {e}")))?;

            trees.push(tree);
        }

        Ok(Self {
            trees,
            config,
            feature_names: dataset.feature_names.clone(),
            n_classes,
            class_labels,
        })
    }

    /// Predict class label for a single feature vector.
    /// Returns (predicted_class, signal_probability).
    #[must_use]
    pub fn predict_one(&self, features: &[Feature]) -> (String, f64) {
        let mut votes: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for tree in &self.trees {
            if let Ok(result) = predict(tree, features) {
                *votes.entry(result.prediction.clone()).or_insert(0) += 1;
            }
        }

        let total_votes = self.trees.len() as f64;
        let signal_votes = votes.get("signal").copied().unwrap_or(0) as f64;
        let signal_prob = signal_votes / total_votes;

        let predicted = votes
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(class, _)| class)
            .unwrap_or_else(|| "noise".into());

        (predicted, signal_prob)
    }

    /// Predict on a batch of samples, returning (predictions, signal_probabilities).
    #[must_use]
    pub fn predict_batch(&self, data: &[Vec<Feature>]) -> Vec<(String, f64)> {
        data.iter()
            .map(|features| self.predict_one(features))
            .collect()
    }

    /// Compute aggregate feature importance across all trees.
    #[must_use]
    pub fn feature_importance(&self) -> Vec<(String, f64)> {
        let n_features = self.feature_names.len();
        let mut importances = vec![0.0_f64; n_features];

        for tree in &self.trees {
            let tree_imp = feature_importance(tree);
            for imp in tree_imp {
                if imp.index < n_features {
                    importances[imp.index] += imp.importance;
                }
            }
        }

        // Normalize
        let total: f64 = importances.iter().sum();
        if total > f64::EPSILON {
            for imp in &mut importances {
                *imp /= total;
            }
        }

        self.feature_names
            .iter()
            .zip(importances)
            .map(|(name, imp)| (name.clone(), imp))
            .collect()
    }

    /// Number of trees in the forest.
    #[must_use]
    pub fn n_trees(&self) -> usize {
        self.trees.len()
    }

    /// Evaluate the model on a dataset, returning metrics.
    #[must_use]
    pub fn evaluate(&self, dataset: &Dataset) -> Metrics {
        let (data, labels) = dataset.to_dtree_data();
        let predictions = self.predict_batch(&data);

        crate::evaluate::compute_metrics(
            &labels,
            &predictions
                .iter()
                .map(|(p, _)| p.clone())
                .collect::<Vec<_>>(),
            &predictions
                .iter()
                .map(|(_, prob)| *prob)
                .collect::<Vec<_>>(),
        )
    }
}

// ---------------------------------------------------------------------------
// Bootstrap sampling (deterministic PRNG)
// ---------------------------------------------------------------------------

/// Generate bootstrap sample indices using a simple LCG PRNG.
fn bootstrap_indices(n: usize, seed: u64, tree_idx: u64) -> Vec<usize> {
    let mut state = seed
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(tree_idx);
    let mut indices = Vec::with_capacity(n);

    for _ in 0..n {
        // LCG step
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let idx = (state >> 33) as usize % n;
        indices.push(idx);
    }

    indices
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Sample;

    fn make_training_dataset() -> Dataset {
        // Create a simple linearly-separable PV dataset
        let mut samples = Vec::new();

        // "signal" samples: high PRR, high ROR
        for i in 0..30 {
            let noise = (i as f64) * 0.01;
            samples.push(Sample {
                drug: format!("drug_{i}"),
                event: "serious_event".into(),
                features: vec![
                    5.0 + noise, // prr (high)
                    4.0 + noise, // ror (high)
                    2.5 + noise, // ic (high)
                    3.0 + noise, // ebgm (high)
                    4.0,         // log_cases
                    0.7,         // hcp_ratio
                    0.2,         // consumer_ratio
                    0.8,         // serious_ratio
                    0.15,        // death_ratio
                    0.5,         // hosp_ratio
                    7.0,         // tto
                    8.0 + noise, // velocity
                ],
                label: Some("signal".into()),
            });
        }

        // "noise" samples: low PRR, low ROR
        for i in 0..30 {
            let noise = (i as f64) * 0.01;
            samples.push(Sample {
                drug: format!("drug_noise_{i}"),
                event: "mild_event".into(),
                features: vec![
                    0.8 + noise,  // prr (low)
                    0.9 + noise,  // ror (low)
                    -0.5 + noise, // ic (low/negative)
                    0.7 + noise,  // ebgm (low)
                    2.0,          // log_cases
                    0.3,          // hcp_ratio
                    0.6,          // consumer_ratio
                    0.2,          // serious_ratio
                    0.01,         // death_ratio
                    0.1,          // hosp_ratio
                    30.0,         // tto
                    1.0 + noise,  // velocity
                ],
                label: Some("noise".into()),
            });
        }

        let feature_names = crate::types::FEATURE_NAMES
            .iter()
            .map(|s| s.to_string())
            .collect();
        Dataset::new(samples, feature_names)
    }

    #[test]
    fn train_random_forest() {
        let dataset = make_training_dataset();
        let config = ForestConfig {
            n_trees: 10,
            max_depth: Some(5),
            seed: 42,
            ..ForestConfig::default()
        };
        let forest = RandomForest::train(&dataset, config);
        assert!(forest.is_ok());
        let forest = forest.unwrap_or_else(|_| unreachable!());
        assert_eq!(forest.n_trees(), 10);
    }

    #[test]
    fn predict_signal() {
        let dataset = make_training_dataset();
        let config = ForestConfig {
            n_trees: 20,
            max_depth: Some(5),
            seed: 42,
            ..ForestConfig::default()
        };
        let forest = RandomForest::train(&dataset, config).unwrap_or_else(|_| unreachable!());

        // High-signal features
        let signal_features: Vec<Feature> =
            vec![5.0, 4.0, 2.5, 3.0, 4.0, 0.7, 0.2, 0.8, 0.15, 0.5, 7.0, 8.0]
                .into_iter()
                .map(Feature::Continuous)
                .collect();

        let (pred, prob) = forest.predict_one(&signal_features);
        assert_eq!(pred, "signal", "Expected signal, got {pred} (prob={prob})");
        assert!(prob > 0.5, "Signal probability should be > 0.5: {prob}");
    }

    #[test]
    fn feature_importance_sums_to_one() {
        let dataset = make_training_dataset();
        let config = ForestConfig {
            n_trees: 10,
            seed: 42,
            ..ForestConfig::default()
        };
        let forest = RandomForest::train(&dataset, config).unwrap_or_else(|_| unreachable!());
        let importances = forest.feature_importance();
        let total: f64 = importances.iter().map(|(_, v)| v).sum();
        assert!(
            (total - 1.0).abs() < 0.01,
            "Importances should sum to ~1.0: {total}"
        );
    }

    #[test]
    fn bootstrap_deterministic() {
        let a = bootstrap_indices(100, 42, 0);
        let b = bootstrap_indices(100, 42, 0);
        assert_eq!(a, b, "Same seed should produce same indices");

        let c = bootstrap_indices(100, 42, 1);
        assert_ne!(a, c, "Different tree_idx should produce different indices");
    }
}
