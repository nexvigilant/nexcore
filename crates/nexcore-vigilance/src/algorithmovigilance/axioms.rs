//! # Algorithmovigilance Axioms (ToV §51-§52)
//!
//! Formal definitions for AI safety axioms under the Theory of Vigilance.
//!
//! # The Four ACA Axioms
//!
//! | Axiom | Statement | ToV Mapping |
//! |-------|-----------|-------------|
//! | I. Temporal Precedence | Cause precedes effect | Definition 6.2 |
//! | II. Causal Chain | O → C → A → H connected | Axiom 5 (Propagation) |
//! | III. Differentiation | Ground truth established | §21 (Configuration C) |
//! | IV. Epistemic Limit | If unknowable, unassessable | §33 (Confidence σ) |
//!
//! # The Alignment Principle (§51)
//!
//! > "Capability cannot be directionally pure. What learns, learns in all directions."
//!
//! The Pharmakon Principle (§34) applies to AI: any system capable enough to
//! provide clinical benefit is capable of causing clinical harm.
//!
//! # Example
//!
//! ```rust
//! use nexcore_vigilance::algorithmovigilance::{
//!     CausalChain, CausalChainLink, ChainLinkEvidence, AcaAxiom, AxiomSatisfaction,
//! };
//!
//! // Build a causal chain: Algorithm → Clinician → Action → Harm
//! let chain = CausalChain::new()
//!     .with_output(ChainLinkEvidence::strong("Model predicted sepsis"))
//!     .with_cognition(ChainLinkEvidence::moderate("Clinician viewed alert"))
//!     .with_action(ChainLinkEvidence::strong("Antibiotics administered"))
//!     .with_outcome(ChainLinkEvidence::strong("Allergic reaction documented"));
//!
//! // Check if temporal precedence is satisfied
//! let temporal = AcaAxiom::TemporalPrecedence;
//! assert!(chain.output().is_some());
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// CONSTANTS (ToV §53)
// ============================================================================

/// Sigmoid mu parameter for ACA → R_causality transformation.
///
/// # ToV §53.5
///
/// ```text
/// R_causality = sigmoid(ACA_score, μ=3.5, σ=1.5)
/// ```
pub const ACA_SIGMOID_MU: f64 = 3.5;

/// Sigmoid sigma parameter for ACA → R_causality transformation.
pub const ACA_SIGMOID_SIGMA: f64 = 1.5;

// ============================================================================
// CAUSAL CHAIN LINKS (T2-P)
// ============================================================================

/// Link in the O → C → A → H causal chain.
///
/// # Tier: T2-P
///
/// Cross-domain primitive enum grounding to T1 (u8 repr).
///
/// # ToV §52.2.2
///
/// The causal chain has four mandatory links. Algorithm causality
/// requires all links to be connected.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CausalChainLink {
    /// O: Algorithm produces recommendation/prediction
    Output = 0,
    /// C: Clinician perceives and processes output
    Cognition = 1,
    /// A: Clinician takes action (follow or override)
    Action = 2,
    /// H: Patient experiences harm or near-miss
    Outcome = 3,
}

impl CausalChainLink {
    /// Get the next link in the chain, if any.
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Output => Some(Self::Cognition),
            Self::Cognition => Some(Self::Action),
            Self::Action => Some(Self::Outcome),
            Self::Outcome => None,
        }
    }

    /// Get the previous link in the chain, if any.
    #[must_use]
    pub const fn previous(self) -> Option<Self> {
        match self {
            Self::Output => None,
            Self::Cognition => Some(Self::Output),
            Self::Action => Some(Self::Cognition),
            Self::Outcome => Some(Self::Action),
        }
    }

    /// Symbol used in ToV notation.
    #[must_use]
    pub const fn symbol(self) -> char {
        match self {
            Self::Output => 'O',
            Self::Cognition => 'C',
            Self::Action => 'A',
            Self::Outcome => 'H',
        }
    }
}

impl std::fmt::Display for CausalChainLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Output => write!(f, "Output (O)"),
            Self::Cognition => write!(f, "Cognition (C)"),
            Self::Action => write!(f, "Action (A)"),
            Self::Outcome => write!(f, "Outcome (H)"),
        }
    }
}

// ============================================================================
// EVIDENCE STRENGTH (T2-P)
// ============================================================================

/// Evidence strength for a causal chain link.
///
/// # Tier: T2-P
///
/// Newtype over u8 representing evidence quality (0-100).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EvidenceStrength(u8);

impl EvidenceStrength {
    /// No evidence.
    pub const NONE: Self = Self(0);
    /// Weak evidence (hearsay, indirect).
    pub const WEAK: Self = Self(25);
    /// Moderate evidence (documented but incomplete).
    pub const MODERATE: Self = Self(50);
    /// Strong evidence (independently verifiable).
    pub const STRONG: Self = Self(75);
    /// Conclusive evidence (gold standard).
    pub const CONCLUSIVE: Self = Self(100);

    /// Create from raw value, clamping to 0-100.
    #[must_use]
    pub const fn new(value: u8) -> Self {
        Self(if value > 100 { 100 } else { value })
    }

    /// Get raw value.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }

    /// Convert to 0.0-1.0 scale.
    #[must_use]
    pub fn as_f64(self) -> f64 {
        f64::from(self.0) / 100.0
    }

    /// Check if evidence is sufficient (≥50).
    #[must_use]
    pub const fn is_sufficient(self) -> bool {
        self.0 >= 50
    }
}

impl Default for EvidenceStrength {
    fn default() -> Self {
        Self::NONE
    }
}

// ============================================================================
// CHAIN LINK EVIDENCE (T2-C)
// ============================================================================

/// Evidence for a single causal chain link.
///
/// # Tier: T2-C
///
/// Composite type with strength and description.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChainLinkEvidence {
    /// Evidence strength (0-100).
    pub strength: EvidenceStrength,
    /// Description of the evidence.
    pub description: String,
}

impl ChainLinkEvidence {
    /// Create evidence with given strength and description.
    #[must_use]
    pub fn new(strength: EvidenceStrength, description: impl Into<String>) -> Self {
        Self {
            strength,
            description: description.into(),
        }
    }

    /// Create strong evidence.
    #[must_use]
    pub fn strong(description: impl Into<String>) -> Self {
        Self::new(EvidenceStrength::STRONG, description)
    }

    /// Create moderate evidence.
    #[must_use]
    pub fn moderate(description: impl Into<String>) -> Self {
        Self::new(EvidenceStrength::MODERATE, description)
    }

    /// Create weak evidence.
    #[must_use]
    pub fn weak(description: impl Into<String>) -> Self {
        Self::new(EvidenceStrength::WEAK, description)
    }

    /// Create with no evidence.
    #[must_use]
    pub fn none() -> Self {
        Self::new(EvidenceStrength::NONE, "")
    }
}

impl Default for ChainLinkEvidence {
    fn default() -> Self {
        Self::none()
    }
}

// ============================================================================
// CAUSAL CHAIN COMPONENT TYPES (T2-C)
// ============================================================================

/// Algorithm Output (O) - First link in causal chain.
///
/// # Tier: T2-C
///
/// # ToV §52.2.2
///
/// Algorithm produces recommendation/prediction that initiates the chain.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlgorithmOutput {
    /// Evidence for this link.
    pub evidence: ChainLinkEvidence,
    /// Timestamp of output (ISO 8601).
    pub timestamp: Option<String>,
    /// Whether output matched ground truth (if known).
    pub matched_ground_truth: Option<bool>,
}

impl AlgorithmOutput {
    /// Create with evidence.
    #[must_use]
    pub fn new(evidence: ChainLinkEvidence) -> Self {
        Self {
            evidence,
            timestamp: None,
            matched_ground_truth: None,
        }
    }

    /// Set timestamp.
    #[must_use]
    pub fn with_timestamp(mut self, timestamp: impl Into<String>) -> Self {
        self.timestamp = Some(timestamp.into());
        self
    }

    /// Set ground truth match status.
    #[must_use]
    pub const fn with_ground_truth(mut self, matched: bool) -> Self {
        self.matched_ground_truth = Some(matched);
        self
    }
}

/// Clinician Cognition (C) - Second link in causal chain.
///
/// # Tier: T2-C
///
/// # ToV §52.2.2
///
/// Clinician perceives and processes the algorithm output.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClinicianCognition {
    /// Evidence for this link.
    pub evidence: ChainLinkEvidence,
    /// Whether clinician acknowledged viewing the output.
    pub acknowledged: Option<bool>,
}

impl ClinicianCognition {
    /// Create with evidence.
    #[must_use]
    pub fn new(evidence: ChainLinkEvidence) -> Self {
        Self {
            evidence,
            acknowledged: None,
        }
    }

    /// Set acknowledgment status.
    #[must_use]
    pub const fn with_acknowledged(mut self, acknowledged: bool) -> Self {
        self.acknowledged = Some(acknowledged);
        self
    }
}

/// Clinician Action (A) - Third link in causal chain.
///
/// # Tier: T2-C
///
/// # ToV §52.2.2
///
/// Clinician takes action - either following or overriding the algorithm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClinicianAction {
    /// Evidence for this link.
    pub evidence: ChainLinkEvidence,
    /// Whether clinician followed the algorithm recommendation.
    pub followed_algorithm: Option<bool>,
}

impl ClinicianAction {
    /// Create with evidence.
    #[must_use]
    pub fn new(evidence: ChainLinkEvidence) -> Self {
        Self {
            evidence,
            followed_algorithm: None,
        }
    }

    /// Set whether algorithm was followed.
    #[must_use]
    pub const fn with_followed(mut self, followed: bool) -> Self {
        self.followed_algorithm = Some(followed);
        self
    }

    /// Check if clinician overrode the algorithm.
    #[must_use]
    pub fn overrode_algorithm(&self) -> Option<bool> {
        self.followed_algorithm.map(|f| !f)
    }
}

/// Harm Outcome (H) - Fourth link in causal chain.
///
/// # Tier: T2-C
///
/// # ToV §52.2.2
///
/// Patient experiences harm or near-miss.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HarmOutcome {
    /// Evidence for this link.
    pub evidence: ChainLinkEvidence,
    /// Severity of harm (if harm occurred).
    pub severity: Option<HarmSeverity>,
}

impl HarmOutcome {
    /// Create with evidence.
    #[must_use]
    pub fn new(evidence: ChainLinkEvidence) -> Self {
        Self {
            evidence,
            severity: None,
        }
    }

    /// Set severity.
    #[must_use]
    pub const fn with_severity(mut self, severity: HarmSeverity) -> Self {
        self.severity = Some(severity);
        self
    }
}

/// Harm severity categories.
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HarmSeverity {
    /// No harm (near-miss only).
    NearMiss = 0,
    /// Mild harm (temporary, no intervention).
    Mild = 1,
    /// Moderate harm (intervention required).
    Moderate = 2,
    /// Severe harm (hospitalization or permanent damage).
    Severe = 3,
    /// Fatal.
    Fatal = 4,
}

impl std::fmt::Display for HarmSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NearMiss => write!(f, "Near-Miss"),
            Self::Mild => write!(f, "Mild"),
            Self::Moderate => write!(f, "Moderate"),
            Self::Severe => write!(f, "Severe"),
            Self::Fatal => write!(f, "Fatal"),
        }
    }
}

// ============================================================================
// CAUSAL CHAIN (T3)
// ============================================================================

/// Complete O → C → A → H causal chain.
///
/// # Tier: T3
///
/// Domain-specific composite for ACA.
///
/// # ToV §52.2.2 (Axiom II)
///
/// Algorithmic contribution to harm requires a connected causal chain.
/// All four links must be established for algorithm causality attribution.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CausalChain {
    /// O: Algorithm output.
    output: Option<AlgorithmOutput>,
    /// C: Clinician cognition.
    cognition: Option<ClinicianCognition>,
    /// A: Clinician action.
    action: Option<ClinicianAction>,
    /// H: Harm outcome.
    outcome: Option<HarmOutcome>,
}

impl CausalChain {
    /// Create empty chain.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set output (O) with evidence.
    #[must_use]
    pub fn with_output(mut self, evidence: ChainLinkEvidence) -> Self {
        self.output = Some(AlgorithmOutput::new(evidence));
        self
    }

    /// Set output (O) with full struct.
    #[must_use]
    pub fn with_algorithm_output(mut self, output: AlgorithmOutput) -> Self {
        self.output = Some(output);
        self
    }

    /// Set cognition (C) with evidence.
    #[must_use]
    pub fn with_cognition(mut self, evidence: ChainLinkEvidence) -> Self {
        self.cognition = Some(ClinicianCognition::new(evidence));
        self
    }

    /// Set cognition (C) with full struct.
    #[must_use]
    pub fn with_clinician_cognition(mut self, cognition: ClinicianCognition) -> Self {
        self.cognition = Some(cognition);
        self
    }

    /// Set action (A) with evidence.
    #[must_use]
    pub fn with_action(mut self, evidence: ChainLinkEvidence) -> Self {
        self.action = Some(ClinicianAction::new(evidence));
        self
    }

    /// Set action (A) with full struct.
    #[must_use]
    pub fn with_clinician_action(mut self, action: ClinicianAction) -> Self {
        self.action = Some(action);
        self
    }

    /// Set outcome (H) with evidence.
    #[must_use]
    pub fn with_outcome(mut self, evidence: ChainLinkEvidence) -> Self {
        self.outcome = Some(HarmOutcome::new(evidence));
        self
    }

    /// Set outcome (H) with full struct.
    #[must_use]
    pub fn with_harm_outcome(mut self, outcome: HarmOutcome) -> Self {
        self.outcome = Some(outcome);
        self
    }

    /// Get output reference.
    #[must_use]
    pub fn output(&self) -> Option<&AlgorithmOutput> {
        self.output.as_ref()
    }

    /// Get cognition reference.
    #[must_use]
    pub fn cognition(&self) -> Option<&ClinicianCognition> {
        self.cognition.as_ref()
    }

    /// Get action reference.
    #[must_use]
    pub fn action(&self) -> Option<&ClinicianAction> {
        self.action.as_ref()
    }

    /// Get outcome reference.
    #[must_use]
    pub fn outcome(&self) -> Option<&HarmOutcome> {
        self.outcome.as_ref()
    }

    /// Check if chain is complete (all links present).
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.output.is_some()
            && self.cognition.is_some()
            && self.action.is_some()
            && self.outcome.is_some()
    }

    /// Get list of missing links.
    #[must_use]
    pub fn missing_links(&self) -> Vec<CausalChainLink> {
        let mut missing = Vec::new();
        if self.output.is_none() {
            missing.push(CausalChainLink::Output);
        }
        if self.cognition.is_none() {
            missing.push(CausalChainLink::Cognition);
        }
        if self.action.is_none() {
            missing.push(CausalChainLink::Action);
        }
        if self.outcome.is_none() {
            missing.push(CausalChainLink::Outcome);
        }
        missing
    }

    /// Get minimum evidence strength across all links.
    #[must_use]
    pub fn minimum_evidence(&self) -> EvidenceStrength {
        let strengths = [
            self.output.as_ref().map(|o| o.evidence.strength),
            self.cognition.as_ref().map(|c| c.evidence.strength),
            self.action.as_ref().map(|a| a.evidence.strength),
            self.outcome.as_ref().map(|h| h.evidence.strength),
        ];

        strengths
            .into_iter()
            .flatten()
            .min()
            .unwrap_or(EvidenceStrength::NONE)
    }

    /// Check if chain is sufficiently evidenced (all links ≥50).
    #[must_use]
    pub fn is_sufficiently_evidenced(&self) -> bool {
        self.is_complete() && self.minimum_evidence().is_sufficient()
    }
}

// ============================================================================
// ACA AXIOMS (T2-P)
// ============================================================================

/// The Four ACA Axioms from ToV §52.2.
///
/// # Tier: T2-P
///
/// Cross-domain primitive enum.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AcaAxiom {
    /// Axiom I: Cause must precede effect in time.
    ///
    /// ToV Mapping: Definition 6.2 (Temporal ordering in propagation)
    TemporalPrecedence = 1,

    /// Axiom II: O → C → A → H chain must be connected.
    ///
    /// ToV Mapping: Axiom 5 (Hierarchical Propagation)
    CausalChain = 2,

    /// Axiom III: Ground truth must be established.
    ///
    /// ToV Mapping: §21 (Configuration C)
    Differentiation = 3,

    /// Axiom IV: If unknowable, unassessable (Gödel's Clause).
    ///
    /// ToV Mapping: §33 (Confidence σ)
    EpistemicLimit = 4,
}

impl AcaAxiom {
    /// Get all axioms in order.
    #[must_use]
    pub const fn all() -> [Self; 4] {
        [
            Self::TemporalPrecedence,
            Self::CausalChain,
            Self::Differentiation,
            Self::EpistemicLimit,
        ]
    }

    /// Get axiom by number (1-4).
    #[must_use]
    pub const fn from_number(n: u8) -> Option<Self> {
        match n {
            1 => Some(Self::TemporalPrecedence),
            2 => Some(Self::CausalChain),
            3 => Some(Self::Differentiation),
            4 => Some(Self::EpistemicLimit),
            _ => None,
        }
    }

    /// Get axiom number (1-4).
    #[must_use]
    pub const fn number(self) -> u8 {
        self as u8
    }

    /// Get the ToV mapping for this axiom.
    #[must_use]
    pub const fn tov_mapping(self) -> &'static str {
        match self {
            Self::TemporalPrecedence => "Definition 6.2",
            Self::CausalChain => "Axiom 5",
            Self::Differentiation => "§21 (Configuration C)",
            Self::EpistemicLimit => "§33 (Confidence σ)",
        }
    }
}

impl std::fmt::Display for AcaAxiom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TemporalPrecedence => write!(f, "Axiom I: Temporal Precedence"),
            Self::CausalChain => write!(f, "Axiom II: Causal Chain"),
            Self::Differentiation => write!(f, "Axiom III: Differentiation"),
            Self::EpistemicLimit => write!(f, "Axiom IV: Epistemic Limit"),
        }
    }
}

// ============================================================================
// AXIOM SATISFACTION (T2-C)
// ============================================================================

/// Result of checking an ACA axiom.
///
/// # Tier: T2-C
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AxiomSatisfaction {
    /// Which axiom was checked.
    pub axiom: AcaAxiom,
    /// Whether the axiom is satisfied.
    pub satisfied: bool,
    /// Reason for satisfaction/failure.
    pub reason: String,
}

impl AxiomSatisfaction {
    /// Create a satisfied axiom result.
    #[must_use]
    pub fn satisfied(axiom: AcaAxiom, reason: impl Into<String>) -> Self {
        Self {
            axiom,
            satisfied: true,
            reason: reason.into(),
        }
    }

    /// Create a failed axiom result.
    #[must_use]
    pub fn failed(axiom: AcaAxiom, reason: impl Into<String>) -> Self {
        Self {
            axiom,
            satisfied: false,
            reason: reason.into(),
        }
    }
}

// ============================================================================
// OVERRIDE PARADOX (T2-C)
// ============================================================================

/// The Override Paradox analysis (ToV §51.3).
///
/// # Tier: T2-C
///
/// > A correct algorithm that is overridden cannot logically cause harm
/// > from that override.
///
/// This type encapsulates the analysis of whether the override paradox
/// applies to a given incident.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OverrideParadox {
    /// Was the algorithm correct (matched ground truth)?
    pub algorithm_correct: bool,
    /// Did the clinician override the algorithm?
    pub clinician_overrode: bool,
    /// Did harm occur?
    pub harm_occurred: bool,
}

impl OverrideParadox {
    /// Create a new override paradox analysis.
    #[must_use]
    pub const fn new(
        algorithm_correct: bool,
        clinician_overrode: bool,
        harm_occurred: bool,
    ) -> Self {
        Self {
            algorithm_correct,
            clinician_overrode,
            harm_occurred,
        }
    }

    /// Check if the override paradox applies (algorithm is exculpated).
    ///
    /// Returns true if: Algorithm Correct + Clinician Overrode + Harm
    ///
    /// In this case, the algorithm cannot logically be assigned causality.
    #[must_use]
    pub const fn is_algorithm_exculpated(&self) -> bool {
        self.algorithm_correct && self.clinician_overrode && self.harm_occurred
    }

    /// Get the Four-Case Logic Engine classification (ToV §54).
    ///
    /// | Case | Algorithm | Clinician | Outcome |
    /// |------|-----------|-----------|---------|
    /// | I | Wrong | Followed | Harm → Scoring |
    /// | II | Correct | Overrode | Harm → Exculpated |
    /// | III | Wrong | Overrode | No Harm → Near-Miss |
    /// | IV | Correct | Followed | No Harm → Baseline |
    #[must_use]
    pub const fn logic_case(&self) -> LogicCase {
        match (
            self.algorithm_correct,
            self.clinician_overrode,
            self.harm_occurred,
        ) {
            (false, false, true) => LogicCase::CaseI, // Wrong, Followed, Harm
            (true, true, true) => LogicCase::CaseII,  // Correct, Overrode, Harm
            (false, true, false) => LogicCase::CaseIII, // Wrong, Overrode, No Harm
            (true, false, false) => LogicCase::CaseIV, // Correct, Followed, No Harm
            // Edge cases - map to closest logical category
            (false, false, false) => LogicCase::CaseIV, // Wrong, Followed, No Harm (lucky)
            (true, true, false) => LogicCase::CaseIV,   // Correct, Overrode, No Harm
            (true, false, true) => LogicCase::CaseI,    // Correct, Followed, Harm (algo failed?)
            (false, true, true) => LogicCase::CaseI,    // Wrong, Overrode, Harm (both failed)
        }
    }
}

/// Four-Case Logic Engine classification (ToV §54).
///
/// # Tier: T2-P
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LogicCase {
    /// Algorithm Wrong + Clinician Followed + Harm → Proceed to Scoring.
    CaseI = 1,
    /// Algorithm Correct + Clinician Overrode + Harm → Algorithm Exculpated.
    CaseII = 2,
    /// Algorithm Wrong + Clinician Overrode + Harm Averted → Near-Miss Signal.
    CaseIII = 3,
    /// Algorithm Correct + Clinician Followed + Good Outcome → Success Baseline.
    CaseIV = 4,
}

impl std::fmt::Display for LogicCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CaseI => write!(f, "Case I: Algorithm-Contributed Incident"),
            Self::CaseII => write!(f, "Case II: Algorithm Exculpated (Override Paradox)"),
            Self::CaseIII => write!(f, "Case III: Near-Miss Signal"),
            Self::CaseIV => write!(f, "Case IV: Success Baseline"),
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_causal_chain_link_traversal() {
        assert_eq!(
            CausalChainLink::Output.next(),
            Some(CausalChainLink::Cognition)
        );
        assert_eq!(
            CausalChainLink::Cognition.next(),
            Some(CausalChainLink::Action)
        );
        assert_eq!(
            CausalChainLink::Action.next(),
            Some(CausalChainLink::Outcome)
        );
        assert_eq!(CausalChainLink::Outcome.next(), None);

        assert_eq!(CausalChainLink::Output.previous(), None);
        assert_eq!(
            CausalChainLink::Cognition.previous(),
            Some(CausalChainLink::Output)
        );
    }

    #[test]
    fn test_evidence_strength() {
        assert_eq!(EvidenceStrength::new(50).value(), 50);
        assert_eq!(EvidenceStrength::new(150).value(), 100); // Clamped
        assert!(EvidenceStrength::MODERATE.is_sufficient());
        assert!(!EvidenceStrength::WEAK.is_sufficient());
    }

    #[test]
    fn test_causal_chain_completeness() {
        let incomplete = CausalChain::new().with_output(ChainLinkEvidence::strong("Output"));
        assert!(!incomplete.is_complete());
        assert_eq!(incomplete.missing_links().len(), 3);

        let complete = CausalChain::new()
            .with_output(ChainLinkEvidence::strong("Output"))
            .with_cognition(ChainLinkEvidence::moderate("Cognition"))
            .with_action(ChainLinkEvidence::strong("Action"))
            .with_outcome(ChainLinkEvidence::strong("Outcome"));
        assert!(complete.is_complete());
        assert!(complete.missing_links().is_empty());
    }

    #[test]
    fn test_minimum_evidence() {
        let chain = CausalChain::new()
            .with_output(ChainLinkEvidence::strong("O")) // 75
            .with_cognition(ChainLinkEvidence::weak("C")) // 25
            .with_action(ChainLinkEvidence::moderate("A")) // 50
            .with_outcome(ChainLinkEvidence::strong("H")); // 75

        assert_eq!(chain.minimum_evidence(), EvidenceStrength::WEAK);
        assert!(!chain.is_sufficiently_evidenced()); // 25 < 50
    }

    #[test]
    fn test_aca_axioms() {
        assert_eq!(AcaAxiom::all().len(), 4);
        assert_eq!(AcaAxiom::from_number(1), Some(AcaAxiom::TemporalPrecedence));
        assert_eq!(AcaAxiom::from_number(5), None);
        assert_eq!(AcaAxiom::TemporalPrecedence.number(), 1);
    }

    #[test]
    fn test_override_paradox_exculpation() {
        // Case II: Correct + Overrode + Harm → Exculpated
        let case_ii = OverrideParadox::new(true, true, true);
        assert!(case_ii.is_algorithm_exculpated());
        assert_eq!(case_ii.logic_case(), LogicCase::CaseII);

        // Case I: Wrong + Followed + Harm → Scoring
        let case_i = OverrideParadox::new(false, false, true);
        assert!(!case_i.is_algorithm_exculpated());
        assert_eq!(case_i.logic_case(), LogicCase::CaseI);
    }

    #[test]
    fn test_logic_cases() {
        // All four canonical cases
        assert_eq!(
            OverrideParadox::new(false, false, true).logic_case(),
            LogicCase::CaseI
        );
        assert_eq!(
            OverrideParadox::new(true, true, true).logic_case(),
            LogicCase::CaseII
        );
        assert_eq!(
            OverrideParadox::new(false, true, false).logic_case(),
            LogicCase::CaseIII
        );
        assert_eq!(
            OverrideParadox::new(true, false, false).logic_case(),
            LogicCase::CaseIV
        );
    }

    #[test]
    fn test_sigmoid_constants() {
        // ToV §53.5: R_causality = sigmoid(ACA_score, μ=3.5, σ=1.5)
        assert!((ACA_SIGMOID_MU - 3.5).abs() < f64::EPSILON);
        assert!((ACA_SIGMOID_SIGMA - 1.5).abs() < f64::EPSILON);
    }
}
