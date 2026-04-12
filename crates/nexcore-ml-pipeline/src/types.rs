//! Core types for the ML pipeline.
//!
//! ## Primitive Foundation
//! - T1: State (ς) — model state, training metrics
//! - T1: Mapping (μ) — drug-event pair → feature vector → prediction
//! - T1: Sequence (σ) — pipeline stage ordering

use nexcore_dtree::prelude::Feature;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Feature vector
// ---------------------------------------------------------------------------

/// Names of the 12 PV features extracted from FAERS contingency data.
pub const FEATURE_NAMES: [&str; 12] = [
    "prr",
    "ror",
    "ic",
    "ebgm",
    "log_case_count",
    "reporter_hcp_ratio",
    "reporter_consumer_ratio",
    "serious_ratio",
    "death_ratio",
    "hospitalization_ratio",
    "time_to_onset_median_days",
    "reporting_velocity",
];

/// A drug-event pair with its extracted features and optional ground-truth label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sample {
    /// Drug name (generic).
    pub drug: String,
    /// MedDRA preferred term.
    pub event: String,
    /// 12-element PV feature vector.
    pub features: Vec<f64>,
    /// Ground-truth label: "signal" or "noise" (None if unlabeled).
    pub label: Option<String>,
}

impl Sample {
    /// Convert features to dtree Feature format.
    #[must_use]
    pub fn to_dtree_features(&self) -> Vec<Feature> {
        self.features
            .iter()
            .map(|&v| Feature::Continuous(v))
            .collect()
    }
}

// ---------------------------------------------------------------------------
// Dataset
// ---------------------------------------------------------------------------

/// A labeled dataset for training and evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    /// All samples.
    pub samples: Vec<Sample>,
    /// Feature names (length must match sample feature vectors).
    pub feature_names: Vec<String>,
}

impl Dataset {
    /// Create a new dataset from samples.
    #[must_use]
    pub fn new(samples: Vec<Sample>, feature_names: Vec<String>) -> Self {
        Self {
            samples,
            feature_names,
        }
    }

    /// Number of samples.
    #[must_use]
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// Whether the dataset is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Split into train/test by ratio (e.g., 0.8 = 80% train).
    /// Uses deterministic split (first N for train, rest for test).
    #[must_use]
    pub fn split(&self, train_ratio: f64) -> (Self, Self) {
        let n_train = (self.samples.len() as f64 * train_ratio).round() as usize;
        let train = Self {
            samples: self.samples[..n_train].to_vec(),
            feature_names: self.feature_names.clone(),
        };
        let test = Self {
            samples: self.samples[n_train..].to_vec(),
            feature_names: self.feature_names.clone(),
        };
        (train, test)
    }

    /// Extract dtree-compatible data and labels from labeled samples.
    /// Skips any sample without a label.
    #[must_use]
    pub fn to_dtree_data(&self) -> (Vec<Vec<Feature>>, Vec<String>) {
        let mut data = Vec::new();
        let mut labels = Vec::new();
        for sample in &self.samples {
            if let Some(ref label) = sample.label {
                data.push(sample.to_dtree_features());
                labels.push(label.clone());
            }
        }
        (data, labels)
    }

    /// K-fold cross-validation splits. Returns Vec of (train, test) pairs.
    #[must_use]
    pub fn k_fold(&self, k: usize) -> Vec<(Self, Self)> {
        let k = k.max(2).min(self.samples.len());
        let fold_size = self.samples.len() / k;
        let mut folds = Vec::with_capacity(k);

        for i in 0..k {
            let test_start = i * fold_size;
            let test_end = if i == k - 1 {
                self.samples.len()
            } else {
                test_start + fold_size
            };

            let mut train_samples = Vec::new();
            let mut test_samples = Vec::new();

            for (j, sample) in self.samples.iter().enumerate() {
                if j >= test_start && j < test_end {
                    test_samples.push(sample.clone());
                } else {
                    train_samples.push(sample.clone());
                }
            }

            folds.push((
                Self::new(train_samples, self.feature_names.clone()),
                Self::new(test_samples, self.feature_names.clone()),
            ));
        }

        folds
    }
}

// ---------------------------------------------------------------------------
// FAERS contingency table input
// ---------------------------------------------------------------------------

/// Raw FAERS 2x2 contingency table for a drug-event pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContingencyTable {
    /// Drug name.
    pub drug: String,
    /// Event (MedDRA PT).
    pub event: String,
    /// Cases with both drug and event (a).
    pub a: u64,
    /// Cases with drug but not event (b).
    pub b: u64,
    /// Cases with event but not drug (c).
    pub c: u64,
    /// Cases with neither drug nor event (d).
    pub d: u64,
}

/// Reporter breakdown for a drug-event pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporterBreakdown {
    /// Healthcare professional reports.
    pub hcp: u64,
    /// Consumer reports.
    pub consumer: u64,
    /// Other/unknown reporter type.
    pub other: u64,
}

/// Outcome breakdown for a drug-event pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeBreakdown {
    /// Total case count.
    pub total: u64,
    /// Serious outcome count.
    pub serious: u64,
    /// Death outcome count.
    pub death: u64,
    /// Hospitalization count.
    pub hospitalization: u64,
}

/// Temporal data for a drug-event pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalData {
    /// Median time-to-onset in days (None if unknown).
    pub median_tto_days: Option<f64>,
    /// Reporting velocity: cases per quarter (recent 4 quarters average).
    pub velocity: f64,
}

/// Complete raw input for feature extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPairData {
    /// 2x2 contingency table.
    pub contingency: ContingencyTable,
    /// Reporter breakdown.
    pub reporters: ReporterBreakdown,
    /// Outcome breakdown.
    pub outcomes: OutcomeBreakdown,
    /// Temporal data.
    pub temporal: TemporalData,
}

// ---------------------------------------------------------------------------
// Model configuration
// ---------------------------------------------------------------------------

/// Configuration for the random forest ensemble.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForestConfig {
    /// Number of trees in the forest.
    pub n_trees: usize,
    /// Maximum depth per tree (None = unlimited).
    pub max_depth: Option<usize>,
    /// Number of features to consider at each split (None = sqrt(n_features)).
    pub max_features: Option<usize>,
    /// Minimum samples to split a node.
    pub min_samples_split: usize,
    /// Minimum samples per leaf.
    pub min_samples_leaf: usize,
    /// Seed for reproducibility.
    pub seed: u64,
}

impl Default for ForestConfig {
    fn default() -> Self {
        Self {
            n_trees: 100,
            max_depth: Some(10),
            max_features: None, // auto = sqrt(n_features)
            min_samples_split: 5,
            min_samples_leaf: 2,
            seed: 42,
        }
    }
}

// ---------------------------------------------------------------------------
// Prediction output
// ---------------------------------------------------------------------------

/// Prediction for a single drug-event pair.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    /// Drug name.
    pub drug: String,
    /// Event name.
    pub event: String,
    /// Predicted class: "signal" or "noise".
    pub prediction: String,
    /// Probability of being a signal (0.0 - 1.0).
    pub signal_probability: f64,
    /// Feature importances for this prediction (top 5).
    pub top_features: Vec<(String, f64)>,
}

// ---------------------------------------------------------------------------
// Pipeline result
// ---------------------------------------------------------------------------

/// Full pipeline execution result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineResult {
    /// Training metrics.
    pub train_metrics: Metrics,
    /// Test metrics (from holdout or cross-validation).
    pub test_metrics: Metrics,
    /// Cross-validation metrics (None if CV not used).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cv_metrics: Option<CvMetrics>,
    /// Number of trees trained.
    pub n_trees: usize,
    /// Number of training samples.
    pub n_train_samples: usize,
    /// Number of test samples.
    pub n_test_samples: usize,
    /// Model version identifier.
    pub model_version: String,
    /// Predictions on test set.
    pub test_predictions: Vec<Prediction>,
}

/// Cross-validation metrics (mean and std across k folds).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CvMetrics {
    /// Number of folds.
    pub k: usize,
    /// Mean AUC across folds.
    pub mean_auc: f64,
    /// Std dev of AUC across folds.
    pub std_auc: f64,
    /// Mean F1 across folds.
    pub mean_f1: f64,
    /// Std dev of F1 across folds.
    pub std_f1: f64,
    /// Mean accuracy across folds.
    pub mean_accuracy: f64,
    /// Per-fold AUC values.
    pub fold_aucs: Vec<f64>,
}

/// Evaluation metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metrics {
    /// Area under the ROC curve.
    pub auc: f64,
    /// Precision (positive predictive value).
    pub precision: f64,
    /// Recall (sensitivity / true positive rate).
    pub recall: f64,
    /// F1 score (harmonic mean of precision and recall).
    pub f1: f64,
    /// Overall accuracy.
    pub accuracy: f64,
    /// Confusion matrix: [[TN, FP], [FN, TP]].
    pub confusion_matrix: [[u64; 2]; 2],
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_split_ratio() {
        let samples: Vec<Sample> = (0..10)
            .map(|i| Sample {
                drug: format!("drug_{i}"),
                event: "headache".into(),
                features: vec![0.0; 12],
                label: Some("noise".into()),
            })
            .collect();
        let ds = Dataset::new(
            samples,
            FEATURE_NAMES.iter().map(|s| s.to_string()).collect(),
        );
        let (train, test) = ds.split(0.8);
        assert_eq!(train.len(), 8);
        assert_eq!(test.len(), 2);
    }

    #[test]
    fn dataset_k_fold() {
        let samples: Vec<Sample> = (0..20)
            .map(|i| Sample {
                drug: format!("drug_{i}"),
                event: "nausea".into(),
                features: vec![0.0; 12],
                label: Some("signal".into()),
            })
            .collect();
        let ds = Dataset::new(
            samples,
            FEATURE_NAMES.iter().map(|s| s.to_string()).collect(),
        );
        let folds = ds.k_fold(5);
        assert_eq!(folds.len(), 5);
        for (train, test) in &folds {
            assert_eq!(train.len() + test.len(), 20);
        }
    }

    #[test]
    fn sample_to_dtree_features() {
        let s = Sample {
            drug: "metformin".into(),
            event: "lactic_acidosis".into(),
            features: vec![3.5, 2.1, 1.8, 4.2, 2.3, 0.7, 0.2, 0.6, 0.1, 0.3, 14.0, 5.2],
            label: Some("signal".into()),
        };
        let feats = s.to_dtree_features();
        assert_eq!(feats.len(), 12);
        assert_eq!(feats[0], Feature::Continuous(3.5));
    }
}
