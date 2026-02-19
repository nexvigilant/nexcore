//! WHO Pillar 4: Prevention — "the prevention of adverse effects."
//!
//! 15 T3 concepts covering risk management, safety communications,
//! regulatory actions, and scope terms (adverse effects, drug-related problems).

use crate::lex::{LexSymbol, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::fmt;

/// T3 prevention and risk management concepts — WHO Pillar 4.
///
/// Tier: T3 | Dominant: ∂ (Boundary) — prevention enforces thresholds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PreventionConcept {
    /// Structured plan for identified/potential risks and missing information.
    RiskManagementPlan,
    /// FDA-mandated risk evaluation and mitigation strategy.
    Rems,
    /// Routine (labeling) and additional (restricted access) measures.
    RiskMinimizationMeasures,
    /// Systematic safety monitoring throughout product lifecycle.
    PharmacovigilancePlan,
    /// Dear Healthcare Professional Letter: urgent safety communication.
    Dhpc,
    /// Modification of product information for new safety knowledge.
    LabelUpdate,
    /// Removal of product from market due to unacceptable risk.
    WithdrawalSuspension,
    /// Limiting drug access to specific prescribers/patients/settings.
    RestrictedDistribution,
}

/// T3 scope concepts — the "adverse effects or any other drug-related problem."
///
/// Tier: T3 | These define WHAT pharmacovigilance monitors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ScopeConcept {
    /// Unintended harmful consequence of intervention (generic).
    AdverseEffect,
    /// Broader: misuse, abuse, error, interaction, lack of efficacy, off-label.
    DrugRelatedProblem,
    /// Unintended failure in drug treatment process.
    MedicationError,
    /// Pharmacological modification by concurrent substances.
    DrugInteraction,
    /// Use outside approved indication.
    OffLabelUse,
    /// Intentional excessive use for non-therapeutic purpose.
    DrugAbuse,
    /// Failure to produce intended therapeutic effect.
    LackOfEfficacy,
}

impl PreventionConcept {
    /// All 8 prevention concepts.
    pub const ALL: &'static [Self] = &[
        Self::RiskManagementPlan,
        Self::Rems,
        Self::RiskMinimizationMeasures,
        Self::PharmacovigilancePlan,
        Self::Dhpc,
        Self::LabelUpdate,
        Self::WithdrawalSuspension,
        Self::RestrictedDistribution,
    ];

    /// Lex Primitiva grounding.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::RiskManagementPlan => &[Boundary, Sum, Persistence, Sequence, Mapping, Quantity],
            Self::Rems => &[Boundary, Sequence, Mapping, Persistence, Causality, Sum],
            Self::RiskMinimizationMeasures => &[Boundary, Mapping, Sum, Location, Persistence],
            Self::PharmacovigilancePlan => &[
                Sequence,
                Persistence,
                Existence,
                Location,
                Mapping,
                Boundary,
            ],
            Self::Dhpc => &[Causality, Location, Sequence, Boundary, Persistence],
            Self::LabelUpdate => &[Mapping, Persistence, State, Boundary, Sequence],
            Self::WithdrawalSuspension => {
                &[Irreversibility, Boundary, Void, Causality, Persistence]
            }
            Self::RestrictedDistribution => &[Boundary, Location, Sum, Persistence, Mapping],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::RiskManagementPlan => {
                "Structured plan: identified risks + potential risks + missing information + minimization measures"
            }
            Self::Rems => {
                "FDA-mandated: medication guide, communication plan, elements to assure safe use (ETASU)"
            }
            Self::RiskMinimizationMeasures => {
                "Routine (SmPC, PIL, packaging) and additional (controlled distribution, registries)"
            }
            Self::PharmacovigilancePlan => {
                "Systematic approach to monitoring safety throughout entire product lifecycle"
            }
            Self::Dhpc => "Urgent direct communication to prescribers about new safety risk",
            Self::LabelUpdate => {
                "Modification of SmPC/prescribing information to reflect new safety knowledge"
            }
            Self::WithdrawalSuspension => {
                "Removal or suspension of marketing authorization due to unacceptable B-R profile"
            }
            Self::RestrictedDistribution => {
                "Limiting drug access to certified prescribers, pharmacies, or treatment settings"
            }
        }
    }

    /// ICH/regulatory source reference.
    #[must_use]
    pub const fn source(&self) -> &'static str {
        match self {
            Self::RiskManagementPlan => "ICH E2E, GVP Module V",
            Self::Rems => "FDA, 21 USC 355-1",
            Self::RiskMinimizationMeasures => "GVP Module XVI",
            Self::PharmacovigilancePlan => "ICH E2E",
            Self::Dhpc => "GVP Module XV",
            Self::LabelUpdate => "ICH E2C(R2), GVP Module XVI",
            Self::WithdrawalSuspension => "Regulatory action (national authority)",
            Self::RestrictedDistribution => "REMS ETASU, GVP Module XVI",
        }
    }
}

impl ScopeConcept {
    /// All 7 scope concepts.
    pub const ALL: &'static [Self] = &[
        Self::AdverseEffect,
        Self::DrugRelatedProblem,
        Self::MedicationError,
        Self::DrugInteraction,
        Self::OffLabelUse,
        Self::DrugAbuse,
        Self::LackOfEfficacy,
    ];

    /// Lex Primitiva grounding.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::AdverseEffect => &[Causality, Irreversibility, State, Existence, Boundary],
            Self::DrugRelatedProblem => &[Sum, Causality, Existence, Boundary, Location, Mapping],
            Self::MedicationError => &[Void, Sequence, Causality, State, Boundary],
            Self::DrugInteraction => &[Causality, Product, Mapping, Boundary, State],
            Self::OffLabelUse => &[Boundary, Location, Existence, Mapping, Causality],
            Self::DrugAbuse => &[Causality, Boundary, State, Irreversibility, Quantity],
            Self::LackOfEfficacy => &[Void, Comparison, Existence, Causality, Boundary],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::AdverseEffect => "Unintended harmful consequence of pharmaceutical intervention",
            Self::DrugRelatedProblem => {
                "Any event or circumstance involving drug therapy that interferes with desired outcomes"
            }
            Self::MedicationError => {
                "Unintended failure in drug treatment process (prescribing, dispensing, administration)"
            }
            Self::DrugInteraction => {
                "Pharmacological modification of one drug's effect by another concurrent substance"
            }
            Self::OffLabelUse => {
                "Use of a drug outside the approved indication, population, dose, or route"
            }
            Self::DrugAbuse => {
                "Intentional, persistent, or sporadic excessive use inconsistent with medical practice"
            }
            Self::LackOfEfficacy => {
                "Failure of drug to produce intended therapeutic effect at approved dose"
            }
        }
    }

    /// ICH/regulatory source reference.
    #[must_use]
    pub const fn source(&self) -> &'static str {
        match self {
            Self::AdverseEffect => "WHO definition",
            Self::DrugRelatedProblem => "WHO definition, PCNE classification",
            Self::MedicationError => "ICH E2A note, NCC MERP",
            Self::DrugInteraction => "ICH E2A",
            Self::OffLabelUse => "Regulatory (off-label)",
            Self::DrugAbuse => "ICH E2A, WHO",
            Self::LackOfEfficacy => "ICH E2D, GVP Module VI",
        }
    }
}

impl fmt::Display for PreventionConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Prevention::{:?} [{}]", self, self.grounding().tier())
    }
}

impl fmt::Display for ScopeConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Scope::{:?} [{}]", self, self.grounding().tier())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prevention_count_is_8() {
        assert_eq!(PreventionConcept::ALL.len(), 8);
    }

    #[test]
    fn scope_count_is_7() {
        assert_eq!(ScopeConcept::ALL.len(), 7);
    }

    #[test]
    fn withdrawal_is_irreversible() {
        let g = PreventionConcept::WithdrawalSuspension.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Irreversibility),
            "Withdrawal is an irreversible regulatory action"
        );
    }

    #[test]
    fn medication_error_contains_void() {
        let g = ScopeConcept::MedicationError.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Void),
            "Medication error = absence of intended outcome (∅)"
        );
    }

    #[test]
    fn lack_of_efficacy_contains_void() {
        let g = ScopeConcept::LackOfEfficacy.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Void),
            "Lack of efficacy = void where benefit should be"
        );
    }

    #[test]
    fn all_prevention_have_sources() {
        for c in PreventionConcept::ALL {
            assert!(!c.source().is_empty(), "{:?} missing source", c);
        }
    }

    #[test]
    fn all_scope_have_sources() {
        for c in ScopeConcept::ALL {
            assert!(!c.source().is_empty(), "{:?} missing source", c);
        }
    }
}
