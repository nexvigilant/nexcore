//! Core anti-vector types.
//!
//! The data model for structured countermeasures.

use nexcore_harm_taxonomy::HarmTypeId;
use serde::{Deserialize, Serialize};

// =============================================================================
// HARM VECTOR (the thing we're annihilating)
// =============================================================================

/// A harm vector: directed force carrying signal from drug to adverse event.
///
/// `→(cause) × N(magnitude)` — the fundamental unit of pharmacovigilance detection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarmVector {
    /// Drug or intervention causing harm
    pub source: String,
    /// Adverse event or harm outcome
    pub target: String,
    /// Harm type classification (A-H)
    pub harm_type: HarmTypeId,
    /// Signal magnitude (composite of PRR, ROR, IC, EBGM — normalized 0..1)
    pub magnitude: f64,
    /// Confidence in the signal (0..1)
    pub confidence: f64,
    /// The causal pathway description
    pub pathway: Option<String>,
}

// =============================================================================
// ANTI-VECTOR TYPES
// =============================================================================

/// The three classes of anti-vector, each annihilating a different aspect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AntiVectorClass {
    /// Breaks the causal chain. Output: specific intervention.
    /// Example: dose titration schedule that keeps acinar stimulation below threshold.
    Mechanistic,
    /// Cancels a false signal with structured counter-evidence.
    /// Example: indication bias packet showing T2DM comorbidity inflates pancreatitis reports.
    Epistemic,
    /// Engineers risk minimization that increases d(s).
    /// Example: DHCP letter + medication guide update (proportionate to signal magnitude).
    Architectural,
}

/// A mechanistic anti-vector: breaks the harm pathway.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MechanisticAntiVector {
    /// The specific mechanism being targeted
    pub pathway_target: String,
    /// The intervention that breaks the chain
    pub intervention: String,
    /// How the intervention neutralizes the pathway
    pub mechanism_of_action: String,
    /// Expected reduction in harm magnitude (0..1)
    pub expected_attenuation: f64,
    /// Evidence supporting this anti-vector
    pub evidence: Vec<EvidenceItem>,
}

/// An epistemic anti-vector: structured counter-evidence that cancels noise.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpistemicAntiVector {
    /// The bias or confounder being countered
    pub bias_type: BiasType,
    /// Counter-evidence magnitude (0..1, compared against signal magnitude)
    pub counter_magnitude: f64,
    /// Structured evidence packet
    pub evidence: Vec<EvidenceItem>,
    /// Net signal after anti-vector application (signal_magnitude - counter_magnitude)
    pub residual_signal: f64,
    /// Verdict: is the original signal likely real or noise?
    pub verdict: EpistemicVerdict,
}

/// An architectural anti-vector: risk minimization measure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitecturalAntiVector {
    /// The risk minimization measure class
    pub measure: RiskMinimizationMeasure,
    /// Proportionality score (anti-vector magnitude / harm vector magnitude)
    /// < 1.0 = insufficient, 1.0 = matched, > 1.0 = disproportionate
    pub proportionality: f64,
    /// Specific actions comprising this measure
    pub actions: Vec<String>,
    /// Expected increase in safety distance d(s)
    pub delta_safety_distance: f64,
}

// =============================================================================
// COMPOSITE ANTI-VECTOR
// =============================================================================

/// A complete anti-vector: one or more of the three classes combined.
///
/// Most real-world signals need all three: mechanistic understanding of WHY
/// it happens, epistemic assessment of WHETHER the signal is real, and
/// architectural planning for WHAT to do about it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiVector {
    /// The harm vector this anti-vector targets
    pub harm_vector: HarmVector,
    /// Mechanistic component (how to break the pathway)
    pub mechanistic: Option<MechanisticAntiVector>,
    /// Epistemic component (whether the signal is real)
    pub epistemic: Option<EpistemicAntiVector>,
    /// Architectural component (what risk minimization to deploy)
    pub architectural: Option<ArchitecturalAntiVector>,
    /// Overall anti-vector magnitude (combined from components)
    pub magnitude: f64,
    /// Net result after annihilation
    pub annihilation_result: AnnihilationResult,
}

// =============================================================================
// ANNIHILATION
// =============================================================================

/// Result of harm vector meeting anti-vector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnnihilationResult {
    /// |H| > |A|: residual harm remains
    ResidualHarm {
        /// Remaining harm magnitude after partial neutralization
        residual: f64,
        /// What additional anti-vector is needed
        gap: String,
    },
    /// |H| ≈ |A|: complete neutralization
    Annihilated {
        /// Knowledge released by the annihilation
        knowledge: String,
    },
    /// |H| < |A|: surplus protection
    SurplusProtection {
        /// Excess safety margin
        surplus: f64,
        /// Whether the measure is disproportionate (REMS when a label change suffices)
        disproportionate: bool,
    },
}

// =============================================================================
// SUPPORTING TYPES
// =============================================================================

/// Known bias types that generate false signals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BiasType {
    /// Drug prescribed FOR the condition it appears to cause
    IndicationBias,
    /// Media coverage inflates reporting (Weber effect)
    NotorietyBias,
    /// Early post-launch reporting surge
    WeberEffect,
    /// Stimulated reporting from regulatory action
    StimulatedReporting,
    /// Channeling bias — drug given to sicker patients
    ChannelingBias,
    /// Protopathic bias — drug given for early symptoms of the event
    ProtopathicBias,
    /// Depletion of susceptibles over time
    DepletionOfSusceptibles,
    /// Duplicate report inflation
    DuplicateReporting,
}

/// Epistemic verdict after applying counter-evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EpistemicVerdict {
    /// Signal survives counter-evidence — likely real
    SignalConfirmed,
    /// Signal partially attenuated — warrants investigation
    SignalAttenuated,
    /// Signal annihilated by counter-evidence — likely noise
    SignalRefuted,
}

/// Risk minimization measure categories (ICH E2C(R2) aligned).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskMinimizationMeasure {
    /// Label update (SmPC/PI section revision)
    LabelUpdate,
    /// Patient medication guide
    MedicationGuide,
    /// Dear Healthcare Provider letter
    DhcpLetter,
    /// Risk Evaluation and Mitigation Strategy (or EU equivalent)
    Rems,
    /// Restricted distribution program
    RestrictedDistribution,
    /// Required monitoring (lab tests, ECG, etc.)
    RequiredMonitoring,
    /// Dose modification or titration protocol
    DoseModification,
    /// Contraindication addition
    Contraindication,
    /// Market withdrawal (ultimate anti-vector)
    Withdrawal,
}

/// A piece of evidence supporting an anti-vector component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    /// Source type
    pub source: EvidenceSource,
    /// Description of the evidence
    pub description: String,
    /// Strength of evidence (0..1)
    pub strength: f64,
}

/// Evidence source classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EvidenceSource {
    /// Randomized controlled trial
    Rct,
    /// Observational study
    Observational,
    /// Case reports / case series
    CaseReport,
    /// Pharmacoepidemiological database analysis
    DatabaseAnalysis,
    /// Mechanistic / in vitro data
    Mechanistic,
    /// Regulatory assessment (PRAC, FDA review)
    RegulatoryAssessment,
    /// Published literature
    Literature,
    /// Label / SmPC information
    LabelInformation,
}
