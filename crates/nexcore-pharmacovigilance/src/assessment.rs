//! WHO Pillar 2: Assessment — "the assessment of adverse effects."
//!
//! 10 T3 domain-specific concepts for evaluating individual cases,
//! determining causality, and coding medical terminology.

use crate::lex::{LexSymbol, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::fmt;

/// T3 assessment concepts — WHO Pillar 2.
///
/// Tier: T3 | Dominant: → (Causality) — assessment asks "did the drug cause this?"
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssessmentConcept {
    /// Naranjo algorithm: 10 weighted causality questions, score 0-13.
    NaranjoAlgorithm,
    /// WHO-UMC system: Certain/Probable/Possible/Unlikely/Conditional/Unassessable.
    WhoUmcSystem,
    /// Bradford Hill criteria: 9 viewpoints for causal inference.
    BradfordHillCriteria,
    /// Individual Case Safety Report: structured AE documentation.
    Icsr,
    /// Serious Adverse Event: AE meeting seriousness criteria.
    SeriousAdverseEvent,
    /// Suspected Unexpected Serious ADR.
    Susar,
    /// PV-specific causality: "reasonable possibility" non-exclusion standard.
    ReasonablePossibility,
    /// MedDRA hierarchical coding: SOC > HLGT > HLT > PT > LLT.
    MeddraCoding,
    /// Free-text clinical description of AE circumstances.
    CaseNarrative,
    /// Dechallenge (drug stop → AE resolve?) / Rechallenge (restart → recur?).
    RechallengeDechallenge,
}

impl AssessmentConcept {
    /// All 10 assessment concepts.
    pub const ALL: &'static [Self] = &[
        Self::NaranjoAlgorithm,
        Self::WhoUmcSystem,
        Self::BradfordHillCriteria,
        Self::Icsr,
        Self::SeriousAdverseEvent,
        Self::Susar,
        Self::ReasonablePossibility,
        Self::MeddraCoding,
        Self::CaseNarrative,
        Self::RechallengeDechallenge,
    ];

    /// Lex Primitiva grounding.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::NaranjoAlgorithm => &[Causality, Quantity, Sum, Comparison, Sequence, Boundary],
            Self::WhoUmcSystem => &[Causality, Sum, Comparison, Boundary, Existence],
            Self::BradfordHillCriteria => &[
                Causality, Comparison, Sequence, Frequency, Recursion, Mapping,
            ],
            Self::Icsr => &[Persistence, Sequence, Location, Causality, State, Existence],
            Self::SeriousAdverseEvent => {
                &[State, Existence, Causality, Irreversibility, Sum, Boundary]
            }
            Self::Susar => &[
                Causality,
                Irreversibility,
                Sum,
                Boundary,
                Comparison,
                Existence,
            ],
            Self::ReasonablePossibility => &[Causality, Void, Boundary, Existence, Comparison],
            Self::MeddraCoding => &[Sum, Mapping, Location, Recursion, Existence],
            Self::CaseNarrative => &[Sequence, Mapping, Persistence, Causality],
            Self::RechallengeDechallenge => &[Causality, Sequence, State, Comparison, Boundary],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::NaranjoAlgorithm => {
                "10-question weighted algorithm: score 0-13 maps to Definite/Probable/Possible/Doubtful"
            }
            Self::WhoUmcSystem => {
                "6-category causality: Certain, Probable/Likely, Possible, Unlikely, Conditional/Unclassified, Unassessable"
            }
            Self::BradfordHillCriteria => {
                "9 viewpoints: strength, consistency, specificity, temporality, biological gradient, plausibility, coherence, experiment, analogy"
            }
            Self::Icsr => {
                "Structured report: identifiable patient + identifiable reporter + suspect drug + adverse event"
            }
            Self::SeriousAdverseEvent => {
                "AE resulting in death, life-threatening, hospitalization, disability, or congenital anomaly"
            }
            Self::Susar => {
                "ADR that is serious AND unexpected (not in reference safety information) AND suspected causal"
            }
            Self::ReasonablePossibility => {
                "FDA non-exclusion standard: report unless drug causation can be ruled out (∅+→+∂)"
            }
            Self::MeddraCoding => {
                "5-level hierarchy: System Organ Class > HLGT > HLT > Preferred Term > Lowest Level Term"
            }
            Self::CaseNarrative => {
                "Free-text clinical description: patient history, drug details, event course, outcome"
            }
            Self::RechallengeDechallenge => {
                "Temporal causality test: stop drug (dechallenge) → AE resolves? Restart (rechallenge) → recurs?"
            }
        }
    }

    /// ICH/regulatory source reference.
    #[must_use]
    pub const fn source(&self) -> &'static str {
        match self {
            Self::NaranjoAlgorithm => "Naranjo et al. 1981, Clinical Pharmacology & Therapeutics",
            Self::WhoUmcSystem => "WHO-UMC, Uppsala Monitoring Centre",
            Self::BradfordHillCriteria => "Hill 1965, Proceedings of the Royal Society of Medicine",
            Self::Icsr => "ICH E2B(R3), HL7/ICH M2",
            Self::SeriousAdverseEvent => "ICH E2A",
            Self::Susar => "ICH E2A, EU Directive 2001/20/EC",
            Self::ReasonablePossibility => "21 CFR 314.80, FDA",
            Self::MeddraCoding => "MedDRA (MSSO), ICH E2B(R3)",
            Self::CaseNarrative => "ICH E2B(R3), GVP Module VI",
            Self::RechallengeDechallenge => "ICH E2B(R3), Naranjo Q6-Q7",
        }
    }
}

impl fmt::Display for AssessmentConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Assessment::{:?} [{}]", self, self.grounding().tier())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_is_10() {
        assert_eq!(AssessmentConcept::ALL.len(), 10);
    }

    #[test]
    fn bradford_hill_has_6_symbols() {
        let g = AssessmentConcept::BradfordHillCriteria.grounding();
        assert_eq!(
            g.unique_count(),
            6,
            "Bradford Hill should be T3 (6+ symbols)"
        );
    }

    #[test]
    fn reasonable_possibility_contains_void() {
        let g = AssessmentConcept::ReasonablePossibility.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Void),
            "Reasonable possibility grounds to ∅ (cannot exclude)"
        );
    }

    #[test]
    fn all_contain_causality_or_mapping() {
        // Assessment is fundamentally about causality or classification
        for c in AssessmentConcept::ALL {
            let g = c.grounding();
            let has_causality = g.symbols().contains(&LexSymbol::Causality);
            let has_mapping = g.symbols().contains(&LexSymbol::Mapping);
            assert!(has_causality || has_mapping, "{:?} must contain → or μ", c);
        }
    }

    #[test]
    fn icsr_has_persistence() {
        let g = AssessmentConcept::Icsr.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Persistence),
            "ICSR must persist (regulatory record)"
        );
    }
}
