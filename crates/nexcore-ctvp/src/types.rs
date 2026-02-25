//! Core type definitions for CTVP validation.
//!
//! This module contains the fundamental types used throughout the CTVP library,
//! including validation phases, outcomes, severity levels, and evidence types.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation phase in the CTVP framework.
///
/// Maps pharmaceutical drug development phases to software validation stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationPhase {
    /// Phase 0: Preclinical - Mechanism validity testing
    /// Software: Unit tests, mocks, property-based tests, static analysis
    Preclinical,

    /// Phase 1: Safety - Failure mode validation
    /// Software: Chaos engineering, fault injection, boundary testing
    Phase1Safety,

    /// Phase 2: Efficacy - Capability achievement validation
    /// Software: Real data validation, SLO measurement, workflow testing
    Phase2Efficacy,

    /// Phase 3: Confirmation - Scale validation
    /// Software: Shadow deployment, canary deployment, A/B testing
    Phase3Confirmation,

    /// Phase 4: Surveillance - Ongoing correctness
    /// Software: Drift detection, continuous validation, observability
    Phase4Surveillance,
}

impl ValidationPhase {
    fn info(&self) -> (u8, f64, &'static str, &'static str, &'static str) {
        match self {
            Self::Preclinical => (
                0,
                0.05,
                "Preclinical",
                "In-Vitro + In-Vivo Animal Testing",
                "Does the mechanism work under controlled conditions?",
            ),
            Self::Phase1Safety => (
                1,
                0.15,
                "Safety",
                "First-in-Human (20-100 volunteers)",
                "Does it fail gracefully under stress?",
            ),
            Self::Phase2Efficacy => (
                2,
                0.30,
                "Efficacy",
                "Proof of Concept (up to several hundred)",
                "Does it achieve its intended purpose?",
            ),
            Self::Phase3Confirmation => (
                3,
                0.30,
                "Confirmation",
                "Pivotal Trials (300-3000 patients)",
                "Does it perform at least as well at scale?",
            ),
            Self::Phase4Surveillance => (
                4,
                0.20,
                "Surveillance",
                "Post-Market Surveillance (ongoing)",
                "Does it continue working correctly over time?",
            ),
        }
    }

    /// Returns the phase number (0-4)
    pub fn get_number(&self) -> u8 {
        self.info().0
    }

    /// Returns the phase weight for Reality Gradient calculation
    pub fn get_weight(&self) -> f64 {
        self.info().1
    }

    /// Returns the human-readable name
    pub fn get_name(&self) -> &'static str {
        self.info().2
    }

    /// Returns the pharmaceutical equivalent
    pub fn get_pharma_equivalent(&self) -> &'static str {
        self.info().3
    }

    /// Returns the key question this phase answers
    pub fn get_key_question(&self) -> &'static str {
        self.info().4
    }

    /// Returns all phases in order
    pub fn get_all() -> [Self; 5] {
        [
            Self::Preclinical,
            Self::Phase1Safety,
            Self::Phase2Efficacy,
            Self::Phase3Confirmation,
            Self::Phase4Surveillance,
        ]
    }
}

impl std::fmt::Display for ValidationPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Phase {}: {}", self.get_number(), self.get_name())
    }
}

/// Outcome of a validation attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ValidationOutcome {
    /// Validation passed with evidence
    Validated,

    /// Validation failed with reason
    Failed {
        /// Reason for failure
        reason: String,
    },

    /// Validation inconclusive
    Inconclusive {
        /// Reason for inconclusive result
        reason: String,
    },

    /// Validation not applicable for this deliverable
    NotApplicable,
}

impl ValidationOutcome {
    /// Returns true if validated
    pub fn is_validated(&self) -> bool {
        matches!(self, Self::Validated)
    }

    /// Returns true if failed
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed { .. })
    }
}

/// Quality of evidence supporting a validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceQuality {
    /// No evidence present
    None,
    /// Weak evidence (partial, questionable)
    Weak,
    /// Moderate evidence (substantial but incomplete)
    Moderate,
    /// Strong evidence (comprehensive, verified)
    Strong,
}

impl EvidenceQuality {
    /// Returns the numerical value of the quality (0.0 - 1.0)
    pub fn get_value(&self) -> f64 {
        match self {
            Self::None => 0.0,
            Self::Weak => 0.33,
            Self::Moderate => 0.66,
            Self::Strong => 1.0,
        }
    }
}

impl From<EvidenceQuality> for f64 {
    fn from(quality: EvidenceQuality) -> Self {
        quality.get_value()
    }
}

/// Classification of validation/testing issue severity.
///
/// Tier: T2-P (κ + ∂ — comparison with boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticLevel {
    /// Minor issue, improvement opportunity
    Low,
    /// Degraded experience, technical debt
    Medium,
    /// Significant user/business impact
    High,
    /// System failure, data loss, security breach
    Critical,
}

/// Backward-compatible alias.
#[deprecated(note = "use DiagnosticLevel — F2 equivocation fix")]
pub type Severity = DiagnosticLevel;

impl DiagnosticLevel {
    fn info(&self) -> (&'static str, &'static str) {
        match self {
            Self::Low => ("Low", "Backlog"),
            Self::Medium => ("Medium", "Within quarter"),
            Self::High => ("High", "Within sprint"),
            Self::Critical => ("Critical", "Immediate"),
        }
    }

    /// Returns the response time expectation
    pub fn get_response_time(&self) -> &'static str {
        self.info().1
    }
}

impl std::fmt::Display for DiagnosticLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.info().0)
    }
}

/// Type of evidence collected during validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EvidenceType {
    /// Metric measurement
    Metric {
        /// Name of the metric
        name: String,
        /// Measured value
        value: f64,
        /// Unit of measurement
        unit: String,
    },

    /// Log entry
    Log {
        /// Log level (info, warn, error)
        level: String,
        /// Log message
        message: String,
    },

    /// Comparison between expected and actual
    Comparison {
        /// Expected value/state
        expected: String,
        /// Actual value/state
        actual: String,
        /// Similarity score (0.0 - 1.0)
        match_score: f64,
    },

    /// Exception or error
    Exception {
        /// Type of exception
        error_type: String,
        /// Error message
        message: String,
        /// Stack trace if available
        trace: Option<String>,
    },

    /// State snapshot
    Snapshot {
        /// Component name
        component: String,
        /// Captured state data
        state: serde_json::Value,
    },

    /// Test result
    TestResult {
        /// Test name
        name: String,
        /// Whether test passed
        passed: bool,
        /// Test duration
        duration_ms: u64,
    },

    /// Coverage report
    Coverage {
        /// Percentage of line coverage (0.0 - 1.0)
        line_coverage: f64,
        /// Percentage of branch coverage if available
        branch_coverage: Option<f64>,
        /// Percentage of function coverage if available
        function_coverage: Option<f64>,
    },
}

/// A piece of evidence supporting validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Evidence {
    /// Unique identifier
    pub id: NexId,

    /// Evidence type and data
    pub evidence_type: EvidenceType,

    /// Human-readable description
    pub description: String,

    /// When this evidence was collected
    pub collected_at: DateTime,

    /// Source of the evidence (file, tool, etc.)
    pub source: String,

    /// Additional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl Evidence {
    /// Creates new evidence with auto-generated ID and timestamp
    pub fn new(
        evidence_type: EvidenceType,
        description: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            id: NexId::v4(),
            evidence_type,
            description: description.into(),
            collected_at: DateTime::now(),
            source: source.into(),
            metadata: HashMap::new(),
        }
    }

    /// Adds metadata to the evidence
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

/// Result of a validation phase execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Capability being validated
    pub capability_id: String,

    /// Phase that was validated
    pub phase: ValidationPhase,

    /// Outcome of validation
    pub outcome: ValidationOutcome,

    /// Evidence supporting the outcome
    pub evidence: Vec<Evidence>,

    /// Quality assessment of the evidence
    pub evidence_quality: EvidenceQuality,

    /// Duration of validation
    pub duration: std::time::Duration,

    /// When validation was performed
    pub timestamp: DateTime,

    /// Additional notes
    #[serde(default)]
    pub notes: Vec<String>,
}

impl ValidationResult {
    /// Creates a new validation result
    pub fn new(
        capability_id: impl Into<String>,
        phase: ValidationPhase,
        outcome: ValidationOutcome,
        evidence_quality: EvidenceQuality,
    ) -> Self {
        Self {
            capability_id: capability_id.into(),
            phase,
            outcome,
            evidence: Vec::new(),
            evidence_quality,
            duration: std::time::Duration::ZERO,
            timestamp: DateTime::now(),
            notes: Vec::new(),
        }
    }

    /// Adds evidence to the result
    pub fn with_evidence(mut self, evidence: Evidence) -> Self {
        self.evidence.push(evidence);
        self
    }

    /// Sets the duration
    pub fn with_duration(mut self, duration: std::time::Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Adds a note
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }
}

/// Threshold definition for capability validation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Threshold {
    /// Comparison operator
    pub operator: ThresholdOperator,

    /// Target value
    pub value: f64,

    /// Optional minimum value (for Between operator)
    pub min: Option<f64>,

    /// Optional maximum value (for Between operator)
    pub max: Option<f64>,

    /// Percentile to measure (for latency metrics)
    pub percentile: Option<f64>,
}

impl Threshold {
    /// Creates a greater-than threshold
    pub fn gt(value: f64) -> Self {
        Self {
            operator: ThresholdOperator::GreaterThan,
            value,
            min: None,
            max: None,
            percentile: None,
        }
    }

    /// Creates a greater-than-or-equal threshold
    pub fn gte(value: f64) -> Self {
        Self {
            operator: ThresholdOperator::GreaterThanOrEqual,
            value,
            min: None,
            max: None,
            percentile: None,
        }
    }

    /// Creates a less-than threshold
    pub fn lt(value: f64) -> Self {
        Self {
            operator: ThresholdOperator::LessThan,
            value,
            min: None,
            max: None,
            percentile: None,
        }
    }

    /// Creates a less-than-or-equal threshold
    pub fn lte(value: f64) -> Self {
        Self {
            operator: ThresholdOperator::LessThanOrEqual,
            value,
            min: None,
            max: None,
            percentile: None,
        }
    }

    /// Creates a between threshold
    pub fn between(min: f64, max: f64) -> Self {
        Self {
            operator: ThresholdOperator::Between,
            value: (min + max) / 2.0,
            min: Some(min),
            max: Some(max),
            percentile: None,
        }
    }

    /// Sets the percentile for this threshold
    pub fn at_percentile(mut self, p: f64) -> Self {
        self.percentile = Some(p);
        self
    }

    /// Checks if a value meets this threshold
    pub fn is_met(&self, actual: f64) -> bool {
        self.check_operator(actual)
    }

    fn check_operator(&self, actual: f64) -> bool {
        match self.operator {
            ThresholdOperator::GreaterThan => actual > self.value,
            ThresholdOperator::GreaterThanOrEqual => actual >= self.value,
            ThresholdOperator::LessThan => actual < self.value,
            ThresholdOperator::LessThanOrEqual => actual <= self.value,
            ThresholdOperator::Equal => (actual - self.value).abs() < f64::EPSILON,
            ThresholdOperator::NotEqual => (actual - self.value).abs() >= f64::EPSILON,
            ThresholdOperator::Between => self.check_between(actual),
        }
    }

    fn check_between(&self, actual: f64) -> bool {
        let min = self.min.unwrap_or(f64::MIN);
        let max = self.max.unwrap_or(f64::MAX);
        actual >= min && actual <= max
    }
}

/// Threshold comparison operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThresholdOperator {
    /// Greater than (>)
    GreaterThan,
    /// Greater than or equal (>=)
    GreaterThanOrEqual,
    /// Less than (<)
    LessThan,
    /// Less than or equal (<=)
    LessThanOrEqual,
    /// Equal (==)
    Equal,
    /// Not equal (!=)
    NotEqual,
    /// Between min and max (inclusive)
    Between,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_ordering() {
        assert_eq!(ValidationPhase::Preclinical.get_number(), 0);
        assert_eq!(ValidationPhase::Phase4Surveillance.get_number(), 4);
    }

    #[test]
    fn test_phase_weights_sum_to_one() {
        let total: f64 = ValidationPhase::get_all()
            .into_iter()
            .map(|p| p.get_weight())
            .sum();
        assert!((total - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_evidence_quality_ordering() {
        assert!(EvidenceQuality::None < EvidenceQuality::Weak);
        assert!(EvidenceQuality::Weak < EvidenceQuality::Moderate);
        assert!(EvidenceQuality::Moderate < EvidenceQuality::Strong);
    }

    #[test]
    fn test_threshold_comparison() {
        assert!(Threshold::gte(0.99).is_met(0.99));
        assert!(Threshold::gte(0.99).is_met(1.0));
        assert!(!Threshold::gte(0.99).is_met(0.98));

        assert!(Threshold::lt(100.0).is_met(50.0));
        assert!(!Threshold::lt(100.0).is_met(100.0));

        assert!(Threshold::between(0.0, 1.0).is_met(0.5));
        assert!(!Threshold::between(0.0, 1.0).is_met(1.5));
    }
}
