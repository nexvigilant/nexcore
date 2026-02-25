//! CPA (Critical Practice Activity) Types
//!
//! Migrated from Python `domains/regulatory/caba/caba/models/cpa.py`.
//!
//! ## CPA Hierarchy
//!
//! KSB → Competency → EPA → **CPA** (complete business process)
//!
//! ## The 8 CPAs
//!
//! 1. CPA1: Foundational Operations Excellence
//! 2. CPA2: Analysis and Intelligence Excellence
//! 3. CPA3: Risk and Intervention Excellence
//! 4. CPA4: Quality and Compliance Excellence
//! 5. CPA5: Information and Technology Excellence
//! 6. CPA6: Communication and Stakeholder Excellence
//! 7. CPA7: Strategic Leadership and Development Excellence
//! 8. CPA8: Integrated Excellence and Transformation (Capstone)
//!
//! ## Components
//!
//! - [`CPACategory`] - The 8 Critical Practice Activities
//! - [`IntegrationModule`] - 3 integrated learning modules
//! - [`CPAExecutionStatus`] - Overall CPA execution status
//! - [`CPARequirement`] - CPA specification
//! - [`CPAValidationResult`] - CPA validation results
//! - [`CPAExecutionPlan`] - Detailed plan for CPA execution
//! - [`CPAExecutionState`] - Real-time execution state tracking

use crate::caba::Score;
use crate::caba::domain::DomainRequirement;
use crate::caba::proficiency::ProficiencyLevel;
use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// The 8 Critical Practice Activities.
///
/// # L0 Quark - CPA enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CPACategory {
    /// CPA1: Foundational Operations Excellence
    #[serde(rename = "CPA1: Foundational Operations Excellence")]
    Cpa1FoundationalOperations,
    /// CPA2: Analysis and Intelligence Excellence
    #[serde(rename = "CPA2: Analysis and Intelligence Excellence")]
    Cpa2AnalysisIntelligence,
    /// CPA3: Risk and Intervention Excellence
    #[serde(rename = "CPA3: Risk and Intervention Excellence")]
    Cpa3RiskIntervention,
    /// CPA4: Quality and Compliance Excellence
    #[serde(rename = "CPA4: Quality and Compliance Excellence")]
    Cpa4QualityCompliance,
    /// CPA5: Information and Technology Excellence
    #[serde(rename = "CPA5: Information and Technology Excellence")]
    Cpa5InformationTechnology,
    /// CPA6: Communication and Stakeholder Excellence
    #[serde(rename = "CPA6: Communication and Stakeholder Excellence")]
    Cpa6CommunicationStakeholder,
    /// CPA7: Strategic Leadership and Development Excellence
    #[serde(rename = "CPA7: Strategic Leadership and Development Excellence")]
    Cpa7StrategicLeadership,
    /// CPA8: Integrated Excellence and Transformation (Capstone)
    #[serde(rename = "CPA8: Integrated Excellence and Transformation (Capstone)")]
    Cpa8IntegratedExcellence,
}

impl CPACategory {
    /// Get the CPA number (1-8).
    #[must_use]
    pub const fn number(&self) -> u8 {
        match self {
            Self::Cpa1FoundationalOperations => 1,
            Self::Cpa2AnalysisIntelligence => 2,
            Self::Cpa3RiskIntervention => 3,
            Self::Cpa4QualityCompliance => 4,
            Self::Cpa5InformationTechnology => 5,
            Self::Cpa6CommunicationStakeholder => 6,
            Self::Cpa7StrategicLeadership => 7,
            Self::Cpa8IntegratedExcellence => 8,
        }
    }

    /// Get display string for the category.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Cpa1FoundationalOperations => "CPA1: Foundational Operations Excellence",
            Self::Cpa2AnalysisIntelligence => "CPA2: Analysis and Intelligence Excellence",
            Self::Cpa3RiskIntervention => "CPA3: Risk and Intervention Excellence",
            Self::Cpa4QualityCompliance => "CPA4: Quality and Compliance Excellence",
            Self::Cpa5InformationTechnology => "CPA5: Information and Technology Excellence",
            Self::Cpa6CommunicationStakeholder => "CPA6: Communication and Stakeholder Excellence",
            Self::Cpa7StrategicLeadership => {
                "CPA7: Strategic Leadership and Development Excellence"
            }
            Self::Cpa8IntegratedExcellence => {
                "CPA8: Integrated Excellence and Transformation (Capstone)"
            }
        }
    }

    /// Check if this is the capstone CPA (CPA8).
    #[must_use]
    pub const fn is_capstone(&self) -> bool {
        matches!(self, Self::Cpa8IntegratedExcellence)
    }
}

impl std::fmt::Display for CPACategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// The 3 integrated learning modules that weave through all development.
///
/// # L0 Quark - Module enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IntegrationModule {
    /// Module 1: Foundation-Communication Integration
    #[serde(rename = "Module 1: Foundation-Communication Integration")]
    Module1FoundationCommunication,
    /// Module 2: Technical-Communication Integration
    #[serde(rename = "Module 2: Technical-Communication Integration")]
    Module2TechnicalCommunication,
    /// Module 3: Strategic-Communication Integration
    #[serde(rename = "Module 3: Strategic-Communication Integration")]
    Module3StrategicCommunication,
}

impl IntegrationModule {
    /// Get the module number (1-3).
    #[must_use]
    pub const fn number(&self) -> u8 {
        match self {
            Self::Module1FoundationCommunication => 1,
            Self::Module2TechnicalCommunication => 2,
            Self::Module3StrategicCommunication => 3,
        }
    }

    /// Get display string for the module.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Module1FoundationCommunication => {
                "Module 1: Foundation-Communication Integration"
            }
            Self::Module2TechnicalCommunication => "Module 2: Technical-Communication Integration",
            Self::Module3StrategicCommunication => "Module 3: Strategic-Communication Integration",
        }
    }
}

impl std::fmt::Display for IntegrationModule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Overall CPA execution status.
///
/// # L0 Quark - Status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CPAExecutionStatus {
    /// CPA not started
    #[default]
    NotStarted,
    /// Assessing readiness
    AssessingReadiness,
    /// Orchestrating EPAs
    OrchestratingEpas,
    /// Integrating competencies
    IntegratingCompetencies,
    /// Validating excellence
    ValidatingExcellence,
    /// Successfully completed
    Completed,
    /// Execution failed
    Failed,
    /// Some components complete, some incomplete
    PartiallyComplete,
}

impl CPAExecutionStatus {
    /// Check if this status represents a terminal state.
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::PartiallyComplete
        )
    }

    /// Check if this status represents success.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }
}

/// Specification for a CPA to be orchestrated.
///
/// Defines which EPAs and domains are required for CPA completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPARequirement {
    /// CPA ID
    pub cpa_id: String,
    /// CPA category
    pub cpa_category: CPACategory,
    /// Title
    pub title: String,
    /// Description
    pub description: String,

    /// Domain requirements with proficiency levels
    pub required_domains: Vec<DomainRequirement>,

    /// EPA IDs that must be completed
    pub required_epas: Vec<String>,

    /// Integration modules required
    pub integration_modules: Vec<IntegrationModule>,

    /// Progression evidence requirements
    #[serde(default)]
    pub progression_criteria: Vec<String>,
    /// Validation criteria
    #[serde(default)]
    pub validation_criteria: Vec<String>,

    /// Target proficiency for this CPA
    #[serde(default)]
    pub target_proficiency: ProficiencyLevel,

    /// Executive sponsorship required (for CPA7/8)
    #[serde(default)]
    pub executive_sponsorship_required: bool,
    /// Transformation initiative required (for CPA8)
    #[serde(default)]
    pub transformation_initiative_required: bool,
}

impl CPARequirement {
    /// Create a new CPA requirement.
    #[must_use]
    pub fn new(
        cpa_id: String,
        cpa_category: CPACategory,
        title: String,
        description: String,
        required_domains: Vec<DomainRequirement>,
        required_epas: Vec<String>,
        integration_modules: Vec<IntegrationModule>,
    ) -> Self {
        Self {
            cpa_id,
            cpa_category,
            title,
            description,
            required_domains,
            required_epas,
            integration_modules,
            progression_criteria: Vec::new(),
            validation_criteria: Vec::new(),
            target_proficiency: ProficiencyLevel::L3Competent,
            executive_sponsorship_required: cpa_category.number() >= 7,
            transformation_initiative_required: cpa_category.is_capstone(),
        }
    }
}

/// Results of CPA validation.
///
/// Verifies integrated excellence across EPAs and domains.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPAValidationResult {
    /// CPA ID
    pub cpa_id: String,
    /// Whether excellence was achieved
    pub excellence_achieved: bool,
    /// Validation score [0.0, 1.0]
    pub validation_score: Score,

    /// EPA completion status (epa_id → completed)
    pub epa_completion: std::collections::HashMap<String, bool>,
    /// Domain proficiency achieved (domain → level)
    pub domain_proficiency: std::collections::HashMap<String, ProficiencyLevel>,

    /// Criteria evaluation results
    pub criteria_results: std::collections::HashMap<String, bool>,

    /// Evidence package
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
}

impl CPAValidationResult {
    /// Create a new validation result.
    ///
    /// # Errors
    /// Returns error if validation_score is not in [0.0, 1.0].
    pub fn new(
        cpa_id: String,
        excellence_achieved: bool,
        validation_score: f64,
        validated_at: String,
    ) -> Result<Self, crate::caba::ScoreError> {
        Ok(Self {
            cpa_id,
            excellence_achieved,
            validation_score: Score::new(validation_score)?,
            epa_completion: std::collections::HashMap::new(),
            domain_proficiency: std::collections::HashMap::new(),
            criteria_results: std::collections::HashMap::new(),
            evidence_package: serde_json::Value::Object(serde_json::Map::new()),
            errors: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
            validated_at,
        })
    }

    /// Calculate percentage of EPAs completed.
    #[must_use]
    pub fn epa_completion_percentage(&self) -> f64 {
        if self.epa_completion.is_empty() {
            return 0.0;
        }
        let completed = self.epa_completion.values().filter(|&&v| v).count();
        completed as f64 / self.epa_completion.len() as f64
    }

    /// Record EPA completion status.
    pub fn record_epa_completion(&mut self, epa_id: String, completed: bool) {
        self.epa_completion.insert(epa_id, completed);
    }

    /// Record domain proficiency level.
    pub fn record_domain_proficiency(&mut self, domain: String, level: ProficiencyLevel) {
        self.domain_proficiency.insert(domain, level);
    }

    /// Check if validation passed with no errors.
    #[must_use]
    pub fn is_successful(&self) -> bool {
        self.excellence_achieved && self.errors.is_empty()
    }
}

/// Prerequisites for entering an EPA within the CPA framework.
///
/// Validates professional readiness before EPA entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPAPrerequisite {
    /// Which EPA this applies to
    pub epa_id: String,

    /// Domain competency requirements
    pub required_domains: Vec<DomainRequirement>,

    /// Other EPA prerequisites
    #[serde(default)]
    pub prerequisite_epas: Vec<String>,

    /// Special certifications required
    #[serde(default)]
    pub required_certifications: Vec<String>,

    /// Innovation readiness threshold for EPA10 [0.0, 1.0]
    #[serde(default)]
    pub innovation_readiness_threshold: Option<f64>,

    /// Ethical certification required
    #[serde(default)]
    pub ethical_certification_required: bool,

    /// Leadership experience years (for Executive EPAs)
    #[serde(default)]
    pub leadership_experience_years: Option<u32>,

    /// Organizational impact required
    #[serde(default)]
    pub organizational_impact_required: bool,
}

impl EPAPrerequisite {
    /// Create a new EPA prerequisite.
    #[must_use]
    pub fn new(epa_id: String, required_domains: Vec<DomainRequirement>) -> Self {
        Self {
            epa_id,
            required_domains,
            prerequisite_epas: Vec::new(),
            required_certifications: Vec::new(),
            innovation_readiness_threshold: None,
            ethical_certification_required: false,
            leadership_experience_years: None,
            organizational_impact_required: false,
        }
    }

    /// Check if this is an executive-level EPA prerequisite.
    #[must_use]
    pub fn is_executive_level(&self) -> bool {
        self.leadership_experience_years.is_some() || self.organizational_impact_required
    }
}

// =============================================================================
// Execution Types (type definitions - services migrate separately)
// =============================================================================

/// Detailed plan for CPA execution.
///
/// Coordinates multiple EPAs to achieve integrated excellence.
/// Type definition only - orchestration services migrate separately.
///
/// # L2 Molecule - CPA execution planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPAExecutionPlan {
    /// CPA identifier
    pub cpa_id: String,
    /// CPA title
    pub cpa_title: String,

    // EPA orchestration
    /// EPA IDs required for this CPA
    pub required_epas: Vec<String>,
    /// Execution order of EPAs
    pub epa_execution_order: Vec<String>,

    // Domain development plan
    /// Target proficiency levels per domain (domain → level)
    #[serde(default)]
    pub domain_development: std::collections::HashMap<String, ProficiencyLevel>,

    // Integration approach
    /// Cross-EPA integration points
    #[serde(default)]
    pub integration_points: Vec<serde_json::Value>,

    // Timeline
    /// Estimated duration in weeks
    #[serde(default)]
    pub estimated_duration_weeks: u32,
    /// Key milestones
    #[serde(default)]
    pub milestones: Vec<serde_json::Value>,

    // Resources
    /// Required resources
    #[serde(default)]
    pub required_resources: Vec<String>,
    /// Estimated cost in USD
    #[serde(default)]
    pub estimated_cost_usd: f64,

    /// Plan creation timestamp
    #[serde(default)]
    pub created_at: Option<DateTime>,
}

impl CPAExecutionPlan {
    /// Create a new CPA execution plan.
    #[must_use]
    pub fn new(
        cpa_id: String,
        cpa_title: String,
        required_epas: Vec<String>,
        epa_execution_order: Vec<String>,
    ) -> Self {
        Self {
            cpa_id,
            cpa_title,
            required_epas,
            epa_execution_order,
            domain_development: std::collections::HashMap::new(),
            integration_points: Vec::new(),
            estimated_duration_weeks: 0,
            milestones: Vec::new(),
            required_resources: Vec::new(),
            estimated_cost_usd: 0.0,
            created_at: Some(DateTime::now()),
        }
    }

    /// Get total number of required EPAs.
    #[must_use]
    pub fn total_epas(&self) -> usize {
        self.required_epas.len()
    }

    /// Check if this plan has domain development targets.
    #[must_use]
    pub fn has_domain_targets(&self) -> bool {
        !self.domain_development.is_empty()
    }
}

/// Real-time state of CPA execution.
///
/// Tracks progress across multiple EPAs and domain development.
/// Type definition only - persistence services migrate separately.
///
/// # L2 Molecule - CPA execution state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CPAExecutionState {
    /// CPA identifier
    pub cpa_id: String,
    /// Overall execution status
    pub status: CPAExecutionStatus,

    // Progress
    /// Currently executing EPA ID
    #[serde(default)]
    pub current_epa: Option<String>,
    /// Completed EPA IDs
    #[serde(default)]
    pub epas_completed: Vec<String>,
    /// Total number of EPAs
    #[serde(default)]
    pub total_epas: u32,
    /// Completion percentage [0.0, 100.0]
    #[serde(default)]
    pub percent_complete: f64,

    // Timeline
    /// When execution started
    #[serde(default)]
    pub started_at: Option<DateTime>,
    /// When state was last updated
    #[serde(default)]
    pub last_updated: Option<DateTime>,
    /// When execution completed
    #[serde(default)]
    pub completed_at: Option<DateTime>,

    // Domain progress
    /// Current proficiency levels per domain
    #[serde(default)]
    pub domain_proficiency: std::collections::HashMap<String, ProficiencyLevel>,

    // EPA results
    /// Results from each EPA (epa_id → result)
    #[serde(default)]
    pub epa_results: std::collections::HashMap<String, serde_json::Value>,

    // Integration tracking
    /// Completed integration points
    #[serde(default)]
    pub integration_completions: Vec<String>,

    // Issues
    /// Errors encountered during execution
    #[serde(default)]
    pub errors: Vec<String>,
    /// Warnings encountered during execution
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl CPAExecutionState {
    /// Create a new execution state in NotStarted status.
    #[must_use]
    pub fn new(cpa_id: String, total_epas: u32) -> Self {
        Self {
            cpa_id,
            status: CPAExecutionStatus::NotStarted,
            current_epa: None,
            epas_completed: Vec::new(),
            total_epas,
            percent_complete: 0.0,
            started_at: None,
            last_updated: Some(DateTime::now()),
            completed_at: None,
            domain_proficiency: std::collections::HashMap::new(),
            epa_results: std::collections::HashMap::new(),
            integration_completions: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Check if execution has any errors.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get count of completed EPAs.
    #[must_use]
    pub fn completed_epa_count(&self) -> usize {
        self.epas_completed.len()
    }

    /// Calculate completion percentage based on EPAs.
    #[must_use]
    pub fn calculated_completion(&self) -> f64 {
        if self.total_epas == 0 {
            return 0.0;
        }
        (self.epas_completed.len() as f64 / self.total_epas as f64) * 100.0
    }

    /// Check if execution is in a terminal state.
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.status.is_terminal()
    }

    /// Record EPA completion.
    pub fn record_epa_completion(&mut self, epa_id: String, result: serde_json::Value) {
        self.epas_completed.push(epa_id.clone());
        self.epa_results.insert(epa_id, result);
        self.percent_complete = self.calculated_completion();
        self.last_updated = Some(DateTime::now());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpa_category_number() {
        assert_eq!(CPACategory::Cpa1FoundationalOperations.number(), 1);
        assert_eq!(CPACategory::Cpa8IntegratedExcellence.number(), 8);
    }

    #[test]
    fn test_cpa_category_capstone() {
        assert!(!CPACategory::Cpa1FoundationalOperations.is_capstone());
        assert!(CPACategory::Cpa8IntegratedExcellence.is_capstone());
    }

    #[test]
    fn test_integration_module_number() {
        assert_eq!(
            IntegrationModule::Module1FoundationCommunication.number(),
            1
        );
        assert_eq!(IntegrationModule::Module3StrategicCommunication.number(), 3);
    }

    #[test]
    fn test_cpa_execution_status() {
        assert!(CPAExecutionStatus::Completed.is_terminal());
        assert!(CPAExecutionStatus::Completed.is_success());
        assert!(!CPAExecutionStatus::Failed.is_success());
    }

    #[test]
    fn test_cpa_validation_result() {
        let result = CPAValidationResult::new(
            "CPA-1".to_string(),
            true,
            0.92,
            "2026-01-29T00:00:00Z".to_string(),
        );
        assert!(result.is_ok());

        if let Ok(mut result) = result {
            result.record_epa_completion("EPA-1".to_string(), true);
            result.record_epa_completion("EPA-2".to_string(), true);
            result.record_epa_completion("EPA-3".to_string(), false);

            assert!((result.epa_completion_percentage() - 0.666_666).abs() < 0.001);
        }
    }

    #[test]
    fn test_cpa_execution_plan() {
        let plan = CPAExecutionPlan::new(
            "CPA-HIPAA-001".to_string(),
            "HIPAA Compliance Excellence".to_string(),
            vec![
                "EPA-1".to_string(),
                "EPA-2".to_string(),
                "EPA-3".to_string(),
            ],
            vec![
                "EPA-1".to_string(),
                "EPA-2".to_string(),
                "EPA-3".to_string(),
            ],
        );

        assert_eq!(plan.total_epas(), 3);
        assert!(!plan.has_domain_targets());
    }

    #[test]
    fn test_cpa_execution_plan_with_domain_targets() {
        let mut plan = CPAExecutionPlan::new(
            "CPA-HIPAA-001".to_string(),
            "HIPAA Compliance Excellence".to_string(),
            vec!["EPA-1".to_string()],
            vec!["EPA-1".to_string()],
        );

        plan.domain_development
            .insert("D1".to_string(), ProficiencyLevel::L3Competent);

        assert!(plan.has_domain_targets());
    }

    #[test]
    fn test_cpa_execution_state() {
        let state = CPAExecutionState::new("CPA-HIPAA-001".to_string(), 3);

        assert_eq!(state.status, CPAExecutionStatus::NotStarted);
        assert_eq!(state.total_epas, 3);
        assert_eq!(state.completed_epa_count(), 0);
        assert!(!state.is_terminal());
        assert!(!state.has_errors());
        assert_eq!(state.calculated_completion(), 0.0);
    }

    #[test]
    fn test_cpa_execution_state_progress() {
        let mut state = CPAExecutionState::new("CPA-HIPAA-001".to_string(), 3);

        state.record_epa_completion("EPA-1".to_string(), serde_json::json!({"success": true}));
        assert_eq!(state.completed_epa_count(), 1);
        assert!((state.percent_complete - 33.333_333).abs() < 0.001);

        state.record_epa_completion("EPA-2".to_string(), serde_json::json!({"success": true}));
        assert_eq!(state.completed_epa_count(), 2);
        assert!((state.percent_complete - 66.666_666).abs() < 0.001);
    }
}
