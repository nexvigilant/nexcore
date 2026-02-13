//! EPA (Entrustable Professional Activities) Types
//!
//! 20 EPAs: 10 Core + 10 Executive
//! Each EPA has entrustment levels from Level 1 (observation) to Level 5+ (supervision)

use serde::{Deserialize, Serialize};

use super::domain::{CompetencyLevel, Domain};

/// 20 Entrustable Professional Activities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Epa {
    // Core EPAs (1-10)
    /// EPA1: Process ICSRs
    #[serde(rename = "EPA1")]
    Epa1ProcessIcsrs,
    /// EPA2: Conduct Literature Review
    #[serde(rename = "EPA2")]
    Epa2ConductLiteratureReview,
    /// EPA3: Present Safety Information
    #[serde(rename = "EPA3")]
    Epa3PresentSafetyInformation,
    /// EPA4: Prepare Aggregate Reports
    #[serde(rename = "EPA4")]
    Epa4PrepareAggregateReports,
    /// EPA5: Conduct Signal Detection
    #[serde(rename = "EPA5")]
    Epa5ConductSignalDetection,
    /// EPA6: Manage Safety Database
    #[serde(rename = "EPA6")]
    Epa6ManageSafetyDatabase,
    /// EPA7: Develop Risk Minimization
    #[serde(rename = "EPA7")]
    Epa7DevelopRiskMinimization,
    /// EPA8: Conduct Signal Evaluation
    #[serde(rename = "EPA8")]
    Epa8ConductSignalEvaluation,
    /// EPA9: Ensure Quality Compliance
    #[serde(rename = "EPA9")]
    Epa9EnsureQualityCompliance,
    /// EPA10: Implement AI Tools (AI Gateway)
    #[serde(rename = "EPA10")]
    Epa10ImplementAiTools,

    // Executive EPAs (11-20)
    /// EPA11: Design Benefit-Risk Assessment
    #[serde(rename = "EPA11")]
    Epa11DesignBenefitRisk,
    /// EPA12: Lead Technology Adoption
    #[serde(rename = "EPA12")]
    Epa12LeadTechnologyAdoption,
    /// EPA13: Develop Regulatory Strategy
    #[serde(rename = "EPA13")]
    Epa13DevelopRegulatoryStrategy,
    /// EPA14: Conduct Crisis Management
    #[serde(rename = "EPA14")]
    Epa14ConductCrisisManagement,
    /// EPA15: Design Safety Studies
    #[serde(rename = "EPA15")]
    Epa15DesignSafetyStudies,
    /// EPA16: Develop Safety Strategy
    #[serde(rename = "EPA16")]
    Epa16DevelopSafetyStrategy,
    /// EPA17: Lead Organizational Change
    #[serde(rename = "EPA17")]
    Epa17LeadOrganizationalChange,
    /// EPA18: Manage Stakeholder Engagement
    #[serde(rename = "EPA18")]
    Epa18ManageStakeholderEngagement,
    /// EPA19: Provide Expert Testimony
    #[serde(rename = "EPA19")]
    Epa19ProvideExpertTestimony,
    /// EPA20: Lead Innovation
    #[serde(rename = "EPA20")]
    Epa20LeadInnovation,
}

/// EPA category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EpaCategory {
    /// Core EPAs (1-10)
    Core,
    /// Executive EPAs (11-20)
    Executive,
}

impl Epa {
    /// All core EPAs
    pub const CORE: [Self; 10] = [
        Self::Epa1ProcessIcsrs,
        Self::Epa2ConductLiteratureReview,
        Self::Epa3PresentSafetyInformation,
        Self::Epa4PrepareAggregateReports,
        Self::Epa5ConductSignalDetection,
        Self::Epa6ManageSafetyDatabase,
        Self::Epa7DevelopRiskMinimization,
        Self::Epa8ConductSignalEvaluation,
        Self::Epa9EnsureQualityCompliance,
        Self::Epa10ImplementAiTools,
    ];

    /// All executive EPAs
    pub const EXECUTIVE: [Self; 10] = [
        Self::Epa11DesignBenefitRisk,
        Self::Epa12LeadTechnologyAdoption,
        Self::Epa13DevelopRegulatoryStrategy,
        Self::Epa14ConductCrisisManagement,
        Self::Epa15DesignSafetyStudies,
        Self::Epa16DevelopSafetyStrategy,
        Self::Epa17LeadOrganizationalChange,
        Self::Epa18ManageStakeholderEngagement,
        Self::Epa19ProvideExpertTestimony,
        Self::Epa20LeadInnovation,
    ];

    /// Get the EPA number (1-20)
    #[must_use]
    pub const fn number(self) -> u8 {
        match self {
            Self::Epa1ProcessIcsrs => 1,
            Self::Epa2ConductLiteratureReview => 2,
            Self::Epa3PresentSafetyInformation => 3,
            Self::Epa4PrepareAggregateReports => 4,
            Self::Epa5ConductSignalDetection => 5,
            Self::Epa6ManageSafetyDatabase => 6,
            Self::Epa7DevelopRiskMinimization => 7,
            Self::Epa8ConductSignalEvaluation => 8,
            Self::Epa9EnsureQualityCompliance => 9,
            Self::Epa10ImplementAiTools => 10,
            Self::Epa11DesignBenefitRisk => 11,
            Self::Epa12LeadTechnologyAdoption => 12,
            Self::Epa13DevelopRegulatoryStrategy => 13,
            Self::Epa14ConductCrisisManagement => 14,
            Self::Epa15DesignSafetyStudies => 15,
            Self::Epa16DevelopSafetyStrategy => 16,
            Self::Epa17LeadOrganizationalChange => 17,
            Self::Epa18ManageStakeholderEngagement => 18,
            Self::Epa19ProvideExpertTestimony => 19,
            Self::Epa20LeadInnovation => 20,
        }
    }

    /// Get the category (core or executive)
    #[must_use]
    pub const fn category(self) -> EpaCategory {
        if self.number() <= 10 {
            EpaCategory::Core
        } else {
            EpaCategory::Executive
        }
    }

    /// Check if this is the AI Gateway EPA (EPA10)
    #[must_use]
    pub const fn is_gateway(self) -> bool {
        matches!(self, Self::Epa10ImplementAiTools)
    }

    /// Check if this is a core EPA
    #[must_use]
    pub const fn is_core(self) -> bool {
        matches!(self.category(), EpaCategory::Core)
    }

    /// Check if this is an executive EPA
    #[must_use]
    pub const fn is_executive(self) -> bool {
        matches!(self.category(), EpaCategory::Executive)
    }
}

/// EPA entrustment levels
///
/// Progression from observation to executive supervision
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum EntrustmentLevel {
    /// Level 1: Observation only
    #[serde(rename = "Level_1")]
    #[default]
    Level1,
    /// Level 2: Direct supervision
    #[serde(rename = "Level_2")]
    Level2,
    /// Level 3: Indirect supervision
    #[serde(rename = "Level_3")]
    Level3,
    /// Level 4: Supervision available
    #[serde(rename = "Level_4")]
    Level4,
    /// Level 5: Supervise others
    #[serde(rename = "Level_5")]
    Level5,
    /// Level 5+: Executive supervision
    #[serde(rename = "Level_5+")]
    Level5Plus,
}

impl EntrustmentLevel {
    /// All levels in order
    pub const ALL: [Self; 6] = [
        Self::Level1,
        Self::Level2,
        Self::Level3,
        Self::Level4,
        Self::Level5,
        Self::Level5Plus,
    ];

    /// Get the next level, if any
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::Level1 => Some(Self::Level2),
            Self::Level2 => Some(Self::Level3),
            Self::Level3 => Some(Self::Level4),
            Self::Level4 => Some(Self::Level5),
            Self::Level5 => Some(Self::Level5Plus),
            Self::Level5Plus => None,
        }
    }

    /// Get the previous level, if any
    #[must_use]
    pub const fn previous(self) -> Option<Self> {
        match self {
            Self::Level1 => None,
            Self::Level2 => Some(Self::Level1),
            Self::Level3 => Some(Self::Level2),
            Self::Level4 => Some(Self::Level3),
            Self::Level5 => Some(Self::Level4),
            Self::Level5Plus => Some(Self::Level5),
        }
    }

    /// Get numeric index (0-5) for ordering
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::Level1 => 0,
            Self::Level2 => 1,
            Self::Level3 => 2,
            Self::Level4 => 3,
            Self::Level5 => 4,
            Self::Level5Plus => 5,
        }
    }

    /// Check if this level qualifies for CPA8 gateway (Level 4+)
    #[must_use]
    pub const fn qualifies_for_cpa8_gateway(self) -> bool {
        matches!(self, Self::Level4 | Self::Level5 | Self::Level5Plus)
    }
}

/// Entrustment level definition with criteria
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntrustmentLevelDefinition {
    /// The entrustment level
    pub level: EntrustmentLevel,
    /// Description of what this level means
    pub description: String,
    /// Criteria for assessment
    pub assessment_criteria: Vec<String>,
    /// Required evidence types
    pub required_evidence: Vec<String>,
}

/// Complete EPA definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpaDefinition {
    /// EPA identifier
    pub id: Epa,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Category (core or executive)
    pub category: EpaCategory,
    /// Required domain competencies
    pub required_domains: Vec<DomainRequirement>,
    /// Entrustment level definitions
    pub entrustment_levels: Vec<EntrustmentLevelDefinition>,
    /// CPAs this EPA contributes to
    pub contributes_to_cpas: Vec<String>,
    /// Whether this is a gateway activity
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_gateway_activity: Option<bool>,
    /// Gateway requirement details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gateway_requirement: Option<GatewayRequirement>,
}

/// Domain requirement for an EPA
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainRequirement {
    /// Required domain
    pub domain: Domain,
    /// Minimum competency level needed
    pub minimum_level: CompetencyLevel,
}

/// Gateway requirement for CPA entry
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayRequirement {
    /// Target CPA
    pub target_cpa: String,
    /// Minimum entrustment required
    pub minimum_entrustment: EntrustmentLevel,
}

/// User's EPA progress tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpaProgress {
    /// The EPA being tracked
    pub epa: Epa,
    /// Current entrustment level
    pub current_entrustment: EntrustmentLevel,
    /// History of entrustment achievements
    pub progression_history: Vec<EntrustmentRecord>,
    /// Next milestone target
    pub next_milestone: EpaMilestone,
}

/// Entrustment achievement record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntrustmentRecord {
    /// The level achieved
    pub level: EntrustmentLevel,
    /// Date achieved (ISO 8601)
    pub achieved_date: String,
    /// Supervisor who assessed
    pub assessed_by: String,
    /// Evidence artifact IDs
    pub evidence: Vec<String>,
}

/// Next EPA milestone
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpaMilestone {
    /// Target entrustment level
    pub target_level: EntrustmentLevel,
    /// Required evidence types
    pub required_evidence: Vec<String>,
    /// Estimated completion date (ISO 8601)
    pub estimated_completion: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entrustment_level_ordering() {
        assert!(EntrustmentLevel::Level1 < EntrustmentLevel::Level2);
        assert!(EntrustmentLevel::Level4 < EntrustmentLevel::Level5);
        assert!(EntrustmentLevel::Level5 < EntrustmentLevel::Level5Plus);
    }

    #[test]
    fn test_epa_category() {
        assert!(Epa::Epa1ProcessIcsrs.is_core());
        assert!(Epa::Epa10ImplementAiTools.is_core());
        assert!(Epa::Epa11DesignBenefitRisk.is_executive());
        assert!(Epa::Epa20LeadInnovation.is_executive());
    }

    #[test]
    fn test_epa_gateway() {
        assert!(Epa::Epa10ImplementAiTools.is_gateway());
        assert!(!Epa::Epa1ProcessIcsrs.is_gateway());
    }

    #[test]
    fn test_cpa8_gateway_qualification() {
        assert!(!EntrustmentLevel::Level3.qualifies_for_cpa8_gateway());
        assert!(EntrustmentLevel::Level4.qualifies_for_cpa8_gateway());
        assert!(EntrustmentLevel::Level5.qualifies_for_cpa8_gateway());
        assert!(EntrustmentLevel::Level5Plus.qualifies_for_cpa8_gateway());
    }
}
