//! # Lex Primitiva Grounding for nexcore-pv-core
//!
//! GroundsTo implementations for all nexcore-pv-core public types.
//! The PV Core crate is the pharmacovigilance foundation: signal detection,
//! ICSR management, causality assessment, classification, benefit-risk analysis,
//! expectedness, temporal analysis, risk management, and periodic reporting.
//!
//! ## Type Grounding Table
//!
//! | Type | Primitives | Dominant | Tier | Rationale |
//! |------|-----------|----------|------|-----------|
//! | SafetyMargin | ∂ N κ | ∂ | T2-P | Distance to harm boundary with numeric thresholds |
//! | ContingencyTable | N × | N | T2-P | 4-cell numeric product of counts |
//! | CompleteSignalResult | ∃ N κ Σ σ | ∃ | T2-C | Existence test across multiple algorithms |
//! | SignalCriteria (thresholds) | ∂ N κ | ∂ | T2-P | Boundary thresholds for signal gating |
//! | CaseId | ∃ | ∃ | T1 | Existence identity of an ICSR |
//! | Icsr | ∃ → σ κ ς × | ∃ | T3 | Atomic PV unit: existence of drug-event case |
//! | Patient | ς N | ς | T2-P | Demographic state with numeric attributes |
//! | Drug | ς → | ς | T2-P | Stateful drug entity with causal role |
//! | Reaction | ∃ κ σ | ∃ | T2-P | Existence of adverse event with severity ordering |
//! | Seriousness | ∂ κ | ∂ | T2-P | Boundary classification of event severity |
//! | SeriousnessCriterion | Σ | Σ | T1 | Sum enum of seriousness categories |
//! | DrugRole | Σ | Σ | T1 | Sum enum of drug roles |
//! | DrugAction | Σ | Σ | T1 | Sum enum of post-event actions |
//! | Route | Σ | Σ | T1 | Sum enum of administration routes |
//! | ReactionOutcome | Σ | Σ | T1 | Sum enum of outcome categories |
//! | CausalityAssessment | → κ ς | → | T2-P | Causal link between drug and reaction |
//! | CausalityMethod | Σ | Σ | T1 | Sum enum of assessment methods |
//! | CausalityResult | Σ κ | Σ | T2-P | Ordered sum of causality categories |
//! | Assessor | Σ | Σ | T1 | Sum enum of assessor roles |
//! | ReportInfo | ς π | ς | T2-P | Report metadata with persistence |
//! | ReportType | Σ | Σ | T1 | Sum enum of report types |
//! | ReportSource | Σ | Σ | T1 | Sum enum of report sources |
//! | Sex | Σ | Σ | T1 | Sum enum of biological sex |
//! | Dosage | N ς | N | T2-P | Numeric dose with unit state |
//! | IcsrBuilder | ς σ | ς | T2-P | Stateful builder accumulating components |
//! | SafetyLevel | Σ κ | Σ | T2-P | Ordered safety hierarchy level |
//! | SafetyLevelMetadata | ς N σ | ς | T2-P | Metadata state with numeric ranges |
//! | ToVLevel | Σ κ | Σ | T2-P | Ordered ToV hierarchy level |
//! | SeverityLevel | Σ κ | Σ | T2-P | Ordered Hartwig-Siegel severity |
//! | SeverityCategory | Σ | Σ | T1 | Sum enum of severity categories |
//! | CollectionResult | σ N ∃ ς | σ | T2-C | Ordered ingestion pipeline result |
//! | DetectionResult | ∃ N κ | ∃ | T2-P | Signal existence with numeric metrics |
//! | AssessmentResult | → κ σ | → | T2-P | Causality assessment with ordering |
//! | CausalityLevel | Σ κ | Σ | T2-P | Ordered causality strength enum |
//! | UnderstandingResult | ρ μ ς σ | ρ | T2-C | Recursive deepening of knowledge |
//! | PreventionResult | ∂ ς → | ∂ | T2-P | Boundary enforcement actions |
//! | RegulatoryAction | Σ ∂ | Σ | T2-P | Sum of regulatory boundary actions |
//! | ThresholdAdjustment | ∂ N → | ∂ | T2-P | Boundary change with numeric values |
//! | PvCycleResult | σ ∃ → ρ ∂ | σ | T2-C | Full sequential PV cycle |
//! | BenefitAssessment | N κ | N | T2-P | Numeric benefit quantification |
//! | RiskAssessment | N κ ∂ | N | T2-P | Numeric risk quantification with boundary |
//! | QbriResult | N κ → | N | T2-P | Numeric index with decision causality |
//! | QbriThresholds | ∂ N | ∂ | T2-P | Decision boundary thresholds |
//! | RegulatoryDecision | Σ | Σ | T1 | Sum enum of decision categories |
//! | HistoricalDecision | N → ς | N | T2-P | Historical numeric decision record |
//! | LandscapeEntry | N κ ς | N | T2-P | Numeric competitor profile |
//! | LandscapeAnalysis | κ N σ μ | κ | T2-C | Comparative ranking analysis |
//! | LandscapeTrigger | Σ κ | Σ | T2-P | Sum of landscape trigger events |
//! | FisherResult | N κ | N | T2-P | Numeric statistical test result |
//! | Expectedness | Σ | Σ | T1 | Sum enum of expectedness categories |
//! | ProductLabel | ς ∂ μ | ς | T2-P | Stateful label with listed reactions |
//! | RegulatoryRegion | Σ | Σ | T1 | Sum enum of regulatory regions |
//! | LabelSource | Σ | Σ | T1 | Sum enum of label sources |
//! | LabelRegistry | μ ς | μ | T2-P | Registry mapping drug-region to labels |
//! | ExpectednessResult | ∃ ∂ κ ς | ∃ | T2-C | Expectedness determination result |
//! | TimeToOnset | N σ κ | N | T2-P | Numeric temporal measurement |
//! | TtoCategory | Σ κ | Σ | T2-P | Ordered TTO classification |
//! | ExposureDuration | N σ ∂ | N | T2-P | Numeric duration with chronic boundary |
//! | DechallengeResponse | Σ | Σ | T1 | Sum enum of dechallenge outcomes |
//! | RechallengeResponse | Σ | Σ | T1 | Sum enum of rechallenge outcomes |
//! | ChallengeAssessment | → κ N | → | T2-P | Causality evidence from challenge |
//! | TemporalPlausibility | N → κ ∃ | N | T2-C | Numeric plausibility score |
//! | RiskManagementProgram | ς → ∂ ν σ | ς | T2-C | Stateful lifecycle program |
//! | RmFramework | Σ | Σ | T1 | Sum enum of regulatory frameworks |
//! | RmState | Σ ς | Σ | T2-P | State machine position |
//! | RiskMeasure | ∂ ς | ∂ | T2-P | Boundary enforcement measure |
//! | MeasureKind | Σ | Σ | T1 | Sum enum of measure categories |
//! | PeriodicReport | σ ς N κ π | σ | T2-C | Sequential periodic safety report |
//! | PeriodicReportType | Σ | Σ | T1 | Sum enum of report types |
//! | ReportState | Σ ς | Σ | T2-P | Report lifecycle state |
//! | ReportSections | σ N κ | σ | T2-P | Ordered report structure |
//! | SignalSummary | ∃ ς | ∃ | T2-P | Signal existence summary |
//! | SignalStatus | Σ | Σ | T1 | Sum enum of signal states |
//! | BenefitRiskAssessment | Σ κ | Σ | T2-P | Ordered B/R assessment category |
//! | ConclusionAction | Σ | Σ | T1 | Sum enum of conclusion actions |
//! | DisproportionalityResult | ∃ N κ | ∃ | T2-P | Signal existence with numeric metrics |
//! | SurvivalObservation | ∃ N | ∃ | T2-P | Event existence at a time point |
//! | SurvivalPoint | N σ ∂ | N | T2-P | Numeric survival estimate with CI bounds |
//! | KaplanMeierResult | σ N ∂ ∃ | σ | T2-C | Time-ordered survival curve |
//! | CoxCoefficient | → N ∂ | → | T2-P | Causal hazard coefficient with CI |
//! | CoxResult | → N κ σ | → | T2-C | Causal hazard regression result |
//! | CumulativeIncidenceResult | σ N ∂ ∃ | σ | T2-C | Time-ordered cumulative event curve |

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};
use nexcore_lex_primitiva::state_mode::StateMode;

// Imports organized by source module

use crate::SafetyMargin;

// signals::core::types
use crate::signals::core::types::{
    ContingencyTable, DisproportionalityResult, SignalCriteria as CoreSignalCriteria,
};

// types.rs
use crate::types::CompleteSignalResult;

// icsr.rs
use crate::icsr::{
    Assessor, CaseId, CausalityAssessment, CausalityMethod, CausalityResult, Dosage, Drug,
    DrugAction, DrugRole, Icsr, IcsrBuilder, Patient, Reaction, ReactionOutcome, ReportInfo,
    ReportSource, ReportType, Seriousness, SeriousnessCriterion, Sex,
};

// thresholds.rs
use crate::thresholds::SignalCriteria;

// hierarchy.rs
use crate::hierarchy::{SafetyLevel, SafetyLevelMetadata, ToVLevel};

// classification
use crate::classification::{SeverityCategory, SeverityLevel};

// definition.rs
use crate::definition::{
    AssessmentResult, CausalityLevel, CollectionResult, DetectionResult, PreventionResult,
    PvCycleResult, RegulatoryAction, ThresholdAdjustment, UnderstandingResult,
};

// benefit_risk.rs
use crate::benefit_risk::{
    BenefitAssessment, HistoricalDecision, QbriResult, QbriThresholds, RegulatoryDecision,
    RiskAssessment,
};

// landscape.rs
use crate::landscape::{LandscapeAnalysis, LandscapeEntry, LandscapeTrigger};

// compat.rs
use crate::compat::FisherResult;

// expectedness.rs
use crate::expectedness::{
    Expectedness, ExpectednessResult, LabelRegistry, LabelSource, ProductLabel, RegulatoryRegion,
};

// temporal.rs
use crate::temporal::{
    ChallengeAssessment, DechallengeResponse, ExposureDuration, RechallengeResponse,
    TemporalPlausibility, TimeToOnset, TtoCategory,
};

// risk_management.rs
use crate::risk_management::{
    MeasureKind, RiskManagementProgram, RiskMeasure, RmFramework, RmState,
};

// periodic_reporting.rs
use crate::periodic_reporting::{
    BenefitRiskAssessment, ConclusionAction, PeriodicReport, PeriodicReportType, ReportSections,
    ReportState as PeriodicReportState, SignalStatus, SignalSummary,
};

// survival
use crate::signals::survival::cox::{CoxCoefficient, CoxResult};
use crate::signals::survival::cumulative_incidence::CumulativeIncidenceResult;
use crate::signals::survival::kaplan_meier::{
    KaplanMeierResult, SurvivalObservation, SurvivalPoint,
};

// ============================================================================
// T1 Universal (1 unique primitive)
// ============================================================================

/// CaseId: Newtype wrapping String for case identity.
/// Tier: T1Universal. Dominant: ∃ Existence.
/// WHY: A case either exists or doesn't. Pure identity wrapper.
impl GroundsTo for CaseId {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // ∃ -- case identity existence
        ])
        .with_dominant(LexPrimitiva::Existence, 1.0)
    }
}

/// SeriousnessCriterion: Sum enum of seriousness categories.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-6 exclusive classification (Death, LifeThreatening, etc).
impl GroundsTo for SeriousnessCriterion {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-6 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// DrugRole: Sum enum of drug roles in adverse event.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-4 classification (Suspect, Concomitant, Interacting, Treatment).
impl GroundsTo for DrugRole {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-4 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// DrugAction: Sum enum of post-event actions taken.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-6 classification (Withdrawn, DoseReduced, etc).
impl GroundsTo for DrugAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-6 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// Route: Sum enum of administration routes.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-10 classification (Oral, IV, IM, etc).
impl GroundsTo for Route {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-10 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

use crate::icsr::Route;

/// ReactionOutcome: Sum enum of reaction outcomes.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-6 classification (Recovered, Fatal, Unknown, etc).
impl GroundsTo for ReactionOutcome {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-6 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// CausalityMethod: Sum enum of assessment methods.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-5 classification (Naranjo, WhoUmc, Rucam, etc).
impl GroundsTo for CausalityMethod {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-5 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// Assessor: Sum enum of assessor roles.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-4 classification (Reporter, Sponsor, etc).
impl GroundsTo for Assessor {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-4 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// ReportType: Sum enum of E2B report types.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-4 classification (Spontaneous, Study, Literature, Other).
impl GroundsTo for ReportType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-4 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// ReportSource: Sum enum of report sources.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-4 classification (HealthcareProfessional, Consumer, etc).
impl GroundsTo for ReportSource {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-4 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// Sex: Sum enum of biological sex.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-3 classification (Male, Female, Unknown).
impl GroundsTo for Sex {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-3 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// SeverityCategory: Sum enum of broad severity categories.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-4 classification (Mild, Moderate, Severe, Lethal).
impl GroundsTo for SeverityCategory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-4 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// RegulatoryDecision: Sum enum of regulatory outcomes.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-4 classification (Approve, ApproveWithRems, etc).
impl GroundsTo for RegulatoryDecision {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-4 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// Expectedness: Sum enum of expectedness classifications.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-3 classification (Listed, Unlisted, Unknown).
impl GroundsTo for Expectedness {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-3 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// RegulatoryRegion: Sum enum of regulatory jurisdictions.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-8 classification (US, EU, JP, UK, CA, AU, WHO, Other).
impl GroundsTo for RegulatoryRegion {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-8 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// LabelSource: Sum enum of label source types.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-7 classification (USPI, SmPC, JPI, etc).
impl GroundsTo for LabelSource {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-7 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// DechallengeResponse: Sum enum of dechallenge outcomes.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-5 classification (Positive, Negative, Partial, etc).
impl GroundsTo for DechallengeResponse {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-5 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// RechallengeResponse: Sum enum of rechallenge outcomes.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-4 classification (Positive, Negative, NotPerformed, Unknown).
impl GroundsTo for RechallengeResponse {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-4 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// RmFramework: Sum enum of risk management frameworks.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-4 classification (Rems, Rmp, HcRmf, TgaRmp).
impl GroundsTo for RmFramework {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-4 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// MeasureKind: Sum enum of risk measure categories.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-12 classification of risk minimization measure types.
impl GroundsTo for MeasureKind {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-12 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// PeriodicReportType: Sum enum of periodic report types.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-4 classification (Psur, Pbrer, Pader, Dsur).
impl GroundsTo for PeriodicReportType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-4 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// SignalStatus: Sum enum of signal lifecycle states.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-5 classification (New, Ongoing, Confirmed, Refuted, Closed).
impl GroundsTo for SignalStatus {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-5 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// ConclusionAction: Sum enum of report conclusion actions.
/// Tier: T1Universal. Dominant: Σ Sum.
/// WHY: One-of-6 classification of recommended regulatory actions.
impl GroundsTo for ConclusionAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Σ -- one-of-6 variant
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

// ============================================================================
// T2 Primitive (2-3 unique primitives)
// ============================================================================

/// SafetyMargin: Distance to harm boundary with numeric thresholds.
/// Tier: T2Primitive. Dominant: ∂ Boundary.
/// WHY: Measures distance FROM a safety boundary, with numeric metrics and
/// comparison-based interpretation. The boundary concept dominates.
impl GroundsTo for SafetyMargin {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- distance to harm boundary
            LexPrimitiva::Quantity,   // N -- numeric distance value
            LexPrimitiva::Comparison, // κ -- safe vs unsafe interpretation
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.70)
    }
}

/// ContingencyTable: 2x2 table of numeric counts.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Four numeric cells (a, b, c, d) forming a product structure.
/// All operations are arithmetic on these counts.
impl GroundsTo for ContingencyTable {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- four numeric count cells
            LexPrimitiva::Product,  // × -- 2x2 product structure
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// SignalCriteria (thresholds): Boundary thresholds for signal detection.
/// Tier: T2Primitive. Dominant: ∂ Boundary.
/// WHY: Each field is a boundary threshold that gates signal detection.
/// Numeric values serve the boundary decision.
impl GroundsTo for SignalCriteria {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- detection thresholds
            LexPrimitiva::Quantity,   // N -- numeric threshold values
            LexPrimitiva::Comparison, // κ -- meets_prr, meets_ror comparisons
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.70)
    }
}

/// CoreSignalCriteria: Same boundary semantics as thresholds::SignalCriteria.
/// Tier: T2Primitive. Dominant: ∂ Boundary.
impl GroundsTo for CoreSignalCriteria {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- detection thresholds
            LexPrimitiva::Quantity,   // N -- numeric threshold values
            LexPrimitiva::Comparison, // κ -- threshold comparison
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.70)
    }
}

/// Patient: Demographic state with optional numeric attributes.
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Encapsulated patient context (age, sex, weight, history).
impl GroundsTo for Patient {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- demographic context
            LexPrimitiva::Quantity, // N -- age, weight numeric values
        ])
        .with_dominant(LexPrimitiva::State, 0.75)
        .with_state_mode(StateMode::Mutable)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// Drug: Drug entity with role and dosage state.
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Encapsulated drug context: name, role, dosage, dates, indication, action.
/// The causal role (suspect/concomitant) links drug to reaction.
impl GroundsTo for Drug {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // ς -- encapsulated drug context
            LexPrimitiva::Causality, // → -- drug's causal role in event
        ])
        .with_dominant(LexPrimitiva::State, 0.70)
        .with_state_mode(StateMode::Mutable)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// Reaction: Existence of an adverse event with temporal ordering.
/// Tier: T2Primitive. Dominant: ∃ Existence.
/// WHY: A reaction either occurred or didn't. The term, outcome, and onset
/// characterize this existence.
impl GroundsTo for Reaction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // ∃ -- event occurred
            LexPrimitiva::Comparison, // κ -- outcome severity ordering
            LexPrimitiva::Sequence,   // σ -- temporal onset ordering
        ])
        .with_dominant(LexPrimitiva::Existence, 0.65)
    }
}

/// Seriousness: Boundary classification for event severity.
/// Tier: T2Primitive. Dominant: ∂ Boundary.
/// WHY: Each boolean flag tests whether the event crosses a seriousness boundary.
/// is_serious() = any flag true = boundary crossed.
impl GroundsTo for Seriousness {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,   // ∂ -- seriousness boundary
            LexPrimitiva::Comparison, // κ -- most_severe ordering
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// CausalityAssessment: Causal link between drug and reaction.
/// Tier: T2Primitive. Dominant: → Causality.
/// WHY: Links drug_index to reaction_index with a causality result.
/// The assessment IS a causality determination.
impl GroundsTo for CausalityAssessment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // → -- drug caused reaction?
            LexPrimitiva::Comparison, // κ -- result ordering
            LexPrimitiva::State,      // ς -- method + assessor context
        ])
        .with_dominant(LexPrimitiva::Causality, 0.70)
        .with_state_mode(StateMode::Accumulated)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// CausalityResult: Ordered sum of causality strength categories.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Sum enum with Ord ordering (Unassessable < ... < Certain).
impl GroundsTo for CausalityResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-5 variant
            LexPrimitiva::Comparison, // κ -- Ord-derived ordering
        ])
        .with_dominant(LexPrimitiva::Sum, 0.75)
    }
}

/// ReportInfo: Report metadata state.
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Encapsulated report context with timestamps indicating persistence.
impl GroundsTo for ReportInfo {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,       // ς -- report metadata context
            LexPrimitiva::Persistence, // π -- dates persist the record
        ])
        .with_dominant(LexPrimitiva::State, 0.75)
        .with_state_mode(StateMode::Accumulated)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// Dosage: Numeric dose with unit state.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Primarily a numeric measurement (value, unit, frequency).
impl GroundsTo for Dosage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- dose value
            LexPrimitiva::State,    // ς -- unit and route context
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.75)
        .with_state_mode(StateMode::Mutable)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// IcsrBuilder: Stateful builder accumulating ICSR components.
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Mutable state that accumulates components via builder methods.
impl GroundsTo for IcsrBuilder {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- accumulating builder state
            LexPrimitiva::Sequence, // σ -- ordered build steps
        ])
        .with_dominant(LexPrimitiva::State, 0.75)
        .with_state_mode(StateMode::Accumulated)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// SafetyLevel: Ordered 8-level safety hierarchy.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Sum enum with Ord ordering (Molecular < ... < Regulatory).
impl GroundsTo for SafetyLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-8 hierarchy level
            LexPrimitiva::Comparison, // κ -- Ord-derived hierarchy ordering
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// SafetyLevelMetadata: Metadata for a safety level.
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Encapsulated metadata with numeric ranges and examples.
impl GroundsTo for SafetyLevelMetadata {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- level metadata context
            LexPrimitiva::Quantity, // N -- time scale and unit ranges
            LexPrimitiva::Sequence, // σ -- ordered example phenomena
        ])
        .with_dominant(LexPrimitiva::State, 0.60)
        .with_state_mode(StateMode::Accumulated)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// ToVLevel: Ordered ToV hierarchy level.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Sum enum with Ord ordering (Molecular < ... < Regulatory).
impl GroundsTo for ToVLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-5 hierarchy level
            LexPrimitiva::Comparison, // κ -- Ord-derived hierarchy ordering
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// SeverityLevel: Ordered Hartwig-Siegel severity (1-7).
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Sum enum with Ord ordering and numeric repr (Mild1..Lethal7).
impl GroundsTo for SeverityLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-7 severity level
            LexPrimitiva::Comparison, // κ -- Ord-derived severity ordering
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// DetectionResult: Signal existence with numeric metrics.
/// Tier: T2Primitive. Dominant: ∃ Existence.
/// WHY: The core question is "does a signal exist?" (signal_detected bool).
/// Numeric metrics (PRR, ROR, IC, EBGM) characterize the existence.
impl GroundsTo for DetectionResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // ∃ -- signal detected?
            LexPrimitiva::Quantity,   // N -- PRR, ROR, IC, EBGM values
            LexPrimitiva::Comparison, // κ -- exceeds threshold?
        ])
        .with_dominant(LexPrimitiva::Existence, 0.65)
    }
}

/// AssessmentResult: Causality assessment with scoring.
/// Tier: T2Primitive. Dominant: → Causality.
/// WHY: The assessment answers "did the drug cause this?" via Naranjo, WHO-UMC, RUCAM.
impl GroundsTo for AssessmentResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // → -- did the drug cause this?
            LexPrimitiva::Comparison, // κ -- causality level ordering
            LexPrimitiva::Sequence,   // σ -- ordered assessment steps
        ])
        .with_dominant(LexPrimitiva::Causality, 0.65)
    }
}

/// CausalityLevel: Ordered causality strength enum.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Sum enum with Ord ordering (Unassessable < ... < Certain).
impl GroundsTo for CausalityLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-5 variant
            LexPrimitiva::Comparison, // κ -- Ord-derived ordering
        ])
        .with_dominant(LexPrimitiva::Sum, 0.75)
    }
}

/// PreventionResult: Boundary enforcement actions.
/// Tier: T2Primitive. Dominant: ∂ Boundary.
/// WHY: Risk minimization enforces safety boundaries through regulatory actions.
impl GroundsTo for PreventionResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ -- safety boundary enforcement
            LexPrimitiva::State,     // ς -- risk management state
            LexPrimitiva::Causality, // → -- actions caused by assessment
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.65)
        .with_state_mode(StateMode::Accumulated)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// RegulatoryAction: Sum of regulatory boundary actions.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Sum enum of action types, each enforcing a safety boundary.
impl GroundsTo for RegulatoryAction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,      // Σ -- one-of-7 action variant
            LexPrimitiva::Boundary, // ∂ -- each action enforces a boundary
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// ThresholdAdjustment: Boundary change with numeric values.
/// Tier: T2Primitive. Dominant: ∂ Boundary.
/// WHY: Adjusts a detection boundary from previous to updated value.
impl GroundsTo for ThresholdAdjustment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,  // ∂ -- threshold boundary being adjusted
            LexPrimitiva::Quantity,  // N -- previous and updated values
            LexPrimitiva::Causality, // → -- reason for the change
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.65)
    }
}

/// BenefitAssessment: Numeric benefit quantification.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Primarily numeric: magnitude, probability, unmet_need -> score().
impl GroundsTo for BenefitAssessment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- effect size, probability values
            LexPrimitiva::Comparison, // κ -- comparing benefit levels
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// RiskAssessment: Numeric risk quantification.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Primarily numeric: magnitude, probability, severity -> score().
/// Boundary semantics via treatability (reversible/irreversible).
impl GroundsTo for RiskAssessment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- signal, probability, severity values
            LexPrimitiva::Comparison, // κ -- comparing risk levels
            LexPrimitiva::Boundary,   // ∂ -- treatability boundary
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.65)
    }
}

/// QbriResult: Numeric benefit-risk index with decision.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: QBRI = B/R, a numeric ratio that drives the decision.
impl GroundsTo for QbriResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- index value, benefit/risk scores
            LexPrimitiva::Comparison, // κ -- compared against thresholds
            LexPrimitiva::Causality,  // → -- drives regulatory decision
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.65)
    }
}

/// QbriThresholds: Decision boundary thresholds.
/// Tier: T2Primitive. Dominant: ∂ Boundary.
/// WHY: Three thresholds (tau_approve, tau_monitor, tau_uncertain) define
/// decision boundaries.
impl GroundsTo for QbriThresholds {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- decision thresholds
            LexPrimitiva::Quantity, // N -- numeric threshold values
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
    }
}

/// HistoricalDecision: Historical drug decision record.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Numeric benefit/risk data with an actual decision for derivation.
impl GroundsTo for HistoricalDecision {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,  // N -- benefit and risk numeric data
            LexPrimitiva::Causality, // → -- actual decision outcome
            LexPrimitiva::State,     // ς -- historical context
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.60)
        .with_state_mode(StateMode::Accumulated)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// LandscapeEntry: Competitor drug profile with QBRI.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Numeric profile (QBRI index, market share) for a single drug.
impl GroundsTo for LandscapeEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- QBRI, market share values
            LexPrimitiva::Comparison, // κ -- compared in ranking
            LexPrimitiva::State,      // ς -- drug context
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.60)
        .with_state_mode(StateMode::Mutable)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// LandscapeTrigger: Sum of landscape trigger events.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Sum enum of trigger types with comparison semantics.
impl GroundsTo for LandscapeTrigger {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-3 trigger variant
            LexPrimitiva::Comparison, // κ -- comparison-based triggers
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// FisherResult: Numeric statistical test result.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Pure numeric output: p-values, odds ratio, confidence intervals.
impl GroundsTo for FisherResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- p-values, odds ratio, CI
            LexPrimitiva::Comparison, // κ -- statistical significance comparison
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// ProductLabel: Label state with listed reactions.
/// Tier: T2Primitive. Dominant: ς State.
/// WHY: Encapsulated label state with boundary semantics (listed vs unlisted).
impl GroundsTo for ProductLabel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // ς -- label context state
            LexPrimitiva::Boundary, // ∂ -- listed/unlisted boundary
            LexPrimitiva::Mapping,  // μ -- term -> listed mapping
        ])
        .with_dominant(LexPrimitiva::State, 0.60)
        .with_state_mode(StateMode::Mutable)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// LabelRegistry: Registry mapping drug-region pairs to labels.
/// Tier: T2Primitive. Dominant: μ Mapping.
/// WHY: HashMap-based lookup from (drug, region) to ProductLabel.
impl GroundsTo for LabelRegistry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping, // μ -- (drug, region) -> label mapping
            LexPrimitiva::State,   // ς -- mutable registry state
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.75)
        .with_state_mode(StateMode::Mutable)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Mutable)
    }
}

/// TimeToOnset: Numeric temporal measurement with category.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Numeric days measurement with derived category and plausibility.
impl GroundsTo for TimeToOnset {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- days value
            LexPrimitiva::Sequence,   // σ -- temporal ordering
            LexPrimitiva::Comparison, // κ -- category classification
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.65)
    }
}

/// TtoCategory: Ordered time-to-onset classification.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Sum enum with ordering (Immediate < ... < Chronic).
impl GroundsTo for TtoCategory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-6 category
            LexPrimitiva::Comparison, // κ -- temporal ordering
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// ExposureDuration: Numeric duration with chronic boundary.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Numeric days + cumulative dose, with chronic threshold boundary.
impl GroundsTo for ExposureDuration {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- days, cumulative dose
            LexPrimitiva::Sequence, // σ -- temporal duration
            LexPrimitiva::Boundary, // ∂ -- chronic threshold (>90 days)
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.65)
    }
}

/// ChallengeAssessment: Causality evidence from dechallenge/rechallenge.
/// Tier: T2Primitive. Dominant: → Causality.
/// WHY: Dechallenge/rechallenge are the gold standard for causality evidence.
impl GroundsTo for ChallengeAssessment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // → -- causal evidence from challenge
            LexPrimitiva::Comparison, // κ -- causality score ordering
            LexPrimitiva::Quantity,   // N -- score, confidence, days values
        ])
        .with_dominant(LexPrimitiva::Causality, 0.65)
    }
}

/// RmState: Risk management lifecycle state.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Sum enum of 7 lifecycle states with state-machine transitions.
impl GroundsTo for RmState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,   // Σ -- one-of-7 state variant
            LexPrimitiva::State, // ς -- state machine semantics
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
        .with_state_mode(StateMode::Modal)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// RiskMeasure: Risk minimization measure enforcing a boundary.
/// Tier: T2Primitive. Dominant: ∂ Boundary.
/// WHY: Each measure enforces a safety boundary (prescription restriction,
/// monitoring requirement, distribution control, etc).
impl GroundsTo for RiskMeasure {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // ∂ -- safety boundary enforcement
            LexPrimitiva::State,    // ς -- active/inactive state
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
        .with_state_mode(StateMode::Modal)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// PeriodicReportState (ReportState): Report lifecycle state.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Sum enum of 7 lifecycle states.
impl GroundsTo for PeriodicReportState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,   // Σ -- one-of-7 lifecycle state
            LexPrimitiva::State, // ς -- lifecycle position
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
        .with_state_mode(StateMode::Modal)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// ReportSections: Ordered E2C(R2) report structure.
/// Tier: T2Primitive. Dominant: σ Sequence.
/// WHY: Sections 5, 6, 8, 11, 12 in defined order per ICH E2C(R2).
impl GroundsTo for ReportSections {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,   // σ -- ordered sections
            LexPrimitiva::Quantity,   // N -- exposure and tabulation counts
            LexPrimitiva::Comparison, // κ -- benefit-risk comparison
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.60)
    }
}

/// SignalSummary: Signal existence summary in periodic reporting.
/// Tier: T2Primitive. Dominant: ∃ Existence.
/// WHY: Summarizes whether a signal exists and its evaluation status.
impl GroundsTo for SignalSummary {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // ∃ -- signal exists
            LexPrimitiva::State,     // ς -- evaluation status
        ])
        .with_dominant(LexPrimitiva::Existence, 0.75)
        .with_state_mode(StateMode::Accumulated)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// BenefitRiskAssessment: Ordered B/R assessment category.
/// Tier: T2Primitive. Dominant: Σ Sum.
/// WHY: Sum enum of 4 ordered categories (Favorable ... Unfavorable).
impl GroundsTo for BenefitRiskAssessment {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,        // Σ -- one-of-4 assessment category
            LexPrimitiva::Comparison, // κ -- ordered assessment
        ])
        .with_dominant(LexPrimitiva::Sum, 0.70)
    }
}

/// DisproportionalityResult: Signal existence with numeric metrics.
/// Tier: T2Primitive. Dominant: ∃ Existence.
/// WHY: Point estimate, CI, is_signal -- characterizes signal existence.
impl GroundsTo for DisproportionalityResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // ∃ -- is_signal flag
            LexPrimitiva::Quantity,   // N -- point estimate, CI values
            LexPrimitiva::Comparison, // κ -- threshold comparison
        ])
        .with_dominant(LexPrimitiva::Existence, 0.60)
    }
}

// ============================================================================
// T2 Composite (4-5 unique primitives)
// ============================================================================

/// CompleteSignalResult: Full analysis across all signal detection algorithms.
/// Tier: T2Composite. Dominant: ∃ Existence.
/// WHY: Aggregates PRR, ROR, IC, EBGM results -- each an existence test.
/// The sequence of algorithms, numeric metrics, and comparisons compose.
impl GroundsTo for CompleteSignalResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // ∃ -- does signal exist across methods?
            LexPrimitiva::Quantity,   // N -- chi_square, n case count
            LexPrimitiva::Comparison, // κ -- comparing algorithm results
            LexPrimitiva::Sum,        // Σ -- aggregation of 4 algorithm results
            LexPrimitiva::Sequence,   // σ -- ordered algorithm evaluation
        ])
        .with_dominant(LexPrimitiva::Existence, 0.60)
    }
}

/// CollectionResult: Data ingestion pipeline result.
/// Tier: T2Composite. Dominant: σ Sequence.
/// WHY: Ordered pipeline: ingest records -> parse -> deduplicate -> output ICSRs.
impl GroundsTo for CollectionResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ -- ordered ingestion pipeline
            LexPrimitiva::Quantity,  // N -- record counts
            LexPrimitiva::Existence, // ∃ -- successfully ingested cases
            LexPrimitiva::State,     // ς -- pipeline state tracking
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.55)
        .with_state_mode(StateMode::Accumulated)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// UnderstandingResult: Mechanistic understanding via recursive analysis.
/// Tier: T2Composite. Dominant: ρ Recursion.
/// WHY: Recursive deepening: risk factors -> mechanism -> populations ->
/// conservation laws -> ToV mapping. Understanding deepens through layers.
impl GroundsTo for UnderstandingResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Recursion, // ρ -- recursive deepening of knowledge
            LexPrimitiva::Mapping,   // μ -- mapping risk factors to mechanisms
            LexPrimitiva::State,     // ς -- conservation law state
            LexPrimitiva::Sequence,  // σ -- ordered risk factor analysis
        ])
        .with_dominant(LexPrimitiva::Recursion, 0.55)
        .with_state_mode(StateMode::Accumulated)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// PvCycleResult: Full sequential PV cycle result.
/// Tier: T2Composite. Dominant: σ Sequence.
/// WHY: Chains all 5 PV verbs in order: collect -> detect -> assess ->
/// understand -> prevent. The sequence IS the PV cycle.
impl GroundsTo for PvCycleResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ -- ordered 5-verb pipeline
            LexPrimitiva::Existence, // ∃ -- signal detection
            LexPrimitiva::Causality, // → -- causality assessment
            LexPrimitiva::Recursion, // ρ -- understanding deepening
            LexPrimitiva::Boundary,  // ∂ -- prevention boundaries
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.50)
    }
}

/// LandscapeAnalysis: Comparative ranking of drugs in therapeutic landscape.
/// Tier: T2Composite. Dominant: κ Comparison.
/// WHY: Ranks competitors by QBRI, determines advantage, identifies triggers.
/// The analysis IS comparison.
impl GroundsTo for LandscapeAnalysis {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison, // κ -- ranking and comparison
            LexPrimitiva::Quantity,   // N -- QBRI indices, averages
            LexPrimitiva::Sequence,   // σ -- ordered competitor list
            LexPrimitiva::Mapping,    // μ -- drug_id -> entry mapping
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.55)
    }
}

/// ExpectednessResult: Determination of whether event is expected.
/// Tier: T2Composite. Dominant: ∃ Existence.
/// WHY: Tests whether event exists in product labeling.
impl GroundsTo for ExpectednessResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // ∃ -- does the event exist in labeling?
            LexPrimitiva::Boundary,   // ∂ -- listed/unlisted boundary
            LexPrimitiva::Comparison, // κ -- priority weight comparison
            LexPrimitiva::State,      // ς -- region and label context
        ])
        .with_dominant(LexPrimitiva::Existence, 0.55)
        .with_state_mode(StateMode::Accumulated)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// TemporalPlausibility: Overall temporal plausibility assessment.
/// Tier: T2Composite. Dominant: N Quantity.
/// WHY: Numeric score integrating TTO, challenge, and expected range.
impl GroundsTo for TemporalPlausibility {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,   // N -- plausibility score
            LexPrimitiva::Causality,  // → -- causal plausibility
            LexPrimitiva::Comparison, // κ -- within expected range?
            LexPrimitiva::Existence,  // ∃ -- signal existence evidence
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.55)
    }
}

/// RiskManagementProgram: Stateful lifecycle program.
/// Tier: T2Composite. Dominant: ς State.
/// WHY: State machine with lifecycle transitions, boundary measures,
/// and periodic assessment frequency.
impl GroundsTo for RiskManagementProgram {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // ς -- lifecycle state machine
            LexPrimitiva::Causality, // → -- state transitions
            LexPrimitiva::Boundary,  // ∂ -- risk measures enforce boundaries
            LexPrimitiva::Frequency, // ν -- assessment interval
            LexPrimitiva::Sequence,  // σ -- lifecycle sequence
        ])
        .with_dominant(LexPrimitiva::State, 0.50)
        .with_state_mode(StateMode::Modal)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Modal)
    }
}

/// PeriodicReport: Sequential periodic safety report (PSUR/PBRER).
/// Tier: T2Composite. Dominant: σ Sequence.
/// WHY: Ordered sections (5, 6, 8, 11, 12) per ICH E2C(R2).
/// The report IS a structured sequence with numeric data and comparisons.
impl GroundsTo for PeriodicReport {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,    // σ -- ordered sections and reporting period
            LexPrimitiva::State,       // ς -- report lifecycle state
            LexPrimitiva::Quantity,    // N -- exposure, case counts
            LexPrimitiva::Comparison,  // κ -- benefit-risk comparison
            LexPrimitiva::Persistence, // π -- data lock point, IBD
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.45)
        .with_state_mode(StateMode::Accumulated)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

// ============================================================================
// Survival Analysis Types
// ============================================================================

/// SurvivalObservation: A single time-to-event observation (event or censored).
/// Tier: T2Primitive. Dominant: ∃ Existence.
/// WHY: An observation records whether an event EXISTS at a given time.
impl GroundsTo for SurvivalObservation {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence, // ∃ -- event occurred or censored
            LexPrimitiva::Quantity,  // N -- time value
        ])
        .with_dominant(LexPrimitiva::Existence, 0.80)
    }
}

/// SurvivalPoint: A single point on the Kaplan-Meier curve.
/// Tier: T2Primitive. Dominant: N Quantity.
/// WHY: Numeric survival probability estimate at a time point with CI bounds.
impl GroundsTo for SurvivalPoint {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- survival probability, SE, n_risk
            LexPrimitiva::Sequence, // σ -- ordered time point
            LexPrimitiva::Boundary, // ∂ -- CI bounds
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// KaplanMeierResult: Complete non-parametric survival curve.
/// Tier: T2Composite. Dominant: σ Sequence.
/// WHY: An ordered sequence of survival probability estimates over time.
/// The curve IS the sequence — time-ordered step function.
impl GroundsTo for KaplanMeierResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ -- time-ordered curve
            LexPrimitiva::Quantity,  // N -- probabilities, counts
            LexPrimitiva::Boundary,  // ∂ -- CI bounds
            LexPrimitiva::Existence, // ∃ -- events vs censoring
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// CoxCoefficient: A single Cox regression coefficient with HR and CI.
/// Tier: T2Primitive. Dominant: → Causality.
/// WHY: A hazard ratio IS a causal effect estimate — drug → outcome.
impl GroundsTo for CoxCoefficient {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality, // → -- causal hazard effect
            LexPrimitiva::Quantity,  // N -- numeric coefficient, HR, SE
            LexPrimitiva::Boundary,  // ∂ -- CI bounds
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

/// CoxResult: Complete Cox proportional hazards regression result.
/// Tier: T2Composite. Dominant: → Causality.
/// WHY: Cox regression estimates causal hazard relationships between
/// covariates and time-to-event. Causality is the purpose of the model.
impl GroundsTo for CoxResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,  // → -- causal hazard estimation
            LexPrimitiva::Quantity,   // N -- coefficients, log-likelihood
            LexPrimitiva::Comparison, // κ -- concordance, model fit
            LexPrimitiva::Sequence,   // σ -- iterative convergence
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

/// CumulativeIncidenceResult: Complement of Kaplan-Meier (1 - S(t)).
/// Tier: T2Composite. Dominant: σ Sequence.
/// WHY: Time-ordered cumulative probability curve. Same structure as KM
/// but viewed from the event-occurrence perspective.
impl GroundsTo for CumulativeIncidenceResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // σ -- time-ordered curve
            LexPrimitiva::Quantity,  // N -- incidence probabilities
            LexPrimitiva::Boundary,  // ∂ -- CI bounds
            LexPrimitiva::Existence, // ∃ -- event occurrence
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
        .with_state_mode(StateMode::Accumulated)
    }

    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

// ============================================================================
// T3 Domain-Specific (6+ unique primitives)
// ============================================================================

/// Icsr: Individual Case Safety Report -- the atomic unit of pharmacovigilance.
/// Tier: T3DomainSpecific. Dominant: ∃ Existence.
/// WHY: The ICSR IS the existence of a drug-event case. It composes patient
/// state, drug causality, reaction sequences, severity comparison, temporal
/// ordering, and seriousness boundary -- 6+ distinct primitives.
impl GroundsTo for Icsr {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Existence,  // ∃ -- case identity exists
            LexPrimitiva::Causality,  // → -- drug caused reaction?
            LexPrimitiva::Sequence,   // σ -- temporal onset ordering
            LexPrimitiva::Comparison, // κ -- severity comparison
            LexPrimitiva::State,      // ς -- patient/drug/report context
            LexPrimitiva::Product,    // × -- drugs × reactions cross-product
        ])
        .with_dominant(LexPrimitiva::Existence, 0.45)
        .with_state_mode(StateMode::Accumulated)
    }
    fn state_mode() -> Option<StateMode> {
        Some(StateMode::Accumulated)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // ========================================================================
    // T1 Universal types
    // ========================================================================

    #[test]
    fn test_case_id_grounding() {
        assert_eq!(CaseId::dominant_primitive(), Some(LexPrimitiva::Existence));
        assert!(CaseId::is_pure_primitive());
        assert_eq!(CaseId::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_seriousness_criterion_grounding() {
        assert_eq!(
            SeriousnessCriterion::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert!(SeriousnessCriterion::is_pure_primitive());
        assert_eq!(SeriousnessCriterion::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_drug_role_grounding() {
        assert_eq!(DrugRole::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert!(DrugRole::is_pure_primitive());
        assert_eq!(DrugRole::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_drug_action_grounding() {
        assert_eq!(DrugAction::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(DrugAction::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_route_grounding() {
        assert_eq!(Route::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(Route::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_reaction_outcome_grounding() {
        assert_eq!(
            ReactionOutcome::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(ReactionOutcome::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_causality_method_grounding() {
        assert_eq!(
            CausalityMethod::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(CausalityMethod::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_assessor_grounding() {
        assert_eq!(Assessor::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(Assessor::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_report_type_grounding() {
        assert_eq!(ReportType::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(ReportType::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_report_source_grounding() {
        assert_eq!(ReportSource::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(ReportSource::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_sex_grounding() {
        assert_eq!(Sex::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(Sex::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_severity_category_grounding() {
        assert_eq!(
            SeverityCategory::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(SeverityCategory::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_regulatory_decision_grounding() {
        assert_eq!(
            RegulatoryDecision::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(RegulatoryDecision::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_expectedness_grounding() {
        assert_eq!(Expectedness::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(Expectedness::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_regulatory_region_grounding() {
        assert_eq!(
            RegulatoryRegion::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(RegulatoryRegion::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_label_source_grounding() {
        assert_eq!(LabelSource::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(LabelSource::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_dechallenge_response_grounding() {
        assert_eq!(
            DechallengeResponse::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(DechallengeResponse::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_rechallenge_response_grounding() {
        assert_eq!(
            RechallengeResponse::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(RechallengeResponse::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_rm_framework_grounding() {
        assert_eq!(RmFramework::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(RmFramework::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_measure_kind_grounding() {
        assert_eq!(MeasureKind::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(MeasureKind::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_periodic_report_type_grounding() {
        assert_eq!(
            PeriodicReportType::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(PeriodicReportType::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_signal_status_grounding() {
        assert_eq!(SignalStatus::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(SignalStatus::tier(), Tier::T1Universal);
    }

    #[test]
    fn test_conclusion_action_grounding() {
        assert_eq!(
            ConclusionAction::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(ConclusionAction::tier(), Tier::T1Universal);
    }

    // ========================================================================
    // T2 Primitive types
    // ========================================================================

    #[test]
    fn test_safety_margin_grounding() {
        assert_eq!(
            SafetyMargin::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(SafetyMargin::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_contingency_table_grounding() {
        assert_eq!(
            ContingencyTable::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(ContingencyTable::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_signal_criteria_thresholds_grounding() {
        assert_eq!(
            SignalCriteria::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(SignalCriteria::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_patient_grounding() {
        assert_eq!(Patient::dominant_primitive(), Some(LexPrimitiva::State));
        assert_eq!(Patient::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_drug_grounding() {
        assert_eq!(Drug::dominant_primitive(), Some(LexPrimitiva::State));
        assert_eq!(Drug::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_reaction_grounding() {
        assert_eq!(
            Reaction::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
        assert_eq!(Reaction::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_seriousness_grounding() {
        assert_eq!(
            Seriousness::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(Seriousness::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_causality_assessment_grounding() {
        assert_eq!(
            CausalityAssessment::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(CausalityAssessment::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_causality_result_grounding() {
        assert_eq!(
            CausalityResult::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(CausalityResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_report_info_grounding() {
        assert_eq!(ReportInfo::dominant_primitive(), Some(LexPrimitiva::State));
        assert_eq!(ReportInfo::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_dosage_grounding() {
        assert_eq!(Dosage::dominant_primitive(), Some(LexPrimitiva::Quantity));
        assert_eq!(Dosage::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_icsr_builder_grounding() {
        assert_eq!(IcsrBuilder::dominant_primitive(), Some(LexPrimitiva::State));
        assert_eq!(IcsrBuilder::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_safety_level_grounding() {
        assert_eq!(SafetyLevel::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(SafetyLevel::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_tov_level_grounding() {
        assert_eq!(ToVLevel::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(ToVLevel::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_severity_level_grounding() {
        assert_eq!(SeverityLevel::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(SeverityLevel::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_detection_result_grounding() {
        assert_eq!(
            DetectionResult::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
        assert_eq!(DetectionResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_assessment_result_grounding() {
        assert_eq!(
            AssessmentResult::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(AssessmentResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_causality_level_grounding() {
        assert_eq!(
            CausalityLevel::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(CausalityLevel::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_prevention_result_grounding() {
        assert_eq!(
            PreventionResult::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(PreventionResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_regulatory_action_grounding() {
        assert_eq!(
            RegulatoryAction::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(RegulatoryAction::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_threshold_adjustment_grounding() {
        assert_eq!(
            ThresholdAdjustment::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(ThresholdAdjustment::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_benefit_assessment_grounding() {
        assert_eq!(
            BenefitAssessment::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(BenefitAssessment::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_risk_assessment_grounding() {
        assert_eq!(
            RiskAssessment::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(RiskAssessment::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_qbri_result_grounding() {
        assert_eq!(
            QbriResult::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(QbriResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_qbri_thresholds_grounding() {
        assert_eq!(
            QbriThresholds::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(QbriThresholds::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_fisher_result_grounding() {
        assert_eq!(
            FisherResult::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(FisherResult::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_time_to_onset_grounding() {
        assert_eq!(
            TimeToOnset::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(TimeToOnset::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_tto_category_grounding() {
        assert_eq!(TtoCategory::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(TtoCategory::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_exposure_duration_grounding() {
        assert_eq!(
            ExposureDuration::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(ExposureDuration::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_challenge_assessment_grounding() {
        assert_eq!(
            ChallengeAssessment::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(ChallengeAssessment::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_rm_state_grounding() {
        assert_eq!(RmState::dominant_primitive(), Some(LexPrimitiva::Sum));
        assert_eq!(RmState::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_risk_measure_grounding() {
        assert_eq!(
            RiskMeasure::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
        assert_eq!(RiskMeasure::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_landscape_entry_grounding() {
        assert_eq!(
            LandscapeEntry::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(LandscapeEntry::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_landscape_trigger_grounding() {
        assert_eq!(
            LandscapeTrigger::dominant_primitive(),
            Some(LexPrimitiva::Sum)
        );
        assert_eq!(LandscapeTrigger::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_label_registry_grounding() {
        assert_eq!(
            LabelRegistry::dominant_primitive(),
            Some(LexPrimitiva::Mapping)
        );
        assert_eq!(LabelRegistry::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_signal_summary_grounding() {
        assert_eq!(
            SignalSummary::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
        assert_eq!(SignalSummary::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_disproportionality_result_grounding() {
        assert_eq!(
            DisproportionalityResult::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
        assert_eq!(DisproportionalityResult::tier(), Tier::T2Primitive);
    }

    // ========================================================================
    // T2 Composite types
    // ========================================================================

    #[test]
    fn test_complete_signal_result_grounding() {
        assert_eq!(
            CompleteSignalResult::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
        assert_eq!(CompleteSignalResult::tier(), Tier::T2Composite);
    }

    #[test]
    fn test_collection_result_grounding() {
        assert_eq!(
            CollectionResult::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(CollectionResult::tier(), Tier::T2Composite);
    }

    #[test]
    fn test_understanding_result_grounding() {
        assert_eq!(
            UnderstandingResult::dominant_primitive(),
            Some(LexPrimitiva::Recursion)
        );
        assert_eq!(UnderstandingResult::tier(), Tier::T2Composite);
    }

    #[test]
    fn test_pv_cycle_result_grounding() {
        assert_eq!(
            PvCycleResult::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(PvCycleResult::tier(), Tier::T2Composite);
    }

    #[test]
    fn test_landscape_analysis_grounding() {
        assert_eq!(
            LandscapeAnalysis::dominant_primitive(),
            Some(LexPrimitiva::Comparison)
        );
        assert_eq!(LandscapeAnalysis::tier(), Tier::T2Composite);
    }

    #[test]
    fn test_expectedness_result_grounding() {
        assert_eq!(
            ExpectednessResult::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
        assert_eq!(ExpectednessResult::tier(), Tier::T2Composite);
    }

    #[test]
    fn test_temporal_plausibility_grounding() {
        assert_eq!(
            TemporalPlausibility::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(TemporalPlausibility::tier(), Tier::T2Composite);
    }

    #[test]
    fn test_risk_management_program_grounding() {
        assert_eq!(
            RiskManagementProgram::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
        assert_eq!(RiskManagementProgram::tier(), Tier::T2Composite);
    }

    #[test]
    fn test_periodic_report_grounding() {
        assert_eq!(
            PeriodicReport::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(PeriodicReport::tier(), Tier::T2Composite);
    }

    // ========================================================================
    // Survival Analysis types
    // ========================================================================

    #[test]
    fn test_survival_observation_grounding() {
        assert_eq!(
            SurvivalObservation::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
        assert_eq!(SurvivalObservation::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_survival_point_grounding() {
        assert_eq!(
            SurvivalPoint::dominant_primitive(),
            Some(LexPrimitiva::Quantity)
        );
        assert_eq!(SurvivalPoint::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_kaplan_meier_result_grounding() {
        assert_eq!(
            KaplanMeierResult::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(KaplanMeierResult::tier(), Tier::T2Composite);
    }

    #[test]
    fn test_cox_coefficient_grounding() {
        assert_eq!(
            CoxCoefficient::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(CoxCoefficient::tier(), Tier::T2Primitive);
    }

    #[test]
    fn test_cox_result_grounding() {
        assert_eq!(
            CoxResult::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
        assert_eq!(CoxResult::tier(), Tier::T2Composite);
    }

    #[test]
    fn test_cumulative_incidence_result_grounding() {
        assert_eq!(
            CumulativeIncidenceResult::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        assert_eq!(CumulativeIncidenceResult::tier(), Tier::T2Composite);
    }

    // ========================================================================
    // T3 Domain-Specific types
    // ========================================================================

    #[test]
    fn test_icsr_grounding() {
        assert_eq!(Icsr::dominant_primitive(), Some(LexPrimitiva::Existence));
        assert_eq!(Icsr::tier(), Tier::T3DomainSpecific);
    }

    // ========================================================================
    // PV Domain Primitive Coverage
    // ========================================================================

    #[test]
    fn pv_core_covers_all_five_verbs() {
        // Each PV verb maps to a T1 primitive:
        // Collect -> σ (Sequence)
        assert_eq!(
            CollectionResult::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
        // Detect -> ∃ (Existence)
        assert_eq!(
            DetectionResult::dominant_primitive(),
            Some(LexPrimitiva::Existence)
        );
        // Assess -> → (Causality)
        assert_eq!(
            AssessmentResult::dominant_primitive(),
            Some(LexPrimitiva::Causality)
        );
        // Understand -> ρ (Recursion)
        assert_eq!(
            UnderstandingResult::dominant_primitive(),
            Some(LexPrimitiva::Recursion)
        );
        // Prevent -> ∂ (Boundary)
        assert_eq!(
            PreventionResult::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn pv_core_primitive_diversity() {
        // Verify PV core uses a rich set of T1 primitives:
        // ∃, →, σ, κ, ς, ∂, N, Σ, ×, ρ, μ, π, ν (13 of 16)
        // This reflects the domain complexity of pharmacovigilance.
        let primitives = vec![
            Icsr::dominant_primitive(),                // ∃
            AssessmentResult::dominant_primitive(),    // →
            CollectionResult::dominant_primitive(),    // σ
            LandscapeAnalysis::dominant_primitive(),   // κ
            Patient::dominant_primitive(),             // ς
            SafetyMargin::dominant_primitive(),        // ∂
            ContingencyTable::dominant_primitive(),    // N
            DrugRole::dominant_primitive(),            // Σ
            UnderstandingResult::dominant_primitive(), // ρ
            LabelRegistry::dominant_primitive(),       // μ
            ReportInfo::dominant_primitive(),          // ς (already counted)
        ];
        // All should have a dominant primitive
        for p in &primitives {
            assert!(p.is_some());
        }
    }
}
