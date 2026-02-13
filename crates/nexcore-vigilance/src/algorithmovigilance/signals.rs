//! # AI Signal Detection (ToV §56)
//!
//! Signal detection framework for AI/ML-enabled systems, including drift detection,
//! subgroup disparity analysis, failure mode clustering, and override pattern monitoring.
//!
//! # Signal Types (§56.1)
//!
//! | Signal Type | Definition | Detection Method |
//! |-------------|------------|------------------|
//! | Performance Drift | Accuracy degradation over time | Statistical process control |
//! | Subgroup Disparity | Differential performance by demographic | Stratified analysis |
//! | Failure Mode Cluster | Similar incidents clustering | Pattern recognition |
//! | Override Pattern | Systematic clinician rejection | Override rate monitoring |
//! | Context Shift | Performance change with deployment context | Multi-site comparison |
//!
//! # Example
//!
//! ```rust
//! use nexcore_vigilance::algorithmovigilance::signals::{
//!     AiSignalType, DriftIndicator, SubgroupDimension, AiSignal, AiSignalSeverity,
//! };
//!
//! let signal = AiSignal::new(AiSignalType::PerformanceDrift)
//!     .with_severity(AiSignalSeverity::High)
//!     .with_drift_indicator(DriftIndicator::ConceptDrift)
//!     .with_description("CUSUM exceeded control limit");
//!
//! assert!(signal.requires_immediate_review());
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// SIGNAL TYPES (T2-P)
// ============================================================================

/// AI-specific signal types (ToV §56.1).
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiSignalType {
    /// Performance degradation over time (statistical process control).
    PerformanceDrift = 1,
    /// Differential performance by demographic (stratified analysis).
    SubgroupDisparity = 2,
    /// Similar incidents clustering (pattern recognition).
    FailureModeCluster = 3,
    /// Systematic clinician rejection (override rate monitoring).
    OverridePattern = 4,
    /// Performance change with deployment context (multi-site comparison).
    ContextShift = 5,
}

impl AiSignalType {
    /// Get all signal types.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::PerformanceDrift,
            Self::SubgroupDisparity,
            Self::FailureModeCluster,
            Self::OverridePattern,
            Self::ContextShift,
        ]
    }

    /// Get the detection method for this signal type.
    #[must_use]
    pub const fn detection_method(&self) -> &'static str {
        match self {
            Self::PerformanceDrift => "Statistical process control (CUSUM, EWMA)",
            Self::SubgroupDisparity => "Stratified analysis",
            Self::FailureModeCluster => "Pattern recognition, clustering",
            Self::OverridePattern => "Override rate monitoring",
            Self::ContextShift => "Multi-site comparison",
        }
    }

    /// Get the typical U contribution for this signal type.
    ///
    /// # ToV §56.4
    #[must_use]
    pub const fn typical_u_contribution(&self) -> UContribution {
        match self {
            Self::PerformanceDrift => UContribution::Moderate, // Known risk, confirmed
            Self::SubgroupDisparity => UContribution::High,    // Fairness concern
            Self::FailureModeCluster => UContribution::High,   // Previously unseen pattern
            Self::OverridePattern => UContribution::Variable,  // Depends on validity
            Self::ContextShift => UContribution::Moderate,     // Context-dependent
        }
    }
}

impl std::fmt::Display for AiSignalType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PerformanceDrift => write!(f, "Performance Drift"),
            Self::SubgroupDisparity => write!(f, "Subgroup Disparity"),
            Self::FailureModeCluster => write!(f, "Failure Mode Cluster"),
            Self::OverridePattern => write!(f, "Override Pattern"),
            Self::ContextShift => write!(f, "Context Shift"),
        }
    }
}

/// U contribution level for signal types (ToV §56.4).
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UContribution {
    /// Low U contribution.
    Low = 1,
    /// Moderate U contribution.
    Moderate = 2,
    /// High U contribution (systematic, fairness concern).
    High = 3,
    /// Variable U contribution (depends on context).
    Variable = 4,
}

impl std::fmt::Display for UContribution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Moderate => write!(f, "Moderate"),
            Self::High => write!(f, "High"),
            Self::Variable => write!(f, "Variable"),
        }
    }
}

// ============================================================================
// DRIFT DETECTION (T2-P)
// ============================================================================

/// Drift detection indicators (ToV §56.2, §56.6).
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DriftIndicator {
    /// Input feature distribution shift (KL divergence, KS test).
    CovariateShift = 1,
    /// Output label distribution shift.
    ConceptDrift = 2,
    /// Model performance degradation over time.
    PerformanceDecay = 3,
    /// Calibration degradation (predicted vs actual probabilities).
    CalibrationDrift = 4,
    /// Prediction confidence distribution shift.
    ConfidenceDrift = 5,
}

impl DriftIndicator {
    /// Get the recommended detection method.
    #[must_use]
    pub const fn detection_method(&self) -> &'static str {
        match self {
            Self::CovariateShift => "KL divergence, Kolmogorov-Smirnov test",
            Self::ConceptDrift => "CUSUM, EWMA control charts",
            Self::PerformanceDecay => "Moving average AUC/accuracy tracking",
            Self::CalibrationDrift => "Brier score, calibration curve analysis",
            Self::ConfidenceDrift => "Confidence distribution histogram comparison",
        }
    }

    /// Get the recommended action when detected.
    #[must_use]
    pub const fn recommended_action(&self) -> &'static str {
        match self {
            Self::CovariateShift => "Review input pipeline; consider retraining",
            Self::ConceptDrift => "Immediate performance review; potential deployment pause",
            Self::PerformanceDecay => "Investigate root cause; schedule revalidation",
            Self::CalibrationDrift => "Recalibrate model; review probability thresholds",
            Self::ConfidenceDrift => "Investigate uncertainty estimation; consider retraining",
        }
    }
}

impl std::fmt::Display for DriftIndicator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CovariateShift => write!(f, "Covariate Shift"),
            Self::ConceptDrift => write!(f, "Concept Drift"),
            Self::PerformanceDecay => write!(f, "Performance Decay"),
            Self::CalibrationDrift => write!(f, "Calibration Drift"),
            Self::ConfidenceDrift => write!(f, "Confidence Drift"),
        }
    }
}

// ============================================================================
// SUBGROUP ANALYSIS (T2-P)
// ============================================================================

/// Subgroup dimensions for stratified analysis (ToV §56.3).
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubgroupDimension {
    /// Age group stratification.
    AgeGroup = 1,
    /// Sex stratification.
    Sex = 2,
    /// Race/ethnicity stratification.
    RaceEthnicity = 3,
    /// Comorbidity burden stratification.
    ComorbidityBurden = 4,
    /// Clinical setting stratification.
    ClinicalSetting = 5,
    /// Geographic region.
    GeographicRegion = 6,
    /// Insurance/payer type.
    PayerType = 7,
}

impl SubgroupDimension {
    /// Get the alert threshold for disparity (ToV §56.3).
    #[must_use]
    pub const fn alert_threshold_percent(&self) -> u8 {
        match self {
            Self::AgeGroup => 10,
            Self::Sex => 10,
            Self::RaceEthnicity => 5, // Lower threshold for fairness-critical
            Self::ComorbidityBurden => 15,
            Self::ClinicalSetting => 10,
            Self::GeographicRegion => 10,
            Self::PayerType => 10,
        }
    }

    /// Check if this is a fairness-critical dimension.
    #[must_use]
    pub const fn is_fairness_critical(&self) -> bool {
        matches!(self, Self::RaceEthnicity | Self::Sex)
    }
}

impl std::fmt::Display for SubgroupDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AgeGroup => write!(f, "Age Group"),
            Self::Sex => write!(f, "Sex"),
            Self::RaceEthnicity => write!(f, "Race/Ethnicity"),
            Self::ComorbidityBurden => write!(f, "Comorbidity Burden"),
            Self::ClinicalSetting => write!(f, "Clinical Setting"),
            Self::GeographicRegion => write!(f, "Geographic Region"),
            Self::PayerType => write!(f, "Payer Type"),
        }
    }
}

// ============================================================================
// SIGNAL SEVERITY (T2-P)
// ============================================================================

/// AI signal severity levels.
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum AiSignalSeverity {
    /// Informational only, no action required.
    Info = 1,
    /// Low severity, monitor.
    Low = 2,
    /// Moderate severity, investigate.
    Moderate = 3,
    /// High severity, immediate review required.
    High = 4,
    /// Critical severity, deployment pause recommended.
    Critical = 5,
}

impl AiSignalSeverity {
    /// Check if this severity requires immediate review.
    #[must_use]
    pub const fn requires_immediate_review(&self) -> bool {
        matches!(self, Self::High | Self::Critical)
    }

    /// Check if this severity recommends deployment pause.
    #[must_use]
    pub const fn recommends_pause(&self) -> bool {
        matches!(self, Self::Critical)
    }
}

impl Default for AiSignalSeverity {
    fn default() -> Self {
        Self::Info
    }
}

impl std::fmt::Display for AiSignalSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "Info"),
            Self::Low => write!(f, "Low"),
            Self::Moderate => write!(f, "Moderate"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

// ============================================================================
// DRIFT METRICS (T2-C)
// ============================================================================

/// CUSUM (Cumulative Sum) result for drift detection (ToV §56.6.1).
///
/// # Tier: T2-C
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CusumResult {
    /// Cumulative sum statistic (positive side).
    pub cusum_positive: f64,
    /// Cumulative sum statistic (negative side).
    pub cusum_negative: f64,
    /// Control limit threshold.
    pub control_limit: f64,
    /// Whether drift is detected.
    pub drift_detected: bool,
    /// Number of observations.
    pub observations: u32,
}

impl CusumResult {
    /// Create a new CUSUM result.
    #[must_use]
    pub fn new(cusum_positive: f64, cusum_negative: f64, control_limit: f64) -> Self {
        let drift_detected = cusum_positive > control_limit || cusum_negative.abs() > control_limit;
        Self {
            cusum_positive,
            cusum_negative,
            control_limit,
            drift_detected,
            observations: 0,
        }
    }

    /// Set number of observations.
    #[must_use]
    pub const fn with_observations(mut self, n: u32) -> Self {
        self.observations = n;
        self
    }
}

/// KL Divergence result for covariate shift detection.
///
/// # Tier: T2-C
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KlDivergenceResult {
    /// KL divergence value (bits).
    pub divergence: f64,
    /// Threshold for alerting.
    pub threshold: f64,
    /// Whether shift is detected.
    pub shift_detected: bool,
    /// Feature name (if applicable).
    pub feature: Option<String>,
}

impl KlDivergenceResult {
    /// Create a new KL divergence result.
    #[must_use]
    pub fn new(divergence: f64, threshold: f64) -> Self {
        Self {
            divergence,
            threshold,
            shift_detected: divergence > threshold,
            feature: None,
        }
    }

    /// Set feature name.
    #[must_use]
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.feature = Some(feature.into());
        self
    }
}

/// Subgroup disparity result (ToV §56.3).
///
/// # Tier: T2-C
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubgroupDisparityResult {
    /// Subgroup dimension analyzed.
    pub dimension: SubgroupDimension,
    /// Reference subgroup value.
    pub reference_group: String,
    /// Comparison subgroup value.
    pub comparison_group: String,
    /// Reference group metric (e.g., sensitivity).
    pub reference_metric: f64,
    /// Comparison group metric.
    pub comparison_metric: f64,
    /// Absolute disparity (percentage points).
    pub disparity_percent: f64,
    /// Whether disparity exceeds threshold.
    pub exceeds_threshold: bool,
}

impl SubgroupDisparityResult {
    /// Create a new subgroup disparity result.
    #[must_use]
    pub fn new(
        dimension: SubgroupDimension,
        reference_group: impl Into<String>,
        comparison_group: impl Into<String>,
        reference_metric: f64,
        comparison_metric: f64,
    ) -> Self {
        let disparity_percent =
            ((comparison_metric - reference_metric) / reference_metric * 100.0).abs();
        let threshold = f64::from(dimension.alert_threshold_percent());
        Self {
            dimension,
            reference_group: reference_group.into(),
            comparison_group: comparison_group.into(),
            reference_metric,
            comparison_metric,
            disparity_percent,
            exceeds_threshold: disparity_percent > threshold,
        }
    }
}

// ============================================================================
// AI SIGNAL (T3)
// ============================================================================

/// Complete AI signal record (ToV §56).
///
/// # Tier: T3
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AiSignal {
    /// Signal type.
    pub signal_type: AiSignalType,
    /// Severity level.
    pub severity: AiSignalSeverity,
    /// Drift indicator (if applicable).
    pub drift_indicator: Option<DriftIndicator>,
    /// Subgroup dimension (if disparity signal).
    pub subgroup_dimension: Option<SubgroupDimension>,
    /// Description of the signal.
    pub description: Option<String>,
    /// Recommended action.
    pub recommended_action: Option<String>,
    /// U contribution estimate.
    pub u_contribution: UContribution,
    /// Detection timestamp (ISO 8601).
    pub detected_at: Option<String>,
    /// Algorithm identifier.
    pub algorithm_id: Option<String>,
    /// Correlated signal count (for aggregation).
    pub correlated_signal_count: u32,
}

impl AiSignal {
    /// Create a new AI signal.
    #[must_use]
    pub fn new(signal_type: AiSignalType) -> Self {
        Self {
            signal_type,
            severity: AiSignalSeverity::default(),
            drift_indicator: None,
            subgroup_dimension: None,
            description: None,
            recommended_action: None,
            u_contribution: signal_type.typical_u_contribution(),
            detected_at: None,
            algorithm_id: None,
            correlated_signal_count: 1,
        }
    }

    /// Set severity.
    #[must_use]
    pub const fn with_severity(mut self, severity: AiSignalSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set drift indicator.
    #[must_use]
    pub fn with_drift_indicator(mut self, indicator: DriftIndicator) -> Self {
        self.drift_indicator = Some(indicator);
        if self.recommended_action.is_none() {
            self.recommended_action = Some(indicator.recommended_action().to_string());
        }
        self
    }

    /// Set subgroup dimension.
    #[must_use]
    pub const fn with_subgroup_dimension(mut self, dimension: SubgroupDimension) -> Self {
        self.subgroup_dimension = Some(dimension);
        self
    }

    /// Set description.
    #[must_use]
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set recommended action.
    #[must_use]
    pub fn with_action(mut self, action: impl Into<String>) -> Self {
        self.recommended_action = Some(action.into());
        self
    }

    /// Set algorithm ID.
    #[must_use]
    pub fn with_algorithm_id(mut self, id: impl Into<String>) -> Self {
        self.algorithm_id = Some(id.into());
        self
    }

    /// Set correlated signal count.
    #[must_use]
    pub const fn with_correlated_signals(mut self, count: u32) -> Self {
        self.correlated_signal_count = count;
        self
    }

    /// Check if signal requires immediate review.
    #[must_use]
    pub const fn requires_immediate_review(&self) -> bool {
        self.severity.requires_immediate_review()
    }

    /// Check if signal recommends deployment pause.
    #[must_use]
    pub const fn recommends_pause(&self) -> bool {
        self.severity.recommends_pause()
    }

    /// Compute aggregate U based on correlated signals (ToV §56.4).
    ///
    /// ```text
    /// U_aggregate = max(U_individual) + log₂(1 + count_correlated)
    /// ```
    #[must_use]
    pub fn aggregate_u(&self, base_u: f64) -> f64 {
        base_u + (1.0 + f64::from(self.correlated_signal_count)).log2()
    }
}

// ============================================================================
// SIGNAL AGGREGATION (T3)
// ============================================================================

/// Aggregated AI signals for an algorithm (ToV §56.4).
///
/// # Tier: T3
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AiSignalAggregate {
    /// Algorithm identifier.
    pub algorithm_id: String,
    /// Individual signals.
    pub signals: Vec<AiSignal>,
    /// Maximum individual U.
    pub max_u: f64,
    /// Aggregate U (with correlation factor).
    pub aggregate_u: f64,
    /// Overall severity.
    pub overall_severity: AiSignalSeverity,
}

impl AiSignalAggregate {
    /// Create a new aggregate for an algorithm.
    #[must_use]
    pub fn new(algorithm_id: impl Into<String>) -> Self {
        Self {
            algorithm_id: algorithm_id.into(),
            signals: Vec::new(),
            max_u: 0.0,
            aggregate_u: 0.0,
            overall_severity: AiSignalSeverity::Info,
        }
    }

    /// Add a signal with its U value.
    pub fn add_signal(&mut self, signal: AiSignal, u_value: f64) {
        if u_value > self.max_u {
            self.max_u = u_value;
        }
        if signal.severity > self.overall_severity {
            self.overall_severity = signal.severity;
        }
        self.signals.push(signal);
        self.recompute_aggregate();
    }

    /// Recompute aggregate U.
    fn recompute_aggregate(&mut self) {
        let correlated_count = self.signals.len();
        if correlated_count > 0 {
            self.aggregate_u = self.max_u + (1.0 + correlated_count as f64).log2();
        }
    }

    /// Get signal count.
    #[must_use]
    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    /// Check if any signal requires immediate review.
    #[must_use]
    pub fn requires_immediate_review(&self) -> bool {
        self.overall_severity.requires_immediate_review()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_types() {
        assert_eq!(AiSignalType::all().len(), 5);
        assert_eq!(
            AiSignalType::PerformanceDrift.detection_method(),
            "Statistical process control (CUSUM, EWMA)"
        );
    }

    #[test]
    fn test_drift_indicators() {
        assert_eq!(
            DriftIndicator::ConceptDrift.recommended_action(),
            "Immediate performance review; potential deployment pause"
        );
    }

    #[test]
    fn test_subgroup_thresholds() {
        // Race/ethnicity has lower threshold for fairness
        assert_eq!(
            SubgroupDimension::RaceEthnicity.alert_threshold_percent(),
            5
        );
        assert_eq!(SubgroupDimension::AgeGroup.alert_threshold_percent(), 10);
        assert!(SubgroupDimension::RaceEthnicity.is_fairness_critical());
        assert!(!SubgroupDimension::AgeGroup.is_fairness_critical());
    }

    #[test]
    fn test_severity_levels() {
        assert!(!AiSignalSeverity::Moderate.requires_immediate_review());
        assert!(AiSignalSeverity::High.requires_immediate_review());
        assert!(!AiSignalSeverity::High.recommends_pause());
        assert!(AiSignalSeverity::Critical.recommends_pause());
    }

    #[test]
    fn test_cusum_result() {
        let result = CusumResult::new(5.5, -2.0, 5.0).with_observations(100);
        assert!(result.drift_detected); // 5.5 > 5.0
        assert_eq!(result.observations, 100);
    }

    #[test]
    fn test_kl_divergence_result() {
        let result = KlDivergenceResult::new(0.15, 0.1).with_feature("age");
        assert!(result.shift_detected); // 0.15 > 0.1
        assert_eq!(result.feature, Some("age".to_string()));
    }

    #[test]
    fn test_subgroup_disparity() {
        let result = SubgroupDisparityResult::new(
            SubgroupDimension::Sex,
            "Male",
            "Female",
            0.85, // reference sensitivity
            0.72, // comparison sensitivity
        );
        // Disparity = |0.72 - 0.85| / 0.85 * 100 = 15.3%
        assert!(result.disparity_percent > 15.0);
        assert!(result.exceeds_threshold); // 15.3% > 10%
    }

    #[test]
    fn test_ai_signal_builder() {
        let signal = AiSignal::new(AiSignalType::PerformanceDrift)
            .with_severity(AiSignalSeverity::High)
            .with_drift_indicator(DriftIndicator::ConceptDrift)
            .with_description("CUSUM exceeded control limit")
            .with_algorithm_id("sepsis-predictor-v1.2");

        assert!(signal.requires_immediate_review());
        assert!(!signal.recommends_pause());
        assert_eq!(signal.drift_indicator, Some(DriftIndicator::ConceptDrift));
    }

    #[test]
    fn test_aggregate_u_computation() {
        let signal = AiSignal::new(AiSignalType::FailureModeCluster).with_correlated_signals(3);

        // U_aggregate = base_u + log₂(1 + 3) = 10 + log₂(4) = 10 + 2 = 12
        let aggregate = signal.aggregate_u(10.0);
        assert!((aggregate - 12.0).abs() < 0.001);
    }

    #[test]
    fn test_signal_aggregate() {
        let mut aggregate = AiSignalAggregate::new("test-algo");

        aggregate.add_signal(
            AiSignal::new(AiSignalType::PerformanceDrift).with_severity(AiSignalSeverity::Moderate),
            5.0,
        );
        aggregate.add_signal(
            AiSignal::new(AiSignalType::SubgroupDisparity).with_severity(AiSignalSeverity::High),
            8.0,
        );

        assert_eq!(aggregate.signal_count(), 2);
        assert_eq!(aggregate.max_u, 8.0);
        // aggregate_u = 8.0 + log₂(1 + 2) = 8.0 + 1.585 ≈ 9.585
        assert!(aggregate.aggregate_u > 9.5 && aggregate.aggregate_u < 9.6);
        assert_eq!(aggregate.overall_severity, AiSignalSeverity::High);
    }
}
