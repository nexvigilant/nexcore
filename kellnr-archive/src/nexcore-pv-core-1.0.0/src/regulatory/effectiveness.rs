//! FDA Clinical Trial Effectiveness Endpoints
//!
//! Translates FDA guidance on proving effectiveness into code.
//! Based on FDA "Multiple Endpoints in Clinical Trials" (2024) and
//! "Accelerated Approval" Draft Guidance (December 2024).
//!
//! ## Endpoint Hierarchy
//!
//! 1. **Primary** - Required for approval (efficacy)
//! 2. **Secondary** - Supports primary or additional effects
//! 3. **Exploratory** - Future research, not approval basis
//!
//! ## Approval Pathways
//!
//! - **Traditional** - Clinical endpoint (mortality, morbidity)
//! - **Accelerated** - Surrogate or intermediate clinical endpoint

use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// ENDPOINT TIER (FDA Hierarchy)
// ═══════════════════════════════════════════════════════════════════════════

/// FDA endpoint hierarchy tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EndpointTier {
    /// Primary endpoint - required for approval
    Primary,
    /// Secondary endpoint - supports primary or shows additional effects
    Secondary,
    /// Exploratory endpoint - hypothesis generation only
    Exploratory,
}

impl EndpointTier {
    /// Whether this tier is required for approval.
    #[must_use]
    pub const fn required_for_approval(&self) -> bool {
        matches!(self, Self::Primary)
    }

    /// Multiplicity adjustment required.
    #[must_use]
    pub const fn requires_multiplicity_control(&self) -> bool {
        matches!(self, Self::Primary | Self::Secondary)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ENDPOINT TYPE
// ═══════════════════════════════════════════════════════════════════════════

/// Type of clinical trial endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EndpointType {
    /// Clinical endpoint - direct patient benefit (survival, symptoms)
    Clinical,
    /// Surrogate endpoint - reasonably likely to predict clinical benefit
    Surrogate,
    /// Intermediate clinical - measures before final outcome
    IntermediateClinical,
    /// Biomarker - biological indicator
    Biomarker,
    /// Patient-reported outcome
    PatientReported,
    /// Digital health technology measurement
    DigitalHealth,
}

impl EndpointType {
    /// Suitable for traditional approval.
    #[must_use]
    pub const fn supports_traditional_approval(&self) -> bool {
        matches!(self, Self::Clinical | Self::PatientReported)
    }

    /// Suitable for accelerated approval.
    #[must_use]
    pub const fn supports_accelerated_approval(&self) -> bool {
        matches!(
            self,
            Self::Surrogate | Self::IntermediateClinical | Self::Biomarker
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// APPROVAL PATHWAY
// ═══════════════════════════════════════════════════════════════════════════

/// FDA approval pathway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ApprovalPathway {
    /// Traditional approval - established clinical benefit
    Traditional,
    /// Accelerated approval - surrogate/intermediate endpoints
    Accelerated,
    /// Breakthrough therapy designation
    Breakthrough,
    /// Fast track designation
    FastTrack,
    /// Priority review
    PriorityReview,
}

impl ApprovalPathway {
    /// Requires confirmatory trial post-approval.
    #[must_use]
    pub const fn requires_confirmatory_trial(&self) -> bool {
        matches!(self, Self::Accelerated)
    }

    /// CFR section reference.
    #[must_use]
    pub const fn cfr_section(&self) -> &'static str {
        match self {
            Self::Traditional => "21 CFR 314.50",
            Self::Accelerated => "21 CFR 314.510",
            Self::Breakthrough => "21 CFR 312.320",
            Self::FastTrack => "21 CFR 312.300",
            Self::PriorityReview => "21 CFR 314.107",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EFFECTIVENESS ENDPOINT
// ═══════════════════════════════════════════════════════════════════════════

/// A defined effectiveness endpoint for clinical trial.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivenessEndpoint {
    /// Unique identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Tier in hierarchy
    pub tier: EndpointTier,
    /// Type of endpoint
    pub endpoint_type: EndpointType,
    /// Measurement description
    pub measurement: String,
    /// Statistical success criterion
    pub success_criterion: SuccessCriterion,
    /// Clinically meaningful difference (effect size)
    pub clinically_meaningful_difference: Option<f64>,
    /// Time to assessment (weeks)
    pub assessment_time_weeks: Option<u32>,
}

// ═══════════════════════════════════════════════════════════════════════════
// SUCCESS CRITERION
// ═══════════════════════════════════════════════════════════════════════════

/// Statistical success criterion for endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    /// Type of criterion
    pub criterion_type: CriterionType,
    /// Alpha level (type I error rate)
    pub alpha: f64,
    /// Power (1 - beta, type II error rate)
    pub power: f64,
    /// Threshold value (e.g., hazard ratio, difference)
    pub threshold: f64,
    /// Direction (superiority, non-inferiority, equivalence)
    pub direction: TestDirection,
}

impl Default for SuccessCriterion {
    fn default() -> Self {
        Self {
            criterion_type: CriterionType::Superiority,
            alpha: 0.05,
            power: 0.80,
            threshold: 0.0,
            direction: TestDirection::TwoSided,
        }
    }
}

/// Type of statistical criterion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CriterionType {
    /// Treatment better than control
    Superiority,
    /// Treatment not worse than control by margin
    NonInferiority,
    /// Treatment equivalent to control within margin
    Equivalence,
}

/// Direction of statistical test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestDirection {
    /// One-sided (treatment > control)
    OneSided,
    /// Two-sided (treatment ≠ control)
    TwoSided,
}

// ═══════════════════════════════════════════════════════════════════════════
// MULTIPLICITY ADJUSTMENT
// ═══════════════════════════════════════════════════════════════════════════

/// Multiplicity adjustment method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MultiplicityMethod {
    /// Bonferroni correction
    Bonferroni,
    /// Holm step-down procedure
    Holm,
    /// Hochberg step-up procedure
    Hochberg,
    /// Fixed-sequence (gatekeeping)
    FixedSequence,
    /// Fallback procedure
    Fallback,
    /// Graphical approach
    Graphical,
}

impl MultiplicityMethod {
    /// Adjusted alpha for given number of comparisons.
    #[must_use]
    pub fn adjusted_alpha(&self, alpha: f64, n_comparisons: usize) -> f64 {
        match self {
            Self::Bonferroni => alpha / n_comparisons as f64,
            Self::Holm | Self::Hochberg => alpha, // Applied sequentially
            Self::FixedSequence => alpha,         // Full alpha if gate passes
            Self::Fallback | Self::Graphical => alpha, // Depends on weights
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ENDPOINT RESULT
// ═══════════════════════════════════════════════════════════════════════════

/// Result of endpoint analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointResult {
    /// Endpoint analyzed
    pub endpoint_id: String,
    /// Observed effect size
    pub effect_size: f64,
    /// Confidence interval lower bound
    pub ci_lower: f64,
    /// Confidence interval upper bound
    pub ci_upper: f64,
    /// P-value (unadjusted)
    pub p_value: f64,
    /// P-value (multiplicity-adjusted)
    pub p_value_adjusted: Option<f64>,
    /// Whether endpoint met success criterion
    pub success: bool,
}

impl EndpointResult {
    /// Check if result is statistically significant at alpha level.
    #[must_use]
    pub fn is_significant(&self, alpha: f64) -> bool {
        self.p_value_adjusted.unwrap_or(self.p_value) < alpha
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EFFECTIVENESS ASSESSMENT
// ═══════════════════════════════════════════════════════════════════════════

/// Complete effectiveness assessment for approval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivenessAssessment {
    /// Intended approval pathway
    pub pathway: ApprovalPathway,
    /// All endpoints in trial
    pub endpoints: Vec<EffectivenessEndpoint>,
    /// Multiplicity adjustment method
    pub multiplicity_method: MultiplicityMethod,
    /// Results for each endpoint
    pub results: Vec<EndpointResult>,
}

impl EffectivenessAssessment {
    /// Check if primary endpoints met.
    /// Returns false if no primary endpoints defined (prevents vacuous truth).
    #[must_use]
    pub fn primary_endpoints_met(&self) -> bool {
        let primary_ids: Vec<_> = self
            .endpoints
            .iter()
            .filter(|e| e.tier == EndpointTier::Primary)
            .map(|e| &e.id)
            .collect();

        // Guard: must have at least one primary endpoint
        if primary_ids.is_empty() {
            return false;
        }

        self.results
            .iter()
            .filter(|r| primary_ids.contains(&&r.endpoint_id))
            .all(|r| r.success)
    }

    /// Check if suitable for intended pathway.
    #[must_use]
    pub fn supports_pathway(&self) -> bool {
        let primary: Vec<_> = self
            .endpoints
            .iter()
            .filter(|e| e.tier == EndpointTier::Primary)
            .collect();

        match self.pathway {
            ApprovalPathway::Traditional => primary
                .iter()
                .all(|e| e.endpoint_type.supports_traditional_approval()),
            ApprovalPathway::Accelerated => primary
                .iter()
                .all(|e| e.endpoint_type.supports_accelerated_approval()),
            _ => true, // Other pathways accept either
        }
    }

    /// FDA regulatory reference.
    #[must_use]
    pub fn cfr_reference(&self) -> &'static str {
        self.pathway.cfr_section()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SURROGATE ENDPOINT VALIDATION
// ═══════════════════════════════════════════════════════════════════════════

/// Level of surrogate endpoint validation (FDA framework).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurrogateValidationLevel {
    /// Biological plausibility only
    BiologicalPlausibility,
    /// Correlates with clinical outcome in trials
    TrialCorrelation,
    /// Treatment effect predicts clinical benefit
    TreatmentEffectPrediction,
    /// Fully validated across multiple drugs
    FullyValidated,
}

impl SurrogateValidationLevel {
    /// Minimum level for accelerated approval.
    #[must_use]
    pub const fn acceptable_for_accelerated() -> Self {
        Self::TreatmentEffectPrediction
    }
}

/// Surrogate endpoint specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurrogateEndpoint {
    /// The surrogate marker
    pub surrogate: String,
    /// The clinical outcome it predicts
    pub clinical_outcome: String,
    /// Validation level
    pub validation_level: SurrogateValidationLevel,
    /// Correlation coefficient with outcome
    pub correlation: Option<f64>,
    /// Disease context
    pub disease_context: String,
    /// FDA reference (if formally recognized)
    pub fda_reference: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_tier_approval() {
        assert!(EndpointTier::Primary.required_for_approval());
        assert!(!EndpointTier::Secondary.required_for_approval());
        assert!(!EndpointTier::Exploratory.required_for_approval());
    }

    #[test]
    fn test_endpoint_type_pathways() {
        assert!(EndpointType::Clinical.supports_traditional_approval());
        assert!(!EndpointType::Clinical.supports_accelerated_approval());
        assert!(EndpointType::Surrogate.supports_accelerated_approval());
        assert!(!EndpointType::Surrogate.supports_traditional_approval());
    }

    #[test]
    fn test_multiplicity_bonferroni() {
        let adjusted = MultiplicityMethod::Bonferroni.adjusted_alpha(0.05, 5);
        assert!((adjusted - 0.01).abs() < 1e-10);
    }

    #[test]
    fn test_no_primary_returns_false() {
        // Empty endpoints should return false, not vacuous true
        let assessment = EffectivenessAssessment {
            pathway: ApprovalPathway::Traditional,
            endpoints: vec![],
            multiplicity_method: MultiplicityMethod::Bonferroni,
            results: vec![],
        };
        assert!(!assessment.primary_endpoints_met());
    }

    #[test]
    fn test_effectiveness_assessment() {
        let endpoint = EffectivenessEndpoint {
            id: "OS".to_string(),
            name: "Overall Survival".to_string(),
            tier: EndpointTier::Primary,
            endpoint_type: EndpointType::Clinical,
            measurement: "Time to death".to_string(),
            success_criterion: SuccessCriterion::default(),
            clinically_meaningful_difference: Some(3.0),
            assessment_time_weeks: Some(52),
        };

        let result = EndpointResult {
            endpoint_id: "OS".to_string(),
            effect_size: 0.75,
            ci_lower: 0.60,
            ci_upper: 0.95,
            p_value: 0.01,
            p_value_adjusted: Some(0.02),
            success: true,
        };

        let assessment = EffectivenessAssessment {
            pathway: ApprovalPathway::Traditional,
            endpoints: vec![endpoint],
            multiplicity_method: MultiplicityMethod::FixedSequence,
            results: vec![result],
        };

        assert!(assessment.primary_endpoints_met());
        assert!(assessment.supports_pathway());
    }
}
