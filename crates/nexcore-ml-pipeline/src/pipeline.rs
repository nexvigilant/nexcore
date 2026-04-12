//! Full autonomous pipeline: ingest → features → train → evaluate → persist → predict.
//!
//! ## Primitive Foundation
//! - T1: Sequence (σ) — strict pipeline stage ordering
//! - T1: State (ς) — pipeline state transitions
//! - T1: Exists (∃) — pipeline produces a trained, evaluated model

use crate::ensemble::{EnsembleError, RandomForest};
use crate::feature::{self, FeatureError};
use crate::persist::{ModelArtifact, PersistError};
use crate::types::{Dataset, ForestConfig, PipelineResult, Prediction, RawPairData};

/// Errors during pipeline execution.
#[derive(Debug, nexcore_error::Error)]
pub enum PipelineError {
    /// Feature extraction failed.
    #[error("feature extraction: {0}")]
    Feature(#[from] FeatureError),
    /// Training failed.
    #[error("training: {0}")]
    Training(#[from] EnsembleError),
    /// Persistence failed.
    #[error("persistence: {0}")]
    Persist(#[from] PersistError),
    /// Insufficient data.
    #[error("insufficient data: need at least {needed}, got {got}")]
    InsufficientData {
        /// Minimum required.
        needed: usize,
        /// Actually provided.
        got: usize,
    },
    /// No labeled data.
    #[error("no labeled samples in dataset")]
    NoLabels,
}

/// Pipeline configuration.
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Random forest configuration.
    pub forest: ForestConfig,
    /// Train/test split ratio (0.0 - 1.0).
    pub train_ratio: f64,
    /// Whether to run k-fold cross-validation.
    pub cross_validate: bool,
    /// Number of folds for cross-validation.
    pub n_folds: usize,
    /// Path to save the model (None = don't save).
    pub save_path: Option<std::path::PathBuf>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            forest: ForestConfig::default(),
            train_ratio: 0.8,
            cross_validate: false,
            n_folds: 5,
            save_path: None,
        }
    }
}

/// Run the full autonomous ML pipeline.
///
/// Stages:
/// 1. Feature extraction from raw FAERS data
/// 2. Dataset construction with labels
/// 3. Train/test split
/// 4. Random forest training
/// 5. Evaluation (metrics + baseline comparison)
/// 6. Model persistence
/// 7. Test set predictions
///
/// # Errors
/// Returns `PipelineError` if any stage fails.
pub fn run(
    raw_data: &[RawPairData],
    labels: &[(String, String, String)], // (drug, event, label)
    config: PipelineConfig,
) -> Result<PipelineResult, PipelineError> {
    // Stage 1: Feature extraction
    let mut samples = Vec::with_capacity(raw_data.len());
    for raw in raw_data {
        let mut sample = feature::extract_features(raw)?;

        // Match label by (drug, event)
        for (drug, event, label) in labels {
            if sample.drug == *drug && sample.event == *event {
                sample.label = Some(label.clone());
                break;
            }
        }

        samples.push(sample);
    }

    // Stage 2: Dataset construction
    let feature_names = feature::feature_names();
    let dataset = Dataset::new(samples, feature_names);

    let labeled_count = dataset.samples.iter().filter(|s| s.label.is_some()).count();

    if labeled_count < 10 {
        return Err(PipelineError::InsufficientData {
            needed: 10,
            got: labeled_count,
        });
    }

    // Stage 3: Cross-validation (if enabled) — run BEFORE final model
    // so CV metrics describe the same data the final model trains on.
    let cv_metrics = if config.cross_validate {
        Some(cross_validate(&dataset, &config)?)
    } else {
        None
    };

    // Stage 4: Train/test split
    let (train_set, test_set) = dataset.split(config.train_ratio);

    if train_set.is_empty() {
        return Err(PipelineError::NoLabels);
    }

    // Stage 5: Training
    // When CV is enabled, CV already validated performance on the full dataset.
    // The train/test split still produces holdout predictions for inspection.
    let forest = RandomForest::train(&train_set, config.forest.clone())?;

    // Stage 6: Evaluation
    let train_metrics = forest.evaluate(&train_set);
    let test_metrics = forest.evaluate(&test_set);

    // Stage 7: Persistence
    let artifact = ModelArtifact::new(
        forest.clone(),
        train_metrics.clone(),
        Some(test_metrics.clone()),
        train_set.len(),
    );

    if let Some(ref path) = config.save_path {
        artifact.save(path)?;
    }

    // Stage 8: Test predictions
    let importances = forest.feature_importance();
    let top_features: Vec<(String, f64)> = {
        let mut imp = importances.clone();
        imp.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        imp.into_iter().take(5).collect()
    };

    let test_predictions: Vec<Prediction> = test_set
        .samples
        .iter()
        .map(|sample| {
            let features = sample.to_dtree_features();
            let (prediction, signal_probability) = forest.predict_one(&features);

            Prediction {
                drug: sample.drug.clone(),
                event: sample.event.clone(),
                prediction,
                signal_probability,
                top_features: top_features.clone(),
            }
        })
        .collect();

    Ok(PipelineResult {
        train_metrics,
        test_metrics,
        cv_metrics,
        n_trees: forest.n_trees(),
        n_train_samples: train_set.len(),
        n_test_samples: test_set.len(),
        model_version: artifact.version,
        test_predictions,
    })
}

/// Run k-fold cross-validation on the dataset.
///
/// Trains a separate model on each fold, evaluates on the held-out fold,
/// and returns mean/std metrics across all folds.
///
/// # Errors
/// Returns `PipelineError` if any fold fails to train.
pub fn cross_validate(
    dataset: &Dataset,
    config: &PipelineConfig,
) -> Result<crate::types::CvMetrics, PipelineError> {
    let folds = dataset.k_fold(config.n_folds);
    let mut fold_aucs = Vec::with_capacity(folds.len());
    let mut fold_f1s = Vec::with_capacity(folds.len());
    let mut fold_accs = Vec::with_capacity(folds.len());

    for (train_fold, test_fold) in &folds {
        let forest = RandomForest::train(train_fold, config.forest.clone())?;
        let metrics = forest.evaluate(test_fold);
        fold_aucs.push(metrics.auc);
        fold_f1s.push(metrics.f1);
        fold_accs.push(metrics.accuracy);
    }

    let k = folds.len();
    let mean_auc = fold_aucs.iter().sum::<f64>() / k as f64;
    let mean_f1 = fold_f1s.iter().sum::<f64>() / k as f64;
    let mean_accuracy = fold_accs.iter().sum::<f64>() / k as f64;

    let std_auc = std_dev(&fold_aucs, mean_auc);
    let std_f1 = std_dev(&fold_f1s, mean_f1);

    Ok(crate::types::CvMetrics {
        k,
        mean_auc,
        std_auc,
        mean_f1,
        std_f1,
        mean_accuracy,
        fold_aucs,
    })
}

/// Compute sample standard deviation.
fn std_dev(values: &[f64], mean: f64) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }
    let variance =
        values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (values.len() - 1) as f64;
    variance.sqrt()
}

/// Run prediction-only using a pre-trained model.
///
/// # Errors
/// Returns `PipelineError` if feature extraction fails.
pub fn predict_with_model(
    model: &RandomForest,
    raw_data: &[RawPairData],
) -> Result<Vec<Prediction>, PipelineError> {
    let importances = model.feature_importance();
    let top_features: Vec<(String, f64)> = {
        let mut imp = importances;
        imp.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        imp.into_iter().take(5).collect()
    };

    let mut predictions = Vec::with_capacity(raw_data.len());
    for raw in raw_data {
        let sample = feature::extract_features(raw)?;
        let features = sample.to_dtree_features();
        let (prediction, signal_probability) = model.predict_one(&features);

        predictions.push(Prediction {
            drug: sample.drug,
            event: sample.event,
            prediction,
            signal_probability,
            top_features: top_features.clone(),
        });
    }

    Ok(predictions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    fn make_pipeline_data() -> (Vec<RawPairData>, Vec<(String, String, String)>) {
        let mut raw_data = Vec::new();
        let mut labels = Vec::new();

        // Generate signal pairs (high counts, disproportionate)
        for i in 0..15 {
            let drug = format!("signal_drug_{i}");
            let event = format!("serious_event_{i}");
            raw_data.push(RawPairData {
                contingency: ContingencyTable {
                    drug: drug.clone(),
                    event: event.clone(),
                    a: 150 + i * 10,
                    b: 5000,
                    c: 3000,
                    d: 500_000,
                },
                reporters: ReporterBreakdown {
                    hcp: 70,
                    consumer: 20,
                    other: 10,
                },
                outcomes: OutcomeBreakdown {
                    total: 150 + i * 10,
                    serious: 100 + i * 5,
                    death: 10 + i,
                    hospitalization: 50 + i * 3,
                },
                temporal: TemporalData {
                    median_tto_days: Some(7.0 + i as f64),
                    velocity: 8.0 + i as f64 * 0.5,
                },
            });
            labels.push((drug, event, "signal".into()));
        }

        // Generate noise pairs (low counts, proportionate)
        for i in 0..15 {
            let drug = format!("noise_drug_{i}");
            let event = format!("mild_event_{i}");
            raw_data.push(RawPairData {
                contingency: ContingencyTable {
                    drug: drug.clone(),
                    event: event.clone(),
                    a: 10 + i,
                    b: 5000,
                    c: 50_000,
                    d: 500_000,
                },
                reporters: ReporterBreakdown {
                    hcp: 30,
                    consumer: 60,
                    other: 10,
                },
                outcomes: OutcomeBreakdown {
                    total: 10 + i,
                    serious: 2 + i / 3,
                    death: 0,
                    hospitalization: 1,
                },
                temporal: TemporalData {
                    median_tto_days: Some(30.0 + i as f64 * 2.0),
                    velocity: 1.0 + i as f64 * 0.1,
                },
            });
            labels.push((drug, event, "noise".into()));
        }

        (raw_data, labels)
    }

    #[test]
    fn full_pipeline_runs() {
        let (raw_data, labels) = make_pipeline_data();
        let config = PipelineConfig {
            forest: ForestConfig {
                n_trees: 10,
                max_depth: Some(5),
                seed: 42,
                ..ForestConfig::default()
            },
            train_ratio: 0.8,
            ..PipelineConfig::default()
        };

        let result = run(&raw_data, &labels, config);
        assert!(result.is_ok(), "Pipeline failed: {result:?}");
        let result = result.unwrap_or_else(|_| unreachable!());

        assert_eq!(result.n_trees, 10);
        assert!(result.n_train_samples > 0);
        assert!(result.n_test_samples > 0);
        assert!(result.test_metrics.auc >= 0.0);
        assert!(!result.test_predictions.is_empty());
    }

    #[test]
    fn pipeline_insufficient_data() {
        let raw_data = vec![RawPairData {
            contingency: ContingencyTable {
                drug: "x".into(),
                event: "y".into(),
                a: 10,
                b: 100,
                c: 100,
                d: 10000,
            },
            reporters: ReporterBreakdown {
                hcp: 5,
                consumer: 3,
                other: 2,
            },
            outcomes: OutcomeBreakdown {
                total: 10,
                serious: 3,
                death: 0,
                hospitalization: 1,
            },
            temporal: TemporalData {
                median_tto_days: None,
                velocity: 1.0,
            },
        }];
        let labels = vec![("x".into(), "y".into(), "signal".into())];

        let result = run(&raw_data, &labels, PipelineConfig::default());
        assert!(result.is_err());
    }

    #[test]
    fn predict_with_trained_model() {
        let (raw_data, labels) = make_pipeline_data();

        // Train
        let mut samples = Vec::new();
        for raw in &raw_data {
            if let Ok(mut s) = feature::extract_features(raw) {
                for (drug, event, label) in &labels {
                    if s.drug == *drug && s.event == *event {
                        s.label = Some(label.clone());
                    }
                }
                samples.push(s);
            }
        }

        let dataset = Dataset::new(samples, feature::feature_names());
        let forest = RandomForest::train(
            &dataset,
            ForestConfig {
                n_trees: 10,
                max_depth: Some(5),
                seed: 42,
                ..ForestConfig::default()
            },
        )
        .unwrap_or_else(|_| unreachable!());

        // Predict on first 3
        let predictions = predict_with_model(&forest, &raw_data[..3]);
        assert!(predictions.is_ok());
        let predictions = predictions.unwrap_or_else(|_| unreachable!());
        assert_eq!(predictions.len(), 3);
    }
}
