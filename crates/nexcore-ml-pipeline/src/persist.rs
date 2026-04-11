//! Model persistence: serialize/deserialize trained models with versioning.
//!
//! ## Primitive Foundation
//! - T1: Persistence (π) — model survives across sessions
//! - T1: State (ς) — model parameters, version metadata

use crate::ensemble::RandomForest;
use crate::types::Metrics;
use serde::{Deserialize, Serialize};

/// Errors during model persistence.
#[derive(Debug, nexcore_error::Error)]
pub enum PersistError {
    /// Serialization failed.
    #[error("serialization failed: {0}")]
    Serialize(String),
    /// Deserialization failed.
    #[error("deserialization failed: {0}")]
    Deserialize(String),
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(String),
}

/// A versioned model artifact ready for persistence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelArtifact {
    /// Semantic version of the model.
    pub version: String,
    /// Timestamp of training (ISO 8601).
    pub trained_at: String,
    /// Training metrics.
    pub train_metrics: Metrics,
    /// Test/validation metrics.
    pub test_metrics: Option<Metrics>,
    /// Number of training samples used.
    pub n_train_samples: usize,
    /// Feature names.
    pub feature_names: Vec<String>,
    /// The trained forest model.
    pub model: RandomForest,
}

impl ModelArtifact {
    /// Create a new model artifact.
    #[must_use]
    pub fn new(
        model: RandomForest,
        train_metrics: Metrics,
        test_metrics: Option<Metrics>,
        n_train_samples: usize,
    ) -> Self {
        let version = format!(
            "0.1.0-{:08x}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs() as u32)
                .unwrap_or(0)
        );

        let trained_at = format!(
            "{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        );

        Self {
            version,
            trained_at,
            train_metrics,
            test_metrics,
            n_train_samples,
            feature_names: model.feature_names.clone(),
            model,
        }
    }

    /// Serialize to JSON string.
    ///
    /// # Errors
    /// Returns `PersistError::Serialize` on failure.
    pub fn to_json(&self) -> Result<String, PersistError> {
        serde_json::to_string_pretty(self).map_err(|e| PersistError::Serialize(e.to_string()))
    }

    /// Deserialize from JSON string.
    ///
    /// # Errors
    /// Returns `PersistError::Deserialize` on failure.
    pub fn from_json(json: &str) -> Result<Self, PersistError> {
        serde_json::from_str(json).map_err(|e| PersistError::Deserialize(e.to_string()))
    }

    /// Save to a file path.
    ///
    /// # Errors
    /// Returns `PersistError::Io` on write failure.
    pub fn save(&self, path: &std::path::Path) -> Result<(), PersistError> {
        let json = self.to_json()?;
        std::fs::write(path, json).map_err(|e| PersistError::Io(e.to_string()))
    }

    /// Load from a file path.
    ///
    /// # Errors
    /// Returns `PersistError` on read or parse failure.
    pub fn load(path: &std::path::Path) -> Result<Self, PersistError> {
        let json = std::fs::read_to_string(path).map_err(|e| PersistError::Io(e.to_string()))?;
        Self::from_json(&json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Dataset, FEATURE_NAMES, ForestConfig, Metrics, Sample};

    fn make_test_artifact() -> ModelArtifact {
        let samples: Vec<Sample> = (0..20)
            .map(|i| {
                let is_signal = i < 10;
                Sample {
                    drug: format!("drug_{i}"),
                    event: "event".into(),
                    features: if is_signal {
                        vec![5.0, 4.0, 2.0, 3.0, 4.0, 0.7, 0.2, 0.8, 0.1, 0.5, 7.0, 8.0]
                    } else {
                        vec![
                            0.8, 0.9, -0.5, 0.7, 2.0, 0.3, 0.6, 0.2, 0.01, 0.1, 30.0, 1.0,
                        ]
                    },
                    label: Some(if is_signal { "signal" } else { "noise" }.into()),
                }
            })
            .collect();

        let dataset = Dataset::new(
            samples,
            FEATURE_NAMES.iter().map(|s| s.to_string()).collect(),
        );

        let config = ForestConfig {
            n_trees: 5,
            max_depth: Some(3),
            seed: 42,
            ..ForestConfig::default()
        };

        let forest = RandomForest::train(&dataset, config).unwrap_or_else(|_| unreachable!());

        let metrics = Metrics {
            auc: 0.95,
            precision: 0.9,
            recall: 0.85,
            f1: 0.87,
            accuracy: 0.9,
            confusion_matrix: [[9, 1], [1, 9]],
        };

        ModelArtifact::new(forest, metrics.clone(), Some(metrics), 20)
    }

    #[test]
    fn roundtrip_json() {
        let artifact = make_test_artifact();
        let json = artifact.to_json();
        assert!(json.is_ok());
        let json = json.unwrap_or_default();

        let loaded = ModelArtifact::from_json(&json);
        assert!(loaded.is_ok());
        let loaded = loaded.unwrap_or_else(|_| unreachable!());
        assert_eq!(loaded.version, artifact.version);
        assert_eq!(loaded.n_train_samples, 20);
    }

    #[test]
    fn save_and_load_file() {
        let artifact = make_test_artifact();
        let path = std::env::temp_dir().join("test_ml_model.json");
        let save_result = artifact.save(&path);
        assert!(save_result.is_ok());

        let loaded = ModelArtifact::load(&path);
        assert!(loaded.is_ok());
        let loaded = loaded.unwrap_or_else(|_| unreachable!());
        assert_eq!(loaded.feature_names.len(), 12);

        // Cleanup
        let _ = std::fs::remove_file(&path);
    }
}
