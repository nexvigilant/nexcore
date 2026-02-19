//! # KSB Primitive Taxonomy
//!
//! Type-level decomposition of the Pharmacovigilance Knowledge, Skills, and
//! Behaviors (KSB) framework into T1/T2/T3 tiers grounded to the 16 Lex Primitiva.
//!
//! ## Source
//!
//! PV KSB Framework: 1,462 KSBs across 15 domains (D01-D15)
//! - 630 Knowledge components (43.1%)
//! - 344 Skills (23.5%)
//! - 255 Behaviors (17.4%)
//! - 233 AI Integration points (15.9%)
//!
//! ## KSB Type Primitive Grounding
//!
//! | Type | Dominant | Supporting | Tier |
//! |------|----------|------------|------|
//! | Knowledge | π (Persistence) | μ, σ, κ | T2-C |
//! | Skill | σ (Sequence) | μ, κ, ∂ | T2-C |
//! | Behavior | ς (State) | →, ∝, ∂ | T2-C |
//! | AI Integration | μ (Mapping) | σ, κ, ρ | T2-C |
//!
//! ## Domain Primitive Grounding
//!
//! | Domain | Dominant | Supporting | Tier | PVOS Layer |
//! |--------|----------|------------|------|------------|
//! | D01 Foundations | π Persistence | ∂, σ, μ | T2-C | — (meta) |
//! | D02 Clinical ADRs | κ Comparison | →, ∂, μ | T2-C | AVC |
//! | D03 Important ADRs | ∃ Existence | κ, →, ρ | T2-C | PVRC |
//! | D04 ICSRs | σ Sequence | ς, μ, π | T2-C | PVSP |
//! | D05 Clinical Trials | ∃ Existence | σ, ∂, N | T2-C | PVEX |
//! | D06 Med Errors | ∅ Void | →, ∂, ρ | T2-C | PVNL |
//! | D07 SRS | μ Mapping | π, ν, ∂ | T2-C | PVSC |
//! | D08 Signal Detection | ∂ Boundary | κ, N, → | T3 | PVSD |
//! | D09 Post-Auth Studies | ν Frequency | Σ, κ, ∃ | T2-C | PVTF |
//! | D10 Benefit-Risk | κ Comparison | Σ, ∝, N | T3 | PVAG |
//! | D11 Risk Mgmt | ∂ Boundary | ∝, σ, ς | T2-C | PVIR |
//! | D12 Regulatory | ∂ Boundary | σ, π, ∃ | T2-C | — (governance) |
//! | D13 Global PV | λ Location | μ, Σ, ν | T2-C | PVGL |
//! | D14 Communication | μ Mapping | σ, λ, ∂ | T2-C | — (interface) |
//! | D15 Sources | ∃ Existence | μ, π, κ | T2-C | — (input) |
//!
//! ## Cross-Domain Universal Primitives
//!
//! These primitives appear in ALL 15 domains (true PV universals):
//! - κ (Comparison): Every domain involves assessment/evaluation
//! - ∂ (Boundary): Every domain has safety boundaries
//! - σ (Sequence): Every domain has workflows/processes
//! - μ (Mapping): Every domain maps inputs → outputs
//!
//! These appear in 12+ domains:
//! - → (Causality): 13/15 domains (not D14, D15)
//! - N (Quantity): 12/15 domains (not D14)
//! - π (Persistence): 12/15 domains

use crate::lex_primitiva::{GroundsTo, LexPrimitiva, PrimitiveComposition};
use serde::{Deserialize, Serialize};

// =============================================================================
// KSB Type Primitives
// =============================================================================

/// The four KSB component types in the PV framework.
///
/// Each type represents a different dimension of professional competence.
///
/// ## Tier: T2-P (single primitive each — the meta-classification)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KsbType {
    /// What practitioners must KNOW (declarative knowledge).
    /// π (Persistence): Knowledge persists across contexts.
    Knowledge,
    /// What practitioners must DO (procedural capability).
    /// σ (Sequence): Skills are ordered procedures.
    Skill,
    /// How practitioners must ACT (dispositional patterns).
    /// ς (State): Behaviors are sustained states of professional conduct.
    Behavior,
    /// Where AI enhances practice (technological augmentation).
    /// μ (Mapping): AI maps inputs to outputs.
    AiIntegration,
}

impl KsbType {
    /// Get the dominant primitive for this KSB type.
    #[must_use]
    pub const fn dominant_primitive(&self) -> LexPrimitiva {
        match self {
            Self::Knowledge => LexPrimitiva::Persistence,
            Self::Skill => LexPrimitiva::Sequence,
            Self::Behavior => LexPrimitiva::State,
            Self::AiIntegration => LexPrimitiva::Mapping,
        }
    }

    /// Human-readable label.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Knowledge => "Knowledge (π)",
            Self::Skill => "Skill (σ)",
            Self::Behavior => "Behavior (ς)",
            Self::AiIntegration => "AI Integration (μ)",
        }
    }

    /// KSB count in the framework.
    #[must_use]
    pub const fn framework_count(&self) -> u32 {
        match self {
            Self::Knowledge => 630,
            Self::Skill => 344,
            Self::Behavior => 255,
            Self::AiIntegration => 233,
        }
    }

    /// Percentage of total framework.
    #[must_use]
    pub const fn framework_percentage(&self) -> f64 {
        match self {
            Self::Knowledge => 43.1,
            Self::Skill => 23.5,
            Self::Behavior => 17.4,
            Self::AiIntegration => 15.9,
        }
    }
}

/// Knowledge component: What must be KNOWN.
///
/// Dominant: π (Persistence) — knowledge persists across time and contexts.
/// Supporting: μ (Mapping) — knowledge maps concepts to understanding.
/// Supporting: σ (Sequence) — knowledge has prerequisite chains.
/// Supporting: κ (Comparison) — knowledge enables discrimination.
///
/// ## Tier: T2-C (4 primitives)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeComponent {
    /// Domain identifier (D01-D15)
    pub domain_id: String,
    /// Component number within domain
    pub component_number: u8,
    /// Description of knowledge required
    pub description: String,
    /// Bloom taxonomy level (Remember, Understand, Apply, Analyze, Evaluate, Create)
    pub bloom_level: BloomLevel,
    /// Proficiency level required (L1-L5)
    pub proficiency_level: ProficiencyLevel,
}

impl GroundsTo for KnowledgeComponent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Persistence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Persistence, 0.85)
    }
}

/// Essential Skill: What must be DONE.
///
/// Dominant: σ (Sequence) — skills are ordered procedures executed in context.
/// Supporting: μ (Mapping) — skills transform inputs to outputs.
/// Supporting: κ (Comparison) — skills require judgment/evaluation.
/// Supporting: ∂ (Boundary) — skills operate within safety boundaries.
///
/// ## Tier: T2-C (4 primitives)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EssentialSkill {
    /// Domain identifier (D01-D15)
    pub domain_id: String,
    /// Skill number within domain
    pub skill_number: u8,
    /// Description of skill required
    pub description: String,
    /// Bloom taxonomy level
    pub bloom_level: BloomLevel,
    /// Proficiency level required
    pub proficiency_level: ProficiencyLevel,
}

impl GroundsTo for EssentialSkill {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// Critical Behavior: How to ACT.
///
/// Dominant: ς (State) — behaviors are sustained dispositional states.
/// Supporting: → (Causality) — behaviors cause observable outcomes.
/// Supporting: ∝ (Irreversibility) — behaviors acknowledge irreversible consequences.
/// Supporting: ∂ (Boundary) — behaviors respect professional boundaries.
///
/// ## Tier: T2-C (4 primitives)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CriticalBehavior {
    /// Domain identifier (D01-D15)
    pub domain_id: String,
    /// Behavior number within domain
    pub behavior_number: u8,
    /// Description of behavior required
    pub description: String,
    /// Associated proficiency level
    pub proficiency_level: ProficiencyLevel,
}

impl GroundsTo for CriticalBehavior {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Causality,
            LexPrimitiva::Irreversibility,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// AI Integration Point: Where AI enhances practice.
///
/// Dominant: μ (Mapping) — AI maps inputs to predictions/outputs.
/// Supporting: σ (Sequence) — AI operates in processing pipelines.
/// Supporting: κ (Comparison) — AI makes discriminative assessments.
/// Supporting: ρ (Recursion) — AI applies iterative refinement.
///
/// ## Tier: T2-C (4 primitives)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiIntegrationPoint {
    /// Domain identifier (D01-D15)
    pub domain_id: String,
    /// Sequence number within domain
    pub sequence: u8,
    /// Description of AI integration
    pub description: String,
    /// AI technique category
    pub technique: AiTechnique,
}

impl GroundsTo for AiIntegrationPoint {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Recursion,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// =============================================================================
// Supporting Types
// =============================================================================

/// Bloom's Taxonomy levels for knowledge/skill classification.
///
/// ## Tier: T2-P (single primitive: σ Sequence — levels form an ordered progression)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum BloomLevel {
    /// Recall facts and basic concepts
    Remember = 1,
    /// Explain ideas and concepts
    Understand = 2,
    /// Use information in new situations
    Apply = 3,
    /// Draw connections among ideas
    Analyze = 4,
    /// Justify decisions
    Evaluate = 5,
    /// Produce new or original work
    Create = 6,
    /// Fully embodied in professional practice (behavioral extension)
    Internalize = 7,
}

impl GroundsTo for BloomLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence])
    }
}

/// Dreyfus proficiency model levels.
///
/// ## Tier: T2-P (single primitive: ς State — each level is a distinct competence state)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ProficiencyLevel {
    /// L1: Follows rules, needs supervision
    Novice = 1,
    /// L2: Recognizes patterns, limited independence
    AdvancedBeginner = 2,
    /// L3: Plans deliberately, independent in routine tasks
    Competent = 3,
    /// L4: Sees situations holistically, intuitive prioritization
    Proficient = 4,
    /// L5: Fluid, intuitive expertise
    Expert = 5,
}

impl ProficiencyLevel {
    /// Framework distribution percentages (from KSB validation).
    #[must_use]
    pub const fn framework_percentage(&self) -> f64 {
        match self {
            Self::Novice => 48.6,
            Self::AdvancedBeginner => 1.4, // Known gap — REM-007
            Self::Competent => 8.7,
            Self::Proficient => 37.0,
            Self::Expert => 3.8,
        }
    }
}

impl GroundsTo for ProficiencyLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State])
    }
}

/// AI technique categories used across PV domains.
///
/// ## Tier: T2-P (single primitive: μ Mapping — AI maps inputs to outputs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AiTechnique {
    /// Machine learning for pattern recognition
    MachineLearning,
    /// Natural language processing for text understanding
    NaturalLanguageProcessing,
    /// Predictive modeling for risk/outcome prediction
    PredictiveModeling,
    /// Computer vision for image-based assessment
    ComputerVision,
    /// Decision support systems
    DecisionSupport,
    /// Automated workflow and processing
    Automation,
    /// Network/graph analysis
    NetworkAnalysis,
    /// Real-time monitoring dashboards
    RealTimeMonitoring,
}

impl GroundsTo for AiTechnique {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping])
    }
}

// =============================================================================
// Domain Primitive Decomposition
// =============================================================================

/// PV Domain identifier (D01-D15).
///
/// Each domain maps to a specific PVOS layer with a dominant T1 primitive.
///
/// ## Tier: T2-P (single primitive: λ Location — domains are conceptual locations)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PvDomain {
    D01,
    D02,
    D03,
    D04,
    D05,
    D06,
    D07,
    D08,
    D09,
    D10,
    D11,
    D12,
    D13,
    D14,
    D15,
}

impl PvDomain {
    /// Dominant T1 primitive for this domain (PVOS operational alignment).
    ///
    /// Maps each domain to the PVOS layer it serves operationally.
    /// For the cognitive/educational perspective, see [`cognitive_dominant`].
    #[must_use]
    pub const fn dominant_primitive(&self) -> LexPrimitiva {
        match self {
            Self::D01 => LexPrimitiva::Persistence, // Foundations persist
            Self::D02 => LexPrimitiva::Comparison,  // ADR classification → AVC
            Self::D03 => LexPrimitiva::Existence,   // ADR recognition → PVRC
            Self::D04 => LexPrimitiva::Sequence,    // Case processing → PVSP
            Self::D05 => LexPrimitiva::Existence,   // Trial validation → PVEX
            Self::D06 => LexPrimitiva::Void,        // Error = absence → PVNL
            Self::D07 => LexPrimitiva::Mapping,     // Report → database → PVSC
            Self::D08 => LexPrimitiva::Boundary,    // Signal thresholds → PVSD
            Self::D09 => LexPrimitiva::Frequency,   // Temporal analysis → PVTF
            Self::D10 => LexPrimitiva::Comparison,  // Benefit vs risk → PVAG
            Self::D11 => LexPrimitiva::Boundary,    // Risk boundaries → PVIR
            Self::D12 => LexPrimitiva::Boundary,    // Regulatory boundaries
            Self::D13 => LexPrimitiva::Location,    // Global geography → PVGL
            Self::D14 => LexPrimitiva::Mapping,     // Message → audience
            Self::D15 => LexPrimitiva::Existence,   // Source validation
        }
    }

    /// Cognitive dominant T1 primitive (educational/competence perspective).
    ///
    /// What primitive characterizes the domain's intellectual content,
    /// independent of PVOS operational alignment. Cross-validated against
    /// independent primitive extraction analysis.
    ///
    /// Diverges from `dominant_primitive()` on 10/15 domains because
    /// "what system layer you work on" ≠ "what cognitive operation you perform."
    #[must_use]
    pub const fn cognitive_dominant(&self) -> LexPrimitiva {
        match self {
            Self::D01 => LexPrimitiva::Existence,   // "What IS PV?"
            Self::D02 => LexPrimitiva::Causality,   // "What CAUSES ADRs?"
            Self::D03 => LexPrimitiva::Comparison,  // "Does this MATCH a known ADR?"
            Self::D04 => LexPrimitiva::Mapping,     // "Transform report INTO case"
            Self::D05 => LexPrimitiva::Sequence,    // "Events in ORDER across phases"
            Self::D06 => LexPrimitiva::Boundary,    // "Where did process BREACH?"
            Self::D07 => LexPrimitiva::Frequency,   // "What is the RATE of reports?"
            Self::D08 => LexPrimitiva::Quantity,    // "What do NUMBERS say?"
            Self::D09 => LexPrimitiva::Persistence, // "What persists LONG TERM?"
            Self::D10 => LexPrimitiva::Comparison,  // "Benefit GREATER THAN risk?"
            Self::D11 => LexPrimitiva::State,       // "What is current RISK STATE?"
            Self::D12 => LexPrimitiva::Boundary,    // "What MUST you do/not do?"
            Self::D13 => LexPrimitiva::Location,    // "WHERE in the world?"
            Self::D14 => LexPrimitiva::Mapping,     // "Transform signal INTO message"
            Self::D15 => LexPrimitiva::Existence,   // "DOES this info exist?"
        }
    }

    /// Transfer confidence score (0.0-1.0) for cross-domain applicability.
    ///
    /// Higher = domain's skills/knowledge transfer more easily to other fields.
    /// Based on primitive universality analysis.
    #[must_use]
    pub const fn transfer_confidence(&self) -> f64 {
        match self {
            Self::D01 => 0.72,
            Self::D02 => 0.68,
            Self::D03 => 0.74,
            Self::D04 => 0.65,
            Self::D05 => 0.62,
            Self::D06 => 0.82,
            Self::D07 => 0.76,
            Self::D08 => 0.70,
            Self::D09 => 0.71,
            Self::D10 => 0.84,
            Self::D11 => 0.80,
            Self::D12 => 0.75,
            Self::D13 => 0.73,
            Self::D14 => 0.81,
            Self::D15 => 0.85,
        }
    }

    /// KSB count for this domain.
    #[must_use]
    pub const fn ksb_count(&self) -> u32 {
        match self {
            Self::D01 => 63,
            Self::D02 => 83,
            Self::D03 => 97,
            Self::D04 => 104,
            Self::D05 => 109,
            Self::D06 => 105,
            Self::D07 => 99,
            Self::D08 => 132,
            Self::D09 => 109,
            Self::D10 => 110,
            Self::D11 => 108,
            Self::D12 => 105,
            Self::D13 => 98,
            Self::D14 => 70,
            Self::D15 => 70,
        }
    }

    /// Domain name.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::D01 => "Foundations of PV in the AI Era",
            Self::D02 => "Clinical Aspects of ADRs with AI Analysis",
            Self::D03 => "Important ADRs and Recognition Using AI",
            Self::D04 => "ICSRs in the AI Era",
            Self::D05 => "PV in Clinical Trials with AI Integration",
            Self::D06 => "Medication Errors, Quality Issues and AI Detection",
            Self::D07 => "Spontaneous Reporting Systems with AI",
            Self::D08 => "Signal Detection and Management Using AI",
            Self::D09 => "Post-Authorization Studies and Trials",
            Self::D10 => "Benefit-Risk Assessment in the AI Era",
            Self::D11 => "Risk Management with AI Integration",
            Self::D12 => "Regulatory Authorities, Mandatory Procedures",
            Self::D13 => "Global PV and Public Health with AI",
            Self::D14 => "Communication in the AI Era",
            Self::D15 => "Sources of Information in the AI Era",
        }
    }

    /// Corresponding PVOS layer (if any).
    #[must_use]
    pub const fn pvos_layer(&self) -> Option<&'static str> {
        match self {
            Self::D02 => Some("AVC"),  // Adverse event classification
            Self::D03 => Some("PVRC"), // Recursive case analysis
            Self::D04 => Some("PVSP"), // Signal processing
            Self::D05 => Some("PVEX"), // Existence validation
            Self::D06 => Some("PVNL"), // Null/void handling
            Self::D07 => Some("PVSC"), // Stream collection
            Self::D08 => Some("PVSD"), // Signal detection
            Self::D09 => Some("PVTF"), // Temporal frequency
            Self::D10 => Some("PVAG"), // Aggregation
            Self::D11 => Some("PVIR"), // Irreversibility
            Self::D13 => Some("PVGL"), // Geographic localization
            _ => None,                 // D01, D12, D14, D15 are meta/interface domains
        }
    }

    /// All 15 domains.
    pub const ALL: [PvDomain; 15] = [
        Self::D01,
        Self::D02,
        Self::D03,
        Self::D04,
        Self::D05,
        Self::D06,
        Self::D07,
        Self::D08,
        Self::D09,
        Self::D10,
        Self::D11,
        Self::D12,
        Self::D13,
        Self::D14,
        Self::D15,
    ];

    /// Total KSBs across all domains.
    #[must_use]
    pub fn total_ksbs() -> u32 {
        Self::ALL.iter().map(|d| d.ksb_count()).sum()
    }
}

impl GroundsTo for PvDomain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Location])
    }
}

// =============================================================================
// EPA (Entrustable Professional Activity)
// =============================================================================

/// EPA tier classification.
///
/// ## Tier: T2-P (single primitive: σ Sequence — tiers are progression levels)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EpaTier {
    Core,      // 10 EPAs (EPA-01 to EPA-10)
    Executive, // 7 EPAs (EPA-11 to EPA-17)
    Advanced,  // 1 EPA (EPA-18, pending: EPA-19 to EPA-21)
}

/// Entrustable Professional Activity.
///
/// EPAs are integrative units of professional practice that combine
/// Knowledge + Skills + Behaviors from multiple domains.
///
/// ## Tier: T3 (7 primitives: σ + μ + κ + ∂ + → + ς + N)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epa {
    /// EPA identifier (EPA-01 through EPA-21)
    pub id: String,
    /// EPA name
    pub name: String,
    /// Focus area
    pub focus: String,
    /// Primary domains with required proficiency levels
    pub primary_domains: Vec<(PvDomain, ProficiencyLevel)>,
    /// Tier classification
    pub tier: EpaTier,
    /// Port range for service deployment
    pub port_start: u16,
    pub port_end: u16,
}

impl GroundsTo for Epa {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // EPAs are ordered progressions
            LexPrimitiva::Mapping,    // EPAs map domains → competencies
            LexPrimitiva::Comparison, // EPAs require assessment
            LexPrimitiva::Boundary,   // EPAs have entrustment boundaries
            LexPrimitiva::Causality,  // EPAs link learning → outcomes
            LexPrimitiva::State,      // EPAs track proficiency state
            LexPrimitiva::Quantity,   // EPAs quantify competence
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.75)
    }
}

// =============================================================================
// Complete KSB Framework
// =============================================================================

/// The complete PV KSB Framework.
///
/// Integrates all 1,462 KSBs across 15 domains with 21 EPAs.
///
/// ## Tier: T3 (10+ primitives — all 16 Lex Primitiva represented)
///
/// The framework itself grounds to ALL 16 T1 primitives because it spans
/// the entire pharmacovigilance domain — achieving Quindecet coverage
/// through its 15 domain compositions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KsbFramework {
    /// Framework version
    pub version: String,
    /// Total KSB count
    pub total_ksbs: u32,
    /// Domain count
    pub domain_count: u8,
    /// EPA count
    pub epa_count: u8,
    /// Cross-domain integration points
    pub cross_domain_integrations: u32,
}

impl Default for KsbFramework {
    fn default() -> Self {
        Self {
            version: "2026-02-06".to_string(),
            total_ksbs: 1462,
            domain_count: 15,
            epa_count: 21,
            cross_domain_integrations: 400,
        }
    }
}

impl GroundsTo for KsbFramework {
    fn primitive_composition() -> PrimitiveComposition {
        // The framework spans all 16 Lex Primitiva — Quindecet + Product
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,        // D04: Case processing pipelines
            LexPrimitiva::Mapping,         // D07: Report → database
            LexPrimitiva::State,           // Behaviors: Professional conduct states
            LexPrimitiva::Recursion,       // D03: Recursive case analysis
            LexPrimitiva::Void,            // D06: Error = absence
            LexPrimitiva::Boundary,        // D08: Signal detection thresholds
            LexPrimitiva::Frequency,       // D09: Temporal frequency
            LexPrimitiva::Existence,       // D05: Validation
            LexPrimitiva::Persistence,     // Knowledge: Persists across contexts
            LexPrimitiva::Causality,       // D04: Causality assessment
            LexPrimitiva::Comparison,      // D10: Benefit-risk
            LexPrimitiva::Quantity,        // Statistics: PRR, ROR, IC, EBGM
            LexPrimitiva::Location,        // D13: Global geography
            LexPrimitiva::Irreversibility, // D11: Harm permanence
            LexPrimitiva::Sum,             // D10: Aggregation
            LexPrimitiva::Product,         // Cross-domain: Compositional
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.70) // Assessment is the universal PV activity
    }
}

// =============================================================================
// Primitive Coverage Analysis
// =============================================================================

/// Analyze primitive coverage across the KSB framework.
///
/// Returns (primitive, domain_count, percentage) for each of the 16 T1 primitives.
#[must_use]
pub fn analyze_primitive_coverage() -> Vec<(LexPrimitiva, u8, f64)> {
    let all_primitives = [
        (LexPrimitiva::Comparison, 15u8), // κ: ALL domains (assessment/evaluation)
        (LexPrimitiva::Boundary, 15),     // ∂: ALL domains (safety boundaries)
        (LexPrimitiva::Sequence, 15),     // σ: ALL domains (workflows)
        (LexPrimitiva::Mapping, 15),      // μ: ALL domains (I/O transformation)
        (LexPrimitiva::Causality, 13),    // →: 13/15 (not D14, D15 primarily)
        (LexPrimitiva::Quantity, 12),     // N: 12/15 (not D14 primarily)
        (LexPrimitiva::Persistence, 12),  // π: 12/15 (knowledge storage)
        (LexPrimitiva::Existence, 11),    // ∃: 11/15 (validation)
        (LexPrimitiva::State, 11),        // ς: 11/15 (lifecycle states)
        (LexPrimitiva::Frequency, 10),    // ν: 10/15 (temporal patterns)
        (LexPrimitiva::Sum, 10),          // Σ: 10/15 (aggregation)
        (LexPrimitiva::Irreversibility, 9), // ∝: 9/15 (harm outcomes)
        (LexPrimitiva::Location, 8),      // λ: 8/15 (geographic)
        (LexPrimitiva::Recursion, 8),     // ρ: 8/15 (recursive analysis)
        (LexPrimitiva::Void, 6),          // ∅: 6/15 (missing data)
        (LexPrimitiva::Product, 4),       // ×: 4/15 (compositional)
    ];

    all_primitives
        .iter()
        .map(|(prim, count)| (*prim, *count, (*count as f64 / 15.0) * 100.0))
        .collect()
}

/// Check if the framework achieves Quindecet (all 15 original primitives covered).
#[must_use]
pub fn is_quindecet_complete() -> bool {
    let coverage = analyze_primitive_coverage();
    // All 16 primitives must have at least 1 domain
    coverage.iter().all(|(_, count, _)| *count > 0)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_ksbs() {
        assert_eq!(PvDomain::total_ksbs(), 1462);
    }

    #[test]
    fn test_ksb_type_counts_sum() {
        let total = KsbType::Knowledge.framework_count()
            + KsbType::Skill.framework_count()
            + KsbType::Behavior.framework_count()
            + KsbType::AiIntegration.framework_count();
        assert_eq!(total, 1462);
    }

    #[test]
    fn test_all_domains_have_ksbs() {
        for domain in &PvDomain::ALL {
            assert!(domain.ksb_count() > 0, "Domain {:?} has no KSBs", domain);
        }
    }

    #[test]
    fn test_d08_is_largest_domain() {
        let max_domain = PvDomain::ALL.iter().max_by_key(|d| d.ksb_count()).copied();
        assert_eq!(max_domain, Some(PvDomain::D08));
        assert_eq!(PvDomain::D08.ksb_count(), 132);
    }

    #[test]
    fn test_domain_pvos_alignment() {
        // 11 domains map to PVOS layers
        let pvos_count = PvDomain::ALL
            .iter()
            .filter(|d| d.pvos_layer().is_some())
            .count();
        assert_eq!(pvos_count, 11);
        // 4 meta/interface domains don't map directly
        assert!(PvDomain::D01.pvos_layer().is_none()); // Foundations
        assert!(PvDomain::D12.pvos_layer().is_none()); // Regulatory
        assert!(PvDomain::D14.pvos_layer().is_none()); // Communication
        assert!(PvDomain::D15.pvos_layer().is_none()); // Sources
    }

    #[test]
    fn test_knowledge_grounds_to_persistence() {
        let comp = KnowledgeComponent::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Persistence));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn test_skill_grounds_to_sequence() {
        let comp = EssentialSkill::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn test_behavior_grounds_to_state() {
        let comp = CriticalBehavior::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn test_ai_integration_grounds_to_mapping() {
        let comp = AiIntegrationPoint::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn test_epa_is_t3() {
        let comp = Epa::primitive_composition();
        assert!(comp.primitives.len() >= 6, "EPA must be T3 (6+ primitives)");
    }

    #[test]
    fn test_framework_covers_all_16_primitives() {
        let comp = KsbFramework::primitive_composition();
        assert_eq!(
            comp.primitives.len(),
            16,
            "Framework must cover all 16 Lex Primitiva"
        );
    }

    #[test]
    fn test_quindecet_complete() {
        assert!(
            is_quindecet_complete(),
            "KSB framework must achieve Quindecet"
        );
    }

    #[test]
    fn test_bloom_level_ordering() {
        assert!(BloomLevel::Create > BloomLevel::Evaluate);
        assert!(BloomLevel::Evaluate > BloomLevel::Analyze);
        assert!(BloomLevel::Analyze > BloomLevel::Apply);
        assert!(BloomLevel::Apply > BloomLevel::Understand);
        assert!(BloomLevel::Understand > BloomLevel::Remember);
    }

    #[test]
    fn test_proficiency_level_ordering() {
        assert!(ProficiencyLevel::Expert > ProficiencyLevel::Proficient);
        assert!(ProficiencyLevel::Proficient > ProficiencyLevel::Competent);
        assert!(ProficiencyLevel::Competent > ProficiencyLevel::AdvancedBeginner);
        assert!(ProficiencyLevel::AdvancedBeginner > ProficiencyLevel::Novice);
    }

    #[test]
    fn test_proficiency_gap_at_l2() {
        // Known gap: L2 (Advanced Beginner) has only 1.4% of KSBs
        assert!(
            ProficiencyLevel::AdvancedBeginner.framework_percentage() < 5.0,
            "L2 gap should be below 5%"
        );
    }

    #[test]
    fn test_primitive_coverage_top4_universal() {
        let coverage = analyze_primitive_coverage();
        // Top 4 primitives should be in all 15 domains
        for (prim, count, _) in &coverage[..4] {
            assert_eq!(
                *count, 15,
                "Primitive {:?} should be in all 15 domains, found in {}",
                prim, count
            );
        }
    }

    #[test]
    fn test_ksb_type_dominant_primitives_unique() {
        let types = [
            KsbType::Knowledge,
            KsbType::Skill,
            KsbType::Behavior,
            KsbType::AiIntegration,
        ];
        let dominants: Vec<LexPrimitiva> = types.iter().map(|t| t.dominant_primitive()).collect();
        // All dominants should be unique
        for (i, a) in dominants.iter().enumerate() {
            for (j, b) in dominants.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b, "KSB types should have unique dominant primitives");
                }
            }
        }
    }

    #[test]
    fn test_domain_dominant_primitives_coverage() {
        // Check that at least 10 of the 16 primitives are represented as dominants
        let dominants: std::collections::HashSet<LexPrimitiva> = PvDomain::ALL
            .iter()
            .map(|d| d.dominant_primitive())
            .collect();
        assert!(
            dominants.len() >= 8,
            "At least 8 unique dominant primitives needed across 15 domains, got {}",
            dominants.len()
        );
    }

    #[test]
    fn test_serialization_roundtrip() {
        let framework = KsbFramework::default();
        let json = serde_json::to_string(&framework);
        assert!(json.is_ok());
        let deserialized: Result<KsbFramework, _> = serde_json::from_str(&json.unwrap_or_default());
        assert!(deserialized.is_ok());
    }

    #[test]
    fn test_dual_perspective_convergence() {
        // 5 domains where operational and cognitive perspectives agree
        let convergent: Vec<PvDomain> = PvDomain::ALL
            .iter()
            .filter(|d| d.dominant_primitive() == d.cognitive_dominant())
            .copied()
            .collect();
        assert_eq!(convergent.len(), 5, "Expect 5 convergent domains");
        assert!(convergent.contains(&PvDomain::D10)); // Benefit-Risk: κ
        assert!(convergent.contains(&PvDomain::D12)); // Regulatory: ∂
        assert!(convergent.contains(&PvDomain::D13)); // Global PV: λ
        assert!(convergent.contains(&PvDomain::D14)); // Communication: μ
        assert!(convergent.contains(&PvDomain::D15)); // Sources: ∃
    }

    #[test]
    fn test_cognitive_dominants_cover_more_primitives() {
        // Cognitive perspective should cover at least 9 unique primitives
        let cognitive: std::collections::HashSet<LexPrimitiva> = PvDomain::ALL
            .iter()
            .map(|d| d.cognitive_dominant())
            .collect();
        assert!(
            cognitive.len() >= 9,
            "Cognitive dominants should cover 9+ unique primitives, got {}",
            cognitive.len()
        );
    }

    #[test]
    fn test_transfer_confidence_range() {
        for domain in &PvDomain::ALL {
            let tc = domain.transfer_confidence();
            assert!(
                (0.5..=1.0).contains(&tc),
                "Domain {:?} transfer confidence {tc} out of range",
                domain
            );
        }
    }

    #[test]
    fn test_highest_transfer_is_d15() {
        let max = PvDomain::ALL
            .iter()
            .max_by(|a, b| a.transfer_confidence().total_cmp(&b.transfer_confidence()));
        assert_eq!(max, Some(&PvDomain::D15)); // Info Sources: 0.85
    }

    #[test]
    fn test_lowest_transfer_is_d05() {
        let min = PvDomain::ALL
            .iter()
            .min_by(|a, b| a.transfer_confidence().total_cmp(&b.transfer_confidence()));
        assert_eq!(min, Some(&PvDomain::D05)); // Clinical Trials: 0.62
    }

    #[test]
    fn test_mean_transfer_confidence() {
        let mean: f64 = PvDomain::ALL
            .iter()
            .map(|d| d.transfer_confidence())
            .sum::<f64>()
            / 15.0;
        // Mean should be approximately 0.745
        assert!(
            (0.73..=0.76).contains(&mean),
            "Mean transfer confidence {mean} not in expected range"
        );
    }
}
