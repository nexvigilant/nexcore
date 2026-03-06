//! Canonical EPA→Domain and CPA→EPA mappings.
//!
//! These encode the structural relationships from the NexVigilant KSB Framework.
//! (source: 04-ksb-competency-framework.md, EPA and CPA tables)
//!
//! ## Relationship Graph
//!
//! ```text
//! CPA-1 Case Management ──→ EPA-01, EPA-02, EPA-03
//! CPA-2 Signal Management ──→ EPA-04, EPA-05
//! CPA-3 Risk Management ──→ EPA-06, EPA-07
//! CPA-4 Quality & Compliance ──→ EPA-08, EPA-09, EPA-12
//! CPA-5 Data & Technology ──→ EPA-10, EPA-11
//! CPA-6 Communication ──→ EPA-17, EPA-19
//! CPA-7 Research & Dev ──→ EPA-16, EPA-20
//! CPA-8 AI-Enhanced PV ──→ EPA-10 (gateway)
//! ```

use crate::caba::cpa::CPACategory;
use crate::caba::domain::DomainCategory;
use crate::caba::epa::EPACategory;

/// Get the primary domains required for an EPA.
///
/// Returns domains that are prerequisites for performing the EPA,
/// ordered by relevance (primary domains first).
/// (source: 04-ksb-competency-framework.md domain-to-primitive and EPA tables)
#[must_use]
pub fn epa_required_domains(epa: EPACategory) -> &'static [DomainCategory] {
    use DomainCategory::*;
    match epa {
        // Core Tier
        EPACategory::Epa01ProcessIcsrs => {
            &[D04IcsrProcessing, D03MedicalTerminology, D01PvFoundations]
        }
        EPACategory::Epa02AssessCausality => &[
            D02ClinicalPharmacology,
            D04IcsrProcessing,
            D03MedicalTerminology,
        ],
        EPACategory::Epa03ManageCaseQuality => &[D04IcsrProcessing, D09QualityManagement],
        EPACategory::Epa04DetectSignals => &[D05SignalDetection, D13AdvancedAnalytics],
        EPACategory::Epa05ValidateSignals => &[
            D05SignalDetection,
            D02ClinicalPharmacology,
            D06RiskAssessment,
        ],
        EPACategory::Epa06AssessBenefitRisk => &[
            D06RiskAssessment,
            D02ClinicalPharmacology,
            D05SignalDetection,
        ],
        EPACategory::Epa07ManageRiskMinimization => &[D06RiskAssessment, D07RegulatoryIntelligence],
        EPACategory::Epa08RegulatorySubmissions => &[D07RegulatoryIntelligence, D04IcsrProcessing],

        // Advanced Tier
        EPACategory::Epa09RegulatoryIntelligence => {
            &[D07RegulatoryIntelligence, D11GlobalOperations]
        }
        EPACategory::Epa10IntegrateAi => &[D08PvSystems, D13AdvancedAnalytics, D01PvFoundations],
        EPACategory::Epa11ManagePvSystems => &[D08PvSystems, D09QualityManagement],
        EPACategory::Epa12ConductAudits => &[D09QualityManagement, D07RegulatoryIntelligence],
        EPACategory::Epa13SpecialPopulations => &[D10SpecialPopulations, D02ClinicalPharmacology],
        EPACategory::Epa14CoordinateGlobalPv => &[D11GlobalOperations, D07RegulatoryIntelligence],
        EPACategory::Epa17StakeholderCommunication => &[D14Communication, D06RiskAssessment],

        // Expert Tier
        EPACategory::Epa15LeadPvStrategy => &[D12ProgramManagement, D11GlobalOperations],
        EPACategory::Epa16AdvancedAnalytics => &[D13AdvancedAnalytics, D05SignalDetection],
        EPACategory::Epa18DevelopTalent => &[D15ProfessionalDevelopment, D14Communication],
        EPACategory::Epa19CrisisCommunication => {
            &[D14Communication, D06RiskAssessment, D12ProgramManagement]
        }
        EPACategory::Epa20PharmacoepidemiologyStudies => &[
            D13AdvancedAnalytics,
            D02ClinicalPharmacology,
            D05SignalDetection,
        ],
        EPACategory::Epa21LeadTransformation => &[
            D12ProgramManagement,
            D08PvSystems,
            D15ProfessionalDevelopment,
        ],
    }
}

/// Get the EPAs that belong to a CPA.
///
/// (source: 04-ksb-competency-framework.md CPA table)
#[must_use]
pub fn cpa_required_epas(cpa: CPACategory) -> &'static [EPACategory] {
    use EPACategory::*;
    match cpa {
        CPACategory::Cpa1CaseManagement => &[
            Epa01ProcessIcsrs,
            Epa02AssessCausality,
            Epa03ManageCaseQuality,
        ],
        CPACategory::Cpa2SignalManagement => &[Epa04DetectSignals, Epa05ValidateSignals],
        CPACategory::Cpa3RiskManagement => &[Epa06AssessBenefitRisk, Epa07ManageRiskMinimization],
        CPACategory::Cpa4QualityCompliance => &[
            Epa08RegulatorySubmissions,
            Epa09RegulatoryIntelligence,
            Epa12ConductAudits,
        ],
        CPACategory::Cpa5DataTechnology => &[Epa10IntegrateAi, Epa11ManagePvSystems],
        CPACategory::Cpa6Communication => {
            &[Epa17StakeholderCommunication, Epa19CrisisCommunication]
        }
        CPACategory::Cpa7ResearchDevelopment => {
            &[Epa16AdvancedAnalytics, Epa20PharmacoepidemiologyStudies]
        }
        CPACategory::Cpa8AiEnhancedPv => &[
            Epa10IntegrateAi,
            Epa13SpecialPopulations,
            Epa14CoordinateGlobalPv,
            Epa15LeadPvStrategy,
            Epa18DevelopTalent,
            Epa21LeadTransformation,
        ],
    }
}

/// Get all domains touched by a CPA (union of its EPA domain requirements).
#[must_use]
pub fn cpa_required_domains(cpa: CPACategory) -> Vec<DomainCategory> {
    let mut domains = Vec::new();
    for epa in cpa_required_epas(cpa) {
        for domain in epa_required_domains(*epa) {
            if !domains.contains(domain) {
                domains.push(*domain);
            }
        }
    }
    domains
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epa_01_requires_icsr_processing() {
        let domains = epa_required_domains(EPACategory::Epa01ProcessIcsrs);
        assert!(domains.contains(&DomainCategory::D04IcsrProcessing));
    }

    #[test]
    fn test_epa_04_requires_signal_detection() {
        let domains = epa_required_domains(EPACategory::Epa04DetectSignals);
        assert!(domains.contains(&DomainCategory::D05SignalDetection));
    }

    #[test]
    fn test_cpa1_contains_case_processing_epas() {
        let epas = cpa_required_epas(CPACategory::Cpa1CaseManagement);
        assert_eq!(epas.len(), 3);
        assert!(epas.contains(&EPACategory::Epa01ProcessIcsrs));
        assert!(epas.contains(&EPACategory::Epa02AssessCausality));
        assert!(epas.contains(&EPACategory::Epa03ManageCaseQuality));
    }

    #[test]
    fn test_cpa8_includes_gateway_and_orphans() {
        let epas = cpa_required_epas(CPACategory::Cpa8AiEnhancedPv);
        // CPA-8 is the capstone: EPA-10 (gateway) + 5 orphan EPAs
        assert_eq!(epas.len(), 6);
        assert!(epas.contains(&EPACategory::Epa10IntegrateAi));
        assert!(epas.contains(&EPACategory::Epa13SpecialPopulations));
        assert!(epas.contains(&EPACategory::Epa15LeadPvStrategy));
        assert!(epas.contains(&EPACategory::Epa21LeadTransformation));
    }

    #[test]
    fn test_cpa_required_domains_deduplicates() {
        let domains = cpa_required_domains(CPACategory::Cpa4QualityCompliance);
        // EPA-08 and EPA-09 both require D07 — should appear only once
        let d07_count = domains
            .iter()
            .filter(|d| **d == DomainCategory::D07RegulatoryIntelligence)
            .count();
        assert_eq!(d07_count, 1);
    }

    #[test]
    fn test_all_21_epas_have_domain_mappings() {
        let all_epas = [
            EPACategory::Epa01ProcessIcsrs,
            EPACategory::Epa02AssessCausality,
            EPACategory::Epa03ManageCaseQuality,
            EPACategory::Epa04DetectSignals,
            EPACategory::Epa05ValidateSignals,
            EPACategory::Epa06AssessBenefitRisk,
            EPACategory::Epa07ManageRiskMinimization,
            EPACategory::Epa08RegulatorySubmissions,
            EPACategory::Epa09RegulatoryIntelligence,
            EPACategory::Epa10IntegrateAi,
            EPACategory::Epa11ManagePvSystems,
            EPACategory::Epa12ConductAudits,
            EPACategory::Epa13SpecialPopulations,
            EPACategory::Epa14CoordinateGlobalPv,
            EPACategory::Epa15LeadPvStrategy,
            EPACategory::Epa16AdvancedAnalytics,
            EPACategory::Epa17StakeholderCommunication,
            EPACategory::Epa18DevelopTalent,
            EPACategory::Epa19CrisisCommunication,
            EPACategory::Epa20PharmacoepidemiologyStudies,
            EPACategory::Epa21LeadTransformation,
        ];
        for epa in &all_epas {
            let domains = epa_required_domains(*epa);
            assert!(!domains.is_empty(), "EPA {:?} has no domain mappings", epa);
        }
    }

    #[test]
    fn test_all_8_cpas_have_epa_mappings() {
        let all_cpas = [
            CPACategory::Cpa1CaseManagement,
            CPACategory::Cpa2SignalManagement,
            CPACategory::Cpa3RiskManagement,
            CPACategory::Cpa4QualityCompliance,
            CPACategory::Cpa5DataTechnology,
            CPACategory::Cpa6Communication,
            CPACategory::Cpa7ResearchDevelopment,
            CPACategory::Cpa8AiEnhancedPv,
        ];
        for cpa in &all_cpas {
            let epas = cpa_required_epas(*cpa);
            assert!(!epas.is_empty(), "CPA {:?} has no EPA mappings", cpa);
        }
    }
}
