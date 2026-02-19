//! WHO Pillar 3: Understanding — "the understanding of adverse effects."
//!
//! 10 T3 domain-specific concepts for deepening knowledge of safety signals
//! through periodic reporting, validation, and causal reasoning.

use crate::lex::{LexSymbol, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::fmt;

/// T3 understanding concepts — WHO Pillar 3.
///
/// Tier: T3 | Dominant: μ (Mapping) — understanding builds causal models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnderstandingConcept {
    /// Periodic Safety Update Report / Periodic Benefit-Risk Evaluation Report.
    PsurPbrer,
    /// Development Safety Update Report (annual during clinical development).
    Dsur,
    /// Confirmation that detected signal is genuine, not artifact.
    SignalValidation,
    /// Ranking validated signals by clinical impact and public health significance.
    SignalPrioritization,
    /// Mechanism of action consistent with known pharmacology.
    BiologicalPlausibility,
    /// Effect magnitude correlates with exposure amount.
    DoseResponseRelationship,
    /// AE onset consistent with drug pharmacokinetics.
    TemporalPlausibility,
    /// AE occurs primarily with this drug, not others.
    SpecificityOfAssociation,
    /// Same association observed across independent studies.
    ConsistencyAcrossStudies,
    /// Association consistent with natural history and biology of disease.
    Coherence,
}

impl UnderstandingConcept {
    /// All 10 understanding concepts.
    pub const ALL: &'static [Self] = &[
        Self::PsurPbrer,
        Self::Dsur,
        Self::SignalValidation,
        Self::SignalPrioritization,
        Self::BiologicalPlausibility,
        Self::DoseResponseRelationship,
        Self::TemporalPlausibility,
        Self::SpecificityOfAssociation,
        Self::ConsistencyAcrossStudies,
        Self::Coherence,
    ];

    /// Lex Primitiva grounding.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::PsurPbrer => &[Sequence, Sum, Comparison, Persistence, Boundary, Quantity],
            Self::Dsur => &[Sequence, State, Quantity, Persistence, Boundary],
            Self::SignalValidation => &[Existence, Comparison, Void, Boundary, Quantity],
            Self::SignalPrioritization => &[Comparison, Quantity, Boundary, Irreversibility, Sum],
            Self::BiologicalPlausibility => &[Causality, Mapping, Recursion, Existence],
            Self::DoseResponseRelationship => {
                &[Quantity, Causality, Boundary, Recursion, Comparison]
            }
            Self::TemporalPlausibility => &[Sequence, Comparison, Boundary, Quantity, Causality],
            Self::SpecificityOfAssociation => &[Comparison, Existence, Void, Boundary, Causality],
            Self::ConsistencyAcrossStudies => &[Comparison, Quantity, Location, Persistence],
            Self::Coherence => &[Mapping, Causality, Recursion, Existence],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::PsurPbrer => {
                "Periodic aggregate safety report with benefit-risk assessment (ICH E2C(R2))"
            }
            Self::Dsur => "Annual safety summary during clinical development phase (ICH E2F)",
            Self::SignalValidation => {
                "Confirming detected signal is genuine, excluding artifacts and confounders"
            }
            Self::SignalPrioritization => {
                "Ranking signals by clinical severity, population impact, and novelty"
            }
            Self::BiologicalPlausibility => {
                "Known mechanism of action supports causal link (Bradford Hill criterion 6)"
            }
            Self::DoseResponseRelationship => {
                "Higher dose → greater effect; Hill equation gradient (Bradford Hill criterion 5)"
            }
            Self::TemporalPlausibility => {
                "AE onset consistent with drug's Tmax and elimination half-life (Bradford Hill criterion 4)"
            }
            Self::SpecificityOfAssociation => {
                "AE occurs primarily with this drug, ruling out alternative causes (Bradford Hill criterion 3)"
            }
            Self::ConsistencyAcrossStudies => {
                "Same association observed across multiple independent populations and study designs (Bradford Hill criterion 2)"
            }
            Self::Coherence => {
                "Association consistent with known natural history, pathophysiology, and lab evidence (Bradford Hill criterion 7)"
            }
        }
    }

    /// Whether this concept is one of the Bradford Hill criteria.
    #[must_use]
    pub const fn is_bradford_hill(&self) -> bool {
        matches!(
            self,
            Self::BiologicalPlausibility
                | Self::DoseResponseRelationship
                | Self::TemporalPlausibility
                | Self::SpecificityOfAssociation
                | Self::ConsistencyAcrossStudies
                | Self::Coherence
        )
    }

    /// ICH/regulatory source reference.
    #[must_use]
    pub const fn source(&self) -> &'static str {
        match self {
            Self::PsurPbrer => "ICH E2C(R2)",
            Self::Dsur => "ICH E2F",
            Self::SignalValidation => "CIOMS VIII, GVP Module IX",
            Self::SignalPrioritization => "GVP Module IX",
            Self::BiologicalPlausibility => "Hill 1965, criterion 6",
            Self::DoseResponseRelationship => "Hill 1965, criterion 5",
            Self::TemporalPlausibility => "Hill 1965, criterion 4",
            Self::SpecificityOfAssociation => "Hill 1965, criterion 3",
            Self::ConsistencyAcrossStudies => "Hill 1965, criterion 2",
            Self::Coherence => "Hill 1965, criterion 7",
        }
    }
}

impl fmt::Display for UnderstandingConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Understanding::{:?} [{}]", self, self.grounding().tier())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_is_10() {
        assert_eq!(UnderstandingConcept::ALL.len(), 10);
    }

    #[test]
    fn six_bradford_hill_criteria() {
        let bh_count = UnderstandingConcept::ALL
            .iter()
            .filter(|c| c.is_bradford_hill())
            .count();
        assert_eq!(
            bh_count, 6,
            "Should have 6 Bradford Hill criteria represented"
        );
    }

    #[test]
    fn psur_has_persistence() {
        let g = UnderstandingConcept::PsurPbrer.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Persistence),
            "PSUR/PBRER must persist as regulatory record"
        );
    }

    #[test]
    fn signal_validation_excludes_artifacts() {
        let g = UnderstandingConcept::SignalValidation.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Void),
            "Signal validation must check for ∅ (artifact exclusion)"
        );
    }

    #[test]
    fn all_have_sources() {
        for c in UnderstandingConcept::ALL {
            assert!(!c.source().is_empty(), "{:?} missing source", c);
        }
    }
}
