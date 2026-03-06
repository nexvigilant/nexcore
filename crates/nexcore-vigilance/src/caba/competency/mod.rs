//! PV Competency Domain Types
//!
//! Core competencies integrate multiple KSBs into working PV capabilities.
//! (source: ~/Vaults/nexvigilant/400-projects/ksb-framework/)
//!
//! ## Competency Hierarchy
//!
//! KSB (atomic) -> Core Competency (integrated) -> EPA (process) -> CPA (complete business)
//!
//! ## Components
//!
//! - [`CompetencyCategory`] - 8 PV competency domain clusters
//! - [`CompetencyRequirement`] - Specification for a required competency
//! - [`IntegrationModel`] - How KSBs integrate into working competency
//! - [`ValidationResult`] - Competency validation results
//! - [`CoreCompetency`] - Complete, validated competency

use crate::caba::domain::DomainCluster;
use crate::caba::proficiency::ProficiencyLevel;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// PV competency domain classifications.
///
/// These align with the DomainCluster groupings from the KSB framework,
/// representing the major functional areas of PV competency.
/// (source: 04-ksb-competency-framework.md)
///
/// # L0 Quark - Category enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompetencyCategory {
    /// PV foundations, clinical pharmacology, medical terminology (D01-D03)
    #[serde(rename = "Foundational PV Science")]
    FoundationalScience,
    /// ICSR processing, signal detection, risk assessment (D04-D06)
    #[serde(rename = "Core PV Operations")]
    CoreOperations,
    /// Regulatory intelligence, PV systems & technology (D07-D08)
    #[serde(rename = "Regulatory & Systems")]
    RegulatorySystems,
    /// Quality management in PV (D09)
    #[serde(rename = "Quality Management")]
    QualityManagement,
    /// Special populations, global PV operations (D10-D11)
    #[serde(rename = "Specialized PV")]
    SpecializedPv,
    /// PV program management & strategy (D12)
    #[serde(rename = "PV Program Management")]
    ProgramManagement,
    /// Advanced analytics & data science (D13)
    #[serde(rename = "Advanced Analytics")]
    AdvancedAnalytics,
    /// Communication, professional development (D14-D15)
    #[serde(rename = "Communication & Development")]
    CommunicationDevelopment,
}

impl CompetencyCategory {
    /// Get display string for the category.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::FoundationalScience => "Foundational PV Science",
            Self::CoreOperations => "Core PV Operations",
            Self::RegulatorySystems => "Regulatory & Systems",
            Self::QualityManagement => "Quality Management",
            Self::SpecializedPv => "Specialized PV",
            Self::ProgramManagement => "PV Program Management",
            Self::AdvancedAnalytics => "Advanced Analytics",
            Self::CommunicationDevelopment => "Communication & Development",
        }
    }

    /// Map to the corresponding DomainCluster.
    #[must_use]
    pub const fn to_domain_cluster(&self) -> DomainCluster {
        match self {
            Self::FoundationalScience => DomainCluster::Foundational,
            Self::CoreOperations => DomainCluster::CoreOperational,
            Self::RegulatorySystems => DomainCluster::Regulatory,
            Self::QualityManagement => DomainCluster::Quality,
            Self::SpecializedPv => DomainCluster::Specialized,
            Self::ProgramManagement => DomainCluster::Management,
            Self::AdvancedAnalytics => DomainCluster::Advanced,
            Self::CommunicationDevelopment => DomainCluster::CrossCutting,
        }
    }
}

impl From<DomainCluster> for CompetencyCategory {
    fn from(cluster: DomainCluster) -> Self {
        match cluster {
            DomainCluster::Foundational => Self::FoundationalScience,
            DomainCluster::CoreOperational => Self::CoreOperations,
            DomainCluster::Regulatory => Self::RegulatorySystems,
            DomainCluster::Quality => Self::QualityManagement,
            DomainCluster::Specialized => Self::SpecializedPv,
            DomainCluster::Management => Self::ProgramManagement,
            DomainCluster::Advanced => Self::AdvancedAnalytics,
            DomainCluster::CrossCutting => Self::CommunicationDevelopment,
        }
    }
}

impl std::fmt::Display for CompetencyCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Specification for a required competency.
///
/// Input to the Competency Composer - defines what competency needs to be built.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetencyRequirement {
    /// Competency title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Domain category
    pub category: CompetencyCategory,
    /// Domain context
    pub domain: String,

    /// Required knowledge items
    pub required_knowledge: Vec<String>,
    /// Required skill items
    pub required_skills: Vec<String>,
    /// Required behavior items
    pub required_behaviors: Vec<String>,

    /// Use case description
    pub use_case: String,
    /// Target proficiency level
    #[serde(default)]
    pub target_proficiency: ProficiencyLevel,

    /// Integration points with other competencies
    #[serde(default)]
    pub integration_points: Vec<String>,
}

/// Builder for creating `CompetencyRequirement`.
#[derive(Debug, Default)]
pub struct CompetencyRequirementBuilder {
    title: Option<String>,
    description: Option<String>,
    category: Option<CompetencyCategory>,
    domain: Option<String>,
    required_knowledge: Vec<String>,
    required_skills: Vec<String>,
    required_behaviors: Vec<String>,
    use_case: Option<String>,
    target_proficiency: ProficiencyLevel,
}

impl CompetencyRequirementBuilder {
    /// Create a new builder.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the title.
    #[must_use]
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the description.
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the category.
    #[must_use]
    pub fn category(mut self, category: CompetencyCategory) -> Self {
        self.category = Some(category);
        self
    }

    /// Set the domain.
    #[must_use]
    pub fn domain(mut self, domain: impl Into<String>) -> Self {
        self.domain = Some(domain.into());
        self
    }

    /// Add a knowledge requirement.
    #[must_use]
    pub fn knowledge(mut self, knowledge: impl Into<String>) -> Self {
        self.required_knowledge.push(knowledge.into());
        self
    }

    /// Add a skill requirement.
    #[must_use]
    pub fn skill(mut self, skill: impl Into<String>) -> Self {
        self.required_skills.push(skill.into());
        self
    }

    /// Add a behavior requirement.
    #[must_use]
    pub fn behavior(mut self, behavior: impl Into<String>) -> Self {
        self.required_behaviors.push(behavior.into());
        self
    }

    /// Set the use case.
    #[must_use]
    pub fn use_case(mut self, use_case: impl Into<String>) -> Self {
        self.use_case = Some(use_case.into());
        self
    }

    /// Set the target proficiency.
    #[must_use]
    pub fn target_proficiency(mut self, level: ProficiencyLevel) -> Self {
        self.target_proficiency = level;
        self
    }

    /// Build the `CompetencyRequirement`.
    ///
    /// # Errors
    /// Returns error if required fields are missing or KSB lists are empty.
    pub fn build(self) -> Result<CompetencyRequirement, CompetencyError> {
        let title = self
            .title
            .ok_or_else(|| CompetencyError::MissingRequirement("title is required".to_string()))?;
        let description = self.description.ok_or_else(|| {
            CompetencyError::MissingRequirement("description is required".to_string())
        })?;
        let category = self.category.ok_or_else(|| {
            CompetencyError::MissingRequirement("category is required".to_string())
        })?;
        let domain = self
            .domain
            .ok_or_else(|| CompetencyError::MissingRequirement("domain is required".to_string()))?;
        let use_case = self.use_case.ok_or_else(|| {
            CompetencyError::MissingRequirement("use_case is required".to_string())
        })?;

        if self.required_knowledge.is_empty() {
            return Err(CompetencyError::MissingRequirement(
                "At least one knowledge requirement needed".to_string(),
            ));
        }
        if self.required_skills.is_empty() {
            return Err(CompetencyError::MissingRequirement(
                "At least one skill requirement needed".to_string(),
            ));
        }
        if self.required_behaviors.is_empty() {
            return Err(CompetencyError::MissingRequirement(
                "At least one behavior requirement needed".to_string(),
            ));
        }

        Ok(CompetencyRequirement {
            title,
            description,
            category,
            domain,
            required_knowledge: self.required_knowledge,
            required_skills: self.required_skills,
            required_behaviors: self.required_behaviors,
            use_case,
            target_proficiency: self.target_proficiency,
            integration_points: Vec::new(),
        })
    }
}

impl CompetencyRequirement {
    /// Create a new builder for `CompetencyRequirement`.
    #[must_use]
    pub fn builder() -> CompetencyRequirementBuilder {
        CompetencyRequirementBuilder::new()
    }
}

/// Defines how KSBs integrate into working competency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationModel {
    /// How knowledge items are applied (KSB ID -> application description)
    pub knowledge_integration: std::collections::HashMap<String, String>,
    /// How skills are executed (KSB ID -> execution description)
    pub skill_integration: std::collections::HashMap<String, String>,
    /// How behaviors are demonstrated (KSB ID -> demonstration description)
    pub behavior_integration: std::collections::HashMap<String, String>,
    /// External dependencies
    #[serde(default)]
    pub external_dependencies: Vec<String>,
    /// Internal dependencies
    #[serde(default)]
    pub internal_dependencies: Vec<String>,
}

/// Competency validation results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Overall validity
    pub valid: bool,
    /// Validation score [0.0, 1.0]
    pub validation_score: f64,

    /// Completeness check results
    pub completeness_check: std::collections::HashMap<String, bool>,
    /// Dependency check results
    pub dependency_check: std::collections::HashMap<String, bool>,

    /// Errors found
    #[serde(default)]
    pub errors: Vec<String>,
    /// Warnings found
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Suggestions for improvement
    #[serde(default)]
    pub suggestions: Vec<String>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            valid: true,
            validation_score: 0.0,
            completeness_check: std::collections::HashMap::new(),
            dependency_check: std::collections::HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        }
    }
}

impl ValidationResult {
    /// Check if validation passed with no errors.
    #[must_use]
    pub fn is_successful(&self) -> bool {
        self.valid && self.errors.is_empty()
    }

    /// Add an error and mark as invalid.
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.valid = false;
    }

    /// Add a warning without affecting validity.
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// Complete, validated competency ready for deployment.
///
/// Output of Competency Composer - fully functional PV capability module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreCompetency {
    /// Unique identifier (format: CC-XXX-001)
    pub id: String,
    /// Competency title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Domain category
    pub category: CompetencyCategory,
    /// Domain context
    pub domain: String,
    /// Version string
    pub version: String,

    /// Required KSB IDs
    pub required_ksbs: Vec<String>,
    /// Integration model
    pub integration_model: IntegrationModel,

    /// Validation result
    pub validation_result: ValidationResult,

    /// Target proficiency level
    #[serde(default)]
    pub target_proficiency: ProficiencyLevel,

    /// External integrations
    #[serde(default)]
    pub external_integrations: Vec<String>,

    /// Creation timestamp
    pub created_at: DateTime,
    /// Last validation timestamp
    pub last_validated: DateTime,
    /// Creator identifier
    pub created_by: String,
}

/// Error type for competency operations.
#[derive(Debug, Clone, nexcore_error::Error)]
pub enum CompetencyError {
    /// Missing required KSB requirement
    #[error("Missing requirement: {0}")]
    MissingRequirement(String),

    /// Validation failed
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Invalid competency ID format
    #[error("Invalid competency ID format: {0}")]
    InvalidId(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_competency_category_display() {
        assert_eq!(
            CompetencyCategory::FoundationalScience.as_str(),
            "Foundational PV Science"
        );
        assert_eq!(
            CompetencyCategory::CoreOperations.as_str(),
            "Core PV Operations"
        );
    }

    #[test]
    fn test_category_domain_cluster_roundtrip() {
        let cat = CompetencyCategory::CoreOperations;
        let cluster = cat.to_domain_cluster();
        let back: CompetencyCategory = cluster.into();
        assert_eq!(cat, back);
    }

    #[test]
    fn test_competency_requirement_validation() {
        let result = CompetencyRequirement::builder()
            .title("Signal Detection Competency")
            .description("Ability to detect safety signals from PV data")
            .category(CompetencyCategory::CoreOperations)
            .domain("Signal Detection")
            .knowledge("PRR calculation methodology")
            .skill("Statistical signal analysis")
            .behavior("Timely escalation of confirmed signals")
            .use_case("Routine signal detection review")
            .build();
        assert!(result.is_ok());

        // Test missing knowledge
        let result = CompetencyRequirement::builder()
            .title("Test")
            .description("Description")
            .category(CompetencyCategory::FoundationalScience)
            .domain("PV Foundations")
            .skill("S1")
            .behavior("B1")
            .use_case("Use case")
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_validation_result_error_handling() {
        let mut result = ValidationResult::default();
        assert!(result.valid);
        assert!(result.is_successful());

        result.add_error("Test error".to_string());
        assert!(!result.valid);
        assert!(!result.is_successful());
    }
}
