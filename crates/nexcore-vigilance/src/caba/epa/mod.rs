//! EPA (Entrustable Professional Activity) Types
//!
//! The 21 PV Entrustable Professional Activities from the NexVigilant KSB Framework.
//! (source: ~/Vaults/nexvigilant/400-projects/ksb-framework/epas/)
//!
//! ## EPA Overview
//!
//! An EPA is a unit of professional practice — a task or responsibility that a
//! PV professional can be trusted to perform at a given proficiency level.
//!
//! ## The 21 PV EPAs by Tier
//!
//! **Core (EPA-01 to EPA-08):** Foundation PV activities
//! **Advanced (EPA-09 to EPA-14, EPA-17):** Specialized PV activities
//! **Expert (EPA-15, EPA-16, EPA-18 to EPA-21):** Leadership PV activities
//!
//! ## Components
//!
//! - [`EPACategory`] - 21 PV entrustable activities
//! - [`EPATier`] - Core, Advanced, Expert classification
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

/// The 21 PV Entrustable Professional Activities.
///
/// (source: 04-ksb-competency-framework.md EPA table)
///
/// # L0 Quark - Category enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EPACategory {
    // Core Tier (EPA-01 to EPA-08)
    /// EPA-01: Process and Evaluate ICSRs
    #[serde(rename = "EPA-01: Process and Evaluate ICSRs")]
    Epa01ProcessIcsrs,
    /// EPA-02: Assess Causality
    #[serde(rename = "EPA-02: Assess Causality")]
    Epa02AssessCausality,
    /// EPA-03: Manage Case Quality
    #[serde(rename = "EPA-03: Manage Case Quality")]
    Epa03ManageCaseQuality,
    /// EPA-04: Detect Safety Signals
    #[serde(rename = "EPA-04: Detect Safety Signals")]
    Epa04DetectSignals,
    /// EPA-05: Validate and Analyze Signals
    #[serde(rename = "EPA-05: Validate and Analyze Signals")]
    Epa05ValidateSignals,
    /// EPA-06: Assess Benefit-Risk
    #[serde(rename = "EPA-06: Assess Benefit-Risk")]
    Epa06AssessBenefitRisk,
    /// EPA-07: Manage Risk Minimization
    #[serde(rename = "EPA-07: Manage Risk Minimization")]
    Epa07ManageRiskMinimization,
    /// EPA-08: Prepare Regulatory Submissions
    #[serde(rename = "EPA-08: Prepare Regulatory Submissions")]
    Epa08RegulatorySubmissions,

    // Advanced Tier (EPA-09 to EPA-14, EPA-17)
    /// EPA-09: Manage Regulatory Intelligence
    #[serde(rename = "EPA-09: Manage Regulatory Intelligence")]
    Epa09RegulatoryIntelligence,
    /// EPA-10: Integrate AI in PV
    #[serde(rename = "EPA-10: Integrate AI in PV")]
    Epa10IntegrateAi,
    /// EPA-11: Manage PV Systems
    #[serde(rename = "EPA-11: Manage PV Systems")]
    Epa11ManagePvSystems,
    /// EPA-12: Conduct PV Audits
    #[serde(rename = "EPA-12: Conduct PV Audits")]
    Epa12ConductAudits,
    /// EPA-13: Manage Special Population Safety
    #[serde(rename = "EPA-13: Manage Special Population Safety")]
    Epa13SpecialPopulations,
    /// EPA-14: Coordinate Global PV
    #[serde(rename = "EPA-14: Coordinate Global PV")]
    Epa14CoordinateGlobalPv,
    /// EPA-17: Manage Stakeholder Communication
    #[serde(rename = "EPA-17: Manage Stakeholder Communication")]
    Epa17StakeholderCommunication,

    // Expert Tier (EPA-15, EPA-16, EPA-18 to EPA-21)
    /// EPA-15: Lead PV Program Strategy
    #[serde(rename = "EPA-15: Lead PV Program Strategy")]
    Epa15LeadPvStrategy,
    /// EPA-16: Conduct Advanced Safety Analytics
    #[serde(rename = "EPA-16: Conduct Advanced Safety Analytics")]
    Epa16AdvancedAnalytics,
    /// EPA-18: Develop PV Talent
    #[serde(rename = "EPA-18: Develop PV Talent")]
    Epa18DevelopTalent,
    /// EPA-19: Manage Crisis Safety Communication
    #[serde(rename = "EPA-19: Manage Crisis Safety Communication")]
    Epa19CrisisCommunication,
    /// EPA-20: Design Pharmacoepidemiology Studies
    #[serde(rename = "EPA-20: Design Pharmacoepidemiology Studies")]
    Epa20PharmacoepidemiologyStudies,
    /// EPA-21: Lead Organizational PV Transformation
    #[serde(rename = "EPA-21: Lead Organizational PV Transformation")]
    Epa21LeadTransformation,
}

impl EPACategory {
    /// All 21 EPA variants.
    pub const ALL: [Self; 21] = [
        Self::Epa01ProcessIcsrs,
        Self::Epa02AssessCausality,
        Self::Epa03ManageCaseQuality,
        Self::Epa04DetectSignals,
        Self::Epa05ValidateSignals,
        Self::Epa06AssessBenefitRisk,
        Self::Epa07ManageRiskMinimization,
        Self::Epa08RegulatorySubmissions,
        Self::Epa09RegulatoryIntelligence,
        Self::Epa10IntegrateAi,
        Self::Epa11ManagePvSystems,
        Self::Epa12ConductAudits,
        Self::Epa13SpecialPopulations,
        Self::Epa14CoordinateGlobalPv,
        Self::Epa15LeadPvStrategy,
        Self::Epa16AdvancedAnalytics,
        Self::Epa17StakeholderCommunication,
        Self::Epa18DevelopTalent,
        Self::Epa19CrisisCommunication,
        Self::Epa20PharmacoepidemiologyStudies,
        Self::Epa21LeadTransformation,
    ];

    /// Get the EPA number (1-21).
    #[must_use]
    pub const fn number(&self) -> u8 {
        match self {
            Self::Epa01ProcessIcsrs => 1,
            Self::Epa02AssessCausality => 2,
            Self::Epa03ManageCaseQuality => 3,
            Self::Epa04DetectSignals => 4,
            Self::Epa05ValidateSignals => 5,
            Self::Epa06AssessBenefitRisk => 6,
            Self::Epa07ManageRiskMinimization => 7,
            Self::Epa08RegulatorySubmissions => 8,
            Self::Epa09RegulatoryIntelligence => 9,
            Self::Epa10IntegrateAi => 10,
            Self::Epa11ManagePvSystems => 11,
            Self::Epa12ConductAudits => 12,
            Self::Epa13SpecialPopulations => 13,
            Self::Epa14CoordinateGlobalPv => 14,
            Self::Epa15LeadPvStrategy => 15,
            Self::Epa16AdvancedAnalytics => 16,
            Self::Epa17StakeholderCommunication => 17,
            Self::Epa18DevelopTalent => 18,
            Self::Epa19CrisisCommunication => 19,
            Self::Epa20PharmacoepidemiologyStudies => 20,
            Self::Epa21LeadTransformation => 21,
        }
    }

    /// Get the EPA tier.
    #[must_use]
    pub const fn tier(&self) -> EPATier {
        match self {
            Self::Epa01ProcessIcsrs
            | Self::Epa02AssessCausality
            | Self::Epa03ManageCaseQuality
            | Self::Epa04DetectSignals
            | Self::Epa05ValidateSignals
            | Self::Epa06AssessBenefitRisk
            | Self::Epa07ManageRiskMinimization
            | Self::Epa08RegulatorySubmissions => EPATier::Core,

            Self::Epa09RegulatoryIntelligence
            | Self::Epa10IntegrateAi
            | Self::Epa11ManagePvSystems
            | Self::Epa12ConductAudits
            | Self::Epa13SpecialPopulations
            | Self::Epa14CoordinateGlobalPv
            | Self::Epa17StakeholderCommunication => EPATier::Advanced,

            Self::Epa15LeadPvStrategy
            | Self::Epa16AdvancedAnalytics
            | Self::Epa18DevelopTalent
            | Self::Epa19CrisisCommunication
            | Self::Epa20PharmacoepidemiologyStudies
            | Self::Epa21LeadTransformation => EPATier::Expert,
        }
    }

    /// Get the PV focus area for this EPA.
    #[must_use]
    pub const fn focus_area(&self) -> &'static str {
        match self {
            Self::Epa01ProcessIcsrs | Self::Epa02AssessCausality | Self::Epa03ManageCaseQuality => {
                "Case Processing"
            }
            Self::Epa04DetectSignals | Self::Epa05ValidateSignals => "Signal Management",
            Self::Epa06AssessBenefitRisk | Self::Epa07ManageRiskMinimization => "Risk Assessment",
            Self::Epa08RegulatorySubmissions | Self::Epa09RegulatoryIntelligence => "Regulatory",
            Self::Epa10IntegrateAi | Self::Epa11ManagePvSystems => "Technology",
            Self::Epa12ConductAudits => "Quality",
            Self::Epa13SpecialPopulations => "Specialized",
            Self::Epa14CoordinateGlobalPv => "Operations",
            Self::Epa15LeadPvStrategy => "Management",
            Self::Epa16AdvancedAnalytics | Self::Epa20PharmacoepidemiologyStudies => "Analytics",
            Self::Epa17StakeholderCommunication | Self::Epa19CrisisCommunication => "Communication",
            Self::Epa18DevelopTalent => "Development",
            Self::Epa21LeadTransformation => "Transformation",
        }
    }

    /// Get display string for the category.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Epa01ProcessIcsrs => "EPA-01: Process and Evaluate ICSRs",
            Self::Epa02AssessCausality => "EPA-02: Assess Causality",
            Self::Epa03ManageCaseQuality => "EPA-03: Manage Case Quality",
            Self::Epa04DetectSignals => "EPA-04: Detect Safety Signals",
            Self::Epa05ValidateSignals => "EPA-05: Validate and Analyze Signals",
            Self::Epa06AssessBenefitRisk => "EPA-06: Assess Benefit-Risk",
            Self::Epa07ManageRiskMinimization => "EPA-07: Manage Risk Minimization",
            Self::Epa08RegulatorySubmissions => "EPA-08: Prepare Regulatory Submissions",
            Self::Epa09RegulatoryIntelligence => "EPA-09: Manage Regulatory Intelligence",
            Self::Epa10IntegrateAi => "EPA-10: Integrate AI in PV",
            Self::Epa11ManagePvSystems => "EPA-11: Manage PV Systems",
            Self::Epa12ConductAudits => "EPA-12: Conduct PV Audits",
            Self::Epa13SpecialPopulations => "EPA-13: Manage Special Population Safety",
            Self::Epa14CoordinateGlobalPv => "EPA-14: Coordinate Global PV",
            Self::Epa15LeadPvStrategy => "EPA-15: Lead PV Program Strategy",
            Self::Epa16AdvancedAnalytics => "EPA-16: Conduct Advanced Safety Analytics",
            Self::Epa17StakeholderCommunication => "EPA-17: Manage Stakeholder Communication",
            Self::Epa18DevelopTalent => "EPA-18: Develop PV Talent",
            Self::Epa19CrisisCommunication => "EPA-19: Manage Crisis Safety Communication",
            Self::Epa20PharmacoepidemiologyStudies => "EPA-20: Design Pharmacoepidemiology Studies",
            Self::Epa21LeadTransformation => "EPA-21: Lead Organizational PV Transformation",
        }
    }
}

impl std::fmt::Display for EPACategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// EPA tier classification.
///
/// (source: 04-ksb-competency-framework.md EPA table, Tier column)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EPATier {
    /// Foundation PV activities (EPA-01 to EPA-08)
    Core,
    /// Specialized PV activities (EPA-09 to EPA-14, EPA-17)
    Advanced,
    /// Leadership PV activities (EPA-15, EPA-16, EPA-18 to EPA-21)
    Expert,
}

impl EPATier {
    /// Get display string.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Core => "Core",
            Self::Advanced => "Advanced",
            Self::Expert => "Expert",
        }
    }
}

impl std::fmt::Display for EPATier {
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

    /// Specific regulation sections (e.g., "ICH E2B(R3)", "GVP Module VI")
    #[serde(default)]
    pub regulatory_citations: Vec<String>,

    /// Priority level
    #[serde(default)]
    pub priority: Priority,
    /// Estimated duration in hours
    #[serde(default)]
    pub estimated_duration_hours: Option<f64>,
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
            regulatory_citations: Vec::new(),
            priority: Priority::default(),
            estimated_duration_hours: None,
        }
    }
}

/// Results of EPA goal validation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPAValidationResult {
    /// EPA ID
    pub epa_id: String,
    /// Whether goal was achieved
    pub goal_achieved: bool,
    /// Validation score [0.0, 1.0]
    pub validation_score: Score,

    /// Success criteria evaluation (criterion -> passed/failed)
    pub criteria_results: std::collections::HashMap<String, bool>,

    /// Errors found
    #[serde(default)]
    pub errors: Vec<String>,
    /// Warnings found
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Recommendations for improvement
    #[serde(default)]
    pub recommendations: Vec<String>,

    /// Validation timestamp
    pub validated_at: String,

    /// How reliable is the deployment [0.0, 1.0]
    #[serde(default)]
    pub reliability_score: Score,
    /// How complete is the implementation [0.0, 1.0]
    #[serde(default)]
    pub completeness_score: Score,
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
            errors: Vec::new(),
            warnings: Vec::new(),
            recommendations: Vec::new(),
            validated_at,
            reliability_score: Score::ZERO,
            completeness_score: Score::ZERO,
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
// Execution Types
// =============================================================================

/// Represents deployment of a single competency within an EPA.
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

    /// When deployment started
    #[serde(default)]
    pub started_at: Option<DateTime>,
    /// When deployment completed
    #[serde(default)]
    pub completed_at: Option<DateTime>,
    /// Total duration in seconds
    #[serde(default)]
    pub duration_seconds: Option<f64>,

    /// Errors encountered during deployment
    #[serde(default)]
    pub deployment_errors: Vec<String>,
    /// Warnings encountered during deployment
    #[serde(default)]
    pub deployment_warnings: Vec<String>,

    /// Other competency IDs this step depends on
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Current progress percentage [0.0, 100.0]
    #[serde(default)]
    pub progress_percent: f64,
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
/// # L2 Molecule - EPA execution planning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPAExecutionPlan {
    /// EPA identifier
    pub epa_id: String,
    /// EPA title
    pub epa_title: String,

    /// Deployment steps in execution order
    pub deployment_steps: Vec<CompetencyDeploymentStep>,

    /// Estimated total duration in minutes
    #[serde(default)]
    pub estimated_duration_minutes: Option<u32>,

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
            estimated_duration_minutes: None,
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
/// # L2 Molecule - EPA execution state tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EPAExecutionState {
    /// EPA identifier
    pub epa_id: String,
    /// Overall execution status
    pub status: EPAExecutionStatus,

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

    /// When execution started
    #[serde(default)]
    pub started_at: Option<DateTime>,
    /// When state was last updated
    #[serde(default)]
    pub last_updated: Option<DateTime>,
    /// When execution completed
    #[serde(default)]
    pub completed_at: Option<DateTime>,

    /// Status of each competency deployment (competency_id -> status)
    #[serde(default)]
    pub competency_states: std::collections::HashMap<String, CompetencyDeploymentStatus>,

    /// Errors encountered during execution
    #[serde(default)]
    pub errors: Vec<String>,
    /// Warnings encountered during execution
    #[serde(default)]
    pub warnings: Vec<String>,
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
    fn test_epa_category_tier() {
        assert_eq!(EPACategory::Epa01ProcessIcsrs.tier(), EPATier::Core);
        assert_eq!(
            EPACategory::Epa08RegulatorySubmissions.tier(),
            EPATier::Core
        );
        assert_eq!(EPACategory::Epa10IntegrateAi.tier(), EPATier::Advanced);
        assert_eq!(EPACategory::Epa21LeadTransformation.tier(), EPATier::Expert);
    }

    #[test]
    fn test_epa_focus_area() {
        assert_eq!(
            EPACategory::Epa01ProcessIcsrs.focus_area(),
            "Case Processing"
        );
        assert_eq!(
            EPACategory::Epa04DetectSignals.focus_area(),
            "Signal Management"
        );
        assert_eq!(EPACategory::Epa12ConductAudits.focus_area(), "Quality");
    }

    #[test]
    fn test_epa_numbers() {
        assert_eq!(EPACategory::Epa01ProcessIcsrs.number(), 1);
        assert_eq!(EPACategory::Epa21LeadTransformation.number(), 21);
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
            "EPA-04".to_string(),
            true,
            0.95,
            "2026-01-29T00:00:00Z".to_string(),
        );
        assert!(result.is_ok());

        if let Ok(mut result) = result {
            result.add_criterion_result("Signal detected".to_string(), true);
            result.add_criterion_result("Threshold validated".to_string(), true);
            result.add_criterion_result("Causality assessed".to_string(), false);

            assert!((result.criteria_met_percentage() - 0.666_666).abs() < 0.001);
        }
    }

    #[test]
    fn test_competency_deployment_step() {
        let step =
            CompetencyDeploymentStep::new("CC-SIG-001".to_string(), "Signal Detection".to_string());

        assert_eq!(step.status, CompetencyDeploymentStatus::Pending);
        assert!(!step.has_errors());
        assert!(!step.has_warnings());
    }

    #[test]
    fn test_epa_execution_plan() {
        let steps = vec![
            CompetencyDeploymentStep::new("CC-SIG-001".to_string(), "Signal Detection".to_string()),
            CompetencyDeploymentStep::new(
                "CC-CAUS-001".to_string(),
                "Causality Assessment".to_string(),
            ),
        ];

        let plan = EPAExecutionPlan::new(
            "EPA-04".to_string(),
            "Detect Safety Signals".to_string(),
            steps,
        );

        assert_eq!(plan.total_steps(), 2);
        assert_eq!(plan.competency_ids(), vec!["CC-SIG-001", "CC-CAUS-001"]);
    }

    #[test]
    fn test_epa_execution_state() {
        let state = EPAExecutionState::new("EPA-04".to_string(), 3);

        assert_eq!(state.status, EPAExecutionStatus::NotStarted);
        assert_eq!(state.steps_completed, 0);
        assert_eq!(state.total_steps, 3);
        assert!(!state.is_terminal());
        assert!(!state.has_errors());
    }

    #[test]
    fn test_epa_execution_state_competency_counts() {
        let mut state = EPAExecutionState::new("EPA-04".to_string(), 3);

        state.competency_states.insert(
            "CC-SIG-001".to_string(),
            CompetencyDeploymentStatus::Completed,
        );
        state.competency_states.insert(
            "CC-CAUS-001".to_string(),
            CompetencyDeploymentStatus::Completed,
        );
        state.competency_states.insert(
            "CC-RISK-001".to_string(),
            CompetencyDeploymentStatus::Failed,
        );

        assert_eq!(state.completed_competency_count(), 2);
        assert_eq!(state.failed_competency_count(), 1);
    }

    #[test]
    fn test_all_21_epas_exist() {
        let epas = [
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
        assert_eq!(epas.len(), 21);
        for epa in &epas {
            assert!(epa.number() >= 1 && epa.number() <= 21);
        }
    }
}
