//! T2-C cross-domain composites for pharmacovigilance.
//!
//! 20 composed concepts built from T2-P primitives. Each grounds to
//! 3-5 unique Lex Primitiva symbols and appears in multiple domains.

use crate::lex::{LexSymbol, PrimitiveComposition};
use crate::primitives::PvPrimitive;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 20 cross-domain composites active in pharmacovigilance.
///
/// Tier: T2-C | Each concept grounds to 3-5 unique symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PvComposite {
    /// 2×2 matrix of counts cross-tabulating exposure and outcome.
    ContingencyTable,
    /// Statistical disproportionality warranting investigation.
    Signal,
    /// State machine tracking signal from detection to closure.
    SignalLifecycle,
    /// Ordinal classification of signal magnitude.
    SignalStrength,
    /// Any untoward medical occurrence in a patient (ICH E2A).
    AdverseEvent,
    /// Response to a drug which is noxious and unintended (WHO 1972).
    AdverseDrugReaction,
    /// AE outcome classification: death, life-threatening, etc. (ICH E2A).
    Seriousness,
    /// Comparative assessment of benefit against risk.
    BenefitRiskEvaluation,
    /// Sigmoidal relationship between exposure and effect magnitude.
    DoseResponseCurve,
    /// Proportion of disease attributable to exposure in population.
    PopulationAttributableFraction,
    /// 2×2 classification performance: TP, FP, FN, TN.
    ConfusionMatrix,
    /// Probability of harm occurring.
    Risk,
    /// Evaluation of causal relationship between exposure and outcome.
    CausalityAssessment,
    /// Ordered relationship between drug start and AE onset.
    TemporalRelationship,
    /// Ordered process with state transitions and boundary conditions.
    Workflow,
    /// Directed message to a specified recipient.
    Notification,
    /// Multi-source data merge into unified representation.
    Aggregation,
    /// Immutable, append-only log of actions.
    AuditTrail,
    /// Concealment of allocation or identity (blinding).
    Masking,
    /// Missing data field requiring inference or follow-up.
    DataGap,
}

impl PvComposite {
    /// All 20 composites.
    pub const ALL: &'static [Self] = &[
        Self::ContingencyTable,
        Self::Signal,
        Self::SignalLifecycle,
        Self::SignalStrength,
        Self::AdverseEvent,
        Self::AdverseDrugReaction,
        Self::Seriousness,
        Self::BenefitRiskEvaluation,
        Self::DoseResponseCurve,
        Self::PopulationAttributableFraction,
        Self::ConfusionMatrix,
        Self::Risk,
        Self::CausalityAssessment,
        Self::TemporalRelationship,
        Self::Workflow,
        Self::Notification,
        Self::Aggregation,
        Self::AuditTrail,
        Self::Masking,
        Self::DataGap,
    ];

    /// T2-P primitive dependencies for this composite.
    #[must_use]
    pub fn dependencies(&self) -> &'static [PvPrimitive] {
        match self {
            Self::ContingencyTable => &[],
            Self::Signal => &[
                PvPrimitive::Threshold,
                PvPrimitive::Association,
                PvPrimitive::Source,
            ],
            Self::SignalLifecycle => &[PvPrimitive::Timestamp],
            Self::SignalStrength => &[PvPrimitive::Threshold],
            Self::AdverseEvent => &[PvPrimitive::Timestamp, PvPrimitive::Association],
            Self::AdverseDrugReaction => &[PvPrimitive::Harm, PvPrimitive::Association],
            Self::Seriousness => &[PvPrimitive::Harm, PvPrimitive::Threshold],
            Self::BenefitRiskEvaluation => &[PvPrimitive::Ratio, PvPrimitive::Threshold],
            Self::DoseResponseCurve => &[PvPrimitive::Threshold],
            Self::PopulationAttributableFraction => {
                &[PvPrimitive::IncidenceRate, PvPrimitive::Prevalence]
            }
            Self::ConfusionMatrix => &[PvPrimitive::Sensitivity, PvPrimitive::Specificity],
            Self::Risk => &[PvPrimitive::Harm, PvPrimitive::ReportingRate],
            Self::CausalityAssessment => &[PvPrimitive::Association],
            Self::TemporalRelationship => &[PvPrimitive::Timestamp, PvPrimitive::Duration],
            Self::Workflow => &[PvPrimitive::Timestamp, PvPrimitive::Validation],
            Self::Notification => &[PvPrimitive::Obligation, PvPrimitive::Source],
            Self::Aggregation => &[PvPrimitive::Source],
            Self::AuditTrail => &[PvPrimitive::Timestamp],
            Self::Masking => &[],
            Self::DataGap => &[PvPrimitive::Validation],
        }
    }

    /// Lex Primitiva grounding (union of constituent symbols).
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::ContingencyTable => &[Quantity, Quantity, Quantity, Quantity],
            Self::Signal => &[
                Existence, Comparison, Quantity, Boundary, Location, Sequence, Causality,
            ],
            Self::SignalLifecycle => &[State, Sequence, Irreversibility],
            Self::SignalStrength => &[Sum, Comparison, Boundary],
            Self::AdverseEvent => &[State, Existence, Sequence, Causality, Location],
            Self::AdverseDrugReaction => &[State, Existence, Sequence, Causality, Boundary],
            Self::Seriousness => &[Irreversibility, Sum, Boundary],
            Self::BenefitRiskEvaluation => &[Comparison, Quantity, Boundary, Sum],
            Self::DoseResponseCurve => &[Recursion, Boundary],
            Self::PopulationAttributableFraction => &[Product, Frequency, Comparison],
            Self::ConfusionMatrix => &[Quantity, Comparison, Boundary],
            Self::Risk => &[Frequency, Irreversibility, Quantity],
            Self::CausalityAssessment => &[Causality, Comparison, Quantity, Sum],
            Self::TemporalRelationship => &[Sequence, Comparison],
            Self::Workflow => &[Sequence, State, Boundary],
            Self::Notification => &[Causality, Location, Sequence, Boundary],
            Self::Aggregation => &[Sum, Location, Mapping],
            Self::AuditTrail => &[Sequence, Persistence, Irreversibility],
            Self::Masking => &[Void, Location],
            Self::DataGap => &[Void, Existence, Comparison],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::ContingencyTable => {
                "2x2 matrix: a(exposed+event), b(exposed+no event), c(unexposed+event), d(unexposed+no event)"
            }
            Self::Signal => {
                "Statistical disproportionality between observed and expected AE frequency warranting investigation"
            }
            Self::SignalLifecycle => {
                "Directed state machine: New → UnderReview → Confirmed → Closed/Refuted"
            }
            Self::SignalStrength => "Ordinal magnitude: Weak, Moderate, Strong, Very Strong",
            Self::AdverseEvent => {
                "Any untoward medical occurrence, not necessarily causally related to treatment (ICH E2A)"
            }
            Self::AdverseDrugReaction => {
                "Noxious, unintended response to a drug at normal doses (WHO 1972)"
            }
            Self::Seriousness => {
                "Outcome-based: death, life-threatening, hospitalization, disability, congenital anomaly (ICH E2A)"
            }
            Self::BenefitRiskEvaluation => {
                "Comparative assessment of therapeutic benefit vs safety risk for a population"
            }
            Self::DoseResponseCurve => {
                "Sigmoidal function (Hill equation) relating exposure amount to effect magnitude"
            }
            Self::PopulationAttributableFraction => {
                "PAF = Pe(RR-1) / [Pe(RR-1)+1]: disease fraction attributable to exposure"
            }
            Self::ConfusionMatrix => {
                "2x2 classification: TP, FP, FN, TN for signal detection performance"
            }
            Self::Risk => {
                "Probability of harm: combines frequency of occurrence with severity of outcome"
            }
            Self::CausalityAssessment => {
                "Systematic evaluation of causal relationship (certain/probable/possible/unlikely)"
            }
            Self::TemporalRelationship => "Ordered relationship: drug start must precede AE onset",
            Self::Workflow => {
                "Ordered process with defined states, transitions, and completion criteria"
            }
            Self::Notification => {
                "Directed safety communication to regulatory authority or stakeholder"
            }
            Self::Aggregation => {
                "Multi-source data merge: spontaneous + clinical + literature → unified signal"
            }
            Self::AuditTrail => {
                "Immutable, timestamped, append-only log of all safety data modifications"
            }
            Self::Masking => {
                "Concealment of treatment allocation or identity in clinical assessment"
            }
            Self::DataGap => {
                "Missing data field requiring inference, imputation, or active follow-up"
            }
        }
    }

    /// Whether this composite involves recursive structure.
    #[must_use]
    pub const fn is_recursive(&self) -> bool {
        matches!(self, Self::DoseResponseCurve | Self::SignalLifecycle)
    }

    /// Dependency depth (how many T2-P levels down).
    #[must_use]
    pub const fn depth(&self) -> u8 {
        match self {
            Self::ContingencyTable | Self::Masking => 1,
            _ => 2,
        }
    }
}

impl fmt::Display for PvComposite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?} [{}]", self, self.grounding().tier())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_is_20() {
        assert_eq!(PvComposite::ALL.len(), 20);
    }

    #[test]
    fn signal_is_t3_by_symbols() {
        // Signal has 7 unique symbols — that's T3 by count
        let g = PvComposite::Signal.grounding();
        assert!(
            g.unique_count() >= 6,
            "Signal has {} symbols",
            g.unique_count()
        );
    }

    #[test]
    fn contingency_table_is_t1_by_symbols() {
        // Only uses Quantity (deduplicated)
        let g = PvComposite::ContingencyTable.grounding();
        assert_eq!(g.unique_count(), 1);
    }

    #[test]
    fn ae_adr_distinction() {
        // AE lacks explicit Boundary; ADR includes it (dose boundary)
        let ae = PvComposite::AdverseEvent.grounding();
        let adr = PvComposite::AdverseDrugReaction.grounding();
        assert!(
            adr.symbols().contains(&LexSymbol::Boundary),
            "ADR must include Boundary (dose constraint)"
        );
        assert!(
            !ae.symbols().contains(&LexSymbol::Boundary),
            "AE should NOT include Boundary (no dose constraint)"
        );
    }

    #[test]
    fn all_have_descriptions() {
        for c in PvComposite::ALL {
            assert!(!c.description().is_empty(), "{:?} has empty description", c);
        }
    }

    #[test]
    fn dose_response_is_recursive() {
        assert!(PvComposite::DoseResponseCurve.is_recursive());
    }

    #[test]
    fn workflow_not_recursive() {
        assert!(!PvComposite::Workflow.is_recursive());
    }
}
