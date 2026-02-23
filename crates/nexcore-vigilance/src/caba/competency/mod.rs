//! Competency Domain Types
//!
//! Migrated from Python `domains/regulatory/caba/caba/models/competency.py`.
//!
//! ## Competency Hierarchy
//!
//! KSB (atomic) → Core Competency (integrated) → EPA (process) → CPA (complete business)
//!
//! ## Components
//!
//! - [`CompetencyCategory`] - 10 competency domain classifications
//! - [`SimpleProficiencyLevel`] - 4-level proficiency (novice → expert)
//! - [`CompetencyRequirement`] - Specification for a required competency
//! - [`IntegrationModel`] - How KSBs integrate into working competency
//! - [`ValidationResult`] - Competency validation results
//! - [`CoreCompetency`] - Complete, validated competency

use serde::{Deserialize, Serialize};

/// Competency domain classifications.
///
/// # L0 Quark - Category enumeration
///
/// These represent the 10 major technical domains for competencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CompetencyCategory {
    /// Authentication & Authorization
    #[serde(rename = "Authentication & Authorization")]
    Authentication,
    /// Data Storage & Retrieval
    #[serde(rename = "Data Storage & Retrieval")]
    DataManagement,
    /// API Integration & Communication
    #[serde(rename = "API Integration & Communication")]
    ApiIntegration,
    /// Payment & Billing
    #[serde(rename = "Payment & Billing")]
    PaymentProcessing,
    /// Email & Messaging
    #[serde(rename = "Email & Messaging")]
    EmailCommunication,
    /// File Storage & Management
    #[serde(rename = "File Storage & Management")]
    FileStorage,
    /// Analytics & Metrics
    #[serde(rename = "Analytics & Metrics")]
    Analytics,
    /// Monitoring & Observability
    #[serde(rename = "Monitoring & Observability")]
    Monitoring,
    /// Security & Compliance
    #[serde(rename = "Security & Compliance")]
    Security,
    /// Workflow Orchestration
    #[serde(rename = "Workflow Orchestration")]
    Workflow,
}

impl CompetencyCategory {
    /// Get display string for the category.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Authentication => "Authentication & Authorization",
            Self::DataManagement => "Data Storage & Retrieval",
            Self::ApiIntegration => "API Integration & Communication",
            Self::PaymentProcessing => "Payment & Billing",
            Self::EmailCommunication => "Email & Messaging",
            Self::FileStorage => "File Storage & Management",
            Self::Analytics => "Analytics & Metrics",
            Self::Monitoring => "Monitoring & Observability",
            Self::Security => "Security & Compliance",
            Self::Workflow => "Workflow Orchestration",
        }
    }
}

impl std::fmt::Display for CompetencyCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Simple 4-level proficiency model for competencies.
///
/// # L0 Quark - Simple proficiency levels
///
/// Note: This is a simplified model. For full PDC 7-level model, use
/// [`crate::proficiency::ProficiencyLevel`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum SimpleProficiencyLevel {
    /// Basic understanding, can use with guidance
    Novice,
    /// Solid understanding, can use independently
    #[default]
    Competent,
    /// Deep understanding, can optimize and adapt
    Proficient,
    /// Mastery, can design and teach
    Expert,
}

impl SimpleProficiencyLevel {
    /// Get numeric value for comparisons.
    #[must_use]
    pub const fn numeric_value(&self) -> u8 {
        match self {
            Self::Novice => 1,
            Self::Competent => 2,
            Self::Proficient => 3,
            Self::Expert => 4,
        }
    }

    /// Get display string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Novice => "novice",
            Self::Competent => "competent",
            Self::Proficient => "proficient",
            Self::Expert => "expert",
        }
    }
}

impl std::cmp::PartialOrd for SimpleProficiencyLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for SimpleProficiencyLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.numeric_value().cmp(&other.numeric_value())
    }
}

impl std::fmt::Display for SimpleProficiencyLevel {
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

    // KSB Requirements (can be IDs or natural language descriptions)
    /// Required knowledge items
    pub required_knowledge: Vec<String>,
    /// Required skill items
    pub required_skills: Vec<String>,
    /// Required behavior items
    pub required_behaviors: Vec<String>,

    /// Use case description
    pub use_case: String,
    /// Constraints on implementation
    #[serde(default)]
    pub constraints: serde_json::Value,
    /// Target proficiency level
    #[serde(default)]
    pub target_proficiency: SimpleProficiencyLevel,

    /// Runtime requirements
    #[serde(default)]
    pub runtime_requirements: serde_json::Value,
    /// Infrastructure needs
    #[serde(default)]
    pub infrastructure_needs: Vec<String>,
    /// Integration points
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
    target_proficiency: SimpleProficiencyLevel,
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
    pub fn target_proficiency(mut self, level: SimpleProficiencyLevel) -> Self {
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
            constraints: serde_json::Value::Object(serde_json::Map::new()),
            target_proficiency: self.target_proficiency,
            runtime_requirements: serde_json::Value::Object(serde_json::Map::new()),
            infrastructure_needs: Vec::new(),
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
    /// How knowledge is applied
    pub knowledge_integration: std::collections::HashMap<String, String>,
    /// How skills are executed
    pub skill_integration: std::collections::HashMap<String, String>,
    /// How behaviors are demonstrated
    pub behavior_integration: std::collections::HashMap<String, String>,
    /// Module structure
    pub module_structure: serde_json::Value,
    /// External dependencies
    #[serde(default)]
    pub external_dependencies: Vec<String>,
    /// Internal dependencies
    #[serde(default)]
    pub internal_dependencies: Vec<String>,
    /// Infrastructure components
    #[serde(default)]
    pub infrastructure_components: Vec<serde_json::Value>,
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
    /// Integration check results
    #[serde(default)]
    pub integration_check: serde_json::Value,
    /// Code quality check results
    #[serde(default)]
    pub code_quality_check: serde_json::Value,
    /// Test coverage check results
    #[serde(default)]
    pub test_coverage_check: std::collections::HashMap<String, f64>,

    /// Errors found
    #[serde(default)]
    pub errors: Vec<String>,
    /// Warnings found
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Suggestions for improvement
    #[serde(default)]
    pub suggestions: Vec<String>,

    /// Automation score [0.0, 1.0]
    #[serde(default)]
    pub automation_score: f64,
    /// Reliability score [0.0, 1.0]
    #[serde(default)]
    pub reliability_score: f64,
    /// Maintainability score [0.0, 1.0]
    #[serde(default)]
    pub maintainability_score: f64,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            valid: true,
            validation_score: 0.0,
            completeness_check: std::collections::HashMap::new(),
            dependency_check: std::collections::HashMap::new(),
            integration_check: serde_json::Value::Object(serde_json::Map::new()),
            code_quality_check: serde_json::Value::Object(serde_json::Map::new()),
            test_coverage_check: std::collections::HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
            automation_score: 0.0,
            reliability_score: 0.0,
            maintainability_score: 0.0,
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
/// Output of Competency Composer - fully functional capability module.
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

    /// Proficiency definitions per level
    #[serde(default)]
    pub proficiency_definitions: std::collections::HashMap<String, serde_json::Value>,
    /// Target proficiency level
    #[serde(default)]
    pub target_proficiency: SimpleProficiencyLevel,

    /// Runtime requirements
    #[serde(default)]
    pub runtime_requirements: serde_json::Value,
    /// Infrastructure requirements
    #[serde(default)]
    pub infrastructure_requirements: Vec<serde_json::Value>,
    /// External integrations
    #[serde(default)]
    pub external_integrations: Vec<String>,

    /// Automation potential [0.0, 1.0]
    #[serde(default)]
    pub automation_potential: f64,
    /// Reliability score [0.0, 1.0]
    #[serde(default)]
    pub reliability_score: f64,
    /// Test coverage [0.0, 1.0]
    #[serde(default)]
    pub test_coverage: f64,
    /// Code quality score [0.0, 1.0]
    #[serde(default)]
    pub code_quality_score: f64,

    /// Creation timestamp (ISO 8601)
    pub created_at: String,
    /// Last validation timestamp (ISO 8601)
    pub last_validated: String,
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
            CompetencyCategory::Authentication.as_str(),
            "Authentication & Authorization"
        );
        assert_eq!(
            CompetencyCategory::Security.as_str(),
            "Security & Compliance"
        );
    }

    #[test]
    fn test_simple_proficiency_ordering() {
        assert!(SimpleProficiencyLevel::Novice < SimpleProficiencyLevel::Competent);
        assert!(SimpleProficiencyLevel::Competent < SimpleProficiencyLevel::Proficient);
        assert!(SimpleProficiencyLevel::Proficient < SimpleProficiencyLevel::Expert);
    }

    #[test]
    fn test_competency_requirement_validation() {
        let result = CompetencyRequirement::builder()
            .title("Test")
            .description("Description")
            .category(CompetencyCategory::Authentication)
            .domain("Auth")
            .knowledge("K1")
            .skill("S1")
            .behavior("B1")
            .use_case("Use case")
            .build();
        assert!(result.is_ok());

        // Test missing knowledge
        let result = CompetencyRequirement::builder()
            .title("Test")
            .description("Description")
            .category(CompetencyCategory::Authentication)
            .domain("Auth")
            // No knowledge added
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
