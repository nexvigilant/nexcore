//! EPA (Entrustable Professional Activity) Types
//!
//! Migrated from Python `domains/regulatory/caba/caba/models/epa.py`.
//!
//! ## EPA Overview
//!
//! An EPA is a complex, multi-step activity requiring coordination of multiple
//! core competencies to achieve a high-level goal (e.g., "Implement HIPAA
//! Technical Safeguards §164.312").
//!
//! ## Components
//!
//! - [`EPACategory`] - 14 compliance framework categories
//! - [`EPAExecutionStatus`] - Overall EPA execution status
//! - [`CompetencyDeploymentStatus`] - Individual competency deployment status
//! - [`EPARequirement`] - Specification for an EPA
//! - [`EPAValidationResult`] - EPA goal validation results
//! - [`CompetencyDeploymentStep`] - Single competency deployment within EPA
//! - [`EPAExecutionPlan`] - Detailed plan for EPA execution
//! - [`EPAExecutionState`] - Real-time execution state tracking

use crate::caba::Score;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// EPA domain classifications aligned with compliance frameworks.
///
/// # L0 Quark - Category enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EPACategory {
    /// HIPAA Technical Safeguards (§164.312)
    #[serde(rename = "HIPAA Technical Safeguards")]
    HipaaTechnicalSafeguards,
    /// HIPAA Administrative Safeguards (§164.308)
    #[serde(rename = "HIPAA Administrative Safeguards")]
    HipaaAdministrativeSafeguards,
    /// HIPAA Physical Safeguards (§164.310)
    #[serde(rename = "HIPAA Physical Safeguards")]
    HipaaPhysicalSafeguards,
    /// SOC2 Security Controls
    #[serde(rename = "SOC2 Security Controls")]
    Soc2Security,
    /// SOC2 Availability Controls
    #[serde(rename = "SOC2 Availability Controls")]
    Soc2Availability,
    /// SOC2 Confidentiality Controls
    #[serde(rename = "SOC2 Confidentiality Controls")]
    Soc2Confidentiality,
    /// GDPR Compliance Controls
    #[serde(rename = "GDPR Compliance Controls")]
    GdprCompliance,
    /// GDPR Data Protection
    #[serde(rename = "GDPR Data Protection")]
    GdprDataProtection,
    /// ISO 27001 Information Security
    #[serde(rename = "ISO 27001 Information Security")]
    Iso27001InformationSecurity,
    /// NIST Cybersecurity Framework
    #[serde(rename = "NIST Cybersecurity Framework")]
    NistCybersecurity,
    /// PCI DSS Compliance
    #[serde(rename = "PCI DSS Compliance")]
    PciDssCompliance,
    /// General Security Implementation
    #[serde(rename = "General Security Implementation")]
    GeneralSecurity,
    /// Business Continuity & Disaster Recovery
    #[serde(rename = "Business Continuity & Disaster Recovery")]
    BusinessContinuity,
    /// Incident Response & Management
    #[serde(rename = "Incident Response & Management")]
    IncidentResponse,
}

impl EPACategory {
    /// Get display string for the category.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::HipaaTechnicalSafeguards => "HIPAA Technical Safeguards",
            Self::HipaaAdministrativeSafeguards => "HIPAA Administrative Safeguards",
            Self::HipaaPhysicalSafeguards => "HIPAA Physical Safeguards",
            Self::Soc2Security => "SOC2 Security Controls",
            Self::Soc2Availability => "SOC2 Availability Controls",
            Self::Soc2Confidentiality => "SOC2 Confidentiality Controls",
            Self::GdprCompliance => "GDPR Compliance Controls",
            Self::GdprDataProtection => "GDPR Data Protection",
            Self::Iso27001InformationSecurity => "ISO 27001 Information Security",
            Self::NistCybersecurity => "NIST Cybersecurity Framework",
            Self::PciDssCompliance => "PCI DSS Compliance",
            Self::GeneralSecurity => "General Security Implementation",
            Self::BusinessContinuity => "Business Continuity & Disaster Recovery",
            Self::IncidentResponse => "Incident Response & Management",
        }
    }

    /// Get the compliance framework this category belongs to.
    #[must_use]
    pub const fn framework(&self) -> &'static str {
        match self {
            Self::HipaaTechnicalSafeguards
            | Self::HipaaAdministrativeSafeguards
            | Self::HipaaPhysicalSafeguards => "HIPAA",
            Self::Soc2Security | Self::Soc2Availability | Self::Soc2Confidentiality => "SOC2",
            Self::GdprCompliance | Self::GdprDataProtection => "GDPR",
            Self::Iso27001InformationSecurity => "ISO27001",
            Self::NistCybersecurity => "NIST",
            Self::PciDssCompliance => "PCI-DSS",
            Self::GeneralSecurity | Self::BusinessContinuity | Self::IncidentResponse => "General",
        }
    }
}

impl std::fmt::Display for EPACategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Status of individual competency deployment within EPA.
///
/// Tracks lifecycle of each competency as EPA orchestrates deployment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CompetencyDeploymentStatus {
    /// Not started yet
    #[default]
    Pending,
    /// Checking prerequisites
    Validating,
    /// Deployment in progress
    Deploying,
    /// Configuring cross-competency access
    Integrating,
    /// Successfully deployed
    Completed,
    /// Deployment failed
    Failed,
    /// Skipped due to dependencies or conditions
    Skipped,
    /// Reversing deployment
    RollingBack,
}

impl CompetencyDeploymentStatus {
    /// Check if this status represents a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Failed | Self::Skipped)
    }

    /// Check if this status represents an active state.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        matches!(
            self,
            Self::Validating | Self::Deploying | Self::Integrating | Self::RollingBack
        )
    }
}

/// Overall EPA execution status.
///
/// Represents high-level state of entire EPA orchestration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EPAExecutionStatus {
    /// EPA created but not executed
    #[default]
    NotStarted,
    /// Analyzing requirements, resolving dependencies
    Planning,
    /// Deploying competencies
    Deploying,
    /// Configuring cross-competency communication
    Integrating,
    /// Verifying goal achievement
    Validating,
    /// Successfully completed
    Completed,
    /// Execution failed
    Failed,
    /// Some competencies deployed, some failed
    PartiallyDeployed,
}

impl EPAExecutionStatus {
    /// Check if this status represents a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::PartiallyDeployed
        )
    }

    /// Check if this status represents success.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// Priority level for EPA requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    /// Low priority
    Low,
    /// Medium priority (default)
    #[default]
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

impl Priority {
    /// Get numeric value for sorting.
    #[must_use]
    pub const fn numeric_value(&self) -> u8 {
        match self {
            Self::Low => 1,
            Self::Medium => 2,
            Self::High => 3,
            Self::Critical => 4,
        }
    }
}

impl std::cmp::PartialOrd for Priority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl std::cmp::Ord for Priority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.numeric_value().cmp(&other.numeric_value())
    }
}

/// Specification for an EPA to be orchestrated.
///
/// Defines the high-level goal and which competencies are needed to achieve it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPARequirement {
    /// EPA title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// EPA category
    pub category: EPACategory,

    /// High-level objective to achieve
    pub goal: String,
    /// How to know goal is achieved
    pub success_criteria: Vec<String>,

    /// Required competency IDs or descriptions
    pub required_competencies: Vec<String>,

    /// Explicit ordering if needed (competency IDs)
    #[serde(default)]
    pub deployment_order: Option<Vec<String>>,
    /// Cross-competency configuration
    #[serde(default)]
    pub integration_requirements: serde_json::Value,

    /// Post-deployment tests
    #[serde(default)]
    pub validation_checks: Vec<serde_json::Value>,

    /// Compliance framework (e.g., "HIPAA", "SOC2", "GDPR")
    #[serde(default)]
    pub compliance_framework: Option<String>,
    /// Specific regulation sections
    #[serde(default)]
    pub regulatory_citations: Vec<String>,

    /// Priority level
    #[serde(default)]
    pub priority: Priority,
    /// Estimated duration in hours
    #[serde(default)]
    pub estimated_duration_hours: Option<f64>,
    /// Estimated cost in USD
    #[serde(default)]
    pub estimated_cost_usd: Option<f64>,
}

impl EPARequirement {
    /// Create a new EPA requirement.
    #[must_use]
    pub fn new(
        title: String,
        description: String,
        category: EPACategory,
        goal: String,
        success_criteria: Vec<String>,
        required_competencies: Vec<String>,
    ) -> Self {
        Self {
            title,
            description,
            category,
            goal,
            success_criteria,
            required_competencies,
            deployment_order: None,
            integration_requirements: serde_json::Value::Object(serde_json::Map::new()),
            validation_checks: Vec::new(),
            compliance_framework: Some(category.framework().to_string()),
            regulatory_citations: Vec::new(),
            priority: Priority::default(),
            estimated_duration_hours: None,
            estimated_cost_usd: None,
        }
    }
}

/// Results of EPA goal validation.
///
/// Verifies that the orchestrated competencies achieve the intended goal.
/// Produces compliance evidence and attestation documentation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPAValidationResult {
    /// EPA ID
    pub epa_id: String,
    /// Whether goal was achieved
    pub goal_achieved: bool,
    /// Validation score [0.0, 1.0]
    pub validation_score: Score,

    /// Success criteria evaluation (criterion → passed/failed)
    pub criteria_results: std::collections::HashMap<String, bool>,

    /// Detailed validation check results
    #[serde(default)]
    pub validation_checks: Vec<serde_json::Value>,

    /// Formal attestation statement
    #[serde(default)]
    pub compliance_attestation: Option<String>,
    /// Supporting evidence
    #[serde(default)]
    pub evidence_package: serde_json::Value,

    /// Errors found
    #[serde(default)]
    pub errors: Vec<String>,
    /// Warnings found
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Recommendations for improvement
    #[serde(default)]
    pub recommendations: Vec<String>,

    /// Validation timestamp (ISO 8601)
    pub validated_at: String,

    /// How reliable is the deployment [0.0, 1.0]
    #[serde(default)]
    pub reliability_score: Score,
    /// How complete is the implementation [0.0, 1.0]
    #[serde(default)]
    pub completeness_score: Score,
    /// How well does it meet compliance requirements [0.0, 1.0]
    #[serde(default)]
    pub compliance_score: Score,
}

impl EPAValidationResult {
    /// Create a new validation result.
    ///
    /// # Errors
    /// Returns error if validation_score is not in [0.0, 1.0].
    pub fn new(
        epa_id: String,
        goal_achieved: bool,
        validation_score: f64,
        validated_at: String,
    ) -> Result<Self, crate::caba::ScoreError> {
        Ok(Self {
            epa_id,
            goal_achieved,
            validation_score: Score::new(validation_score)?,
            criteria_results: std::collections::HashMap::new(),
            validation_checks: Vec::new(),
            compliance_attestation: None,
            evidence_package: serde_json::Value::Object(serde_json::Map::new()),
            errors: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
            validated_at,
            reliability_score: Score::ZERO,
            completeness_score: Score::ZERO,
            compliance_score: Score::ZERO,
        })
    }

    /// Calculate percentage of criteria met.
    #[must_use]
    pub fn criteria_met_percentage(&self) -> f64 {
        if self.criteria_results.is_empty() {
            return 0.0;
        }
        let met_count = self.criteria_results.values().filter(|&&v| v).count();
        met_count as f64 / self.criteria_results.len() as f64
    }

    /// Add a criterion result.
    pub fn add_criterion_result(&mut self, criterion: String, passed: bool) {
        self.criteria_results.insert(criterion, passed);
    }

    /// Check if validation passed with no errors.
    #[must_use]
    pub fn is_successful(&self) -> bool {
        self.goal_achieved && self.errors.is_empty()
    }
}

// =============================================================================
// Execution Types (type definitions - services migrate separately)
// =============================================================================

/// Represents deployment of a single competency within an EPA.
///
/// Tracks status, timing, outputs, and errors for one competency
/// as part of EPA orchestration.
///
/// # L2 Molecule - Competency deployment tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetencyDeploymentStep {
    /// Competency identifier
    pub competency_id: String,
    /// Human-readable title
    pub competency_title: String,
    /// Current deployment status
    pub status: CompetencyDeploymentStatus,

    // Execution timeline
    /// When deployment started (ISO 8601)
    #[serde(default)]
    pub started_at: Option<DateTime>,
    /// When deployment completed (ISO 8601)
    #[serde(default)]
    pub completed_at: Option<DateTime>,
    /// Total duration in seconds
    #[serde(default)]
    pub duration_seconds: Option<f64>,

    // Results (note: terraform_outputs excluded per SOP - that's infrastructure)
    /// Errors encountered during deployment
    #[serde(default)]
    pub deployment_errors: Vec<String>,
    /// Warnings encountered during deployment
    #[serde(default)]
    pub deployment_warnings: Vec<String>,

    // Dependencies
    /// Other competency IDs this step depends on
    #[serde(default)]
    pub depends_on: Vec<String>,

    // Progress
    /// Current progress percentage [0.0, 100.0]
    #[serde(default)]
    pub progress_percent: f64,
    /// Current operation description (e.g., "Running terraform apply")
    #[serde(default)]
    pub current_operation: Option<String>,
}

impl CompetencyDeploymentStep {
    /// Create a new deployment step in pending state.
    #[must_use]
    pub fn new(competency_id: String, competency_title: String) -> Self {
        Self {
            competency_id,
            competency_title,
            status: CompetencyDeploymentStatus::Pending,
            started_at: None,
            completed_at: None,
            duration_seconds: None,
            deployment_errors: Vec::new(),
            deployment_warnings: Vec::new(),
            depends_on: Vec::new(),
            progress_percent: 0.0,
            current_operation: None,
        }
    }

    /// Check if this step has any errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.deployment_errors.is_empty()
    }

    /// Check if this step has any warnings.
    #[must_use]
    pub fn has_warnings(&self) -> bool {
        !self.deployment_warnings.is_empty()
    }
}

/// Detailed plan for EPA execution.
///
/// Generated by analyzing requirements and resolving dependencies.
/// Defines exact order of competency deployment and integration points.
///
/// # L2 Molecule - EPA execution planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPAExecutionPlan {
    /// EPA identifier
    pub epa_id: String,
    /// EPA title
    pub epa_title: String,

    /// Deployment steps in execution order
    pub deployment_steps: Vec<CompetencyDeploymentStep>,

    /// Cross-competency integration points
    #[serde(default)]
    pub integration_points: Vec<serde_json::Value>,

    // Resource estimates
    /// Estimated total duration in minutes
    #[serde(default)]
    pub estimated_duration_minutes: Option<u32>,
    /// Estimated cost in USD
    #[serde(default)]
    pub estimated_cost_usd: Option<f64>,

    // Risk assessment
    /// Identified risks and mitigations
    #[serde(default)]
    pub risks: Vec<String>,
    /// Required pre-conditions
    #[serde(default)]
    pub preconditions: Vec<String>,

    /// Plan creation timestamp
    #[serde(default)]
    pub created_at: Option<DateTime>,
}

impl EPAExecutionPlan {
    /// Create a new execution plan.
    #[must_use]
    pub fn new(
        epa_id: String,
        epa_title: String,
        deployment_steps: Vec<CompetencyDeploymentStep>,
    ) -> Self {
        Self {
            epa_id,
            epa_title,
            deployment_steps,
            integration_points: Vec::new(),
            estimated_duration_minutes: None,
            estimated_cost_usd: None,
            risks: Vec::new(),
            preconditions: Vec::new(),
            created_at: Some(DateTime::now()),
        }
    }

    /// Get total number of deployment steps.
    #[must_use]
    pub fn total_steps(&self) -> usize {
        self.deployment_steps.len()
    }

    /// Get all competency IDs in deployment order.
    #[must_use]
    pub fn competency_ids(&self) -> Vec<&str> {
        self.deployment_steps
            .iter()
            .map(|s| s.competency_id.as_str())
            .collect()
    }
}

/// Real-time state of EPA execution.
///
/// Tracks progress, maintains state, enables recovery from failures.
/// Type definition only - persistence services migrate separately.
///
/// # L2 Molecule - EPA execution state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPAExecutionState {
    /// EPA identifier
    pub epa_id: String,
    /// Overall execution status
    pub status: EPAExecutionStatus,

    // Progress tracking
    /// Current step description
    #[serde(default)]
    pub current_step: Option<String>,
    /// Number of steps completed
    #[serde(default)]
    pub steps_completed: u32,
    /// Total number of steps
    #[serde(default)]
    pub total_steps: u32,
    /// Completion percentage [0.0, 100.0]
    #[serde(default)]
    pub percent_complete: f64,

    // Execution timeline
    /// When execution started
    #[serde(default)]
    pub started_at: Option<DateTime>,
    /// When state was last updated
    #[serde(default)]
    pub last_updated: Option<DateTime>,
    /// When execution completed
    #[serde(default)]
    pub completed_at: Option<DateTime>,

    // Competency-level tracking
    /// Status of each competency deployment (competency_id → status)
    #[serde(default)]
    pub competency_states: std::collections::HashMap<String, CompetencyDeploymentStatus>,

    // Issues
    /// Errors encountered during execution
    #[serde(default)]
    pub errors: Vec<String>,
    /// Warnings encountered during execution
    #[serde(default)]
    pub warnings: Vec<String>,

    /// Elapsed time in seconds
    #[serde(default)]
    pub elapsed_seconds: Option<f64>,
}

impl EPAExecutionState {
    /// Create a new execution state in NotStarted status.
    #[must_use]
    pub fn new(epa_id: String, total_steps: u32) -> Self {
        Self {
            epa_id,
            status: EPAExecutionStatus::NotStarted,
            current_step: None,
            steps_completed: 0,
            total_steps,
            percent_complete: 0.0,
            started_at: None,
            last_updated: Some(DateTime::now()),
            completed_at: None,
            competency_states: std::collections::HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            elapsed_seconds: None,
        }
    }

    /// Check if execution has any errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get count of completed competencies.
    #[must_use]
    pub fn completed_competency_count(&self) -> usize {
        self.competency_states
            .values()
            .filter(|s| **s == CompetencyDeploymentStatus::Completed)
            .count()
    }

    /// Get count of failed competencies.
    #[must_use]
    pub fn failed_competency_count(&self) -> usize {
        self.competency_states
            .values()
            .filter(|s| **s == CompetencyDeploymentStatus::Failed)
            .count()
    }

    /// Check if execution is in a terminal state.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_epa_category_framework() {
        assert_eq!(EPACategory::HipaaTechnicalSafeguards.framework(), "HIPAA");
        assert_eq!(EPACategory::Soc2Security.framework(), "SOC2");
        assert_eq!(EPACategory::GdprCompliance.framework(), "GDPR");
        assert_eq!(EPACategory::NistCybersecurity.framework(), "NIST");
    }

    #[test]
    fn test_deployment_status_terminal() {
        assert!(CompetencyDeploymentStatus::Completed.is_terminal());
        assert!(CompetencyDeploymentStatus::Failed.is_terminal());
        assert!(!CompetencyDeploymentStatus::Pending.is_terminal());
        assert!(!CompetencyDeploymentStatus::Deploying.is_terminal());
    }

    #[test]
    fn test_execution_status_success() {
        assert!(EPAExecutionStatus::Completed.is_success());
        assert!(!EPAExecutionStatus::Failed.is_success());
        assert!(!EPAExecutionStatus::PartiallyDeployed.is_success());
    }

    #[test]
    fn test_priority_ordering() {
        assert!(Priority::Low < Priority::Medium);
        assert!(Priority::Medium < Priority::High);
        assert!(Priority::High < Priority::Critical);
    }

    #[test]
    fn test_epa_validation_result() {
        let result = EPAValidationResult::new(
            "EPA-1".to_string(),
            true,
            0.95,
            "2026-01-29T00:00:00Z".to_string(),
        );
        assert!(result.is_ok());

        if let Ok(mut result) = result {
            result.add_criterion_result("Auth implemented".to_string(), true);
            result.add_criterion_result("Encryption configured".to_string(), true);
            result.add_criterion_result("Audit logging".to_string(), false);

            assert!((result.criteria_met_percentage() - 0.666_666).abs() < 0.001);
        }
    }

    #[test]
    fn test_competency_deployment_step() {
        let step =
            CompetencyDeploymentStep::new("CC-AUTH-001".to_string(), "Authentication".to_string());

        assert_eq!(step.status, CompetencyDeploymentStatus::Pending);
        assert!(!step.has_errors());
        assert!(!step.has_warnings());
        assert_eq!(step.progress_percent, 0.0);
    }

    #[test]
    fn test_competency_deployment_step_with_errors() {
        let mut step =
            CompetencyDeploymentStep::new("CC-AUTH-001".to_string(), "Authentication".to_string());
        step.deployment_errors.push("Connection failed".to_string());
        step.deployment_warnings
            .push("Retry recommended".to_string());

        assert!(step.has_errors());
        assert!(step.has_warnings());
    }

    #[test]
    fn test_epa_execution_plan() {
        let steps = vec![
            CompetencyDeploymentStep::new("CC-AUTH-001".to_string(), "Authentication".to_string()),
            CompetencyDeploymentStep::new("CC-ENCRYPT-001".to_string(), "Encryption".to_string()),
        ];

        let plan = EPAExecutionPlan::new(
            "EPA-HIPAA-001".to_string(),
            "HIPAA Technical Safeguards".to_string(),
            steps,
        );

        assert_eq!(plan.total_steps(), 2);
        assert_eq!(plan.competency_ids(), vec!["CC-AUTH-001", "CC-ENCRYPT-001"]);
    }

    #[test]
    fn test_epa_execution_state() {
        let state = EPAExecutionState::new("EPA-HIPAA-001".to_string(), 3);

        assert_eq!(state.status, EPAExecutionStatus::NotStarted);
        assert_eq!(state.steps_completed, 0);
        assert_eq!(state.total_steps, 3);
        assert!(!state.is_terminal());
        assert!(!state.has_errors());
    }

    #[test]
    fn test_epa_execution_state_competency_counts() {
        let mut state = EPAExecutionState::new("EPA-HIPAA-001".to_string(), 3);

        state.competency_states.insert(
            "CC-AUTH-001".to_string(),
            CompetencyDeploymentStatus::Completed,
        );
        state.competency_states.insert(
            "CC-ENCRYPT-001".to_string(),
            CompetencyDeploymentStatus::Completed,
        );
        state.competency_states.insert(
            "CC-AUDIT-001".to_string(),
            CompetencyDeploymentStatus::Failed,
        );

        assert_eq!(state.completed_competency_count(), 2);
        assert_eq!(state.failed_competency_count(), 1);
    }
}
