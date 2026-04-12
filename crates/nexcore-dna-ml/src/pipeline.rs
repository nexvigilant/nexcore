//! Combined DNA-ML pipeline.
//!
//! Orchestrates the full flow: FAERS data → PV features → DNA encoding →
//! similarity features → augmented feature vector → random forest → prediction.

use crate::encode::{compute_bounds, encode_features};
use crate::similarity::mean_similarity_features;
use nexcore_dna::types::Strand;
use nexcore_ml_pipeline::evaluate::{compute_auc, compute_metrics};
use nexcore_ml_pipeline::feature::extract_features;
use nexcore_ml_pipeline::prelude::{Dataset, FEATURE_NAMES, ForestConfig, Metrics, RandomForest};
use nexcore_ml_pipeline::types::RawPairData;

/// Configuration for the DNA-ML pipeline.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct DnaMlConfig {
    /// Number of trees in the random forest.
    pub n_trees: usize,
    /// Maximum tree depth.
    pub max_depth: usize,
    /// Minimum samples to split a node.
    pub min_samples_split: usize,
    /// Number of features to consider per split (None = sqrt).
    pub max_features: Option<usize>,
    /// Whether to include DNA similarity features.
    pub use_dna_features: bool,
}

impl Default for DnaMlConfig {
    fn default() -> Self {
        Self {
            n_trees: 50,
            max_depth: 8,
            min_samples_split: 5,
            max_features: None,
            use_dna_features: true,
        }
    }
}

/// Result of the DNA-ML pipeline run.
#[derive(Debug, serde::Serialize)]
pub struct DnaMlResult {
    /// Evaluation metrics.
    pub metrics: Metrics,
    /// Number of original PV features.
    pub pv_feature_count: usize,
    /// Number of DNA similarity features added.
    pub dna_feature_count: usize,
    /// Total augmented feature dimension.
    pub total_features: usize,
    /// Number of training samples.
    pub n_samples: usize,
    /// Feature names (PV + DNA).
    pub feature_names: Vec<String>,
}

/// Augment PV feature vectors with DNA-encoded similarity features.
///
/// For each sample:
/// 1. Encode features as DNA strand
/// 2. Compute mean similarity to all other strands
/// 3. Append 5 similarity features to the original 12
///
/// Returns augmented feature vectors (17 dimensions).
pub fn augment_with_dna(raw_features: &[Vec<f64>]) -> Vec<Vec<f64>> {
    if raw_features.is_empty() {
        return vec![];
    }

    let (mins, maxs) = compute_bounds(raw_features);

    // Encode all samples as DNA strands
    let strands: Vec<Strand> = raw_features
        .iter()
        .map(|f| encode_features(f, &mins, &maxs))
        .collect();

    // For each sample, compute similarity to all OTHER samples
    raw_features
        .iter()
        .enumerate()
        .map(|(i, original)| {
            let references: Vec<Strand> = strands
                .iter()
                .enumerate()
                .filter(|(j, _)| *j != i)
                .map(|(_, s)| s.clone())
                .collect();

            let dna_feats = mean_similarity_features(&strands[i], &references);

            let mut augmented = original.clone();
            augmented.extend(dna_feats);
            augmented
        })
        .collect()
}

/// DNA feature names appended to the PV feature vector.
const DNA_FEATURE_NAMES: [&str; 5] = [
    "dna_mean_hamming",
    "dna_gc_content",
    "dna_gc_divergence",
    "dna_lcs_ratio",
    "dna_strand_length",
];

/// Run the full DNA-ML pipeline.
///
/// Takes raw FAERS pair data with labels and produces a trained model
/// with evaluation metrics.
pub fn run(
    raw_data: &[RawPairData],
    labels: &[String],
    config: &DnaMlConfig,
) -> Result<DnaMlResult, nexcore_error::NexError> {
    if raw_data.len() != labels.len() {
        return Err(nexcore_error::NexError::new(
            "data and labels must have equal length",
        ));
    }
    if raw_data.is_empty() {
        return Err(nexcore_error::NexError::new("empty dataset"));
    }

    // Stage 1: Extract PV features (12-dim)
    let mut pv_features: Vec<Vec<f64>> = Vec::with_capacity(raw_data.len());
    for raw in raw_data {
        let sample = extract_features(raw)
            .map_err(|e| nexcore_error::NexError::new(format!("feature extraction: {e}")))?;
        pv_features.push(sample.features);
    }

    let pv_dim = FEATURE_NAMES.len();

    // Stage 2: DNA augmentation (12 → 17 features)
    let augmented = if config.use_dna_features {
        augment_with_dna(&pv_features)
    } else {
        pv_features.clone()
    };

    let dna_dim = if config.use_dna_features { 5 } else { 0 };
    let total_dim = pv_dim + dna_dim;

    // Build feature names
    let mut names: Vec<String> = FEATURE_NAMES.iter().map(|s| s.to_string()).collect();
    if config.use_dna_features {
        names.extend(DNA_FEATURE_NAMES.iter().map(|s| s.to_string()));
    }

    // Stage 3: Build dataset
    let samples: Vec<nexcore_ml_pipeline::types::Sample> = augmented
        .into_iter()
        .zip(raw_data.iter())
        .zip(labels.iter())
        .map(
            |((features, raw), label): ((Vec<f64>, &RawPairData), &String)| {
                nexcore_ml_pipeline::types::Sample {
                    drug: raw.contingency.drug.clone(),
                    event: raw.contingency.event.clone(),
                    features,
                    label: Some(label.clone()),
                }
            },
        )
        .collect();

    let dataset = Dataset::new(samples, names.clone());

    // Stage 4: Leave-one-out cross-validation
    //
    // For each sample, train on all others and predict the held-out sample.
    // This gives unbiased generalization estimates on small datasets.
    let forest_config = ForestConfig {
        n_trees: config.n_trees,
        max_depth: Some(config.max_depth),
        max_features: config.max_features,
        min_samples_split: config.min_samples_split,
        min_samples_leaf: 1,
        seed: 42,
    };

    let true_labels: Vec<String> = labels.to_vec();
    let mut predicted_labels = Vec::with_capacity(true_labels.len());
    let mut probabilities = Vec::with_capacity(true_labels.len());

    for i in 0..dataset.samples.len() {
        let train_samples: Vec<nexcore_ml_pipeline::types::Sample> = dataset
            .samples
            .iter()
            .enumerate()
            .filter(|(j, _)| *j != i)
            .map(|(_, s)| s.clone())
            .collect();
        let train_ds = Dataset::new(train_samples, names.clone());

        let forest = RandomForest::train(&train_ds, forest_config.clone())
            .map_err(|e| nexcore_error::NexError::new(format!("training fold {i}: {e}")))?;

        let test_features = dataset.samples[i].to_dtree_features();
        let (prediction, probability) = forest.predict_one(&test_features);
        predicted_labels.push(prediction);
        probabilities.push(probability);
    }

    let metrics = compute_metrics(&true_labels, &predicted_labels, &probabilities);

    Ok(DnaMlResult {
        metrics,
        pv_feature_count: pv_dim,
        dna_feature_count: dna_dim,
        total_features: total_dim,
        n_samples: dataset.len(),
        feature_names: names,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_ml_pipeline::types::*;

    fn make_test_data() -> (Vec<RawPairData>, Vec<String>) {
        let signal_pair = RawPairData {
            contingency: ContingencyTable {
                drug: "TestDrug".into(),
                event: "TestEvent".into(),
                a: 50,
                b: 1000,
                c: 10,
                d: 50000,
            },
            reporters: ReporterBreakdown {
                hcp: 30,
                consumer: 20,
                other: 0,
            },
            outcomes: OutcomeBreakdown {
                total: 50,
                serious: 25,
                death: 5,
                hospitalization: 15,
            },
            temporal: TemporalData {
                median_tto_days: Some(14.0),
                velocity: 2.0,
            },
        };

        let noise_pair = RawPairData {
            contingency: ContingencyTable {
                drug: "PlaceboDrug".into(),
                event: "CommonEvent".into(),
                a: 5,
                b: 2000,
                c: 500,
                d: 50000,
            },
            reporters: ReporterBreakdown {
                hcp: 2,
                consumer: 3,
                other: 0,
            },
            outcomes: OutcomeBreakdown {
                total: 5,
                serious: 1,
                death: 0,
                hospitalization: 1,
            },
            temporal: TemporalData {
                median_tto_days: None,
                velocity: 0.5,
            },
        };

        let data = vec![
            signal_pair.clone(),
            signal_pair.clone(),
            signal_pair,
            noise_pair.clone(),
            noise_pair.clone(),
            noise_pair,
        ];
        let labels = vec![
            "signal".into(),
            "signal".into(),
            "signal".into(),
            "noise".into(),
            "noise".into(),
            "noise".into(),
        ];
        (data, labels)
    }

    #[test]
    fn pipeline_runs_with_dna_features() {
        let (data, labels) = make_test_data();
        let config = DnaMlConfig {
            n_trees: 10,
            max_depth: 4,
            use_dna_features: true,
            ..Default::default()
        };
        let result = run(&data, &labels, &config).unwrap();
        assert_eq!(result.total_features, 17);
        assert_eq!(result.dna_feature_count, 5);
        assert!(result.metrics.auc >= 0.0);
    }

    #[test]
    fn pipeline_runs_without_dna_features() {
        let (data, labels) = make_test_data();
        let config = DnaMlConfig {
            n_trees: 10,
            max_depth: 4,
            use_dna_features: false,
            ..Default::default()
        };
        let result = run(&data, &labels, &config).unwrap();
        assert_eq!(result.total_features, 12);
        assert_eq!(result.dna_feature_count, 0);
    }

    #[test]
    fn rejects_empty_dataset() {
        let config = DnaMlConfig::default();
        let err = run(&[], &[], &config);
        assert!(err.is_err());
    }

    #[test]
    fn rejects_label_data_mismatch() {
        let (data, _) = make_test_data();
        let labels = vec!["signal".into()]; // 1 label, 6 data
        let err = run(&data, &labels, &DnaMlConfig::default());
        assert!(err.is_err());
    }

    #[test]
    fn single_class_still_runs() {
        let (data, _) = make_test_data();
        // All same label — forest trains but AUC undefined
        let labels: Vec<String> = data.iter().map(|_| "signal".into()).collect();
        let config = DnaMlConfig {
            n_trees: 5,
            max_depth: 3,
            ..Default::default()
        };
        // Should not panic — may error on train or produce degenerate metrics
        let _ = run(&data, &labels, &config);
    }

    #[test]
    fn augment_preserves_original_features() {
        let features = vec![
            vec![1.0, 2.0, 3.0],
            vec![4.0, 5.0, 6.0],
            vec![7.0, 8.0, 9.0],
        ];
        let augmented = augment_with_dna(&features);
        assert_eq!(augmented.len(), 3);
        // Original 3 features + 5 DNA = 8
        assert_eq!(augmented[0].len(), 8);
        // First 3 values unchanged
        assert!((augmented[0][0] - 1.0).abs() < f64::EPSILON);
        assert!((augmented[0][1] - 2.0).abs() < f64::EPSILON);
        assert!((augmented[0][2] - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn augment_empty_returns_empty() {
        let result = augment_with_dna(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn feature_names_match_dimension() {
        let (data, labels) = make_test_data();
        let config = DnaMlConfig {
            n_trees: 5,
            max_depth: 3,
            use_dna_features: true,
            ..Default::default()
        };
        let result = run(&data, &labels, &config).unwrap();
        assert_eq!(result.feature_names.len(), result.total_features);
    }
}
