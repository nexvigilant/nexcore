//! CPA (Critical Practice Areas) Types
//!
//! 8 CPAs integrating multiple EPAs.
//! CPA8 is the capstone requiring EPA10 Level 4+.

use serde::{Deserialize, Serialize};

use super::domain::CompetencyLevel;
use super::epa::{EntrustmentLevel, Epa};

/// 8 Critical Practice Areas
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Cpa {
    /// CPA1: Case Management Activities
    #[serde(rename = "CPA1")]
    Cpa1CaseManagement,
    /// CPA2: Signal Management Activities
    #[serde(rename = "CPA2")]
    Cpa2SignalManagement,
    /// CPA3: Risk Management Activities
    #[serde(rename = "CPA3")]
    Cpa3RiskManagement,
    /// CPA4: Quality and Compliance Activities
    #[serde(rename = "CPA4")]
    Cpa4QualityCompliance,
    /// CPA5: Data and Technology Activities
    #[serde(rename = "CPA5")]
    Cpa5DataTechnology,
    /// CPA6: Communication and Stakeholder Activities
    #[serde(rename = "CPA6")]
    Cpa6CommunicationStakeholder,
    /// CPA7: Research and Development Activities
    #[serde(rename = "CPA7")]
    Cpa7ResearchDevelopment,
    /// CPA8: AI-Enhanced Pharmacovigilance Activities (Capstone)
    #[serde(rename = "CPA8")]
    Cpa8AiEnhancedPv,
}

impl Cpa {
    /// All CPAs
    pub const ALL: [Self; 8] = [
        Self::Cpa1CaseManagement,
        Self::Cpa2SignalManagement,
        Self::Cpa3RiskManagement,
        Self::Cpa4QualityCompliance,
        Self::Cpa5DataTechnology,
        Self::Cpa6CommunicationStakeholder,
        Self::Cpa7ResearchDevelopment,
        Self::Cpa8AiEnhancedPv,
    ];

    /// Foundational CPAs (non-capstone)
    pub const FOUNDATIONAL: [Self; 7] = [
        Self::Cpa1CaseManagement,
        Self::Cpa2SignalManagement,
        Self::Cpa3RiskManagement,
        Self::Cpa4QualityCompliance,
        Self::Cpa5DataTechnology,
        Self::Cpa6CommunicationStakeholder,
        Self::Cpa7ResearchDevelopment,
    ];

    /// Get the CPA number (1-8)
    #[must_use]
    pub const fn number(self) -> u8 {
        match self {
            Self::Cpa1CaseManagement => 1,
            Self::Cpa2SignalManagement => 2,
            Self::Cpa3RiskManagement => 3,
            Self::Cpa4QualityCompliance => 4,
            Self::Cpa5DataTechnology => 5,
            Self::Cpa6CommunicationStakeholder => 6,
            Self::Cpa7ResearchDevelopment => 7,
            Self::Cpa8AiEnhancedPv => 8,
        }
    }

    /// Check if this is the capstone CPA (CPA8)
    #[must_use]
    pub const fn is_capstone(self) -> bool {
        matches!(self, Self::Cpa8AiEnhancedPv)
    }

    /// Get the display name
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Cpa1CaseManagement => "Case Management Activities",
            Self::Cpa2SignalManagement => "Signal Management Activities",
            Self::Cpa3RiskManagement => "Risk Management Activities",
            Self::Cpa4QualityCompliance => "Quality and Compliance Activities",
            Self::Cpa5DataTechnology => "Data and Technology Activities",
            Self::Cpa6CommunicationStakeholder => "Communication and Stakeholder Activities",
            Self::Cpa7ResearchDevelopment => "Research and Development Activities",
            Self::Cpa8AiEnhancedPv => "AI-Enhanced Pharmacovigilance Activities",
        }
    }
}

/// CPA achievement status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CpaStatus {
    /// Not started
    #[default]
    NotStarted,
    /// In progress
    InProgress,
    /// Achieved
    Achieved,
    /// Advanced level
    Advanced,
    /// Expert level
    Expert,
}

/// CPA progression level definition
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CpaProgressionLevel {
    /// Competency level for this CPA stage
    pub level: CompetencyLevel,
    /// What user can do at this level
    pub capabilities: Vec<String>,
    /// How to validate achievement
    pub assessment_criteria: Vec<String>,
}

/// Complete CPA definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpaDefinition {
    /// CPA identifier
    pub id: Cpa,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// EPAs that contribute to this CPA
    pub integrates_epas: Vec<Epa>,
    /// Progression levels
    pub progression_levels: Vec<CpaProgressionLevel>,
    /// Whether this is the capstone CPA
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_capstone: Option<bool>,
}

/// User's CPA progress tracking
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CpaProgress {
    /// The CPA being tracked
    pub cpa: Cpa,
    /// Current status
    pub status: CpaStatus,
    /// Current competency level within CPA
    pub current_level: CompetencyLevel,
    /// Date of CPA entry (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_date: Option<String>,
    /// Date of completion (ISO 8601)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_date: Option<String>,
    /// EPA contributions to this CPA
    pub contributing_epas: Vec<EpaContribution>,
    /// Readiness percentage (0-100)
    pub readiness: u8,
}

/// EPA contribution to a CPA
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpaContribution {
    /// The contributing EPA
    pub epa: Epa,
    /// Current entrustment level
    pub entrustment_level: EntrustmentLevel,
    /// Whether it meets the CPA requirement
    pub meets_requirement: bool,
}

/// Check if user is eligible for CPA8 entry
///
/// Requires EPA10 Level 4+
#[must_use]
pub fn check_cpa8_eligibility(epa10_level: EntrustmentLevel) -> bool {
    epa10_level.qualifies_for_cpa8_gateway()
}

/// Calculate CPA readiness percentage
///
/// Based on how many contributing EPAs meet their requirements
#[must_use]
pub fn calculate_cpa_readiness(contributing_epas: &[EpaContribution]) -> u8 {
    if contributing_epas.is_empty() {
        return 0;
    }

    let met_count = contributing_epas
        .iter()
        .filter(|e| e.meets_requirement)
        .count();

    // INVARIANT: percentage is always 0-100
    let percentage = (met_count * 100) / contributing_epas.len();
    percentage.min(100) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpa_capstone() {
        assert!(Cpa::Cpa8AiEnhancedPv.is_capstone());
        assert!(!Cpa::Cpa1CaseManagement.is_capstone());
    }

    #[test]
    fn test_cpa8_eligibility() {
        assert!(!check_cpa8_eligibility(EntrustmentLevel::Level3));
        assert!(check_cpa8_eligibility(EntrustmentLevel::Level4));
        assert!(check_cpa8_eligibility(EntrustmentLevel::Level5Plus));
    }

    #[test]
    fn test_cpa_readiness_calculation() {
        let contributions = vec![
            EpaContribution {
                epa: Epa::Epa1ProcessIcsrs,
                entrustment_level: EntrustmentLevel::Level4,
                meets_requirement: true,
            },
            EpaContribution {
                epa: Epa::Epa2ConductLiteratureReview,
                entrustment_level: EntrustmentLevel::Level2,
                meets_requirement: false,
            },
        ];

        assert_eq!(calculate_cpa_readiness(&contributions), 50);
    }

    #[test]
    fn test_cpa_readiness_empty() {
        assert_eq!(calculate_cpa_readiness(&[]), 0);
    }

    #[test]
    fn test_cpa_readiness_all_met() {
        let contributions = vec![
            EpaContribution {
                epa: Epa::Epa1ProcessIcsrs,
                entrustment_level: EntrustmentLevel::Level4,
                meets_requirement: true,
            },
            EpaContribution {
                epa: Epa::Epa2ConductLiteratureReview,
                entrustment_level: EntrustmentLevel::Level4,
                meets_requirement: true,
            },
        ];

        assert_eq!(calculate_cpa_readiness(&contributions), 100);
    }
}
