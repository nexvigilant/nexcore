//! Anonymized data collection pipeline for cross-tenant ML training.
//!
//! Handles consent-aware data aggregation with differential privacy guarantees,
//! data quality scoring, and training pool thresholding.

use serde::{Deserialize, Serialize};

/// Configuration for data anonymization before aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnonymizationConfig {
    /// Remove all tenant-identifying information from training data.
    pub remove_tenant_ids: bool,
    /// Canonicalize compound representations (SMILES normalization, etc.).
    pub canonicalize_compounds: bool,
    /// Strip proprietary target names, replacing with generic identifiers.
    pub strip_proprietary_targets: bool,
    /// Aggregate assay-level statistics instead of individual measurements.
    pub aggregate_assay_stats: bool,
    /// Differential privacy epsilon parameter. Lower = more private (more noise).
    /// Typical range: 0.1 (very private) to 10.0 (minimal privacy).
    pub differential_privacy_epsilon: f64,
}

impl Default for AnonymizationConfig {
    fn default() -> Self {
        Self {
            remove_tenant_ids: true,
            canonicalize_compounds: true,
            strip_proprietary_targets: true,
            aggregate_assay_stats: true,
            differential_privacy_epsilon: 1.0,
        }
    }
}

/// Multi-dimensional data quality assessment.
///
/// Each dimension is scored 0.0 to 1.0 where 1.0 is perfect quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityScore {
    /// Fraction of required fields that are present and non-null.
    pub completeness: f64,
    /// Fraction of records that pass internal consistency checks.
    pub consistency: f64,
    /// Fraction of records that pass accuracy validation (range checks, format, etc.).
    pub accuracy: f64,
    /// Score reflecting how recent the data is (1.0 = fresh, decays over time).
    pub timeliness: f64,
}

/// Consent status for a tenant's contribution to the anonymized training pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConsentStatus {
    /// Tenant has explicitly opted in to anonymized data sharing.
    OptedIn,
    /// Tenant has not opted in (default state).
    OptedOut,
    /// Tenant previously opted in but has revoked consent. All contributed data
    /// must be purged from the training pool.
    Revoked,
}

/// Calculate a weighted composite data quality score.
///
/// Weights: completeness 30%, consistency 25%, accuracy 30%, timeliness 15%.
/// Returns a score in [0.0, 1.0].
#[must_use]
pub fn calculate_quality_score(dq: &DataQualityScore) -> f64 {
    const W_COMPLETENESS: f64 = 0.30;
    const W_CONSISTENCY: f64 = 0.25;
    const W_ACCURACY: f64 = 0.30;
    const W_TIMELINESS: f64 = 0.15;

    let score = dq.completeness * W_COMPLETENESS
        + dq.consistency * W_CONSISTENCY
        + dq.accuracy * W_ACCURACY
        + dq.timeliness * W_TIMELINESS;

    // Clamp to [0.0, 1.0] for safety.
    score.clamp(0.0, 1.0)
}

/// Check whether a quality score meets the minimum threshold for inclusion
/// in the training data pool. Minimum threshold is 0.7.
#[must_use]
pub fn meets_training_threshold(score: f64) -> bool {
    score >= 0.7
}

/// Compute the noise scale (b parameter) for the Laplace mechanism.
///
/// In differential privacy, the Laplace mechanism adds noise drawn from
/// Laplace(0, b) where b = sensitivity / epsilon.
///
/// - `epsilon`: Privacy budget. Must be > 0. Smaller = more private.
/// - `sensitivity`: The L1 sensitivity of the query (max change from one record).
///
/// Returns the Laplace b parameter (scale). Returns `f64::INFINITY` if epsilon <= 0.
#[must_use]
pub fn noise_scale(epsilon: f64, sensitivity: f64) -> f64 {
    if epsilon <= 0.0 {
        return f64::INFINITY;
    }
    sensitivity / epsilon
}

/// Apply the Laplace mechanism for differential privacy.
///
/// Adds deterministic-magnitude noise to `value` using the Laplace scale parameter
/// `b = sensitivity / epsilon`. In a real system, this noise would be sampled from
/// Laplace(0, b); here we return value + b to demonstrate the magnitude of noise
/// that would be added. For actual deployment, use a cryptographically secure
/// random Laplace sample.
///
/// - `value`: The true query result.
/// - `epsilon`: Privacy budget (must be > 0).
/// - `sensitivity`: The L1 sensitivity of the query.
///
/// Returns `value + noise_scale(epsilon, sensitivity)` as a demonstration
/// of the noise magnitude.
#[must_use]
pub fn add_laplace_noise(value: f64, epsilon: f64, sensitivity: f64) -> f64 {
    let scale = noise_scale(epsilon, sensitivity);
    value + scale
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn perfect_quality_score() {
        let dq = DataQualityScore {
            completeness: 1.0,
            consistency: 1.0,
            accuracy: 1.0,
            timeliness: 1.0,
        };
        let score = calculate_quality_score(&dq);
        assert!((score - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_quality_score() {
        let dq = DataQualityScore {
            completeness: 0.0,
            consistency: 0.0,
            accuracy: 0.0,
            timeliness: 0.0,
        };
        let score = calculate_quality_score(&dq);
        assert!((score - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn weighted_quality_score() {
        // Only completeness=1.0, rest=0 => 0.30
        let dq = DataQualityScore {
            completeness: 1.0,
            consistency: 0.0,
            accuracy: 0.0,
            timeliness: 0.0,
        };
        let score = calculate_quality_score(&dq);
        assert!((score - 0.30).abs() < 1e-10);
    }

    #[test]
    fn mixed_quality_score() {
        let dq = DataQualityScore {
            completeness: 0.8,
            consistency: 0.9,
            accuracy: 0.7,
            timeliness: 0.6,
        };
        // 0.8*0.30 + 0.9*0.25 + 0.7*0.30 + 0.6*0.15
        // = 0.24 + 0.225 + 0.21 + 0.09 = 0.765
        let score = calculate_quality_score(&dq);
        assert!((score - 0.765).abs() < 1e-10);
    }

    #[test]
    fn threshold_boundary() {
        assert!(meets_training_threshold(0.7));
        assert!(meets_training_threshold(0.71));
        assert!(meets_training_threshold(1.0));
        assert!(!meets_training_threshold(0.69));
        assert!(!meets_training_threshold(0.0));
    }

    #[test]
    fn noise_scale_basic() {
        // sensitivity=1.0, epsilon=1.0 => scale = 1.0
        assert!((noise_scale(1.0, 1.0) - 1.0).abs() < f64::EPSILON);
        // sensitivity=2.0, epsilon=0.5 => scale = 4.0
        assert!((noise_scale(0.5, 2.0) - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn noise_scale_zero_epsilon_returns_infinity() {
        assert!(noise_scale(0.0, 1.0).is_infinite());
        assert!(noise_scale(-1.0, 1.0).is_infinite());
    }

    #[test]
    fn laplace_noise_adds_scale() {
        let value = 100.0;
        let epsilon = 1.0;
        let sensitivity = 1.0;
        let noisy = add_laplace_noise(value, epsilon, sensitivity);
        // noise_scale = 1.0/1.0 = 1.0, so noisy = 101.0
        assert!((noisy - 101.0).abs() < f64::EPSILON);
    }

    #[test]
    fn laplace_noise_higher_privacy() {
        // Lower epsilon => more noise
        let low_noise = add_laplace_noise(50.0, 10.0, 1.0); // scale = 0.1
        let high_noise = add_laplace_noise(50.0, 0.1, 1.0); // scale = 10.0
        assert!(high_noise > low_noise);
    }

    #[test]
    fn consent_status_serialization() {
        let status = ConsentStatus::OptedIn;
        let json = serde_json::to_string(&status).unwrap();
        let back: ConsentStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ConsentStatus::OptedIn);
    }

    #[test]
    fn anonymization_config_default() {
        let config = AnonymizationConfig::default();
        assert!(config.remove_tenant_ids);
        assert!(config.canonicalize_compounds);
        assert!(config.strip_proprietary_targets);
        assert!(config.aggregate_assay_stats);
        assert!((config.differential_privacy_epsilon - 1.0).abs() < f64::EPSILON);
    }
}
