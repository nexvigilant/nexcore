//! FDA Credibility Assessment Metrics
//!
//! ## T1 Primitive Foundation
//!
//! | Type | T1 Grounding | Description |
//! |------|:------------:|-------------|
//! | CredibilityScore | N | 0-10 composite score |
//! | CredibilityRating | κ | Threshold-based rating |
//! | AssessmentMetrics | Σ | Aggregated assessment data |
//! | EvidenceDistribution | N+ν | Quality counts by type |
//! | RiskDistribution | N+κ | Risk level frequencies |
//! | DriftHistory | ν+σ | Temporal drift patterns |

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use super::{EvidenceQuality, EvidenceType, RiskLevel};

// ============================================================================
// Credibility Score (0-10 composite)
// ============================================================================

/// Credibility score on 0-10 scale
///
/// T1 Grounding: N (Quantity)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct CredibilityScore(f64);

impl CredibilityScore {
    /// Create a new score, clamped to [0, 10]
    pub fn new(value: f64) -> Self {
        Self(value.clamp(0.0, 10.0))
    }

    /// Get the raw value
    pub fn value(&self) -> f64 {
        self.0
    }

    /// Convert to rating
    ///
    /// T1 Grounding: κ (Comparison against thresholds)
    pub fn rating(&self) -> CredibilityRating {
        CredibilityRating::from_score(*self)
    }

    /// Zero score (failed assessment)
    pub fn zero() -> Self {
        Self(0.0)
    }

    /// Perfect score
    pub fn perfect() -> Self {
        Self(10.0)
    }
}

impl fmt::Display for CredibilityScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.2}/10", self.0)
    }
}

impl Default for CredibilityScore {
    fn default() -> Self {
        Self::zero()
    }
}

// ============================================================================
// Credibility Rating (threshold bands)
// ============================================================================

/// Credibility rating derived from score
///
/// T1 Grounding: κ (Comparison)
///
/// | Rating | Score Range | Meaning |
/// |--------|-------------|---------|
/// | Critical | 0-2 | Reject — major deficiencies |
/// | Weak | 2-4 | Needs significant revision |
/// | Adequate | 4-6 | Minimum acceptable |
/// | Good | 6-8 | Above requirements |
/// | Excellent | 8-10 | Exemplary credibility |
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CredibilityRating {
    /// 0-2: Major deficiencies, reject
    Critical,
    /// 2-4: Needs significant revision
    Weak,
    /// 4-6: Minimum acceptable
    Adequate,
    /// 6-8: Above requirements
    Good,
    /// 8-10: Exemplary
    Excellent,
}

impl CredibilityRating {
    /// Derive rating from score
    ///
    /// T1 Grounding: κ (Comparison against thresholds)
    pub fn from_score(score: CredibilityScore) -> Self {
        let v = score.value();
        if v < 2.0 {
            Self::Critical
        } else if v < 4.0 {
            Self::Weak
        } else if v < 6.0 {
            Self::Adequate
        } else if v < 8.0 {
            Self::Good
        } else {
            Self::Excellent
        }
    }

    /// Check if rating passes minimum threshold (Adequate or better)
    ///
    /// T1 Grounding: κ (Comparison)
    pub fn is_acceptable(&self) -> bool {
        matches!(self, Self::Adequate | Self::Good | Self::Excellent)
    }

    /// Get recommended action based on rating
    pub fn recommended_action(&self) -> &'static str {
        match self {
            Self::Critical => "HALT: Major revision required before resubmission",
            Self::Weak => "REVISE: Address significant gaps in evidence or documentation",
            Self::Adequate => "PROCEED: Meets minimum requirements, consider improvements",
            Self::Good => "APPROVE: Strong credibility, minor enhancements optional",
            Self::Excellent => "EXEMPLARY: Model for future submissions",
        }
    }
}

impl fmt::Display for CredibilityRating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Critical => write!(f, "Critical"),
            Self::Weak => write!(f, "Weak"),
            Self::Adequate => write!(f, "Adequate"),
            Self::Good => write!(f, "Good"),
            Self::Excellent => write!(f, "Excellent"),
        }
    }
}

// ============================================================================
// Evidence Distribution
// ============================================================================

/// Distribution of evidence by type and quality
///
/// T1 Grounding: N (Quantity) + ν (Frequency)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvidenceDistribution {
    /// Count by evidence type
    by_type: HashMap<String, usize>,
    /// Count by quality level
    by_quality: HashMap<String, usize>,
    /// Total evidence items
    total: usize,
    /// High-quality count
    high_quality_count: usize,
}

impl EvidenceDistribution {
    /// Create empty distribution
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an evidence item
    pub fn record(&mut self, evidence_type: &EvidenceType, quality: EvidenceQuality) {
        let type_key = format!("{}", evidence_type);
        *self.by_type.entry(type_key).or_insert(0) += 1;

        let quality_key = format!("{:?}", quality);
        *self.by_quality.entry(quality_key).or_insert(0) += 1;

        self.total += 1;
        if quality == EvidenceQuality::High {
            self.high_quality_count += 1;
        }
    }

    /// Get total evidence count
    pub fn total(&self) -> usize {
        self.total
    }

    /// Get high-quality evidence count
    pub fn high_quality_count(&self) -> usize {
        self.high_quality_count
    }

    /// Get high-quality ratio
    ///
    /// T1 Grounding: N (Quantity ratio)
    pub fn high_quality_ratio(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.high_quality_count as f64 / self.total as f64
        }
    }

    /// Get distribution by type
    pub fn by_type(&self) -> &HashMap<String, usize> {
        &self.by_type
    }

    /// Get distribution by quality
    pub fn by_quality(&self) -> &HashMap<String, usize> {
        &self.by_quality
    }
}

// ============================================================================
// Risk Distribution
// ============================================================================

/// Distribution of assessments by risk level
///
/// T1 Grounding: N (Quantity) + κ (Comparison)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RiskDistribution {
    /// Count at each risk level
    counts: HashMap<String, usize>,
    /// Total assessments
    total: usize,
}

impl RiskDistribution {
    /// Create empty distribution
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a risk level
    pub fn record(&mut self, level: RiskLevel) {
        let key = level.to_string();
        *self.counts.entry(key).or_insert(0) += 1;
        self.total += 1;
    }

    /// Get count for a risk level
    pub fn count(&self, level: RiskLevel) -> usize {
        self.counts.get(&level.to_string()).copied().unwrap_or(0)
    }

    /// Get total assessments
    pub fn total(&self) -> usize {
        self.total
    }

    /// Get high-risk ratio
    ///
    /// T1 Grounding: N (Quantity ratio)
    pub fn high_risk_ratio(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            self.count(RiskLevel::High) as f64 / self.total as f64
        }
    }

    /// Get distribution map
    pub fn distribution(&self) -> &HashMap<String, usize> {
        &self.counts
    }
}

// ============================================================================
// Assessment Metrics (aggregate)
// ============================================================================

/// Aggregated FDA assessment metrics
///
/// T1 Grounding: Σ (Sum) — Aggregation of multiple measurements
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssessmentMetrics {
    /// Total assessments started
    pub assessments_started: usize,
    /// Total assessments completed
    pub assessments_completed: usize,
    /// Assessments approved
    pub assessments_approved: usize,
    /// Assessments rejected
    pub assessments_rejected: usize,
    /// Assessments needing revision
    pub assessments_revision: usize,
    /// Evidence distribution
    pub evidence: EvidenceDistribution,
    /// Risk distribution
    pub risk: RiskDistribution,
    /// Average completion time (seconds)
    pub avg_completion_time_secs: f64,
    /// Drift alerts triggered
    pub drift_alerts: usize,
}

impl AssessmentMetrics {
    /// Create new metrics tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate approval rate
    ///
    /// T1 Grounding: N (Quantity ratio)
    pub fn approval_rate(&self) -> f64 {
        if self.assessments_completed == 0 {
            0.0
        } else {
            self.assessments_approved as f64 / self.assessments_completed as f64
        }
    }

    /// Calculate completion rate
    ///
    /// T1 Grounding: N (Quantity ratio)
    pub fn completion_rate(&self) -> f64 {
        if self.assessments_started == 0 {
            0.0
        } else {
            self.assessments_completed as f64 / self.assessments_started as f64
        }
    }

    /// Calculate overall health score
    ///
    /// Formula: (approval_rate × 0.35) + (completion_rate × 0.25) + (high_quality_ratio × 0.25) + (low_risk_ratio × 0.15)
    ///
    /// T1 Grounding: N (Weighted sum)
    pub fn health_score(&self) -> CredibilityScore {
        let approval = self.approval_rate();
        let completion = self.completion_rate();
        let quality = self.evidence.high_quality_ratio();
        let low_risk = 1.0 - self.risk.high_risk_ratio();

        let score = (approval * 0.35 + completion * 0.25 + quality * 0.25 + low_risk * 0.15) * 10.0;

        CredibilityScore::new(score)
    }
}

// ============================================================================
// Credibility Score Calculator
// ============================================================================

/// Input parameters for credibility score calculation
///
/// T1 Grounding: N (Quantities for weighted formula)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CredibilityInput {
    /// Evidence quality score (0-1): high_quality_count / required
    pub evidence_quality: f64,
    /// Fit-for-use score (0-1): criteria_passed / 6
    pub fit_for_use: f64,
    /// Risk mitigation score (0-1): controls_implemented / required
    pub risk_mitigation: f64,
    /// Documentation score (0-1): sections_complete / total
    pub documentation: f64,
}

impl CredibilityInput {
    /// Create from raw counts
    pub fn from_counts(
        high_quality_evidence: usize,
        required_evidence: usize,
        fit_criteria_passed: usize,
        risk_controls: usize,
        required_controls: usize,
        doc_sections: usize,
        total_sections: usize,
    ) -> Self {
        Self {
            evidence_quality: if required_evidence == 0 {
                0.0
            } else {
                (high_quality_evidence as f64 / required_evidence as f64).min(1.0)
            },
            fit_for_use: fit_criteria_passed as f64 / 6.0,
            risk_mitigation: if required_controls == 0 {
                1.0
            } else {
                (risk_controls as f64 / required_controls as f64).min(1.0)
            },
            documentation: if total_sections == 0 {
                0.0
            } else {
                doc_sections as f64 / total_sections as f64
            },
        }
    }

    /// Calculate composite credibility score
    ///
    /// Formula: (evidence × 0.30) + (fit_for_use × 0.25) + (risk × 0.25) + (docs × 0.20)
    ///
    /// T1 Grounding: N (Weighted linear combination)
    pub fn calculate_score(&self) -> CredibilityScore {
        let score = (self.evidence_quality * 0.30
            + self.fit_for_use * 0.25
            + self.risk_mitigation * 0.25
            + self.documentation * 0.20)
            * 10.0;

        CredibilityScore::new(score)
    }
}

// ============================================================================
// Drift History
// ============================================================================

/// Historical drift measurement
///
/// T1 Grounding: N (Quantity) + π (Persistence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftMeasurement {
    /// Timestamp (Unix epoch seconds)
    pub timestamp: u64,
    /// Drift percentage (0-100)
    pub drift_percent: f64,
    /// Severity level
    pub severity: String,
    /// Model identifier
    pub model_id: String,
}

/// Drift history tracker
///
/// T1 Grounding: ν (Frequency) + σ (Sequence)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DriftHistory {
    /// Historical measurements
    measurements: Vec<DriftMeasurement>,
    /// Alert count by severity
    alerts: HashMap<String, usize>,
}

impl DriftHistory {
    /// Create new history tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a drift measurement
    pub fn record(&mut self, measurement: DriftMeasurement) {
        let severity = measurement.severity.clone();
        self.measurements.push(measurement);

        if severity != "NONE" {
            *self.alerts.entry(severity).or_insert(0) += 1;
        }
    }

    /// Get total measurement count
    pub fn measurement_count(&self) -> usize {
        self.measurements.len()
    }

    /// Get alert count by severity
    pub fn alert_count(&self, severity: &str) -> usize {
        self.alerts.get(severity).copied().unwrap_or(0)
    }

    /// Get total alerts (non-NONE)
    pub fn total_alerts(&self) -> usize {
        self.alerts.values().sum()
    }

    /// Get recent measurements (last N)
    pub fn recent(&self, n: usize) -> &[DriftMeasurement] {
        let start = self.measurements.len().saturating_sub(n);
        &self.measurements[start..]
    }

    /// Calculate trend (average drift over recent measurements)
    ///
    /// T1 Grounding: N (Average quantity)
    pub fn trend(&self, window: usize) -> f64 {
        let recent = self.recent(window);
        if recent.is_empty() {
            return 0.0;
        }
        recent.iter().map(|m| m.drift_percent).sum::<f64>() / recent.len() as f64
    }

    /// Check if drift is worsening
    ///
    /// T1 Grounding: κ (Comparison of trend vs threshold)
    pub fn is_worsening(&self, threshold: f64) -> bool {
        if self.measurements.len() < 4 {
            return false;
        }
        let recent_trend = self.trend(2);
        let older_trend = {
            let older: Vec<_> = self.measurements.iter().rev().skip(2).take(2).collect();
            if older.is_empty() {
                return false;
            }
            older.iter().map(|m| m.drift_percent).sum::<f64>() / older.len() as f64
        };
        recent_trend > older_trend + threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_credibility_score_clamping() {
        assert!((CredibilityScore::new(-5.0).value() - 0.0).abs() < f64::EPSILON);
        assert!((CredibilityScore::new(15.0).value() - 10.0).abs() < f64::EPSILON);
        assert!((CredibilityScore::new(7.5).value() - 7.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_credibility_rating_thresholds() {
        assert_eq!(
            CredibilityScore::new(1.0).rating(),
            CredibilityRating::Critical
        );
        assert_eq!(CredibilityScore::new(3.0).rating(), CredibilityRating::Weak);
        assert_eq!(
            CredibilityScore::new(5.0).rating(),
            CredibilityRating::Adequate
        );
        assert_eq!(CredibilityScore::new(7.0).rating(), CredibilityRating::Good);
        assert_eq!(
            CredibilityScore::new(9.0).rating(),
            CredibilityRating::Excellent
        );
    }

    #[test]
    fn test_rating_acceptability() {
        assert!(!CredibilityRating::Critical.is_acceptable());
        assert!(!CredibilityRating::Weak.is_acceptable());
        assert!(CredibilityRating::Adequate.is_acceptable());
        assert!(CredibilityRating::Good.is_acceptable());
        assert!(CredibilityRating::Excellent.is_acceptable());
    }

    #[test]
    fn test_evidence_distribution() {
        let mut dist = EvidenceDistribution::new();
        dist.record(&EvidenceType::ValidationMetrics, EvidenceQuality::High);
        dist.record(&EvidenceType::TestResults, EvidenceQuality::Medium);
        dist.record(&EvidenceType::Architecture, EvidenceQuality::High);

        assert_eq!(dist.total(), 3);
        assert_eq!(dist.high_quality_count(), 2);
        assert!((dist.high_quality_ratio() - 0.6667).abs() < 0.01);
    }

    #[test]
    fn test_risk_distribution() {
        let mut dist = RiskDistribution::new();
        dist.record(RiskLevel::High);
        dist.record(RiskLevel::Medium);
        dist.record(RiskLevel::Low);
        dist.record(RiskLevel::Low);

        assert_eq!(dist.total(), 4);
        assert_eq!(dist.count(RiskLevel::High), 1);
        assert_eq!(dist.count(RiskLevel::Low), 2);
        assert!((dist.high_risk_ratio() - 0.25).abs() < f64::EPSILON);
    }

    #[test]
    fn test_credibility_input_calculation() {
        let input = CredibilityInput {
            evidence_quality: 1.0, // 100% high quality
            fit_for_use: 1.0,      // All 6 criteria
            risk_mitigation: 1.0,  // All controls
            documentation: 1.0,    // Complete docs
        };
        let score = input.calculate_score();
        assert!((score.value() - 10.0).abs() < f64::EPSILON);
        assert_eq!(score.rating(), CredibilityRating::Excellent);
    }

    #[test]
    fn test_credibility_input_weighted() {
        let input = CredibilityInput {
            evidence_quality: 0.5,
            fit_for_use: 0.5,
            risk_mitigation: 0.5,
            documentation: 0.5,
        };
        let score = input.calculate_score();
        assert!((score.value() - 5.0).abs() < f64::EPSILON);
        assert_eq!(score.rating(), CredibilityRating::Adequate);
    }

    #[test]
    fn test_drift_history_trend() {
        let mut history = DriftHistory::new();
        history.record(DriftMeasurement {
            timestamp: 1000,
            drift_percent: 5.0,
            severity: "MINOR".to_string(),
            model_id: "model-1".to_string(),
        });
        history.record(DriftMeasurement {
            timestamp: 2000,
            drift_percent: 10.0,
            severity: "MINOR".to_string(),
            model_id: "model-1".to_string(),
        });

        assert_eq!(history.measurement_count(), 2);
        assert!((history.trend(2) - 7.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_assessment_metrics_health() {
        let mut metrics = AssessmentMetrics::new();
        metrics.assessments_started = 10;
        metrics.assessments_completed = 8;
        metrics.assessments_approved = 6;

        // Record some evidence
        metrics
            .evidence
            .record(&EvidenceType::ValidationMetrics, EvidenceQuality::High);
        metrics
            .evidence
            .record(&EvidenceType::TestResults, EvidenceQuality::High);

        // Record some risk
        metrics.risk.record(RiskLevel::Medium);
        metrics.risk.record(RiskLevel::Low);

        let score = metrics.health_score();
        assert!(score.value() > 5.0);
        assert!(score.rating().is_acceptable());
    }
}
