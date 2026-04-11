//! ML pipeline MCP tools — autonomous signal detection via random forest.

use crate::params::ml_pipeline::{
    MlEvaluateParams, MlFeatureExtractParams, MlPipelineRunParams, MlPredictParams, MlTrainParams,
};
use nexcore_dtree::prelude::Feature;
use nexcore_ml_pipeline::prelude::*;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

// ---------------------------------------------------------------------------
// In-memory model store (session-scoped)
// ---------------------------------------------------------------------------

static MODEL_STORE: OnceLock<Mutex<HashMap<String, RandomForest>>> = OnceLock::new();

fn store() -> &'static Mutex<HashMap<String, RandomForest>> {
    MODEL_STORE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn next_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    format!("ml_model_{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

// ---------------------------------------------------------------------------
// Tool: ml_feature_extract
// ---------------------------------------------------------------------------

/// Extract 12-element PV feature vector from FAERS contingency data.
pub fn ml_feature_extract(params: MlFeatureExtractParams) -> Result<CallToolResult, McpError> {
    let total_cases = params.a;
    let raw = RawPairData {
        contingency: ContingencyTable {
            drug: params.drug,
            event: params.event,
            a: params.a,
            b: params.b,
            c: params.c,
            d: params.d,
        },
        reporters: ReporterBreakdown {
            hcp: params.hcp_reports.unwrap_or(0),
            consumer: params.consumer_reports.unwrap_or(0),
            other: 0,
        },
        outcomes: OutcomeBreakdown {
            total: total_cases,
            serious: params.serious_count.unwrap_or(0),
            death: params.death_count.unwrap_or(0),
            hospitalization: params.hospitalization_count.unwrap_or(0),
        },
        temporal: TemporalData {
            median_tto_days: params.median_tto_days,
            velocity: params.velocity.unwrap_or(0.0),
        },
    };

    match extract_features(&raw) {
        Ok(sample) => {
            let feature_map: HashMap<&str, f64> = FEATURE_NAMES
                .iter()
                .zip(sample.features.iter())
                .map(|(name, &val)| (*name, val))
                .collect();

            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({
                    "success": true,
                    "drug": sample.drug,
                    "event": sample.event,
                    "features": feature_map,
                    "feature_vector": sample.features,
                }))
                .unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Feature extraction failed: {e}"
        ))])),
    }
}

// ---------------------------------------------------------------------------
// Tool: ml_train
// ---------------------------------------------------------------------------

/// Train a random forest model on labeled PV feature data.
pub fn ml_train(params: MlTrainParams) -> Result<CallToolResult, McpError> {
    let samples: Vec<Sample> = params
        .samples
        .into_iter()
        .map(|s| Sample {
            drug: s.drug,
            event: s.event,
            features: s.features,
            label: Some(s.label),
        })
        .collect();

    let dataset = Dataset::new(samples, feature_names());

    let config = ForestConfig {
        n_trees: params.n_trees.unwrap_or(100),
        max_depth: params.max_depth.or(Some(10)),
        seed: params.seed.unwrap_or(42),
        ..ForestConfig::default()
    };

    match RandomForest::train(&dataset, config) {
        Ok(forest) => {
            let importances = forest.feature_importance();
            let metrics = forest.evaluate(&dataset);
            let model_id = next_id();

            let result = json!({
                "success": true,
                "model_id": model_id,
                "n_trees": forest.n_trees(),
                "n_samples": dataset.len(),
                "train_accuracy": metrics.accuracy,
                "train_auc": metrics.auc,
                "train_f1": metrics.f1,
                "feature_importance": importances,
            });

            if let Ok(mut s) = store().lock() {
                s.insert(model_id, forest);
            }

            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Training failed: {e}"
        ))])),
    }
}

// ---------------------------------------------------------------------------
// Tool: ml_predict
// ---------------------------------------------------------------------------

/// Predict signal probability for drug-event pairs using a trained model.
pub fn ml_predict(params: MlPredictParams) -> Result<CallToolResult, McpError> {
    let guard = store()
        .lock()
        .map_err(|_| McpError::internal_error("Store lock failed", None))?;

    let Some(forest) = guard.get(&params.model_id) else {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Model not found: {}. Train a model first with ml_train.",
            params.model_id
        ))]));
    };

    let predictions: Vec<serde_json::Value> = params
        .samples
        .iter()
        .map(|s| {
            let features: Vec<Feature> =
                s.features.iter().map(|&v| Feature::Continuous(v)).collect();
            let (prediction, prob) = forest.predict_one(&features);
            json!({
                "drug": s.drug,
                "event": s.event,
                "prediction": prediction,
                "signal_probability": prob,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!({
            "success": true,
            "model_id": params.model_id,
            "predictions": predictions,
        }))
        .unwrap_or_default(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: ml_evaluate
// ---------------------------------------------------------------------------

/// Evaluate a trained model on held-out test data.
pub fn ml_evaluate(params: MlEvaluateParams) -> Result<CallToolResult, McpError> {
    let guard = store()
        .lock()
        .map_err(|_| McpError::internal_error("Store lock failed", None))?;

    let Some(forest) = guard.get(&params.model_id) else {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Model not found: {}",
            params.model_id
        ))]));
    };

    let samples: Vec<Sample> = params
        .test_samples
        .into_iter()
        .map(|s| Sample {
            drug: s.drug,
            event: s.event,
            features: s.features,
            label: Some(s.label),
        })
        .collect();

    let dataset = Dataset::new(samples, feature_names());
    let metrics = forest.evaluate(&dataset);

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&json!({
            "success": true,
            "model_id": params.model_id,
            "auc": metrics.auc,
            "precision": metrics.precision,
            "recall": metrics.recall,
            "f1": metrics.f1,
            "accuracy": metrics.accuracy,
            "confusion_matrix": {
                "true_negative": metrics.confusion_matrix[0][0],
                "false_positive": metrics.confusion_matrix[0][1],
                "false_negative": metrics.confusion_matrix[1][0],
                "true_positive": metrics.confusion_matrix[1][1],
            },
        }))
        .unwrap_or_default(),
    )]))
}

// ---------------------------------------------------------------------------
// Tool: ml_pipeline_run
// ---------------------------------------------------------------------------

/// Run the full autonomous ML pipeline: extract features → train → evaluate → predict.
pub fn ml_pipeline_run(params: MlPipelineRunParams) -> Result<CallToolResult, McpError> {
    let raw_data: Vec<RawPairData> = params
        .data
        .iter()
        .map(|e| RawPairData {
            contingency: ContingencyTable {
                drug: e.drug.clone(),
                event: e.event.clone(),
                a: e.a,
                b: e.b,
                c: e.c,
                d: e.d,
            },
            reporters: ReporterBreakdown {
                hcp: e.hcp_reports.unwrap_or(0),
                consumer: e.consumer_reports.unwrap_or(0),
                other: 0,
            },
            outcomes: OutcomeBreakdown {
                total: e.a,
                serious: e.serious_count.unwrap_or(0),
                death: e.death_count.unwrap_or(0),
                hospitalization: e.hospitalization_count.unwrap_or(0),
            },
            temporal: TemporalData {
                median_tto_days: e.median_tto_days,
                velocity: e.velocity.unwrap_or(0.0),
            },
        })
        .collect();

    let labels: Vec<(String, String, String)> = params
        .labels
        .iter()
        .map(|l| (l.drug.clone(), l.event.clone(), l.label.clone()))
        .collect();

    let config = PipelineConfig {
        forest: ForestConfig {
            n_trees: params.n_trees.unwrap_or(100),
            max_depth: params.max_depth.or(Some(10)),
            seed: 42,
            ..ForestConfig::default()
        },
        train_ratio: params.train_ratio.unwrap_or(0.8),
        ..PipelineConfig::default()
    };

    match pipeline::run(&raw_data, &labels, config) {
        Ok(result) => {
            let predictions: Vec<serde_json::Value> = result
                .test_predictions
                .iter()
                .map(|p| {
                    json!({
                        "drug": p.drug,
                        "event": p.event,
                        "prediction": p.prediction,
                        "signal_probability": p.signal_probability,
                    })
                })
                .collect();

            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&json!({
                    "success": true,
                    "model_version": result.model_version,
                    "n_trees": result.n_trees,
                    "n_train_samples": result.n_train_samples,
                    "n_test_samples": result.n_test_samples,
                    "train_metrics": {
                        "auc": result.train_metrics.auc,
                        "precision": result.train_metrics.precision,
                        "recall": result.train_metrics.recall,
                        "f1": result.train_metrics.f1,
                        "accuracy": result.train_metrics.accuracy,
                    },
                    "test_metrics": {
                        "auc": result.test_metrics.auc,
                        "precision": result.test_metrics.precision,
                        "recall": result.test_metrics.recall,
                        "f1": result.test_metrics.f1,
                        "accuracy": result.test_metrics.accuracy,
                    },
                    "test_predictions": predictions,
                }))
                .unwrap_or_default(),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Pipeline failed: {e}"
        ))])),
    }
}
