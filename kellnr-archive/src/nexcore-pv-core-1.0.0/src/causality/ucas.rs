//! # Universal Causality Assessment Scale (UCAS) - ToV §36
//!
//! Domain-agnostic causality assessment adapted from WHO-UMC and Naranjo.
//!
//! ## ToV Integration
//!
//! UCAS score integrates with the Recognition component (R) of the signal equation:
//!
//! ```text
//! R_causality = sigmoid(UCAS_score, μ=5, σ=2)
//! ```
//!
//! This allows causality assessment to inform signal strength calculations.
//!
//! ## Codex Compliance
//!
//! - **WRAP**: All scores use dedicated newtypes
//! - **CLASSIFY**: Explicit tier annotations
//! - **GROUND**: Traces to T1 primitives (i32, f64)
//!
//! ## Reference
//!
//! - ToV §36: Universal Causality Assessment Scale
//! - Intervention Vigilance Framework v1.0.0, §5.4

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

// =============================================================================
// CONSTANTS - ToV §36
// =============================================================================

/// UCAS sigmoid integration: center point (μ)
/// Tier: T1 (universal constant)
pub const SIGMOID_MU: f64 = 5.0;

/// UCAS sigmoid integration: spread (σ)
/// Tier: T1 (universal constant)
pub const SIGMOID_SIGMA: f64 = 2.0;

/// Maximum possible UCAS score
/// Sum of all positive criteria: 2+2+3+2+1+2+1+1 = 14
pub const UCAS_MAX_SCORE: i32 = 14;

/// Minimum possible UCAS score
/// Sum of all negative criteria: -1 + -2 = -3
pub const UCAS_MIN_SCORE: i32 = -3;

// =============================================================================
// CRITERION RESPONSE - Tier: T2-P
// =============================================================================

/// Response for a UCAS criterion
///
/// ## Tier: T2-P (Cross-Domain Primitive)
///
/// Represents Yes/No/Unknown for criterion assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CriterionResponse {
    /// Yes - criterion is met
    Yes,
    /// No - criterion is not met
    No,
    /// Unknown - insufficient information
    #[default]
    Unknown,
}

impl CriterionResponse {
    /// Check if response is affirmative
    #[must_use]
    pub const fn is_yes(&self) -> bool {
        matches!(self, Self::Yes)
    }

    /// Check if response is negative
    #[must_use]
    pub const fn is_no(&self) -> bool {
        matches!(self, Self::No)
    }

    /// Check if response is unknown
    #[must_use]
    pub const fn is_unknown(&self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl From<bool> for CriterionResponse {
    fn from(value: bool) -> Self {
        if value { Self::Yes } else { Self::No }
    }
}

impl From<Option<bool>> for CriterionResponse {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(true) => Self::Yes,
            Some(false) => Self::No,
            None => Self::Unknown,
        }
    }
}

// =============================================================================
// CRITERION SCORE - Tier: T2-P
// =============================================================================

/// Individual criterion score
///
/// ## Tier: T2-P (Cross-Domain Primitive)
///
/// Newtype for criterion scores to prevent mixing with other integers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct CriterionScore(i32);

impl CriterionScore {
    /// Create new criterion score
    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    /// Get the raw value
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }

    /// Zero score
    pub const ZERO: Self = Self(0);
}

impl From<i32> for CriterionScore {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

impl From<CriterionScore> for i32 {
    fn from(score: CriterionScore) -> i32 {
        score.0
    }
}

impl std::ops::Add for CriterionScore {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

// =============================================================================
// UCAS TOTAL SCORE - Tier: T2-P
// =============================================================================

/// Total UCAS score
///
/// ## Tier: T2-P (Cross-Domain Primitive)
///
/// Range: -3 to +14 (based on criterion weights)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UcasScore(i32);

impl UcasScore {
    /// Create new UCAS score, clamped to valid range
    #[must_use]
    pub fn new(value: i32) -> Self {
        Self(value.clamp(UCAS_MIN_SCORE, UCAS_MAX_SCORE))
    }

    /// Create from raw value without clamping (internal use)
    #[must_use]
    pub const fn from_raw(value: i32) -> Self {
        Self(value)
    }

    /// Get the raw value
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }

    /// Calculate Recognition component (R) for ToV integration
    ///
    /// Uses sigmoid function: R = 1 / (1 + exp(-(score - μ) / σ))
    ///
    /// ## ToV Integration
    ///
    /// This value can be used directly as the R component in S = U × R × T
    #[must_use]
    pub fn to_recognition_r(self) -> f64 {
        let x = f64::from(self.0);
        1.0 / (1.0 + (-(x - SIGMOID_MU) / SIGMOID_SIGMA).exp())
    }
}

impl Default for UcasScore {
    fn default() -> Self {
        Self(0)
    }
}

impl fmt::Display for UcasScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PartialOrd for UcasScore {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for UcasScore {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

// =============================================================================
// UCAS CATEGORY - Tier: T2-C
// =============================================================================

/// UCAS causality category
///
/// ## Tier: T2-C (Cross-Domain Composite)
///
/// Categories map to score ranges as per ToV §36.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UcasCategory {
    /// Score ≥9: Definitive causal relationship
    Certain,
    /// Score 6-8: Likely causal; alternatives unlikely
    Probable,
    /// Score 3-5: Plausible causal; alternatives possible
    Possible,
    /// Score 1-2: Improbable; alternatives likely
    Unlikely,
    /// Score ≤0: Insufficient data
    Unassessable,
}

impl UcasCategory {
    /// Determine category from score
    #[must_use]
    pub fn from_score(score: UcasScore) -> Self {
        match score.value() {
            9..=14 => Self::Certain,
            6..=8 => Self::Probable,
            3..=5 => Self::Possible,
            1..=2 => Self::Unlikely,
            _ => Self::Unassessable,
        }
    }

    /// Get minimum score for this category
    #[must_use]
    pub const fn min_score(self) -> i32 {
        match self {
            Self::Certain => 9,
            Self::Probable => 6,
            Self::Possible => 3,
            Self::Unlikely => 1,
            Self::Unassessable => UCAS_MIN_SCORE,
        }
    }

    /// Get confidence level (0.0-1.0) for this category
    #[must_use]
    pub const fn confidence(self) -> f64 {
        match self {
            Self::Certain => 0.95,
            Self::Probable => 0.80,
            Self::Possible => 0.60,
            Self::Unlikely => 0.30,
            Self::Unassessable => 0.10,
        }
    }

    /// Get recommended action
    #[must_use]
    pub const fn recommended_action(self) -> &'static str {
        match self {
            Self::Certain => "Confirm causal relationship; implement risk minimization",
            Self::Probable => "Investigate further; prepare risk communication",
            Self::Possible => "Continue monitoring; gather additional evidence",
            Self::Unlikely => "Document and archive; no immediate action",
            Self::Unassessable => "Request additional data; defer assessment",
        }
    }
}

impl fmt::Display for UcasCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Certain => write!(f, "Certain"),
            Self::Probable => write!(f, "Probable"),
            Self::Possible => write!(f, "Possible"),
            Self::Unlikely => write!(f, "Unlikely"),
            Self::Unassessable => write!(f, "Unassessable"),
        }
    }
}

// =============================================================================
// UCAS INPUT - Tier: T3
// =============================================================================

/// UCAS assessment input (8 criteria)
///
/// ## Tier: T3 (Domain-Specific)
///
/// Adapted from WHO-UMC and Naranjo for domain-agnostic application.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UcasInput {
    /// 1. Did harm occur after exposure with plausible latency?
    /// Points: Yes=+2, Unknown=0, No=-1
    pub temporal_relationship: CriterionResponse,

    /// 2. Did harm improve when intervention removed?
    /// Points: Yes=+2, Unknown/No=0
    pub dechallenge: CriterionResponse,

    /// 3. Did harm recur when intervention reintroduced?
    /// Points: Yes=+3, Unknown/No=0
    pub rechallenge: CriterionResponse,

    /// 4. Is there a known mechanism for this harm?
    /// Points: Yes=+2, Unknown/No=0
    pub mechanistic_plausibility: CriterionResponse,

    /// 5. Are other plausible causes present?
    /// Points: Yes=-2, Unknown=0, No=+1
    pub alternative_explanations: CriterionResponse,

    /// 6. Relationship between intensity and severity?
    /// Points: Yes=+2, Unknown/No=0
    pub dose_response: CriterionResponse,

    /// 7. Has this association been reported before?
    /// Points: Yes=+1, Unknown/No=0
    pub prior_evidence: CriterionResponse,

    /// 8. Is this harm characteristic of this intervention class?
    /// Points: Yes=+1, Unknown/No=0
    pub specificity: CriterionResponse,
}

impl UcasInput {
    /// Create new input with all unknown responses
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: set temporal relationship
    #[must_use]
    pub fn with_temporal(mut self, response: CriterionResponse) -> Self {
        self.temporal_relationship = response;
        self
    }

    /// Builder: set dechallenge
    #[must_use]
    pub fn with_dechallenge(mut self, response: CriterionResponse) -> Self {
        self.dechallenge = response;
        self
    }

    /// Builder: set rechallenge
    #[must_use]
    pub fn with_rechallenge(mut self, response: CriterionResponse) -> Self {
        self.rechallenge = response;
        self
    }

    /// Builder: set mechanistic plausibility
    #[must_use]
    pub fn with_mechanism(mut self, response: CriterionResponse) -> Self {
        self.mechanistic_plausibility = response;
        self
    }

    /// Builder: set alternative explanations
    #[must_use]
    pub fn with_alternatives(mut self, response: CriterionResponse) -> Self {
        self.alternative_explanations = response;
        self
    }

    /// Builder: set dose-response
    #[must_use]
    pub fn with_dose_response(mut self, response: CriterionResponse) -> Self {
        self.dose_response = response;
        self
    }

    /// Builder: set prior evidence
    #[must_use]
    pub fn with_prior_evidence(mut self, response: CriterionResponse) -> Self {
        self.prior_evidence = response;
        self
    }

    /// Builder: set specificity
    #[must_use]
    pub fn with_specificity(mut self, response: CriterionResponse) -> Self {
        self.specificity = response;
        self
    }
}

// =============================================================================
// UCAS RESULT - Tier: T3
// =============================================================================

/// Individual criterion breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriterionBreakdown {
    /// Criterion name
    pub name: String,
    /// Response given
    pub response: CriterionResponse,
    /// Points awarded
    pub score: CriterionScore,
    /// Maximum possible points
    pub max_points: i32,
}

/// UCAS assessment result
///
/// ## Tier: T3 (Domain-Specific)
///
/// Complete result with score, category, breakdown, and ToV integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UcasResult {
    /// Total UCAS score
    pub score: UcasScore,
    /// Causality category
    pub category: UcasCategory,
    /// Recognition component (R) for ToV S = U × R × T
    pub recognition_r: f64,
    /// Per-criterion breakdown
    pub breakdown: Vec<CriterionBreakdown>,
    /// Assessment confidence (0.0-1.0)
    pub confidence: f64,
}

impl UcasResult {
    /// Check if result indicates likely causality (Certain or Probable)
    #[must_use]
    pub fn is_likely_causal(&self) -> bool {
        matches!(
            self.category,
            UcasCategory::Certain | UcasCategory::Probable
        )
    }

    /// Check if result indicates possible causality (at least Possible)
    #[must_use]
    pub fn is_possibly_causal(&self) -> bool {
        !matches!(
            self.category,
            UcasCategory::Unlikely | UcasCategory::Unassessable
        )
    }
}

// =============================================================================
// ASSESSMENT FUNCTION
// =============================================================================

/// Calculate UCAS score and category
///
/// Implements ToV §36 scoring criteria:
///
/// | Criterion | Yes | Unknown | No |
/// |-----------|-----|---------|-----|
/// | Temporal | +2 | 0 | -1 |
/// | Dechallenge | +2 | 0 | 0 |
/// | Rechallenge | +3 | 0 | 0 |
/// | Mechanism | +2 | 0 | 0 |
/// | Alternatives | -2 | 0 | +1 |
/// | Dose-Response | +2 | 0 | 0 |
/// | Prior Evidence | +1 | 0 | 0 |
/// | Specificity | +1 | 0 | 0 |
#[must_use]
pub fn calculate_ucas(input: &UcasInput) -> UcasResult {
    let mut breakdown = Vec::with_capacity(8);
    let mut total = 0i32;

    // 1. Temporal relationship
    let temporal_score = match input.temporal_relationship {
        CriterionResponse::Yes => 2,
        CriterionResponse::Unknown => 0,
        CriterionResponse::No => -1,
    };
    breakdown.push(CriterionBreakdown {
        name: "Temporal Relationship".to_string(),
        response: input.temporal_relationship,
        score: CriterionScore::new(temporal_score),
        max_points: 2,
    });
    total += temporal_score;

    // 2. Dechallenge
    let dechallenge_score = match input.dechallenge {
        CriterionResponse::Yes => 2,
        CriterionResponse::Unknown | CriterionResponse::No => 0,
    };
    breakdown.push(CriterionBreakdown {
        name: "Dechallenge".to_string(),
        response: input.dechallenge,
        score: CriterionScore::new(dechallenge_score),
        max_points: 2,
    });
    total += dechallenge_score;

    // 3. Rechallenge (strongest criterion)
    let rechallenge_score = match input.rechallenge {
        CriterionResponse::Yes => 3,
        CriterionResponse::Unknown | CriterionResponse::No => 0,
    };
    breakdown.push(CriterionBreakdown {
        name: "Rechallenge".to_string(),
        response: input.rechallenge,
        score: CriterionScore::new(rechallenge_score),
        max_points: 3,
    });
    total += rechallenge_score;

    // 4. Mechanistic plausibility
    let mechanism_score = match input.mechanistic_plausibility {
        CriterionResponse::Yes => 2,
        CriterionResponse::Unknown | CriterionResponse::No => 0,
    };
    breakdown.push(CriterionBreakdown {
        name: "Mechanistic Plausibility".to_string(),
        response: input.mechanistic_plausibility,
        score: CriterionScore::new(mechanism_score),
        max_points: 2,
    });
    total += mechanism_score;

    // 5. Alternative explanations (inverted scoring)
    let alternatives_score = match input.alternative_explanations {
        CriterionResponse::Yes => -2, // Alternatives present = negative
        CriterionResponse::Unknown => 0,
        CriterionResponse::No => 1, // No alternatives = positive
    };
    breakdown.push(CriterionBreakdown {
        name: "Alternative Explanations".to_string(),
        response: input.alternative_explanations,
        score: CriterionScore::new(alternatives_score),
        max_points: 1,
    });
    total += alternatives_score;

    // 6. Dose-response
    let dose_response_score = match input.dose_response {
        CriterionResponse::Yes => 2,
        CriterionResponse::Unknown | CriterionResponse::No => 0,
    };
    breakdown.push(CriterionBreakdown {
        name: "Dose-Response".to_string(),
        response: input.dose_response,
        score: CriterionScore::new(dose_response_score),
        max_points: 2,
    });
    total += dose_response_score;

    // 7. Prior evidence
    let prior_evidence_score = match input.prior_evidence {
        CriterionResponse::Yes => 1,
        CriterionResponse::Unknown | CriterionResponse::No => 0,
    };
    breakdown.push(CriterionBreakdown {
        name: "Prior Evidence".to_string(),
        response: input.prior_evidence,
        score: CriterionScore::new(prior_evidence_score),
        max_points: 1,
    });
    total += prior_evidence_score;

    // 8. Specificity
    let specificity_score = match input.specificity {
        CriterionResponse::Yes => 1,
        CriterionResponse::Unknown | CriterionResponse::No => 0,
    };
    breakdown.push(CriterionBreakdown {
        name: "Specificity".to_string(),
        response: input.specificity,
        score: CriterionScore::new(specificity_score),
        max_points: 1,
    });
    total += specificity_score;

    let score = UcasScore::from_raw(total);
    let category = UcasCategory::from_score(score);
    let recognition_r = score.to_recognition_r();
    let confidence = category.confidence();

    UcasResult {
        score,
        category,
        recognition_r,
        breakdown,
        confidence,
    }
}

/// Quick UCAS assessment with minimal inputs
///
/// For rapid triage when full assessment is not needed.
#[must_use]
pub fn calculate_ucas_quick(
    temporal: bool,
    dechallenge: bool,
    rechallenge: bool,
    mechanism: bool,
    no_alternatives: bool,
) -> UcasResult {
    let input = UcasInput::new()
        .with_temporal(temporal.into())
        .with_dechallenge(dechallenge.into())
        .with_rechallenge(rechallenge.into())
        .with_mechanism(mechanism.into())
        .with_alternatives(if no_alternatives {
            CriterionResponse::No
        } else {
            CriterionResponse::Yes
        });

    calculate_ucas(&input)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ucas_certain() {
        // All positive responses = Certain
        let input = UcasInput::new()
            .with_temporal(CriterionResponse::Yes)
            .with_dechallenge(CriterionResponse::Yes)
            .with_rechallenge(CriterionResponse::Yes)
            .with_mechanism(CriterionResponse::Yes)
            .with_alternatives(CriterionResponse::No)
            .with_dose_response(CriterionResponse::Yes)
            .with_prior_evidence(CriterionResponse::Yes)
            .with_specificity(CriterionResponse::Yes);

        let result = calculate_ucas(&input);
        assert_eq!(result.score.value(), 14); // Max score
        assert_eq!(result.category, UcasCategory::Certain);
        assert!(result.recognition_r > 0.9);
    }

    #[test]
    fn test_ucas_probable() {
        let input = UcasInput::new()
            .with_temporal(CriterionResponse::Yes) // +2
            .with_dechallenge(CriterionResponse::Yes) // +2
            .with_mechanism(CriterionResponse::Yes) // +2
            .with_alternatives(CriterionResponse::No); // +1
        // Total = 7 = Probable

        let result = calculate_ucas(&input);
        assert_eq!(result.score.value(), 7);
        assert_eq!(result.category, UcasCategory::Probable);
    }

    #[test]
    fn test_ucas_possible() {
        let input = UcasInput::new()
            .with_temporal(CriterionResponse::Yes) // +2
            .with_mechanism(CriterionResponse::Yes); // +2
        // Total = 4 = Possible

        let result = calculate_ucas(&input);
        assert_eq!(result.score.value(), 4);
        assert_eq!(result.category, UcasCategory::Possible);
    }

    #[test]
    fn test_ucas_unlikely() {
        // Only 2 weak positive criteria - score = 2 = Unlikely
        let input = UcasInput::new()
            .with_prior_evidence(CriterionResponse::Yes) // +1
            .with_specificity(CriterionResponse::Yes); // +1
        // Total = 2 = Unlikely

        let result = calculate_ucas(&input);
        assert_eq!(result.score.value(), 2);
        assert_eq!(result.category, UcasCategory::Unlikely);
    }

    #[test]
    fn test_ucas_unassessable() {
        let input = UcasInput::new()
            .with_temporal(CriterionResponse::No) // -1
            .with_alternatives(CriterionResponse::Yes); // -2
        // Total = -3 = Unassessable

        let result = calculate_ucas(&input);
        assert!(result.score.value() <= 0);
        assert_eq!(result.category, UcasCategory::Unassessable);
    }

    #[test]
    fn test_recognition_r_sigmoid() {
        // Score 0 → R ≈ 0.076 (sigmoid(0, 5, 2))
        let low_score = UcasScore::new(0);
        assert!(low_score.to_recognition_r() < 0.2);

        // Score 5 → R = 0.5 (exactly at μ)
        let mid_score = UcasScore::new(5);
        assert!((mid_score.to_recognition_r() - 0.5).abs() < 0.01);

        // Score 10 → R ≈ 0.924 (sigmoid(10, 5, 2))
        let high_score = UcasScore::new(10);
        assert!(high_score.to_recognition_r() > 0.9);
    }

    #[test]
    fn test_quick_assessment() {
        let result = calculate_ucas_quick(true, true, true, true, true);
        // temporal=2 + dechallenge=2 + rechallenge=3 + mechanism=2 + no_alternatives=1 = 10
        assert_eq!(result.score.value(), 10);
        assert_eq!(result.category, UcasCategory::Certain);
    }

    #[test]
    fn test_breakdown_present() {
        let input = UcasInput::new().with_temporal(CriterionResponse::Yes);
        let result = calculate_ucas(&input);

        assert_eq!(result.breakdown.len(), 8);
        assert_eq!(result.breakdown[0].name, "Temporal Relationship");
        assert_eq!(result.breakdown[0].score.value(), 2);
    }

    #[test]
    fn test_category_from_score() {
        assert_eq!(
            UcasCategory::from_score(UcasScore::new(14)),
            UcasCategory::Certain
        );
        assert_eq!(
            UcasCategory::from_score(UcasScore::new(9)),
            UcasCategory::Certain
        );
        assert_eq!(
            UcasCategory::from_score(UcasScore::new(8)),
            UcasCategory::Probable
        );
        assert_eq!(
            UcasCategory::from_score(UcasScore::new(6)),
            UcasCategory::Probable
        );
        assert_eq!(
            UcasCategory::from_score(UcasScore::new(5)),
            UcasCategory::Possible
        );
        assert_eq!(
            UcasCategory::from_score(UcasScore::new(3)),
            UcasCategory::Possible
        );
        assert_eq!(
            UcasCategory::from_score(UcasScore::new(2)),
            UcasCategory::Unlikely
        );
        assert_eq!(
            UcasCategory::from_score(UcasScore::new(1)),
            UcasCategory::Unlikely
        );
        assert_eq!(
            UcasCategory::from_score(UcasScore::new(0)),
            UcasCategory::Unassessable
        );
        assert_eq!(
            UcasCategory::from_score(UcasScore::new(-3)),
            UcasCategory::Unassessable
        );
    }
}
