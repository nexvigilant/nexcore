//! T3 regulatory framework, infrastructure, and operational concepts.
//!
//! 29 domain-specific concepts covering ICH guidelines, reporting systems,
//! databases, surveillance infrastructure, and case processing operations.

use crate::lex::{LexSymbol, PrimitiveComposition};
use serde::{Deserialize, Serialize};
use std::fmt;

/// T3 regulatory framework concepts — ICH guidelines and regulations.
///
/// Tier: T3 | Dominant: σ (Sequence) + π (Persistence) — regulations define
/// ordered processes that persist as enforceable standards.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegulatoryConcept {
    /// ICH E2A: foundational definitions (AE, ADR, SAE, unexpected).
    IchE2a,
    /// ICH E2B(R3): electronic ICSR transmission standard.
    IchE2bR3,
    /// ICH E2C(R2): PBRER format and content.
    IchE2cR2,
    /// ICH E2D(R1): post-approval safety data management.
    IchE2dR1,
    /// ICH E2E: pharmacovigilance planning.
    IchE2e,
    /// ICH E2F: development safety update report.
    IchE2f,
    /// EMA Good Pharmacovigilance Practice (16 modules).
    GvpModules,
    /// 21 CFR 314.80: FDA post-marketing safety reporting.
    Cfr31480,
    /// 15-day expedited reporting for serious unexpected ADRs.
    ExpeditedReporting,
    /// E2B(R3) HL7/ICH M2 XML transmission standard.
    E2bTransmission,
}

/// T3 infrastructure concepts — reporting systems and databases.
///
/// Tier: T3 | Dominant: λ (Location) + π (Persistence) — infrastructure
/// stores data at specific geographic/organizational locations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InfrastructureConcept {
    /// Passive surveillance from unsolicited reports.
    SpontaneousReportingSystem,
    /// FDA Adverse Event Reporting System.
    Faers,
    /// EU adverse reaction database.
    EudraVigilance,
    /// WHO-UMC global ICSR database (30M+ reports).
    VigiBase,
    /// Prospective organized data collection.
    ActiveSurveillance,
    /// FDA active surveillance using electronic health data.
    SentinelSystem,
    /// Signal detection from electronic health records.
    EhrMining,
    /// Organized system for uniform outcome data collection.
    PatientRegistry,
    /// NLP/mining of social media for AE mentions.
    SocialMediaMonitoring,
    /// CIOMS standardized international AE reporting form.
    CiomsForm,
}

/// T3 operational process concepts — case handling and quality.
///
/// Tier: T3 | Dominant: σ (Sequence) + μ (Mapping) — operations are
/// ordered transformations of data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum OperationsConcept {
    /// Intake → triage → coding → assessment → submission.
    CaseProcessing,
    /// Initial assessment for completeness and seriousness.
    CaseTriage,
    /// Top-level MedDRA classification (27 SOCs).
    MeddraSoc,
    /// Standard medical concept level in MedDRA hierarchy.
    MeddraPt,
    /// Minimum 4 elements: patient, reporter, drug, AE.
    CaseCompleteness,
    /// Requesting additional information on initial report.
    FollowUp,
    /// Identifying same case from multiple channels.
    DuplicateDetection,
    /// Line listings and statistical analysis for periodic reports.
    AggregateReporting,
    /// Qualified Person for PV: oversight and quality management.
    Qppv,
}

impl RegulatoryConcept {
    /// All 10 regulatory concepts.
    pub const ALL: &'static [Self] = &[
        Self::IchE2a,
        Self::IchE2bR3,
        Self::IchE2cR2,
        Self::IchE2dR1,
        Self::IchE2e,
        Self::IchE2f,
        Self::GvpModules,
        Self::Cfr31480,
        Self::ExpeditedReporting,
        Self::E2bTransmission,
    ];

    /// Lex Primitiva grounding.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::IchE2a => &[Mapping, Boundary, Sum, Persistence, Existence],
            Self::IchE2bR3 => &[Mapping, Sequence, Persistence, Location, Boundary],
            Self::IchE2cR2 => &[Sequence, Comparison, Persistence, Boundary, Sum],
            Self::IchE2dR1 => &[Persistence, Sequence, Boundary, Location, Causality],
            Self::IchE2e => &[
                Sequence,
                Existence,
                Mapping,
                Recursion,
                Boundary,
                Persistence,
            ],
            Self::IchE2f => &[Sequence, Quantity, State, Persistence, Boundary],
            Self::GvpModules => &[Sequence, Mapping, Boundary, Sum, Persistence, Location],
            Self::Cfr31480 => &[Boundary, Sequence, Persistence, Causality, Location],
            Self::ExpeditedReporting => {
                &[Boundary, Sequence, Irreversibility, Causality, Persistence]
            }
            Self::E2bTransmission => &[Mapping, Sequence, Persistence, Location, Recursion],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::IchE2a => {
                "Foundational safety definitions: AE, ADR, SAE, unexpected, severity vs seriousness"
            }
            Self::IchE2bR3 => {
                "Electronic ICSR transmission standard: data elements, XML schema, HL7/ICH M2"
            }
            Self::IchE2cR2 => "PBRER: periodic benefit-risk evaluation, format, content, frequency",
            Self::IchE2dR1 => {
                "Post-approval safety: reporting requirements, signal detection obligation"
            }
            Self::IchE2e => {
                "PV planning: safety specification, PV plan, signal detection methodology"
            }
            Self::IchE2f => "DSUR: annual development safety update during clinical trials",
            Self::GvpModules => {
                "16-module EMA system: quality (I), PSUR (VII), signal management (IX), risk minimization (XVI)"
            }
            Self::Cfr31480 => {
                "FDA post-marketing: 15-day alert reports, periodic reports, field alert reports"
            }
            Self::ExpeditedReporting => {
                "15-day submission for serious unexpected ADRs (fatal/life-threatening: immediate)"
            }
            Self::E2bTransmission => {
                "HL7/ICH M2 electronic ICSR: 450+ data elements, hierarchical XML, acknowledgment"
            }
        }
    }

    /// ICH/regulatory source reference.
    #[must_use]
    pub const fn source(&self) -> &'static str {
        match self {
            Self::IchE2a => "ICH E2A (1994)",
            Self::IchE2bR3 => "ICH E2B(R3) (2014)",
            Self::IchE2cR2 => "ICH E2C(R2) (2012)",
            Self::IchE2dR1 => "ICH E2D(R1) (2003)",
            Self::IchE2e => "ICH E2E (2004)",
            Self::IchE2f => "ICH E2F (2010)",
            Self::GvpModules => "EMA GVP (2012-present)",
            Self::Cfr31480 => "21 CFR 314.80, FDA",
            Self::ExpeditedReporting => "ICH E2A, 21 CFR 314.80",
            Self::E2bTransmission => "ICH E2B(R3), HL7/ICH M2",
        }
    }
}

impl InfrastructureConcept {
    /// All 10 infrastructure concepts.
    pub const ALL: &'static [Self] = &[
        Self::SpontaneousReportingSystem,
        Self::Faers,
        Self::EudraVigilance,
        Self::VigiBase,
        Self::ActiveSurveillance,
        Self::SentinelSystem,
        Self::EhrMining,
        Self::PatientRegistry,
        Self::SocialMediaMonitoring,
        Self::CiomsForm,
    ];

    /// Lex Primitiva grounding.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::SpontaneousReportingSystem => {
                &[Location, Persistence, Sequence, Existence, Causality]
            }
            Self::Faers => &[
                Location,
                Persistence,
                Existence,
                Sequence,
                Quantity,
                Mapping,
            ],
            Self::EudraVigilance => &[
                Location,
                Persistence,
                Existence,
                Sequence,
                Mapping,
                Boundary,
            ],
            Self::VigiBase => &[
                Location,
                Persistence,
                Quantity,
                Sequence,
                Mapping,
                Existence,
            ],
            Self::ActiveSurveillance => &[Sequence, Existence, Mapping, Recursion, Location],
            Self::SentinelSystem => &[
                Location,
                Mapping,
                Recursion,
                Sequence,
                Quantity,
                Persistence,
            ],
            Self::EhrMining => &[
                Mapping,
                Recursion,
                Existence,
                Location,
                Persistence,
                Quantity,
            ],
            Self::PatientRegistry => {
                &[Persistence, Sequence, Location, Quantity, Mapping, Boundary]
            }
            Self::SocialMediaMonitoring => &[Location, Mapping, Existence, Frequency, Recursion],
            Self::CiomsForm => &[Mapping, Persistence, Sequence, Boundary, Location],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::SpontaneousReportingSystem => {
                "Passive surveillance: unsolicited reports from HCPs, patients, and MAHs"
            }
            Self::Faers => "FDA Adverse Event Reporting System: US spontaneous reporting database",
            Self::EudraVigilance => "European adverse reaction database (EMA)",
            Self::VigiBase => "WHO-UMC global ICSR database: 30M+ reports from 140+ countries",
            Self::ActiveSurveillance => {
                "Prospective, organized, systematic data collection for safety"
            }
            Self::SentinelSystem => {
                "FDA Sentinel: active surveillance using claims/EHR data (300M+ lives)"
            }
            Self::EhrMining => {
                "Signal detection from electronic health records via NLP and analytics"
            }
            Self::PatientRegistry => {
                "Organized system collecting uniform data on patient outcomes over time"
            }
            Self::SocialMediaMonitoring => {
                "NLP mining of social media, forums, and patient communities for AE mentions"
            }
            Self::CiomsForm => "CIOMS I standardized international adverse event reporting form",
        }
    }
}

impl OperationsConcept {
    /// All 9 operations concepts.
    pub const ALL: &'static [Self] = &[
        Self::CaseProcessing,
        Self::CaseTriage,
        Self::MeddraSoc,
        Self::MeddraPt,
        Self::CaseCompleteness,
        Self::FollowUp,
        Self::DuplicateDetection,
        Self::AggregateReporting,
        Self::Qppv,
    ];

    /// Lex Primitiva grounding.
    #[must_use]
    pub fn grounding(&self) -> PrimitiveComposition {
        use LexSymbol::*;
        PrimitiveComposition::new(match self {
            Self::CaseProcessing => &[Sequence, Mapping, Causality, State, Boundary, Persistence],
            Self::CaseTriage => &[Comparison, Sum, Boundary, Existence, Sequence],
            Self::MeddraSoc => &[Sum, Location, Mapping, Boundary, Recursion],
            Self::MeddraPt => &[Mapping, Sum, Existence, Boundary, Persistence],
            Self::CaseCompleteness => &[Existence, Existence, Existence, Existence, Boundary],
            Self::FollowUp => &[Sequence, Recursion, Persistence, Void, Location],
            Self::DuplicateDetection => &[Comparison, Location, Mapping, Existence, Quantity],
            Self::AggregateReporting => &[Sum, Quantity, Sequence, Persistence, Boundary, Mapping],
            Self::Qppv => &[Boundary, Persistence, Sequence, Mapping, Location],
        })
    }

    /// Human-readable description.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::CaseProcessing => {
                "End-to-end pipeline: intake → triage → coding → assessment → submission"
            }
            Self::CaseTriage => "Initial assessment: valid case? Serious? Expedited? Complete?",
            Self::MeddraSoc => "System Organ Class: 27 top-level body-system groupings in MedDRA",
            Self::MeddraPt => "Preferred Term: standard medical concept for adverse event coding",
            Self::CaseCompleteness => {
                "Minimum valid case: identifiable patient + reporter + suspect drug + AE"
            }
            Self::FollowUp => "Requesting additional information when initial report has data gaps",
            Self::DuplicateDetection => {
                "Identifying same case reported through multiple channels (HCP + patient + literature)"
            }
            Self::AggregateReporting => {
                "Combined line listings with statistical analysis for PSUR/PBRER/DSUR"
            }
            Self::Qppv => {
                "Qualified Person for PV: single point of accountability for PV system quality"
            }
        }
    }
}

impl fmt::Display for RegulatoryConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Regulatory::{:?} [{}]", self, self.grounding().tier())
    }
}

impl fmt::Display for InfrastructureConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Infrastructure::{:?} [{}]",
            self,
            self.grounding().tier()
        )
    }
}

impl fmt::Display for OperationsConcept {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Operations::{:?} [{}]", self, self.grounding().tier())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regulatory_count_is_10() {
        assert_eq!(RegulatoryConcept::ALL.len(), 10);
    }

    #[test]
    fn infrastructure_count_is_10() {
        assert_eq!(InfrastructureConcept::ALL.len(), 10);
    }

    #[test]
    fn operations_count_is_9() {
        assert_eq!(OperationsConcept::ALL.len(), 9);
    }

    #[test]
    fn all_regulatory_have_persistence() {
        for c in RegulatoryConcept::ALL {
            let g = c.grounding();
            assert!(
                g.symbols().contains(&LexSymbol::Persistence),
                "{:?} should persist as regulatory standard",
                c
            );
        }
    }

    #[test]
    fn all_databases_have_location() {
        let dbs = [
            InfrastructureConcept::Faers,
            InfrastructureConcept::EudraVigilance,
            InfrastructureConcept::VigiBase,
        ];
        for db in &dbs {
            let g = db.grounding();
            assert!(
                g.symbols().contains(&LexSymbol::Location),
                "{:?} must have location (geographic scope)",
                db
            );
        }
    }

    #[test]
    fn case_completeness_needs_existence() {
        let g = OperationsConcept::CaseCompleteness.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Existence),
            "Case completeness checks existence of 4 minimum elements"
        );
    }

    #[test]
    fn follow_up_needs_void() {
        let g = OperationsConcept::FollowUp.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Void),
            "Follow-up addresses missing data (∅)"
        );
    }

    #[test]
    fn expedited_has_irreversibility() {
        let g = RegulatoryConcept::ExpeditedReporting.grounding();
        assert!(
            g.symbols().contains(&LexSymbol::Irreversibility),
            "Expedited reporting triggered by serious (often irreversible) events"
        );
    }
}
