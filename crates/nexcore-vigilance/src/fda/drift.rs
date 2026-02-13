//! Data Drift Detection and Monitoring
//!
//! ## T1 Grounding
//!
//! - **DataDrift**: ∂ (Boundary) + ν (Frequency)
//!   - Boundary: Deviation from training distribution
//!   - Frequency: Rate of drift over time
//!
//! - **DriftMagnitude**: N (Quantity) + κ (Comparison)
//!   - Quantified deviation with threshold comparison

use serde::{Deserialize, Serialize};
use std::fmt;

/// Magnitude of drift from training distribution
///
/// T1 Grounding: N (Quantity) + κ (Comparison)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct DriftMagnitude(f64);

impl DriftMagnitude {
    /// Creates a new drift magnitude
    ///
    /// ## Validation
    ///
    /// - Must be non-negative
    /// - Must be finite
    pub fn new(value: f64) -> Result<Self, DriftError> {
        if !value.is_finite() {
            return Err(DriftError::NonFinite);
        }
        if value < 0.0 {
            return Err(DriftError::Negative);
        }
        Ok(Self(value))
    }

    /// Returns the raw magnitude value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Returns true if drift exceeds threshold
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn exceeds(&self, threshold: f64) -> bool {
        self.0 > threshold
    }

    /// Classifies drift severity
    ///
    /// T1 Grounding: κ (Comparison) — Threshold-based classification
    pub fn severity(&self) -> DriftSeverity {
        match self.0 {
            x if x < 0.05 => DriftSeverity::Negligible,
            x if x < 0.15 => DriftSeverity::Minor,
            x if x < 0.30 => DriftSeverity::Moderate,
            x if x < 0.50 => DriftSeverity::Major,
            _ => DriftSeverity::Critical,
        }
    }
}

impl fmt::Display for DriftMagnitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.4}", self.0)
    }
}

/// Severity classification for drift
///
/// T1 Grounding: κ (Comparison) — Ordered severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum DriftSeverity {
    /// < 5% drift
    Negligible,
    /// 5-15% drift
    Minor,
    /// 15-30% drift
    Moderate,
    /// 30-50% drift
    Major,
    /// > 50% drift
    Critical,
}

impl DriftSeverity {
    /// Returns true if severity requires immediate action
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn requires_action(&self) -> bool {
        matches!(self, Self::Major | Self::Critical)
    }

    /// Returns recommended action interval in days
    ///
    /// T1 Grounding: N (Quantity) + ν (Frequency)
    pub fn action_interval_days(&self) -> Option<u32> {
        match self {
            Self::Negligible => None,
            Self::Minor => Some(90),    // Quarterly review
            Self::Moderate => Some(30), // Monthly review
            Self::Major => Some(7),     // Weekly review
            Self::Critical => Some(1),  // Daily review
        }
    }
}

impl fmt::Display for DriftSeverity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Negligible => write!(f, "Negligible"),
            Self::Minor => write!(f, "Minor"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Major => write!(f, "Major"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Type of drift detected
///
/// T1 Grounding: σ (Sequence) — Different drift categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DriftType {
    /// Covariate drift (X distribution changes)
    Covariate,
    /// Prior probability drift (P(Y) changes)
    Prior,
    /// Concept drift (P(Y|X) changes)
    Concept,
}

impl fmt::Display for DriftType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Covariate => write!(f, "Covariate"),
            Self::Prior => write!(f, "Prior"),
            Self::Concept => write!(f, "Concept"),
        }
    }
}

/// Detected data drift from training distribution
///
/// T1 Grounding: ∂ (Boundary) + ν (Frequency)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataDrift {
    /// Type of drift
    drift_type: DriftType,
    /// Magnitude of drift
    magnitude: DriftMagnitude,
    /// Feature or variable affected
    affected_feature: String,
    /// Timestamp of detection (Unix epoch seconds)
    detected_at: u64,
}

impl DataDrift {
    pub fn new(
        drift_type: DriftType,
        magnitude: DriftMagnitude,
        affected_feature: impl Into<String>,
        detected_at: u64,
    ) -> Self {
        Self {
            drift_type,
            magnitude,
            affected_feature: affected_feature.into(),
            detected_at,
        }
    }

    pub fn drift_type(&self) -> DriftType {
        self.drift_type
    }

    pub fn magnitude(&self) -> DriftMagnitude {
        self.magnitude
    }

    pub fn severity(&self) -> DriftSeverity {
        self.magnitude.severity()
    }

    pub fn affected_feature(&self) -> &str {
        &self.affected_feature
    }

    pub fn detected_at(&self) -> u64 {
        self.detected_at
    }

    /// Returns true if drift requires immediate remediation
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn requires_action(&self) -> bool {
        self.severity().requires_action()
    }
}

impl fmt::Display for DataDrift {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} drift in '{}': {} ({})",
            self.drift_type,
            self.affected_feature,
            self.magnitude,
            self.severity()
        )
    }
}

/// Drift detection strategy
///
/// T1 Grounding: μ (Mapping) — Statistical test selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DriftDetector {
    /// Kolmogorov-Smirnov test
    KolmogorovSmirnov,
    /// Population Stability Index
    Psi,
    /// Kullback-Leibler divergence
    KullbackLeibler,
    /// Chi-squared test
    ChiSquared,
}

impl DriftDetector {
    /// Returns the recommended significance threshold
    ///
    /// T1 Grounding: N (Quantity)
    pub fn default_threshold(&self) -> f64 {
        match self {
            Self::KolmogorovSmirnov => 0.05, // p-value
            Self::Psi => 0.25,               // PSI threshold
            Self::KullbackLeibler => 0.10,   // KL divergence
            Self::ChiSquared => 0.05,        // p-value
        }
    }

    /// Returns true if this detector is suitable for continuous features
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn supports_continuous(&self) -> bool {
        matches!(
            self,
            Self::KolmogorovSmirnov | Self::Psi | Self::KullbackLeibler
        )
    }

    /// Returns true if this detector is suitable for categorical features
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn supports_categorical(&self) -> bool {
        matches!(self, Self::ChiSquared | Self::Psi)
    }
}

impl fmt::Display for DriftDetector {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::KolmogorovSmirnov => write!(f, "Kolmogorov-Smirnov"),
            Self::Psi => write!(f, "PSI"),
            Self::KullbackLeibler => write!(f, "KL Divergence"),
            Self::ChiSquared => write!(f, "Chi-Squared"),
        }
    }
}

/// Errors in drift detection
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriftError {
    NonFinite,
    Negative,
}

impl fmt::Display for DriftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite => write!(f, "Drift magnitude must be finite"),
            Self::Negative => write!(f, "Drift magnitude cannot be negative"),
        }
    }
}

impl std::error::Error for DriftError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_drift_magnitude_valid() {
        let mag = DriftMagnitude::new(0.15);
        assert!(mag.is_ok());
        let m = mag.ok().unwrap_or_else(|| panic!("Should succeed"));
        assert_eq!(m.value(), 0.15);
    }

    #[test]
    fn test_drift_magnitude_negative() {
        let mag = DriftMagnitude::new(-0.1);
        assert!(matches!(mag, Err(DriftError::Negative)));
    }

    #[test]
    fn test_drift_magnitude_non_finite() {
        let mag = DriftMagnitude::new(f64::NAN);
        assert!(matches!(mag, Err(DriftError::NonFinite)));

        let mag2 = DriftMagnitude::new(f64::INFINITY);
        assert!(matches!(mag2, Err(DriftError::NonFinite)));
    }

    #[test]
    fn test_drift_magnitude_exceeds() {
        let mag = DriftMagnitude::new(0.3)
            .ok()
            .unwrap_or_else(|| panic!("Should succeed"));
        assert!(mag.exceeds(0.2));
        assert!(!mag.exceeds(0.4));
    }

    #[test]
    fn test_drift_severity_classification() {
        let negligible = DriftMagnitude::new(0.03)
            .ok()
            .unwrap_or_else(|| panic!("Should succeed"));
        assert_eq!(negligible.severity(), DriftSeverity::Negligible);

        let minor = DriftMagnitude::new(0.10)
            .ok()
            .unwrap_or_else(|| panic!("Should succeed"));
        assert_eq!(minor.severity(), DriftSeverity::Minor);

        let moderate = DriftMagnitude::new(0.25)
            .ok()
            .unwrap_or_else(|| panic!("Should succeed"));
        assert_eq!(moderate.severity(), DriftSeverity::Moderate);

        let major = DriftMagnitude::new(0.40)
            .ok()
            .unwrap_or_else(|| panic!("Should succeed"));
        assert_eq!(major.severity(), DriftSeverity::Major);

        let critical = DriftMagnitude::new(0.60)
            .ok()
            .unwrap_or_else(|| panic!("Should succeed"));
        assert_eq!(critical.severity(), DriftSeverity::Critical);
    }

    #[test]
    fn test_drift_severity_requires_action() {
        assert!(!DriftSeverity::Negligible.requires_action());
        assert!(!DriftSeverity::Minor.requires_action());
        assert!(!DriftSeverity::Moderate.requires_action());
        assert!(DriftSeverity::Major.requires_action());
        assert!(DriftSeverity::Critical.requires_action());
    }

    #[test]
    fn test_drift_severity_action_interval() {
        assert_eq!(DriftSeverity::Negligible.action_interval_days(), None);
        assert_eq!(DriftSeverity::Minor.action_interval_days(), Some(90));
        assert_eq!(DriftSeverity::Moderate.action_interval_days(), Some(30));
        assert_eq!(DriftSeverity::Major.action_interval_days(), Some(7));
        assert_eq!(DriftSeverity::Critical.action_interval_days(), Some(1));
    }

    #[test]
    fn test_data_drift_creation() {
        let mag = DriftMagnitude::new(0.35)
            .ok()
            .unwrap_or_else(|| panic!("Should succeed"));
        let drift = DataDrift::new(DriftType::Covariate, mag, "age", 1738713600);

        assert_eq!(drift.drift_type(), DriftType::Covariate);
        assert_eq!(drift.magnitude().value(), 0.35);
        assert_eq!(drift.severity(), DriftSeverity::Major);
        assert!(drift.requires_action());
    }

    #[test]
    fn test_drift_detector_thresholds() {
        assert_eq!(DriftDetector::KolmogorovSmirnov.default_threshold(), 0.05);
        assert_eq!(DriftDetector::Psi.default_threshold(), 0.25);
        assert_eq!(DriftDetector::KullbackLeibler.default_threshold(), 0.10);
        assert_eq!(DriftDetector::ChiSquared.default_threshold(), 0.05);
    }

    #[test]
    fn test_drift_detector_feature_support() {
        assert!(DriftDetector::KolmogorovSmirnov.supports_continuous());
        assert!(!DriftDetector::KolmogorovSmirnov.supports_categorical());

        assert!(DriftDetector::ChiSquared.supports_categorical());
        assert!(!DriftDetector::ChiSquared.supports_continuous());

        assert!(DriftDetector::Psi.supports_continuous());
        assert!(DriftDetector::Psi.supports_categorical());
    }

    #[test]
    fn test_drift_display() {
        let mag = DriftMagnitude::new(0.42)
            .ok()
            .unwrap_or_else(|| panic!("Should succeed"));
        let drift = DataDrift::new(DriftType::Concept, mag, "outcome_probability", 0);
        let s = drift.to_string();
        assert!(s.contains("Concept"));
        assert!(s.contains("outcome_probability"));
        assert!(s.contains("Major"));
    }
}
