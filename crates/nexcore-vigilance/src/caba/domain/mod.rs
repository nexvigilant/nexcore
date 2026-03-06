//! PV Competency Domain System
//!
//! The 15 pharmacovigilance knowledge domains from the NexVigilant KSB Framework
//! (source: ~/Vaults/nexvigilant/400-projects/ksb-framework/domains/).
//!
//! ## Domain Clusters
//!
//! - **Foundational (D01-D03)**: PV foundations, clinical pharmacology, medical terminology
//! - **Core Operational (D04-D06)**: ICSR processing, signal detection, risk assessment
//! - **Regulatory (D07-D08)**: Regulatory intelligence, PV systems & technology
//! - **Quality (D09)**: Quality management in PV
//! - **Specialized (D10-D11)**: Special populations, global PV operations
//! - **Management (D12)**: PV program management & strategy
//! - **Advanced (D13)**: Advanced analytics & data science
//! - **Cross-cutting (D14-D15)**: Communication, professional development

use crate::caba::proficiency::ProficiencyLevel;
use serde::{Deserialize, Serialize};

/// The 15 PV knowledge domains from the NexVigilant KSB Framework.
///
/// Each domain maps to a cluster of KSBs at proficiency levels L1-L5++.
/// (source: ~/Vaults/nexvigilant/400-projects/ksb-framework/domains/)
///
/// # L0 Quark - Domain enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainCategory {
    // Foundational (D01-D03)
    /// D01: Foundations of Pharmacovigilance in the AI Era
    #[serde(rename = "D01: Foundations of Pharmacovigilance in the AI Era")]
    D01PvFoundations,
    /// D02: Clinical Pharmacology and Drug Safety Science
    #[serde(rename = "D02: Clinical Pharmacology and Drug Safety Science")]
    D02ClinicalPharmacology,
    /// D03: Medical Terminology and Disease Classification
    #[serde(rename = "D03: Medical Terminology and Disease Classification")]
    D03MedicalTerminology,

    // Core Operational (D04-D06)
    /// D04: Individual Case Safety Report Processing
    #[serde(rename = "D04: Individual Case Safety Report Processing")]
    D04IcsrProcessing,
    /// D05: Signal Detection and Analysis
    #[serde(rename = "D05: Signal Detection and Analysis")]
    D05SignalDetection,
    /// D06: Risk Assessment and Communication
    #[serde(rename = "D06: Risk Assessment and Communication")]
    D06RiskAssessment,

    // Regulatory (D07-D08)
    /// D07: Regulatory Intelligence and Compliance
    #[serde(rename = "D07: Regulatory Intelligence and Compliance")]
    D07RegulatoryIntelligence,
    /// D08: Pharmacovigilance Systems and Technology
    #[serde(rename = "D08: Pharmacovigilance Systems and Technology")]
    D08PvSystems,

    // Quality (D09)
    /// D09: Quality Management in Pharmacovigilance
    #[serde(rename = "D09: Quality Management in Pharmacovigilance")]
    D09QualityManagement,

    // Specialized (D10-D11)
    /// D10: Special Populations and Products
    #[serde(rename = "D10: Special Populations and Products")]
    D10SpecialPopulations,
    /// D11: Global PV Operations
    #[serde(rename = "D11: Global PV Operations")]
    D11GlobalOperations,

    // Management (D12)
    /// D12: PV Program Management and Strategy
    #[serde(rename = "D12: PV Program Management and Strategy")]
    D12ProgramManagement,

    // Advanced (D13)
    /// D13: Advanced Analytics and Data Science
    #[serde(rename = "D13: Advanced Analytics and Data Science")]
    D13AdvancedAnalytics,

    // Cross-cutting (D14-D15)
    /// D14: Communication and Stakeholder Management
    #[serde(rename = "D14: Communication and Stakeholder Management")]
    D14Communication,
    /// D15: Professional Development and Ethics
    #[serde(rename = "D15: Professional Development and Ethics")]
    D15ProfessionalDevelopment,
}

impl DomainCategory {
    /// All 15 domain variants, ordered D01–D15.
    pub const ALL: [Self; 15] = [
        Self::D01PvFoundations,
        Self::D02ClinicalPharmacology,
        Self::D03MedicalTerminology,
        Self::D04IcsrProcessing,
        Self::D05SignalDetection,
        Self::D06RiskAssessment,
        Self::D07RegulatoryIntelligence,
        Self::D08PvSystems,
        Self::D09QualityManagement,
        Self::D10SpecialPopulations,
        Self::D11GlobalOperations,
        Self::D12ProgramManagement,
        Self::D13AdvancedAnalytics,
        Self::D14Communication,
        Self::D15ProfessionalDevelopment,
    ];

    /// Get the domain number (1-15).
    #[must_use]
    pub const fn number(&self) -> u8 {
        match self {
            Self::D01PvFoundations => 1,
            Self::D02ClinicalPharmacology => 2,
            Self::D03MedicalTerminology => 3,
            Self::D04IcsrProcessing => 4,
            Self::D05SignalDetection => 5,
            Self::D06RiskAssessment => 6,
            Self::D07RegulatoryIntelligence => 7,
            Self::D08PvSystems => 8,
            Self::D09QualityManagement => 9,
            Self::D10SpecialPopulations => 10,
            Self::D11GlobalOperations => 11,
            Self::D12ProgramManagement => 12,
            Self::D13AdvancedAnalytics => 13,
            Self::D14Communication => 14,
            Self::D15ProfessionalDevelopment => 15,
        }
    }

    /// Get the domain cluster.
    ///
    /// Clusters group domains by function within the PV system.
    #[must_use]
    pub const fn cluster(&self) -> DomainCluster {
        match self {
            Self::D01PvFoundations
            | Self::D02ClinicalPharmacology
            | Self::D03MedicalTerminology => DomainCluster::Foundational,
            Self::D04IcsrProcessing | Self::D05SignalDetection | Self::D06RiskAssessment => {
                DomainCluster::CoreOperational
            }
            Self::D07RegulatoryIntelligence | Self::D08PvSystems => DomainCluster::Regulatory,
            Self::D09QualityManagement => DomainCluster::Quality,
            Self::D10SpecialPopulations | Self::D11GlobalOperations => DomainCluster::Specialized,
            Self::D12ProgramManagement => DomainCluster::Management,
            Self::D13AdvancedAnalytics => DomainCluster::Advanced,
            Self::D14Communication | Self::D15ProfessionalDevelopment => {
                DomainCluster::CrossCutting
            }
        }
    }

    /// Get display string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::D01PvFoundations => "D01: Foundations of Pharmacovigilance in the AI Era",
            Self::D02ClinicalPharmacology => "D02: Clinical Pharmacology and Drug Safety Science",
            Self::D03MedicalTerminology => "D03: Medical Terminology and Disease Classification",
            Self::D04IcsrProcessing => "D04: Individual Case Safety Report Processing",
            Self::D05SignalDetection => "D05: Signal Detection and Analysis",
            Self::D06RiskAssessment => "D06: Risk Assessment and Communication",
            Self::D07RegulatoryIntelligence => "D07: Regulatory Intelligence and Compliance",
            Self::D08PvSystems => "D08: Pharmacovigilance Systems and Technology",
            Self::D09QualityManagement => "D09: Quality Management in Pharmacovigilance",
            Self::D10SpecialPopulations => "D10: Special Populations and Products",
            Self::D11GlobalOperations => "D11: Global PV Operations",
            Self::D12ProgramManagement => "D12: PV Program Management and Strategy",
            Self::D13AdvancedAnalytics => "D13: Advanced Analytics and Data Science",
            Self::D14Communication => "D14: Communication and Stakeholder Management",
            Self::D15ProfessionalDevelopment => "D15: Professional Development and Ethics",
        }
    }
}

impl std::fmt::Display for DomainCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Domain cluster grouping.
///
/// Clusters organize the 15 domains by their role in the PV system.
/// (source: 04-ksb-competency-framework.md domain table)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainCluster {
    /// D01-D03: PV foundations, clinical pharmacology, medical terminology
    Foundational,
    /// D04-D06: ICSR processing, signal detection, risk assessment
    CoreOperational,
    /// D07-D08: Regulatory intelligence, PV systems & technology
    Regulatory,
    /// D09: Quality management in PV
    Quality,
    /// D10-D11: Special populations, global PV operations
    Specialized,
    /// D12: PV program management & strategy
    Management,
    /// D13: Advanced analytics & data science
    Advanced,
    /// D14-D15: Communication, professional development
    CrossCutting,
}

impl DomainCluster {
    /// Get display string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Foundational => "Foundational",
            Self::CoreOperational => "Core Operational",
            Self::Regulatory => "Regulatory",
            Self::Quality => "Quality",
            Self::Specialized => "Specialized",
            Self::Management => "Management",
            Self::Advanced => "Advanced",
            Self::CrossCutting => "Cross-cutting",
        }
    }
}

impl std::fmt::Display for DomainCluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Specifies required proficiency in a specific domain.
///
/// Used for EPA and CPA prerequisite validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRequirement {
    /// The domain category
    pub domain: DomainCategory,
    /// Minimum required proficiency level
    pub minimum_level: ProficiencyLevel,
    /// Requirement type: primary or supporting
    pub requirement_type: RequirementType,
    /// Behavioral anchors for this requirement
    #[serde(default)]
    pub behavioral_anchors: Vec<String>,
}

/// Type of domain requirement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RequirementType {
    /// Primary requirement - must be met
    Primary,
    /// Supporting requirement - recommended but not blocking
    Supporting,
}

impl DomainRequirement {
    /// Create a new primary domain requirement.
    #[must_use]
    pub fn primary(domain: DomainCategory, minimum_level: ProficiencyLevel) -> Self {
        Self {
            domain,
            minimum_level,
            requirement_type: RequirementType::Primary,
            behavioral_anchors: Vec::new(),
        }
    }

    /// Create a new supporting domain requirement.
    #[must_use]
    pub fn supporting(domain: DomainCategory, minimum_level: ProficiencyLevel) -> Self {
        Self {
            domain,
            minimum_level,
            requirement_type: RequirementType::Supporting,
            behavioral_anchors: Vec::new(),
        }
    }

    /// Check if this is a primary requirement.
    #[must_use]
    pub fn is_primary(&self) -> bool {
        self.requirement_type == RequirementType::Primary
    }

    /// Check if a proficiency level meets this requirement.
    #[must_use]
    pub fn is_met_by(&self, level: ProficiencyLevel) -> bool {
        level >= self.minimum_level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_number() {
        assert_eq!(DomainCategory::D01PvFoundations.number(), 1);
        assert_eq!(DomainCategory::D15ProfessionalDevelopment.number(), 15);
    }

    #[test]
    fn test_domain_cluster() {
        assert_eq!(
            DomainCategory::D01PvFoundations.cluster(),
            DomainCluster::Foundational
        );
        assert_eq!(
            DomainCategory::D05SignalDetection.cluster(),
            DomainCluster::CoreOperational
        );
        assert_eq!(
            DomainCategory::D08PvSystems.cluster(),
            DomainCluster::Regulatory
        );
        assert_eq!(
            DomainCategory::D09QualityManagement.cluster(),
            DomainCluster::Quality
        );
        assert_eq!(
            DomainCategory::D11GlobalOperations.cluster(),
            DomainCluster::Specialized
        );
        assert_eq!(
            DomainCategory::D12ProgramManagement.cluster(),
            DomainCluster::Management
        );
        assert_eq!(
            DomainCategory::D13AdvancedAnalytics.cluster(),
            DomainCluster::Advanced
        );
        assert_eq!(
            DomainCategory::D15ProfessionalDevelopment.cluster(),
            DomainCluster::CrossCutting
        );
    }

    #[test]
    fn test_domain_requirement_met() {
        let req = DomainRequirement::primary(
            DomainCategory::D01PvFoundations,
            ProficiencyLevel::L3Competent,
        );

        assert!(req.is_met_by(ProficiencyLevel::L3Competent));
        assert!(req.is_met_by(ProficiencyLevel::L5Expert));
        assert!(!req.is_met_by(ProficiencyLevel::L2AdvancedBeginner));
    }

    #[test]
    fn test_all_15_domains_exist() {
        let domains = [
            DomainCategory::D01PvFoundations,
            DomainCategory::D02ClinicalPharmacology,
            DomainCategory::D03MedicalTerminology,
            DomainCategory::D04IcsrProcessing,
            DomainCategory::D05SignalDetection,
            DomainCategory::D06RiskAssessment,
            DomainCategory::D07RegulatoryIntelligence,
            DomainCategory::D08PvSystems,
            DomainCategory::D09QualityManagement,
            DomainCategory::D10SpecialPopulations,
            DomainCategory::D11GlobalOperations,
            DomainCategory::D12ProgramManagement,
            DomainCategory::D13AdvancedAnalytics,
            DomainCategory::D14Communication,
            DomainCategory::D15ProfessionalDevelopment,
        ];
        for (i, d) in domains.iter().enumerate() {
            assert_eq!(d.number() as usize, i + 1);
        }
    }
}
