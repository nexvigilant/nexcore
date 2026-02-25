//! Model evaluation framework for systematic benchmarking.
//!
//! Provides standardized benchmark datasets, metrics calculation (F1, MCC),
//! and composite scoring for model ranking.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// A standardized benchmark dataset for model evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkDataset {
    /// Human-readable name (e.g., "MoleculeNet-BBBP", "Tox21").
    pub name: String,
    /// Dataset type (e.g., "classification", "regression", "multi-task").
    pub dataset_type: String,
    /// Number of samples in the evaluation set.
    pub sample_count: u64,
    /// Number of input features per sample.
    pub feature_count: u64,
    /// Description of the dataset and its relevance.
    pub description: String,
}

/// Metrics from evaluating a model against a benchmark dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkMetrics {
    /// Classification accuracy [0.0, 1.0].
    pub accuracy: f64,
    /// Precision (positive predictive value) [0.0, 1.0].
    pub precision: f64,
    /// Recall (sensitivity / true positive rate) [0.0, 1.0].
    pub recall: f64,
    /// F1 score (harmonic mean of precision and recall) [0.0, 1.0].
    pub f1: f64,
    /// Area Under ROC Curve [0.0, 1.0].
    pub auc_roc: f64,
    /// Root Mean Squared Error (for regression models).
    pub rmse: Option<f64>,
    /// Mean Absolute Error (for regression models).
    pub mae: Option<f64>,
}

/// The result of evaluating a model on a benchmark dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// The model that was evaluated.
    pub model_id: String,
    /// The benchmark dataset name.
    pub dataset_name: String,
    /// The computed metrics.
    pub metrics: BenchmarkMetrics,
    /// When the evaluation was performed.
    pub evaluated_at: DateTime,
}

/// Calculate F1 score from precision and recall.
///
/// F1 = 2 * (precision * recall) / (precision + recall)
///
/// Returns 0.0 when both precision and recall are zero (avoids division by zero).
#[must_use]
pub fn calculate_f1(precision: f64, recall: f64) -> f64 {
    let denominator = precision + recall;
    if denominator <= 0.0 {
        return 0.0;
    }
    2.0 * (precision * recall) / denominator
}

/// Calculate Matthews Correlation Coefficient (MCC).
///
/// MCC = (TP*TN - FP*FN) / sqrt((TP+FP)*(TP+FN)*(TN+FP)*(TN+FN))
///
/// Returns 0.0 when the denominator is zero (all predictions are the same class,
/// or all samples are the same class).
///
/// MCC ranges from -1 (total disagreement) to +1 (perfect prediction).
/// 0 indicates no better than random.
#[must_use]
pub fn calculate_mcc(tp: u64, tn: u64, fp: u64, fn_count: u64) -> f64 {
    let tp_f = tp as f64;
    let tn_f = tn as f64;
    let fp_f = fp as f64;
    let fn_f = fn_count as f64;

    let numerator = tp_f * tn_f - fp_f * fn_f;

    let denom_product = (tp_f + fp_f) * (tp_f + fn_f) * (tn_f + fp_f) * (tn_f + fn_f);

    if denom_product <= 0.0 {
        return 0.0;
    }

    numerator / denom_product.sqrt()
}

/// Rank models by a composite score.
///
/// Composite score = 0.4 * auc_roc + 0.3 * f1 + 0.3 * accuracy
///
/// Returns a sorted list of (model_id, composite_score) in descending order.
/// If multiple results exist for the same model, only the first occurrence is used.
#[must_use]
pub fn rank_models(results: &[BenchmarkResult]) -> Vec<(String, f64)> {
    let mut scores: Vec<(String, f64)> = Vec::new();
    let mut seen_models = std::collections::HashSet::new();

    for result in results {
        if seen_models.contains(&result.model_id) {
            continue;
        }
        seen_models.insert(result.model_id.clone());

        let composite =
            0.4 * result.metrics.auc_roc + 0.3 * result.metrics.f1 + 0.3 * result.metrics.accuracy;

        scores.push((result.model_id.clone(), composite));
    }

    // Sort descending by score. Use total_cmp for deterministic NaN handling.
    scores.sort_by(|a, b| b.1.total_cmp(&a.1));
    scores
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use nexcore_chrono::DateTime;

    #[test]
    fn f1_basic() {
        // precision=0.8, recall=0.6 => F1 = 2*0.48/1.4 = 0.6857...
        let f1 = calculate_f1(0.8, 0.6);
        let expected = 2.0 * 0.8 * 0.6 / (0.8 + 0.6);
        assert!((f1 - expected).abs() < 1e-10);
    }

    #[test]
    fn f1_perfect() {
        let f1 = calculate_f1(1.0, 1.0);
        assert!((f1 - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn f1_zero_division() {
        assert!((calculate_f1(0.0, 0.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn f1_one_zero() {
        // precision=1.0, recall=0.0 => F1 = 0.0
        assert!((calculate_f1(1.0, 0.0) - 0.0).abs() < f64::EPSILON);
        assert!((calculate_f1(0.0, 1.0) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn f1_symmetric() {
        let f1_a = calculate_f1(0.7, 0.9);
        let f1_b = calculate_f1(0.9, 0.7);
        assert!((f1_a - f1_b).abs() < 1e-10);
    }

    #[test]
    fn mcc_perfect_prediction() {
        // All correct: 50 TP, 50 TN, 0 FP, 0 FN
        let mcc = calculate_mcc(50, 50, 0, 0);
        assert!((mcc - 1.0).abs() < 1e-10);
    }

    #[test]
    fn mcc_total_disagreement() {
        // All wrong: 0 TP, 0 TN, 50 FP, 50 FN
        let mcc = calculate_mcc(0, 0, 50, 50);
        assert!((mcc - (-1.0)).abs() < 1e-10);
    }

    #[test]
    fn mcc_random() {
        // Balanced random: 25 TP, 25 TN, 25 FP, 25 FN => MCC = 0
        let mcc = calculate_mcc(25, 25, 25, 25);
        assert!(mcc.abs() < 1e-10);
    }

    #[test]
    fn mcc_zero_denominator() {
        // All predicted positive, no negatives: TP=50, TN=0, FP=50, FN=0
        // (TP+FP)=100, (TP+FN)=50, (TN+FP)=50, (TN+FN)=0 => denom=0
        let mcc = calculate_mcc(50, 0, 50, 0);
        assert!((mcc - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn mcc_real_values() {
        // TP=90, TN=80, FP=10, FN=20
        // num = 90*80 - 10*20 = 7200 - 200 = 7000
        // denom = sqrt(100 * 110 * 90 * 100) = sqrt(99_000_000) = 9949.87...
        let mcc = calculate_mcc(90, 80, 10, 20);
        let expected_num = 90.0 * 80.0 - 10.0 * 20.0;
        let expected_denom = (100.0_f64 * 110.0 * 90.0 * 100.0).sqrt();
        let expected = expected_num / expected_denom;
        assert!((mcc - expected).abs() < 1e-10);
        // MCC should be around 0.703
        assert!((mcc - 0.703).abs() < 0.001);
    }

    fn make_result(model_id: &str, accuracy: f64, f1: f64, auc_roc: f64) -> BenchmarkResult {
        BenchmarkResult {
            model_id: model_id.to_string(),
            dataset_name: "test-dataset".to_string(),
            metrics: BenchmarkMetrics {
                accuracy,
                precision: 0.0,
                recall: 0.0,
                f1,
                auc_roc,
                rmse: None,
                mae: None,
            },
            evaluated_at: DateTime::now(),
        }
    }

    #[test]
    fn rank_models_sorted_descending() {
        let results = vec![
            make_result("model-c", 0.70, 0.65, 0.75),
            make_result("model-a", 0.90, 0.88, 0.95),
            make_result("model-b", 0.80, 0.78, 0.85),
        ];
        let ranked = rank_models(&results);
        assert_eq!(ranked.len(), 3);
        assert_eq!(ranked[0].0, "model-a");
        assert_eq!(ranked[1].0, "model-b");
        assert_eq!(ranked[2].0, "model-c");
    }

    #[test]
    fn rank_models_composite_score_calculation() {
        let results = vec![make_result("only", 0.80, 0.75, 0.90)];
        let ranked = rank_models(&results);
        let expected = 0.4 * 0.90 + 0.3 * 0.75 + 0.3 * 0.80;
        assert!((ranked[0].1 - expected).abs() < 1e-10);
    }

    #[test]
    fn rank_models_dedup_by_model_id() {
        let results = vec![
            make_result("model-a", 0.80, 0.75, 0.90),
            make_result("model-a", 0.95, 0.93, 0.99), // duplicate — ignored
        ];
        let ranked = rank_models(&results);
        assert_eq!(ranked.len(), 1);
        // Uses first occurrence
        let expected = 0.4 * 0.90 + 0.3 * 0.75 + 0.3 * 0.80;
        assert!((ranked[0].1 - expected).abs() < 1e-10);
    }

    #[test]
    fn rank_models_empty() {
        let ranked = rank_models(&[]);
        assert!(ranked.is_empty());
    }

    #[test]
    fn benchmark_dataset_serialization() {
        let ds = BenchmarkDataset {
            name: "MoleculeNet-BBBP".to_string(),
            dataset_type: "classification".to_string(),
            sample_count: 2039,
            feature_count: 512,
            description: "Blood-brain barrier penetration dataset".to_string(),
        };
        let json = serde_json::to_string(&ds).unwrap();
        let back: BenchmarkDataset = serde_json::from_str(&json).unwrap();
        assert_eq!(back.name, "MoleculeNet-BBBP");
        assert_eq!(back.sample_count, 2039);
    }
}
