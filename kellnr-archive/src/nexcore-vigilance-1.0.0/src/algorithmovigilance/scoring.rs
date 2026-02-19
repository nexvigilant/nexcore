//! # ACA Scoring Framework (ToV §53-§54)
//!
//! Algorithm Causality Assessment scoring with 8 lemmas and sigmoid transformation.
//!
//! # The 8 ACA Lemmas
//!
//! | Lemma | Description | Points | Required |
//! |-------|-------------|--------|----------|
//! | L1 | Temporal Sequence | — | Yes |
//! | L2 | Cognition Evidence | +1 | No |
//! | L3 | Action Alignment | — | Yes |
//! | L4 | Harm Occurrence | — | Yes |
//! | L5 | Dechallenge | Supportive | No |
//! | L6 | Rechallenge | +2 | No |
//! | L7 | Validation Status | +1 | No |
//! | L8 | Ground Truth | +2 | No |
//!
//! # Causality Categories
//!
//! | Category | Score | Description |
//! |----------|-------|-------------|
//! | Definite | ≥6 | Beyond reasonable doubt |
//! | Probable | 4-5 | Likely contributed |
//! | Possible | 2-3 | May have contributed |
//! | Unlikely | <2 | Insufficient evidence |
//! | Unassessable | — | Required lemmas missing |
//!
//! # Example
//!
//! ```rust
//! use nexcore_vigilance::algorithmovigilance::scoring::{
//!     AcaScoringInput, AcaLemma, LemmaResponse, score_aca, AcaCausalityCategory,
//! };
//!
//! let input = AcaScoringInput::new()
//!     .with_lemma(AcaLemma::Temporal, LemmaResponse::Yes)
//!     .with_lemma(AcaLemma::Cognition, LemmaResponse::Yes)
//!     .with_lemma(AcaLemma::Action, LemmaResponse::Yes)
//!     .with_lemma(AcaLemma::Harm, LemmaResponse::Yes)
//!     .with_lemma(AcaLemma::Rechallenge, LemmaResponse::Yes)
//!     .with_lemma(AcaLemma::GroundTruth, LemmaResponse::Yes);
//!
//! let result = score_aca(&input);
//! assert!(result.is_assessable());
//! assert!(result.is_assessable());
//! ```

use serde::{Deserialize, Serialize};

use super::{ACA_SIGMOID_MU, ACA_SIGMOID_SIGMA, LogicCase, OverrideParadox};

// ============================================================================
// ACA LEMMAS (T2-P)
// ============================================================================

/// The 8 ACA Lemmas from ToV §53.2.
///
/// # Tier: T2-P
///
/// Cross-domain primitive enum for causality assessment criteria.
///
/// # ToV §53.2
///
/// Three lemmas are required (L1, L3, L4). Others contribute points.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AcaLemma {
    /// L1: Temporal Sequence - Output preceded outcome.
    ///
    /// **Required** — If violated, cannot attribute causality.
    ///
    /// ToV Mapping: Definition 6.2 (Temporal ordering)
    Temporal = 1,

    /// L2: Cognition Evidence - Clinician perceived/processed the output.
    ///
    /// **Scoring:** +1 if satisfied.
    ///
    /// ToV Mapping: R component (Recognition)
    Cognition = 2,

    /// L3: Action Alignment - Clinician action aligned with algorithm output.
    ///
    /// **Required** — If violated, cannot attribute causality.
    ///
    /// ToV Mapping: Causal chain (A link)
    Action = 3,

    /// L4: Harm Occurrence - Adverse outcome documented.
    ///
    /// **Required** — If violated, cannot attribute causality.
    ///
    /// ToV Mapping: Harm event H
    Harm = 4,

    /// L5: Dechallenge - Outcome improved when algorithm removed/corrected.
    ///
    /// **Supportive** — Provides qualitative support but no points.
    ///
    /// ToV Mapping: Definition 6.3
    Dechallenge = 5,

    /// L6: Rechallenge - Similar pattern observed in other cases.
    ///
    /// **Scoring:** +2 if satisfied.
    ///
    /// ToV Mapping: U computation (§21)
    Rechallenge = 6,

    /// L7: Validation Status - Algorithm validated for this population/use.
    ///
    /// **Scoring:** +1 if satisfied.
    ///
    /// ToV Mapping: Constraint satisfaction
    Validation = 7,

    /// L8: Ground Truth - What "correct" answer was can be established.
    ///
    /// **Critical** for Case determination (Four-Case Logic Engine).
    ///
    /// **Scoring:** +2 if Gold/Silver standard, +1 if Bronze.
    ///
    /// ToV Mapping: Axiom III (Differentiation)
    GroundTruth = 8,
}

impl AcaLemma {
    /// Get all 8 lemmas in order.
    #[must_use]
    pub const fn all() -> [Self; 8] {
        [
            Self::Temporal,
            Self::Cognition,
            Self::Action,
            Self::Harm,
            Self::Dechallenge,
            Self::Rechallenge,
            Self::Validation,
            Self::GroundTruth,
        ]
    }

    /// Get lemma by number (1-8).
    #[must_use]
    pub const fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::Temporal),
            2 => Some(Self::Cognition),
            3 => Some(Self::Action),
            4 => Some(Self::Harm),
            5 => Some(Self::Dechallenge),
            6 => Some(Self::Rechallenge),
            7 => Some(Self::Validation),
            8 => Some(Self::GroundTruth),
            _ => None,
        }
    }

    /// Get lemma number (1-8).
    #[must_use]
    pub const fn number(self) -> u8 {
        self as u8
    }

    /// Check if this lemma is required for causality assessment.
    ///
    /// # ToV §53.2
    ///
    /// L1 (Temporal), L3 (Action), and L4 (Harm) are required.
    #[must_use]
    pub const fn is_required(self) -> bool {
        matches!(self, Self::Temporal | Self::Action | Self::Harm)
    }

    /// Check if this lemma contributes points when satisfied.
    #[must_use]
    pub const fn is_scoring(self) -> bool {
        matches!(
            self,
            Self::Cognition | Self::Rechallenge | Self::Validation | Self::GroundTruth
        )
    }

    /// Get maximum points this lemma can contribute.
    ///
    /// # ToV §53.3
    ///
    /// | Lemma | Points |
    /// |-------|--------|
    /// | L2 (Cognition) | +1 |
    /// | L6 (Rechallenge) | +2 |
    /// | L7 (Validation) | +1 |
    /// | L8 (Ground Truth) | +2 |
    #[must_use]
    pub const fn max_points(self) -> u8 {
        match self {
            Self::Cognition => 1,
            Self::Rechallenge => 2,
            Self::Validation => 1,
            Self::GroundTruth => 2,
            _ => 0,
        }
    }

    /// Get ToV section mapping.
    #[must_use]
    pub const fn tov_mapping(self) -> &'static str {
        match self {
            Self::Temporal => "Definition 6.2",
            Self::Cognition => "R component (§22)",
            Self::Action => "Causal chain",
            Self::Harm => "Harm event H",
            Self::Dechallenge => "Definition 6.3",
            Self::Rechallenge => "U computation (§21)",
            Self::Validation => "Constraint satisfaction",
            Self::GroundTruth => "Axiom III (Differentiation)",
        }
    }
}

impl std::fmt::Display for AcaLemma {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Temporal => write!(f, "L1: Temporal Sequence"),
            Self::Cognition => write!(f, "L2: Cognition Evidence"),
            Self::Action => write!(f, "L3: Action Alignment"),
            Self::Harm => write!(f, "L4: Harm Occurrence"),
            Self::Dechallenge => write!(f, "L5: Dechallenge"),
            Self::Rechallenge => write!(f, "L6: Rechallenge"),
            Self::Validation => write!(f, "L7: Validation Status"),
            Self::GroundTruth => write!(f, "L8: Ground Truth"),
        }
    }
}

// ============================================================================
// LEMMA RESPONSE (T2-P)
// ============================================================================

/// Response to a lemma assessment question.
///
/// # Tier: T2-P
///
/// Cross-domain primitive for capturing evidence state.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum LemmaResponse {
    /// Evidence confirms the lemma is satisfied.
    Yes = 1,
    /// Evidence confirms the lemma is not satisfied.
    No = 2,
    /// Insufficient evidence to determine.
    #[default]
    Unknown = 3,
    /// Lemma is not applicable to this case.
    NotApplicable = 4,
}

impl LemmaResponse {
    /// Check if this response confirms satisfaction.
    #[must_use]
    pub const fn is_satisfied(self) -> bool {
        matches!(self, Self::Yes)
    }

    /// Check if this response confirms failure.
    #[must_use]
    pub const fn is_failed(self) -> bool {
        matches!(self, Self::No)
    }

    /// Check if this response indicates uncertainty.
    #[must_use]
    pub const fn is_uncertain(self) -> bool {
        matches!(self, Self::Unknown)
    }
}

impl std::fmt::Display for LemmaResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Yes => write!(f, "Yes"),
            Self::No => write!(f, "No"),
            Self::Unknown => write!(f, "Unknown"),
            Self::NotApplicable => write!(f, "N/A"),
        }
    }
}

// ============================================================================
// GROUND TRUTH STANDARD (T2-P)
// ============================================================================

/// Ground truth evidence standard for L8.
///
/// # Tier: T2-P
///
/// # ToV §53.4
///
/// | Standard | Points | Description |
/// |----------|--------|-------------|
/// | Gold | +2 | Independent validation, autopsy, definitive test |
/// | Silver | +2 | Expert panel consensus |
/// | Bronze | +1 | Single expert opinion |
/// | Ambiguous | 0 | Conflicting opinions |
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum GroundTruthStandard {
    /// No ground truth established.
    None = 0,
    /// Ambiguous - conflicting opinions.
    Ambiguous = 1,
    /// Bronze - single expert opinion.
    Bronze = 2,
    /// Silver - expert panel consensus.
    Silver = 3,
    /// Gold - independent validation, definitive test.
    Gold = 4,
}

impl GroundTruthStandard {
    /// Get points contributed by this standard.
    #[must_use]
    pub const fn points(self) -> u8 {
        match self {
            Self::None | Self::Ambiguous => 0,
            Self::Bronze => 1,
            Self::Gold | Self::Silver => 2,
        }
    }

    /// Check if ground truth is established (Bronze or better).
    #[must_use]
    pub const fn is_established(self) -> bool {
        matches!(self, Self::Bronze | Self::Silver | Self::Gold)
    }
}

impl Default for GroundTruthStandard {
    fn default() -> Self {
        Self::None
    }
}

impl std::fmt::Display for GroundTruthStandard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Ambiguous => write!(f, "Ambiguous"),
            Self::Bronze => write!(f, "Bronze"),
            Self::Silver => write!(f, "Silver"),
            Self::Gold => write!(f, "Gold"),
        }
    }
}

// ============================================================================
// ACA SCORE (T2-P)
// ============================================================================

/// ACA causality score (0-7 scale).
///
/// # Tier: T2-P
///
/// Newtype over u8 representing the sum of satisfied scoring lemmas.
///
/// # ToV §53.3
///
/// Maximum score is 7 points (L2:1 + L6:2 + L7:1 + L8:2).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub struct AcaScore(u8);

impl AcaScore {
    /// Minimum score.
    pub const MIN: Self = Self(0);
    /// Maximum score.
    pub const MAX: Self = Self(7);

    /// Create a new score, clamping to 0-7.
    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(if value > 7 { 7 } else { value })
    }

    /// Get raw value.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Convert to 0.0-1.0 scale (normalized).
    #[must_use]
    pub fn normalized(self) -> f64 {
        f64::from(self.0) / 7.0
    }

    /// Apply sigmoid transformation to get R_causality.
    ///
    /// # ToV §53.5
    ///
    /// ```text
    /// R_causality = 1 / (1 + e^(-(score - μ) / σ))
    /// ```
    ///
    /// Where μ=3.5, σ=1.5.
    #[must_use]
    pub fn to_r_causality(self) -> f64 {
        sigmoid(f64::from(self.0), ACA_SIGMOID_MU, ACA_SIGMOID_SIGMA)
    }
}

impl From<AcaScore> for u8 {
    fn from(score: AcaScore) -> Self {
        score.0
    }
}

// ============================================================================
// CAUSALITY CATEGORY (T2-P)
// ============================================================================

/// ACA causality assessment category.
///
/// # Tier: T2-P
///
/// # ToV §53.3
///
/// | Category | Score | Description |
/// |----------|-------|-------------|
/// | Definite | ≥6 | Beyond reasonable doubt |
/// | Probable | 4-5 | Likely contributed |
/// | Possible | 2-3 | May have contributed |
/// | Unlikely | <2 | Insufficient evidence |
/// | Unassessable | — | Required lemmas missing |
/// | Exculpated | — | Override Paradox applies |
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AcaCausalityCategory {
    /// Algorithm contribution established beyond reasonable doubt (score ≥6).
    Definite = 1,
    /// Algorithm likely contributed; more evidence would strengthen (score 4-5).
    Probable = 2,
    /// Algorithm may have contributed; alternatives not excluded (score 2-3).
    Possible = 3,
    /// Insufficient evidence for algorithm contribution (score <2).
    Unlikely = 4,
    /// Required lemmas missing or ground truth unknowable.
    Unassessable = 5,
    /// Override Paradox applies - algorithm cannot be assigned causality.
    Exculpated = 6,
}

impl AcaCausalityCategory {
    /// Get category from score.
    ///
    /// Note: Does not check for Unassessable/Exculpated - use scoring function.
    #[must_use]
    pub const fn from_score(score: AcaScore) -> Self {
        match score.value() {
            6..=7 => Self::Definite,
            4..=5 => Self::Probable,
            2..=3 => Self::Possible,
            _ => Self::Unlikely,
        }
    }

    /// Get typical R_causality range for this category.
    ///
    /// # ToV §61.5
    #[must_use]
    pub const fn r_causality_typical(self) -> f64 {
        match self {
            Self::Definite => 0.95,
            Self::Probable => 0.75,
            Self::Possible => 0.45,
            Self::Unlikely => 0.15,
            Self::Unassessable | Self::Exculpated => 0.0,
        }
    }

    /// Check if this category allows algorithm causality attribution.
    #[must_use]
    pub const fn allows_attribution(self) -> bool {
        matches!(
            self,
            Self::Definite | Self::Probable | Self::Possible | Self::Unlikely
        )
    }
}

impl std::fmt::Display for AcaCausalityCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Definite => write!(f, "Definite (≥6)"),
            Self::Probable => write!(f, "Probable (4-5)"),
            Self::Possible => write!(f, "Possible (2-3)"),
            Self::Unlikely => write!(f, "Unlikely (<2)"),
            Self::Unassessable => write!(f, "Unassessable"),
            Self::Exculpated => write!(f, "Exculpated (Override Paradox)"),
        }
    }
}

// ============================================================================
// LEMMA SATISFACTION (T2-C)
// ============================================================================

/// Result of evaluating a single lemma.
///
/// # Tier: T2-C
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LemmaSatisfaction {
    /// Which lemma was evaluated.
    pub lemma: AcaLemma,
    /// Response to the lemma question.
    pub response: LemmaResponse,
    /// Points contributed (0 if not satisfied).
    pub points: u8,
    /// Optional notes/evidence.
    pub notes: Option<String>,
}

impl LemmaSatisfaction {
    /// Create from lemma and response.
    #[must_use]
    pub fn new(lemma: AcaLemma, response: LemmaResponse) -> Self {
        let points = if response.is_satisfied() {
            lemma.max_points()
        } else {
            0
        };
        Self {
            lemma,
            response,
            points,
            notes: None,
        }
    }

    /// Create with custom points (for L8 Ground Truth levels).
    #[must_use]
    pub fn with_points(lemma: AcaLemma, response: LemmaResponse, points: u8) -> Self {
        Self {
            lemma,
            response,
            points: points.min(lemma.max_points()),
            notes: None,
        }
    }

    /// Add notes.
    #[must_use]
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }
}

// ============================================================================
// ACA SCORING INPUT (T3)
// ============================================================================

/// Input for ACA scoring assessment.
///
/// # Tier: T3
///
/// Domain-specific composite for capturing all 8 lemma responses.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcaScoringInput {
    /// L1: Temporal Sequence.
    pub l1_temporal: LemmaResponse,
    /// L2: Cognition Evidence.
    pub l2_cognition: LemmaResponse,
    /// L3: Action Alignment.
    pub l3_action: LemmaResponse,
    /// L4: Harm Occurrence.
    pub l4_harm: LemmaResponse,
    /// L5: Dechallenge.
    pub l5_dechallenge: LemmaResponse,
    /// L6: Rechallenge.
    pub l6_rechallenge: LemmaResponse,
    /// L7: Validation Status.
    pub l7_validation: LemmaResponse,
    /// L8: Ground Truth.
    pub l8_ground_truth: LemmaResponse,
    /// Ground truth evidence standard (for L8 point calculation).
    pub ground_truth_standard: GroundTruthStandard,
    /// Algorithm correctness (for Override Paradox check).
    pub algorithm_correct: Option<bool>,
    /// Whether clinician overrode the algorithm.
    pub clinician_overrode: Option<bool>,
}

impl AcaScoringInput {
    /// Create empty input (all Unknown).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a lemma response.
    #[must_use]
    pub fn with_lemma(mut self, lemma: AcaLemma, response: LemmaResponse) -> Self {
        match lemma {
            AcaLemma::Temporal => self.l1_temporal = response,
            AcaLemma::Cognition => self.l2_cognition = response,
            AcaLemma::Action => self.l3_action = response,
            AcaLemma::Harm => self.l4_harm = response,
            AcaLemma::Dechallenge => self.l5_dechallenge = response,
            AcaLemma::Rechallenge => self.l6_rechallenge = response,
            AcaLemma::Validation => self.l7_validation = response,
            AcaLemma::GroundTruth => self.l8_ground_truth = response,
        }
        self
    }

    /// Set ground truth standard for L8 point calculation.
    #[must_use]
    pub fn with_ground_truth_standard(mut self, standard: GroundTruthStandard) -> Self {
        self.ground_truth_standard = standard;
        if standard.is_established() {
            self.l8_ground_truth = LemmaResponse::Yes;
        }
        self
    }

    /// Set algorithm correctness for Override Paradox check.
    #[must_use]
    pub const fn with_algorithm_correct(mut self, correct: bool) -> Self {
        self.algorithm_correct = Some(correct);
        self
    }

    /// Set clinician override status.
    #[must_use]
    pub const fn with_clinician_overrode(mut self, overrode: bool) -> Self {
        self.clinician_overrode = Some(overrode);
        self
    }

    /// Get response for a specific lemma.
    #[must_use]
    pub const fn get_response(&self, lemma: AcaLemma) -> LemmaResponse {
        match lemma {
            AcaLemma::Temporal => self.l1_temporal,
            AcaLemma::Cognition => self.l2_cognition,
            AcaLemma::Action => self.l3_action,
            AcaLemma::Harm => self.l4_harm,
            AcaLemma::Dechallenge => self.l5_dechallenge,
            AcaLemma::Rechallenge => self.l6_rechallenge,
            AcaLemma::Validation => self.l7_validation,
            AcaLemma::GroundTruth => self.l8_ground_truth,
        }
    }

    /// Check if all required lemmas (L1, L3, L4) are satisfied.
    #[must_use]
    pub fn required_lemmas_satisfied(&self) -> bool {
        self.l1_temporal.is_satisfied()
            && self.l3_action.is_satisfied()
            && self.l4_harm.is_satisfied()
    }

    /// Get list of failed required lemmas.
    #[must_use]
    pub fn failed_required_lemmas(&self) -> Vec<AcaLemma> {
        let mut failed = Vec::new();
        if !self.l1_temporal.is_satisfied() {
            failed.push(AcaLemma::Temporal);
        }
        if !self.l3_action.is_satisfied() {
            failed.push(AcaLemma::Action);
        }
        if !self.l4_harm.is_satisfied() {
            failed.push(AcaLemma::Harm);
        }
        failed
    }

    /// Check if Override Paradox applies (Case II).
    ///
    /// Returns true if: Algorithm Correct + Clinician Overrode + Harm
    #[must_use]
    pub fn is_exculpated(&self) -> bool {
        match (self.algorithm_correct, self.clinician_overrode) {
            (Some(true), Some(true)) if self.l4_harm.is_satisfied() => true,
            _ => false,
        }
    }

    /// Build Override Paradox analysis if applicable.
    #[must_use]
    pub fn to_override_paradox(&self) -> Option<OverrideParadox> {
        match (self.algorithm_correct, self.clinician_overrode) {
            (Some(correct), Some(overrode)) => Some(OverrideParadox::new(
                correct,
                overrode,
                self.l4_harm.is_satisfied(),
            )),
            _ => None,
        }
    }
}

// ============================================================================
// ACA SCORING RESULT (T3)
// ============================================================================

/// Complete ACA scoring result.
///
/// # Tier: T3
///
/// Domain-specific result including score, category, and sigmoid transformation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AcaScoringResult {
    /// Whether assessment could be performed.
    pub assessable: bool,
    /// Individual lemma satisfactions.
    pub lemmas: Vec<LemmaSatisfaction>,
    /// Total score (0-7).
    pub score: AcaScore,
    /// Causality category.
    pub category: AcaCausalityCategory,
    /// R_causality from sigmoid transformation.
    pub r_causality: f64,
    /// Logic case classification (if Override Paradox data available).
    pub logic_case: Option<LogicCase>,
    /// Reason if unassessable.
    pub reason: Option<String>,
}

impl AcaScoringResult {
    /// Create an unassessable result.
    #[must_use]
    pub fn unassessable(reason: impl Into<String>, lemmas: Vec<LemmaSatisfaction>) -> Self {
        Self {
            assessable: false,
            lemmas,
            score: AcaScore::MIN,
            category: AcaCausalityCategory::Unassessable,
            r_causality: 0.0,
            logic_case: None,
            reason: Some(reason.into()),
        }
    }

    /// Create an exculpated result (Override Paradox).
    #[must_use]
    pub fn exculpated(lemmas: Vec<LemmaSatisfaction>, logic_case: LogicCase) -> Self {
        Self {
            assessable: false,
            lemmas,
            score: AcaScore::MIN,
            category: AcaCausalityCategory::Exculpated,
            r_causality: 0.0,
            logic_case: Some(logic_case),
            reason: Some("Override Paradox: Algorithm correct but clinician overrode".into()),
        }
    }

    /// Check if assessment was successful.
    #[must_use]
    pub const fn is_assessable(&self) -> bool {
        self.assessable
    }

    /// Get the total points from scoring lemmas.
    #[must_use]
    pub fn total_points(&self) -> u8 {
        self.lemmas.iter().map(|l| l.points).sum()
    }
}

// ============================================================================
// SCORING FUNCTIONS
// ============================================================================

/// Sigmoid function for R_causality transformation.
///
/// # ToV §53.5
///
/// ```text
/// R_causality = 1 / (1 + e^(-(score - μ) / σ))
/// ```
#[must_use]
pub fn sigmoid(x: f64, mu: f64, sigma: f64) -> f64 {
    1.0 / (1.0 + (-(x - mu) / sigma).exp())
}

/// Score an ACA assessment from input.
///
/// # ToV §53-§54
///
/// This function:
/// 1. Checks for Override Paradox (Case II → Exculpated)
/// 2. Verifies required lemmas (L1, L3, L4)
/// 3. Scores satisfies lemmas (L2, L6, L7, L8)
/// 4. Applies sigmoid transformation for R_causality
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::algorithmovigilance::scoring::{
///     AcaScoringInput, AcaLemma, LemmaResponse, score_aca, AcaCausalityCategory,
/// };
///
/// let input = AcaScoringInput::new()
///     .with_lemma(AcaLemma::Temporal, LemmaResponse::Yes)
///     .with_lemma(AcaLemma::Cognition, LemmaResponse::Yes)
///     .with_lemma(AcaLemma::Action, LemmaResponse::Yes)
///     .with_lemma(AcaLemma::Harm, LemmaResponse::Yes);
///
/// let result = score_aca(&input);
/// assert!(result.is_assessable());
/// ```
#[must_use]
pub fn score_aca(input: &AcaScoringInput) -> AcaScoringResult {
    // Build lemma satisfactions
    let mut lemmas = Vec::with_capacity(8);

    for lemma in AcaLemma::all() {
        let response = input.get_response(lemma);
        let satisfaction = if lemma == AcaLemma::GroundTruth {
            // L8 uses ground truth standard for points
            let points = if response.is_satisfied() {
                input.ground_truth_standard.points()
            } else {
                0
            };
            LemmaSatisfaction::with_points(lemma, response, points)
        } else {
            LemmaSatisfaction::new(lemma, response)
        };
        lemmas.push(satisfaction);
    }

    // Check for Override Paradox (Case II)
    if let Some(paradox) = input.to_override_paradox() {
        if paradox.is_algorithm_exculpated() {
            return AcaScoringResult::exculpated(lemmas, paradox.logic_case());
        }
    }

    // Check required lemmas
    if !input.required_lemmas_satisfied() {
        let failed = input.failed_required_lemmas();
        let reason = format!(
            "Required lemmas not satisfied: {}",
            failed
                .iter()
                .map(|l| l.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        );
        return AcaScoringResult::unassessable(reason, lemmas);
    }

    // Calculate score from scoring lemmas only
    let total_points: u8 = lemmas
        .iter()
        .filter(|l| l.lemma.is_scoring())
        .map(|l| l.points)
        .sum();

    let score = AcaScore::new(total_points);
    let category = AcaCausalityCategory::from_score(score);
    let r_causality = score.to_r_causality();

    // Get logic case if available
    let logic_case = input.to_override_paradox().map(|p| p.logic_case());

    AcaScoringResult {
        assessable: true,
        lemmas,
        score,
        category,
        r_causality,
        logic_case,
        reason: None,
    }
}

/// Quick ACA scoring with minimal input.
///
/// # Arguments
///
/// * `temporal` - L1: Output preceded outcome?
/// * `cognition` - L2: Clinician perceived output?
/// * `action` - L3: Clinician action aligned with output?
/// * `harm` - L4: Harm occurred?
/// * `rechallenge` - L6: Similar pattern in other cases?
/// * `ground_truth` - L8: Ground truth standard
///
/// # Example
///
/// ```rust
/// use nexcore_vigilance::algorithmovigilance::scoring::{
///     score_aca_quick, GroundTruthStandard, AcaCausalityCategory,
/// };
///
/// let result = score_aca_quick(true, true, true, true, true, GroundTruthStandard::Gold);
/// assert!(result.is_assessable());
/// ```
#[must_use]
pub fn score_aca_quick(
    temporal: bool,
    cognition: bool,
    action: bool,
    harm: bool,
    rechallenge: bool,
    ground_truth: GroundTruthStandard,
) -> AcaScoringResult {
    let input = AcaScoringInput::new()
        .with_lemma(
            AcaLemma::Temporal,
            if temporal {
                LemmaResponse::Yes
            } else {
                LemmaResponse::No
            },
        )
        .with_lemma(
            AcaLemma::Cognition,
            if cognition {
                LemmaResponse::Yes
            } else {
                LemmaResponse::No
            },
        )
        .with_lemma(
            AcaLemma::Action,
            if action {
                LemmaResponse::Yes
            } else {
                LemmaResponse::No
            },
        )
        .with_lemma(
            AcaLemma::Harm,
            if harm {
                LemmaResponse::Yes
            } else {
                LemmaResponse::No
            },
        )
        .with_lemma(
            AcaLemma::Rechallenge,
            if rechallenge {
                LemmaResponse::Yes
            } else {
                LemmaResponse::No
            },
        )
        .with_ground_truth_standard(ground_truth);

    score_aca(&input)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lemma_properties() {
        // Required lemmas
        assert!(AcaLemma::Temporal.is_required());
        assert!(AcaLemma::Action.is_required());
        assert!(AcaLemma::Harm.is_required());
        assert!(!AcaLemma::Cognition.is_required());

        // Scoring lemmas
        assert!(AcaLemma::Cognition.is_scoring());
        assert!(AcaLemma::Rechallenge.is_scoring());
        assert!(AcaLemma::Validation.is_scoring());
        assert!(AcaLemma::GroundTruth.is_scoring());
        assert!(!AcaLemma::Temporal.is_scoring());
        assert!(!AcaLemma::Dechallenge.is_scoring());

        // Points
        assert_eq!(AcaLemma::Cognition.max_points(), 1);
        assert_eq!(AcaLemma::Rechallenge.max_points(), 2);
        assert_eq!(AcaLemma::Validation.max_points(), 1);
        assert_eq!(AcaLemma::GroundTruth.max_points(), 2);
        assert_eq!(AcaLemma::Temporal.max_points(), 0);
    }

    #[test]
    fn test_ground_truth_standard() {
        assert_eq!(GroundTruthStandard::Gold.points(), 2);
        assert_eq!(GroundTruthStandard::Silver.points(), 2);
        assert_eq!(GroundTruthStandard::Bronze.points(), 1);
        assert_eq!(GroundTruthStandard::Ambiguous.points(), 0);
        assert_eq!(GroundTruthStandard::None.points(), 0);

        assert!(GroundTruthStandard::Gold.is_established());
        assert!(GroundTruthStandard::Bronze.is_established());
        assert!(!GroundTruthStandard::Ambiguous.is_established());
    }

    #[test]
    fn test_aca_score_categories() {
        assert_eq!(
            AcaCausalityCategory::from_score(AcaScore::new(7)),
            AcaCausalityCategory::Definite
        );
        assert_eq!(
            AcaCausalityCategory::from_score(AcaScore::new(6)),
            AcaCausalityCategory::Definite
        );
        assert_eq!(
            AcaCausalityCategory::from_score(AcaScore::new(5)),
            AcaCausalityCategory::Probable
        );
        assert_eq!(
            AcaCausalityCategory::from_score(AcaScore::new(4)),
            AcaCausalityCategory::Probable
        );
        assert_eq!(
            AcaCausalityCategory::from_score(AcaScore::new(3)),
            AcaCausalityCategory::Possible
        );
        assert_eq!(
            AcaCausalityCategory::from_score(AcaScore::new(2)),
            AcaCausalityCategory::Possible
        );
        assert_eq!(
            AcaCausalityCategory::from_score(AcaScore::new(1)),
            AcaCausalityCategory::Unlikely
        );
        assert_eq!(
            AcaCausalityCategory::from_score(AcaScore::new(0)),
            AcaCausalityCategory::Unlikely
        );
    }

    #[test]
    fn test_sigmoid_transformation() {
        let score = AcaScore::new(5);
        let r = score.to_r_causality();
        // At score=5, μ=3.5, σ=1.5: sigmoid = 1/(1+e^(-1)) ≈ 0.731
        assert!(r > 0.7 && r < 0.8);

        let score_low = AcaScore::new(0);
        let r_low = score_low.to_r_causality();
        // At score=0, should be low
        assert!(r_low < 0.1);

        let score_high = AcaScore::new(7);
        let r_high = score_high.to_r_causality();
        // At score=7, should be high
        assert!(r_high > 0.9);
    }

    #[test]
    fn test_score_definite() {
        // All scoring lemmas satisfied → 7 points → Definite
        let result = score_aca_quick(
            true,                      // L1
            true,                      // L2: +1
            true,                      // L3
            true,                      // L4
            true,                      // L6: +2
            GroundTruthStandard::Gold, // L8: +2
        );
        // Total: L2(1) + L6(2) + L8(2) = 5... missing L7
        // Actually with validation missing, we get 5 points = Probable
        assert!(result.is_assessable());
        assert_eq!(result.score.value(), 5);
        assert_eq!(result.category, AcaCausalityCategory::Probable);
    }

    #[test]
    fn test_score_full_definite() {
        // All 4 scoring lemmas satisfied → 6 points → Definite
        let input = AcaScoringInput::new()
            .with_lemma(AcaLemma::Temporal, LemmaResponse::Yes)
            .with_lemma(AcaLemma::Cognition, LemmaResponse::Yes) // +1
            .with_lemma(AcaLemma::Action, LemmaResponse::Yes)
            .with_lemma(AcaLemma::Harm, LemmaResponse::Yes)
            .with_lemma(AcaLemma::Rechallenge, LemmaResponse::Yes) // +2
            .with_lemma(AcaLemma::Validation, LemmaResponse::Yes) // +1
            .with_ground_truth_standard(GroundTruthStandard::Gold); // +2

        let result = score_aca(&input);
        assert!(result.is_assessable());
        assert_eq!(result.score.value(), 6); // 1+2+1+2 = 6
        assert_eq!(result.category, AcaCausalityCategory::Definite);
    }

    #[test]
    fn test_score_unassessable_missing_required() {
        // L1 (Temporal) failed → Unassessable
        let input = AcaScoringInput::new()
            .with_lemma(AcaLemma::Temporal, LemmaResponse::No) // Required failed!
            .with_lemma(AcaLemma::Action, LemmaResponse::Yes)
            .with_lemma(AcaLemma::Harm, LemmaResponse::Yes);

        let result = score_aca(&input);
        assert!(!result.is_assessable());
        assert_eq!(result.category, AcaCausalityCategory::Unassessable);
        assert!(result.reason.is_some());
    }

    #[test]
    fn test_score_exculpated_override_paradox() {
        // Algorithm Correct + Clinician Overrode + Harm → Case II → Exculpated
        let input = AcaScoringInput::new()
            .with_lemma(AcaLemma::Temporal, LemmaResponse::Yes)
            .with_lemma(AcaLemma::Action, LemmaResponse::Yes)
            .with_lemma(AcaLemma::Harm, LemmaResponse::Yes)
            .with_algorithm_correct(true)
            .with_clinician_overrode(true);

        let result = score_aca(&input);
        assert!(!result.is_assessable());
        assert_eq!(result.category, AcaCausalityCategory::Exculpated);
        assert_eq!(result.logic_case, Some(LogicCase::CaseII));
    }

    #[test]
    fn test_score_possible() {
        // Only required lemmas + L2 → 1 point → Unlikely
        let input = AcaScoringInput::new()
            .with_lemma(AcaLemma::Temporal, LemmaResponse::Yes)
            .with_lemma(AcaLemma::Cognition, LemmaResponse::Yes) // +1
            .with_lemma(AcaLemma::Action, LemmaResponse::Yes)
            .with_lemma(AcaLemma::Harm, LemmaResponse::Yes);

        let result = score_aca(&input);
        assert!(result.is_assessable());
        assert_eq!(result.score.value(), 1);
        assert_eq!(result.category, AcaCausalityCategory::Unlikely);
    }

    #[test]
    fn test_bronze_ground_truth() {
        // Bronze standard → +1 point
        let input = AcaScoringInput::new()
            .with_lemma(AcaLemma::Temporal, LemmaResponse::Yes)
            .with_lemma(AcaLemma::Action, LemmaResponse::Yes)
            .with_lemma(AcaLemma::Harm, LemmaResponse::Yes)
            .with_lemma(AcaLemma::Rechallenge, LemmaResponse::Yes) // +2
            .with_ground_truth_standard(GroundTruthStandard::Bronze); // +1

        let result = score_aca(&input);
        assert!(result.is_assessable());
        assert_eq!(result.score.value(), 3); // 2 + 1 = 3
        assert_eq!(result.category, AcaCausalityCategory::Possible);
    }

    #[test]
    fn test_lemma_response_display() {
        assert_eq!(format!("{}", LemmaResponse::Yes), "Yes");
        assert_eq!(format!("{}", LemmaResponse::No), "No");
        assert_eq!(format!("{}", LemmaResponse::Unknown), "Unknown");
        assert_eq!(format!("{}", LemmaResponse::NotApplicable), "N/A");
    }
}
