//! Competency Domain System
//!
//! Migrated from Python `domains/regulatory/caba/caba/models/cpa.py`.
//!
//! ## 15 Core Competency Domains (PDC Framework)
//!
//! Organized into 5 groups:
//!
//! - **Group 1 (D1-D3)**: Foundational Domains
//! - **Group 2 (D4-D6)**: Technical Operations Domains
//! - **Group 3 (D7-D9)**: Information and Analysis Domains
//! - **Group 4 (D10-D12)**: Strategic Integration Domains
//! - **Group 5 (D13-D15)**: Leadership and Innovation Domains

use crate::caba::proficiency::ProficiencyLevel;
use serde::{Deserialize, Serialize};

/// The 15 core competency domains from PDC framework.
///
/// # L0 Quark - Domain enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DomainCategory {
    // Group 1: Foundational Domains (1-3)
    /// D1: Professional Foundations and Evolution
    #[serde(rename = "D1: Professional Foundations and Evolution")]
    D1ProfessionalFoundations,
    /// D2: Technical Subject Matter Expertise
    #[serde(rename = "D2: Technical Subject Matter Expertise")]
    D2TechnicalExpertise,
    /// D3: Pattern Recognition and Critical Classification
    #[serde(rename = "D3: Pattern Recognition and Critical Classification")]
    D3PatternRecognition,

    // Group 2: Technical Operations Domains (4-6)
    /// D4: Information Processing and Documentation Excellence
    #[serde(rename = "D4: Information Processing and Documentation Excellence")]
    D4InformationProcessing,
    /// D5: Controlled Environment Operations
    #[serde(rename = "D5: Controlled Environment Operations")]
    D5ControlledOperations,
    /// D6: Quality Systems and Error Prevention
    #[serde(rename = "D6: Quality Systems and Error Prevention")]
    D6QualitySystems,

    // Group 3: Information and Analysis Domains (7-9)
    /// D7: Information Systems and Advanced Analytics
    #[serde(rename = "D7: Information Systems and Advanced Analytics")]
    D7InformationSystems,
    /// D8: Pattern Detection and Advanced Validation
    #[serde(rename = "D8: Pattern Detection and Advanced Validation")]
    D8PatternDetection,
    /// D9: Complex Systems Analysis and Modeling
    #[serde(rename = "D9: Complex Systems Analysis and Modeling")]
    D9ComplexSystems,

    // Group 4: Strategic Integration Domains (10-12)
    /// D10: Decision Science and Trade-off Optimization
    #[serde(rename = "D10: Decision Science and Trade-off Optimization")]
    D10DecisionScience,
    /// D11: Strategic Implementation and Risk Excellence
    #[serde(rename = "D11: Strategic Implementation and Risk Excellence")]
    D11StrategicImplementation,
    /// D12: Regulatory Navigation and Compliance Excellence
    #[serde(rename = "D12: Regulatory Navigation and Compliance Excellence")]
    D12RegulatoryNavigation,

    // Group 5: Leadership and Innovation Domains (13-15)
    /// D13: Global Perspective and Cultural Fluency
    #[serde(rename = "D13: Global Perspective and Cultural Fluency")]
    D13GlobalPerspective,
    /// D14: Strategic Communication and Influence
    #[serde(rename = "D14: Strategic Communication and Influence")]
    D14StrategicCommunication,
    /// D15: Innovation Leadership and Transformation
    #[serde(rename = "D15: Innovation Leadership and Transformation")]
    D15InnovationLeadership,
}

impl DomainCategory {
    /// Get the domain number (1-15).
    #[must_use]
    pub const fn number(&self) -> u8 {
        match self {
            Self::D1ProfessionalFoundations => 1,
            Self::D2TechnicalExpertise => 2,
            Self::D3PatternRecognition => 3,
            Self::D4InformationProcessing => 4,
            Self::D5ControlledOperations => 5,
            Self::D6QualitySystems => 6,
            Self::D7InformationSystems => 7,
            Self::D8PatternDetection => 8,
            Self::D9ComplexSystems => 9,
            Self::D10DecisionScience => 10,
            Self::D11StrategicImplementation => 11,
            Self::D12RegulatoryNavigation => 12,
            Self::D13GlobalPerspective => 13,
            Self::D14StrategicCommunication => 14,
            Self::D15InnovationLeadership => 15,
        }
    }

    /// Get the group number (1-5).
    #[must_use]
    pub const fn group(&self) -> u8 {
        match self.number() {
            1..=3 => 1,   // Foundational
            4..=6 => 2,   // Technical Operations
            7..=9 => 3,   // Information and Analysis
            10..=12 => 4, // Strategic Integration
            13..=15 => 5, // Leadership and Innovation
            _ => 0,       // Should never happen
        }
    }

    /// Get the group name.
    #[must_use]
    pub const fn group_name(&self) -> &'static str {
        match self.group() {
            1 => "Foundational Domains",
            2 => "Technical Operations Domains",
            3 => "Information and Analysis Domains",
            4 => "Strategic Integration Domains",
            5 => "Leadership and Innovation Domains",
            _ => "Unknown",
        }
    }

    /// Get display string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::D1ProfessionalFoundations => "D1: Professional Foundations and Evolution",
            Self::D2TechnicalExpertise => "D2: Technical Subject Matter Expertise",
            Self::D3PatternRecognition => "D3: Pattern Recognition and Critical Classification",
            Self::D4InformationProcessing => {
                "D4: Information Processing and Documentation Excellence"
            }
            Self::D5ControlledOperations => "D5: Controlled Environment Operations",
            Self::D6QualitySystems => "D6: Quality Systems and Error Prevention",
            Self::D7InformationSystems => "D7: Information Systems and Advanced Analytics",
            Self::D8PatternDetection => "D8: Pattern Detection and Advanced Validation",
            Self::D9ComplexSystems => "D9: Complex Systems Analysis and Modeling",
            Self::D10DecisionScience => "D10: Decision Science and Trade-off Optimization",
            Self::D11StrategicImplementation => "D11: Strategic Implementation and Risk Excellence",
            Self::D12RegulatoryNavigation => "D12: Regulatory Navigation and Compliance Excellence",
            Self::D13GlobalPerspective => "D13: Global Perspective and Cultural Fluency",
            Self::D14StrategicCommunication => "D14: Strategic Communication and Influence",
            Self::D15InnovationLeadership => "D15: Innovation Leadership and Transformation",
        }
    }
}

impl std::fmt::Display for DomainCategory {
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
    /// Requirement type: "primary" or "supporting"
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
        assert_eq!(DomainCategory::D1ProfessionalFoundations.number(), 1);
        assert_eq!(DomainCategory::D15InnovationLeadership.number(), 15);
    }

    #[test]
    fn test_domain_group() {
        assert_eq!(DomainCategory::D1ProfessionalFoundations.group(), 1);
        assert_eq!(DomainCategory::D6QualitySystems.group(), 2);
        assert_eq!(DomainCategory::D9ComplexSystems.group(), 3);
        assert_eq!(DomainCategory::D12RegulatoryNavigation.group(), 4);
        assert_eq!(DomainCategory::D15InnovationLeadership.group(), 5);
    }

    #[test]
    fn test_domain_requirement_met() {
        let req = DomainRequirement::primary(
            DomainCategory::D1ProfessionalFoundations,
            ProficiencyLevel::L3Competent,
        );

        assert!(req.is_met_by(ProficiencyLevel::L3Competent));
        assert!(req.is_met_by(ProficiencyLevel::L5Expert));
        assert!(!req.is_met_by(ProficiencyLevel::L2AdvancedBeginner));
    }
}
