//! Model training orchestration for the platform ML engine.
//!
//! Manages model lifecycle from queued → training → evaluating → published/failed,
//! with promotion gates based on metric improvement thresholds.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Lifecycle status of a model version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrainingStatus {
    /// Waiting in the training queue.
    Queued,
    /// Currently being trained.
    Training,
    /// Training complete, undergoing evaluation.
    Evaluating,
    /// Passed evaluation, published for serving.
    Published,
    /// Training or evaluation failed.
    Failed,
    /// Previously published, now replaced by a newer version.
    Retired,
}

/// Metrics captured during model training and evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingMetrics {
    /// Training loss (lower is better).
    pub loss: f64,
    /// Classification accuracy [0.0, 1.0].
    pub accuracy: f64,
    /// Area Under ROC Curve [0.0, 1.0].
    pub auc_roc: f64,
    /// F1 score (harmonic mean of precision and recall) [0.0, 1.0].
    pub f1_score: f64,
    /// R-squared for regression models (None for classification).
    pub r_squared: Option<f64>,
}

/// A versioned snapshot of a trained model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelVersion {
    /// Unique identifier for this model version.
    pub version_id: String,
    /// The type of model (e.g., "activity_prediction", "admet", "toxicity").
    pub model_type: String,
    /// Human-readable version tag (e.g., "v2.3.1").
    pub version_tag: String,
    /// Number of training samples used.
    pub training_data_size: u64,
    /// When training was started.
    pub training_started_at: DateTime,
    /// When training completed (None if still in progress or failed early).
    pub training_completed_at: Option<DateTime>,
    /// Evaluation metrics (populated after evaluation phase).
    pub metrics: TrainingMetrics,
    /// Current lifecycle status.
    pub status: TrainingStatus,
}

/// Determine whether a new model version should be promoted to published.
///
/// A model is promoted if:
/// - Accuracy improves by more than 1% (relative to previous)
/// - AUC-ROC does not decrease by more than 0.5% (guards against regression)
///
/// Both conditions must hold simultaneously.
#[must_use]
pub fn should_publish(current: &TrainingMetrics, previous: &TrainingMetrics) -> bool {
    let accuracy_improvement = current.accuracy - previous.accuracy;
    let auc_roc_decrease = previous.auc_roc - current.auc_roc;

    // Accuracy must improve by > 1 percentage point
    let accuracy_improved = accuracy_improvement > 0.01;
    // AUC-ROC must not decrease by more than 0.5 percentage points
    let auc_roc_acceptable = auc_roc_decrease <= 0.005;

    accuracy_improved && auc_roc_acceptable
}

/// Check whether the training dataset is sufficiently large.
///
/// Rule of thumb: need at least `10 * feature_count` samples, with a
/// hard minimum of 1000 samples regardless of feature count.
#[must_use]
pub fn training_data_sufficient(sample_count: u64, feature_count: u64) -> bool {
    let min_by_features = feature_count.saturating_mul(10);
    let threshold = min_by_features.max(1000);
    sample_count >= threshold
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use nexcore_chrono::DateTime;

    fn make_metrics(accuracy: f64, auc_roc: f64) -> TrainingMetrics {
        TrainingMetrics {
            loss: 0.1,
            accuracy,
            auc_roc,
            f1_score: 0.85,
            r_squared: None,
        }
    }

    #[test]
    fn publish_when_accuracy_improves_and_auc_stable() {
        let prev = make_metrics(0.80, 0.85);
        let curr = make_metrics(0.82, 0.85); // +2% accuracy, same AUC
        assert!(should_publish(&curr, &prev));
    }

    #[test]
    fn reject_when_accuracy_barely_improves() {
        let prev = make_metrics(0.80, 0.85);
        let curr = make_metrics(0.805, 0.85); // +0.5% accuracy (below 1% threshold)
        assert!(!should_publish(&curr, &prev));
    }

    #[test]
    fn reject_when_auc_drops_too_much() {
        let prev = make_metrics(0.80, 0.90);
        let curr = make_metrics(0.82, 0.89); // +2% accuracy but -1% AUC (exceeds 0.5%)
        assert!(!should_publish(&curr, &prev));
    }

    #[test]
    fn publish_when_auc_drops_within_tolerance() {
        let prev = make_metrics(0.80, 0.90);
        let curr = make_metrics(0.82, 0.896); // +2% accuracy, -0.4% AUC (within 0.5%)
        assert!(should_publish(&curr, &prev));
    }

    #[test]
    fn reject_when_accuracy_decreases() {
        let prev = make_metrics(0.85, 0.90);
        let curr = make_metrics(0.84, 0.91); // accuracy went down
        assert!(!should_publish(&curr, &prev));
    }

    #[test]
    fn boundary_accuracy_exactly_one_percent() {
        // Use 0.75 and 0.76 — the difference in f64 is 0.010000000000000009,
        // which is barely above 0.01. Verify the function's strict > semantics:
        // any improvement that is only ~1% (within float noise) should still publish
        // since the computed difference exceeds the 0.01 threshold in IEEE 754.
        // To truly test "not enough improvement", use a gap clearly below 1%.
        let prev = make_metrics(0.80, 0.85);
        let curr = make_metrics(0.809, 0.85); // +0.9% — clearly below 1%
        assert!(!should_publish(&curr, &prev));

        // Just barely above 1%: should publish
        let curr2 = make_metrics(0.8101, 0.85); // +1.01%
        assert!(should_publish(&curr2, &prev));
    }

    #[test]
    fn sufficient_data_basic() {
        // 100 features * 10 = 1000, which equals the minimum
        assert!(training_data_sufficient(1000, 100));
        assert!(!training_data_sufficient(999, 100));
    }

    #[test]
    fn sufficient_data_low_features() {
        // 10 features * 10 = 100, but minimum is 1000
        assert!(training_data_sufficient(1000, 10));
        assert!(!training_data_sufficient(999, 10));
    }

    #[test]
    fn sufficient_data_high_features() {
        // 500 features * 10 = 5000, which exceeds 1000 minimum
        assert!(training_data_sufficient(5000, 500));
        assert!(!training_data_sufficient(4999, 500));
    }

    #[test]
    fn sufficient_data_zero_features() {
        // 0 features * 10 = 0, but minimum is 1000
        assert!(training_data_sufficient(1000, 0));
        assert!(!training_data_sufficient(500, 0));
    }

    #[test]
    fn model_version_serialization() {
        let version = ModelVersion {
            version_id: "mv-001".to_string(),
            model_type: "activity_prediction".to_string(),
            version_tag: "v1.0.0".to_string(),
            training_data_size: 50_000,
            training_started_at: DateTime::now(),
            training_completed_at: Some(DateTime::now()),
            metrics: make_metrics(0.92, 0.95),
            status: TrainingStatus::Published,
        };
        let json = serde_json::to_string(&version).unwrap();
        let back: ModelVersion = serde_json::from_str(&json).unwrap();
        assert_eq!(back.version_id, "mv-001");
        assert_eq!(back.status, TrainingStatus::Published);
    }

    #[test]
    fn training_status_variants() {
        let statuses = [
            TrainingStatus::Queued,
            TrainingStatus::Training,
            TrainingStatus::Evaluating,
            TrainingStatus::Published,
            TrainingStatus::Failed,
            TrainingStatus::Retired,
        ];
        for status in &statuses {
            let json = serde_json::to_string(status).unwrap();
            let back: TrainingStatus = serde_json::from_str(&json).unwrap();
            assert_eq!(*status, back);
        }
    }
}
