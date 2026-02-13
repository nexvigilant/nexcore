//! T3 advanced analytics, safety communications, and special populations.
//!
//! 24 domain-specific concepts for statistical methods, regulatory actions,
//! and population-specific safety considerations.

use crate::lex::{LexSymbol, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::fmt;

/// T3 advanced statistical and analytical methods.
///
/// Tier: T3 | Dominant: κ (Comparison) + N (Quantity) — analytics compare quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AnalyticsConcept {
    /// Observed/expected ratio significantly different from 1.
    Disproportionality,
    /// Empirical Bayes adjustment for sparse data.
    BayesianShrinkage,
    /// Distribution of latency between drug start and AE onset.
    TimeToOnsetAnalysis,
    /// Observed-to-expected ratio for background rate comparison.
    ObservedToExpectedRatio,
    /// 1/AR: patients to treat before one additional AE.
    NumberNeededToHarm,
    /// Risk in exposed / risk in unexposed.
    RelativeRisk,
    /// Cross-product ratio (a*d)/(b*c).
    OddsRatio,
    /// Validated groupings of PTs for structured signal detection.
    StandardizedMeddraQuery,
    /// Pre-specified AE warranting enhanced monitoring.
    Aesi,
    /// Serious events so rare they automatically signal.
    DesignatedMedicalEvent,
    /// Risk confirmed by adequate evidence.
    ImportantIdentifiedRisk,
    /// Risk suspected but not confirmed.
    ImportantPotentialRisk,
    /// Key safety data gaps in knowledge.
    MissingInformation,
}

/// T3 safety communication and regulatory action concepts.
///
/// Tier: T3 | Dominant: → (Causality) + ∂ (Boundary) — communications
/// enforce boundary decisions based on causal evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SafetyCommsConcept {
    /// Type II variation to modify marketing authorization.
    SafetyVariation,
    /// EU regulatory review triggered by safety concern.
    ReferralProcedure,
    /// PRAC evaluation of pharmacovigilance data.
    PracAssessment,
    /// End-to-end: detection → validation → prioritization → assessment → recommendation.
    SignalManagementProcess,
    /// Structured messaging to stakeholders about safety risks.
    RiskCommunication,
    /// Measuring whether risk minimization measures work.
    EffectivenessEvaluation,
}

/// T3 special population and context-specific concepts.
///
/// Tier: T3 | Dominant: λ (Location) — special populations define
/// subgroups located within specific clinical/demographic boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SpecialPopulationConcept {
    /// Drug exposure during gestation: teratogenicity risk.
    PregnancyExposure,
    /// Age-stratified AE assessment in children.
    PediatricSafety,
    /// Elderly-specific PK/PD and polypharmacy risk.
    GeriatricSafety,
    /// Spontaneous + active surveillance for immunization reactions.
    VaccinePharmacovigilance,
    /// Mandated post-marketing study for safety concerns.
    PostAuthorizationSafetyStudy,
    /// Drug-induced liver injury: Hy's Law, RUCAM.
    Dili,
    /// Drug effect on cardiac repolarization (QTc).
    QtProlongation,
    /// Highest FDA warning level on product labeling.
    BlackBoxWarning,
    /// ALT>3×ULN + bilirubin>2×ULN = 10-50% mortality risk.
    HysLaw,
    /// Roussel Uclaf Causality Assessment Method for hepatotoxicity.
    RucamScore,
}

impl AnalyticsConcept {
    /// All 13 analytics concepts.
    pub const ALL: &'static [Self] = &[
        Self::Disproportionality,
        Self::BayesianShrinkage,
        Self::TimeToOnsetAnalysis,
        Self::ObservedToExpectedRatio,
        Self::NumberNeededToHarm,
        Self::RelativeRisk,
        Self::OddsRatio,
        Self::StandardizedMeddraQuery,
        Self::Aesi,
        Self::DesignatedMedicalEvent,
        Self::ImportantIdentifiedRisk,
        Self::ImportantPotentialRisk,
        Self::MissingInformation,
    ];

    /// Lex Primitiva grounding.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::Disproportionality => &[Comparison, Quantity, Boundary, Frequency, Sum],
            Self::BayesianShrinkage => &[Recursion, Quantity, Boundary, Comparison, Mapping],
            Self::TimeToOnsetAnalysis => &[Sequence, Frequency, Quantity, Comparison, Boundary],
            Self::ObservedToExpectedRatio => &[Comparison, Quantity, Frequency, Boundary],
            Self::NumberNeededToHarm => &[Comparison, Quantity, Boundary, Irreversibility],
            Self::RelativeRisk => &[Comparison, Frequency, Quantity, Boundary],
            Self::OddsRatio => &[Comparison, Quantity, Product, Boundary],
            Self::StandardizedMeddraQuery => &[Sum, Mapping, Recursion, Existence, Boundary],
            Self::Aesi => &[Existence, Boundary, Persistence, Sequence, Comparison],
            Self::DesignatedMedicalEvent => &[Existence, Irreversibility, Boundary, Frequency],
            Self::ImportantIdentifiedRisk => {
                &[Existence, Causality, Quantity, Boundary, Persistence]
            }
            Self::ImportantPotentialRisk => &[Existence, Causality, Void, Boundary, Persistence],
            Self::MissingInformation => &[Void, Persistence, Boundary, Existence, Mapping],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Disproportionality => "Observed/expected ratio statistically exceeds 1.0",
            Self::BayesianShrinkage => {
                "Empirical Bayes adjustment: shrinks extreme estimates toward prior (MGPS/BCPNN)"
            }
            Self::TimeToOnsetAnalysis => {
                "Distribution of latency: Weibull or log-normal analysis of drug-to-AE interval"
            }
            Self::ObservedToExpectedRatio => {
                "O/E analysis comparing observed AE rate to background incidence rate"
            }
            Self::NumberNeededToHarm => {
                "NNH = 1/Attributable Risk: patients treated per one additional AE"
            }
            Self::RelativeRisk => {
                "RR = incidence_exposed / incidence_unexposed: relative measure of association"
            }
            Self::OddsRatio => "OR = (a×d)/(b×c): cross-product ratio from contingency table",
            Self::StandardizedMeddraQuery => {
                "SMQ: validated PT groupings for broad or narrow signal searches (MSSO)"
            }
            Self::Aesi => {
                "Adverse Event of Special Interest: pre-defined AE requiring enhanced monitoring protocols"
            }
            Self::DesignatedMedicalEvent => {
                "DME: events so rare and serious they automatically constitute a signal (EMA list)"
            }
            Self::ImportantIdentifiedRisk => {
                "Risk confirmed by adequate evidence in the safety specification (ICH E2E)"
            }
            Self::ImportantPotentialRisk => {
                "Risk suspected from non-clinical/clinical data but not yet confirmed (ICH E2E)"
            }
            Self::MissingInformation => {
                "Key gaps in safety knowledge requiring further investigation (ICH E2E)"
            }
        }
    }

    /// ICH/regulatory source reference.
    #[must_use]
    pub const fn source(&self) -> &'static str {
        match self {
            Self::Disproportionality => "CIOMS VIII",
            Self::BayesianShrinkage => "DuMouchel 1999, Bate 1998",
            Self::TimeToOnsetAnalysis => "ICH E2E, GVP Module IX",
            Self::ObservedToExpectedRatio => "GVP Module IX",
            Self::NumberNeededToHarm => "Epidemiology (Altman 1998)",
            Self::RelativeRisk => "Epidemiology",
            Self::OddsRatio => "Epidemiology",
            Self::StandardizedMeddraQuery => "MedDRA MSSO, CIOMS",
            Self::Aesi => "ICH E2F, Brighton Collaboration",
            Self::DesignatedMedicalEvent => "EMA Rev 4 (2020)",
            Self::ImportantIdentifiedRisk => "ICH E2E, GVP Module V",
            Self::ImportantPotentialRisk => "ICH E2E, GVP Module V",
            Self::MissingInformation => "ICH E2E, GVP Module V",
        }
    }
}

impl SafetyCommsConcept {
    /// All 6 safety communication concepts.
    pub const ALL: &'static [Self] = &[
        Self::SafetyVariation,
        Self::ReferralProcedure,
        Self::PracAssessment,
        Self::SignalManagementProcess,
        Self::RiskCommunication,
        Self::EffectivenessEvaluation,
    ];

    /// Lex Primitiva grounding.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::SafetyVariation => &[State, Boundary, Persistence, Mapping, Sequence],
            Self::ReferralProcedure => &[Sequence, Boundary, Sum, Location, Persistence],
            Self::PracAssessment => &[Comparison, Sum, Persistence, Causality, Boundary],
            Self::SignalManagementProcess => {
                &[Sequence, State, Existence, Comparison, Causality, Boundary]
            }
            Self::RiskCommunication => &[Mapping, Location, Boundary, Causality, Sequence],
            Self::EffectivenessEvaluation => &[Comparison, Quantity, Sequence, Boundary, Mapping],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::SafetyVariation => {
                "Type II variation to modify SmPC/PIL based on new safety data"
            }
            Self::ReferralProcedure => {
                "EU-wide regulatory review triggered by safety concern (Art 31/20)"
            }
            Self::PracAssessment => {
                "PRAC evaluation: signal assessment, PSUR review, risk management"
            }
            Self::SignalManagementProcess => {
                "End-to-end: detection → validation → prioritization → assessment → recommendation → communication"
            }
            Self::RiskCommunication => {
                "Structured messaging to HCPs, patients, regulators about safety risks"
            }
            Self::EffectivenessEvaluation => {
                "Measuring impact of risk minimization on prescribing behavior and patient outcomes"
            }
        }
    }
}

impl SpecialPopulationConcept {
    /// All 10 special population concepts.
    pub const ALL: &'static [Self] = &[
        Self::PregnancyExposure,
        Self::PediatricSafety,
        Self::GeriatricSafety,
        Self::VaccinePharmacovigilance,
        Self::PostAuthorizationSafetyStudy,
        Self::Dili,
        Self::QtProlongation,
        Self::BlackBoxWarning,
        Self::HysLaw,
        Self::RucamScore,
    ];

    /// Lex Primitiva grounding.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::PregnancyExposure => &[Location, Causality, Irreversibility, Boundary, Sequence],
            Self::PediatricSafety => &[Location, Comparison, Quantity, Frequency, Boundary],
            Self::GeriatricSafety => &[Location, Product, Causality, Frequency, Boundary],
            Self::VaccinePharmacovigilance => {
                &[Location, Sequence, Frequency, Existence, Sum, Boundary]
            }
            Self::PostAuthorizationSafetyStudy => &[
                Sequence,
                Existence,
                Persistence,
                Boundary,
                Quantity,
                Causality,
            ],
            Self::Dili => &[
                Location,
                Causality,
                Quantity,
                Boundary,
                Irreversibility,
                Comparison,
            ],
            Self::QtProlongation => &[Location, Quantity, Boundary, Causality, Irreversibility],
            Self::BlackBoxWarning => &[Irreversibility, Boundary, Persistence, Causality, Location],
            Self::HysLaw => &[Quantity, Boundary, Irreversibility, Location, Comparison],
            Self::RucamScore => &[Causality, Quantity, Sum, Sequence, Boundary, Location],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::PregnancyExposure => {
                "Drug exposure during gestation: teratogenicity, fetal harm, pregnancy registries"
            }
            Self::PediatricSafety => {
                "Age-stratified safety: neonates, infants, children, adolescents (ICH E11)"
            }
            Self::GeriatricSafety => {
                "Elderly-specific: altered PK/PD, polypharmacy interactions, Beers criteria"
            }
            Self::VaccinePharmacovigilance => {
                "Immunization reactions: spontaneous + active surveillance, Brighton criteria"
            }
            Self::PostAuthorizationSafetyStudy => {
                "PASS: mandated post-marketing study to address specific safety concern"
            }
            Self::Dili => "Drug-Induced Liver Injury: ALT/AST monitoring, Hy's Law, RUCAM scoring",
            Self::QtProlongation => {
                "Cardiac repolarization: QTc > 500ms is critical threshold (ICH E14)"
            }
            Self::BlackBoxWarning => {
                "Highest FDA warning level: enclosed in black border on prescribing information"
            }
            Self::HysLaw => {
                "ALT > 3×ULN + bilirubin > 2×ULN predicts 10-50% drug-related mortality"
            }
            Self::RucamScore => {
                "RUCAM: 7-domain causality scoring (0-14) specific to hepatotoxicity"
            }
        }
    }

    /// ICH/regulatory source reference.
    #[must_use]
    pub const fn source(&self) -> &'static str {
        match self {
            Self::PregnancyExposure => "ICH E2E, FDA pregnancy category",
            Self::PediatricSafety => "ICH E11, GVP",
            Self::GeriatricSafety => "GVP, Beers criteria (AGS)",
            Self::VaccinePharmacovigilance => "CIOMS/WHO, Brighton Collaboration",
            Self::PostAuthorizationSafetyStudy => "GVP Module VIII",
            Self::Dili => "FDA DILI guidance, Hy's Law (Temple)",
            Self::QtProlongation => "ICH E14",
            Self::BlackBoxWarning => "FDA, 21 CFR 201.57(c)",
            Self::HysLaw => "Temple 2006, FDA guidance",
            Self::RucamScore => "Danan & Benichou 1993",
        }
    }
}

impl fmt::Display for AnalyticsConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Analytics::{:?} [{}]", self, self.grounding().tier())
    }
}

impl fmt::Display for SafetyCommsConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SafetyComms::{:?} [{}]", self, self.grounding().tier())
    }
}

impl fmt::Display for SpecialPopulationConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SpecialPopulation::{:?} [{}]",
            self,
            self.grounding().tier()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analytics_count_is_13() {
        assert_eq!(AnalyticsConcept::ALL.len(), 13);
    }

    #[test]
    fn safety_comms_count_is_6() {
        assert_eq!(SafetyCommsConcept::ALL.len(), 6);
    }

    #[test]
    fn special_populations_count_is_10() {
        assert_eq!(SpecialPopulationConcept::ALL.len(), 10);
    }

    #[test]
    fn missing_information_contains_void() {
        let g = AnalyticsConcept::MissingInformation.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Void),
            "Missing information = ∅ (absence of knowledge)"
        );
    }

    #[test]
    fn potential_risk_contains_void() {
        let g = AnalyticsConcept::ImportantPotentialRisk.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Void),
            "Potential risk = suspected but unconfirmed (∅ confirmation)"
        );
    }

    #[test]
    fn dili_has_quantity_and_boundary() {
        let g = SpecialPopulationConcept::Dili.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Quantity),
            "DILI uses ALT/AST numeric values"
        );
        assert!(
            g.symbols().contains(&LexSymbol::Boundary),
            "DILI uses 3×ULN threshold"
        );
    }

    #[test]
    fn hys_law_has_irreversibility() {
        let g = SpecialPopulationConcept::HysLaw.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Irreversibility),
            "Hy's Law predicts mortality (irreversible)"
        );
    }

    #[test]
    fn all_analytics_have_descriptions() {
        for c in AnalyticsConcept::ALL {
            assert!(!c.description().is_empty(), "{:?} missing description", c);
        }
    }
}
