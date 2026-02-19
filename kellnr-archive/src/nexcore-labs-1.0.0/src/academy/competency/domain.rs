//! Competency Domain Types
//!
//! 15 Competency Domains organized in 4 thematic clusters.
//! Each domain has behavioral anchors at 7 levels (L1-L5++).

use serde::{Deserialize, Serialize};

/// 15 Competency Domains for Pharmacovigilance professionals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Domain {
    // Thematic Cluster 1: Foundational Knowledge
    /// Domain 1: PV Foundations
    #[serde(rename = "Domain_1_Foundations")]
    Foundations,
    /// Domain 2: Clinical ADRs
    #[serde(rename = "Domain_2_Clinical_ADRs")]
    ClinicalAdrs,
    /// Domain 3: Important ADRs
    #[serde(rename = "Domain_3_Important_ADRs")]
    ImportantAdrs,

    // Thematic Cluster 2: Core Process Competencies
    /// Domain 4: Case Processing
    #[serde(rename = "Domain_4_Case_Processing")]
    CaseProcessing,
    /// Domain 5: Literature Monitoring
    #[serde(rename = "Domain_5_Literature_Monitoring")]
    LiteratureMonitoring,
    /// Domain 6: Aggregate Reporting
    #[serde(rename = "Domain_6_Aggregate_Reporting")]
    AggregateReporting,
    /// Domain 7: Signal Detection
    #[serde(rename = "Domain_7_Signal_Detection")]
    SignalDetection,
    /// Domain 8: Signal Evaluation
    #[serde(rename = "Domain_8_Signal_Evaluation")]
    SignalEvaluation,
    /// Domain 9: Safety Studies
    #[serde(rename = "Domain_9_Safety_Studies")]
    SafetyStudies,

    // Thematic Cluster 3: Strategic Assessment Competencies
    /// Domain 10: Benefit-Risk Assessment
    #[serde(rename = "Domain_10_Benefit_Risk")]
    BenefitRisk,
    /// Domain 11: Risk Management
    #[serde(rename = "Domain_11_Risk_Management")]
    RiskManagement,
    /// Domain 12: Regulatory Strategy
    #[serde(rename = "Domain_12_Regulatory_Strategy")]
    RegulatoryStrategy,

    // Thematic Cluster 4: Integration and Leadership
    /// Domain 13: Technology & AI
    #[serde(rename = "Domain_13_Technology_AI")]
    TechnologyAi,
    /// Domain 14: Communication
    #[serde(rename = "Domain_14_Communication")]
    Communication,
    /// Domain 15: Leadership & Ethics
    #[serde(rename = "Domain_15_Leadership_Ethics")]
    LeadershipEthics,
}

/// Thematic clusters grouping related domains
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum ThematicCluster {
    /// Cluster 1: Foundational Knowledge (Domains 1-3)
    FoundationalKnowledge = 1,
    /// Cluster 2: Core Process Competencies (Domains 4-9)
    CoreProcess = 2,
    /// Cluster 3: Strategic Assessment (Domains 10-12)
    StrategicAssessment = 3,
    /// Cluster 4: Integration and Leadership (Domains 13-15)
    IntegrationLeadership = 4,
}

impl ThematicCluster {
    /// Get the display name for a thematic cluster
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::FoundationalKnowledge => "Foundational Knowledge",
            Self::CoreProcess => "Core Process Competencies",
            Self::StrategicAssessment => "Strategic Assessment Competencies",
            Self::IntegrationLeadership => "Integration and Leadership Competencies",
        }
    }

    /// Get all domains in this cluster
    #[must_use]
    pub const fn domains(self) -> &'static [Domain] {
        match self {
            Self::FoundationalKnowledge => &[
                Domain::Foundations,
                Domain::ClinicalAdrs,
                Domain::ImportantAdrs,
            ],
            Self::CoreProcess => &[
                Domain::CaseProcessing,
                Domain::LiteratureMonitoring,
                Domain::AggregateReporting,
                Domain::SignalDetection,
                Domain::SignalEvaluation,
                Domain::SafetyStudies,
            ],
            Self::StrategicAssessment => &[
                Domain::BenefitRisk,
                Domain::RiskManagement,
                Domain::RegulatoryStrategy,
            ],
            Self::IntegrationLeadership => &[
                Domain::TechnologyAi,
                Domain::Communication,
                Domain::LeadershipEthics,
            ],
        }
    }
}

impl Domain {
    /// Get the thematic cluster for this domain
    #[must_use]
    pub const fn cluster(self) -> ThematicCluster {
        match self {
            Self::Foundations | Self::ClinicalAdrs | Self::ImportantAdrs => {
                ThematicCluster::FoundationalKnowledge
            }
            Self::CaseProcessing
            | Self::LiteratureMonitoring
            | Self::AggregateReporting
            | Self::SignalDetection
            | Self::SignalEvaluation
            | Self::SafetyStudies => ThematicCluster::CoreProcess,
            Self::BenefitRisk | Self::RiskManagement | Self::RegulatoryStrategy => {
                ThematicCluster::StrategicAssessment
            }
            Self::TechnologyAi | Self::Communication | Self::LeadershipEthics => {
                ThematicCluster::IntegrationLeadership
            }
        }
    }

    /// Get the domain number (1-15)
    #[must_use]
    pub const fn number(self) -> u8 {
        match self {
            Self::Foundations => 1,
            Self::ClinicalAdrs => 2,
            Self::ImportantAdrs => 3,
            Self::CaseProcessing => 4,
            Self::LiteratureMonitoring => 5,
            Self::AggregateReporting => 6,
            Self::SignalDetection => 7,
            Self::SignalEvaluation => 8,
            Self::SafetyStudies => 9,
            Self::BenefitRisk => 10,
            Self::RiskManagement => 11,
            Self::RegulatoryStrategy => 12,
            Self::TechnologyAi => 13,
            Self::Communication => 14,
            Self::LeadershipEthics => 15,
        }
    }
}

/// Competency levels with progression pathway
///
/// - L1-L2: Foundation (Supervised)
/// - L3: Independent Practice
/// - L4-L5: Advanced Leadership
/// - L5+, L5++: Executive Expert
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum CompetencyLevel {
    /// L1: Novice - Observation
    #[default]
    L1,
    /// L2: Advanced Beginner - Supervised Practice
    L2,
    /// L3: Competent - Independent Practice
    L3,
    /// L4: Proficient - Strategic Thinking
    L4,
    /// L5: Expert - Innovation Leadership
    L5,
    /// L5+: Executive Expert - Organizational Impact
    #[serde(rename = "L5+")]
    L5Plus,
    /// L5++: Industry Leader - Global Influence
    #[serde(rename = "L5++")]
    L5PlusPlus,
}

impl CompetencyLevel {
    /// All levels in order from lowest to highest
    pub const ALL: [Self; 7] = [
        Self::L1,
        Self::L2,
        Self::L3,
        Self::L4,
        Self::L5,
        Self::L5Plus,
        Self::L5PlusPlus,
    ];

    /// Get the next level, if any
    #[must_use]
    pub const fn next(self) -> Option<Self> {
        match self {
            Self::L1 => Some(Self::L2),
            Self::L2 => Some(Self::L3),
            Self::L3 => Some(Self::L4),
            Self::L4 => Some(Self::L5),
            Self::L5 => Some(Self::L5Plus),
            Self::L5Plus => Some(Self::L5PlusPlus),
            Self::L5PlusPlus => None,
        }
    }

    /// Get the previous level, if any
    #[must_use]
    pub const fn previous(self) -> Option<Self> {
        match self {
            Self::L1 => None,
            Self::L2 => Some(Self::L1),
            Self::L3 => Some(Self::L2),
            Self::L4 => Some(Self::L3),
            Self::L5 => Some(Self::L4),
            Self::L5Plus => Some(Self::L5),
            Self::L5PlusPlus => Some(Self::L5Plus),
        }
    }

    /// Check if this is a foundation level (L1-L2)
    #[must_use]
    pub const fn is_foundation(self) -> bool {
        matches!(self, Self::L1 | Self::L2)
    }

    /// Check if this is an executive level (L5+ or L5++)
    #[must_use]
    pub const fn is_executive(self) -> bool {
        matches!(self, Self::L5Plus | Self::L5PlusPlus)
    }

    /// Get numeric index (0-6) for ordering
    #[must_use]
    pub const fn ordinal(self) -> u8 {
        match self {
            Self::L1 => 0,
            Self::L2 => 1,
            Self::L3 => 2,
            Self::L4 => 3,
            Self::L5 => 4,
            Self::L5Plus => 5,
            Self::L5PlusPlus => 6,
        }
    }
}

/// Evidence types for behavioral anchor validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    /// Completed case, report, etc.
    WorkProduct,
    /// Supervisor watches performance
    DirectObservation,
    /// Practice scenario
    Simulation,
    /// 360 feedback
    PeerReview,
    /// Reflective practice
    SelfAssessment,
    /// Documentation artifact
    PortfolioArtifact,
}

/// Observable behavioral demonstration
///
/// Each domain has 3-5 behavioral anchors per level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehavioralAnchor {
    /// Unique identifier (e.g., "Domain_1_L2_3")
    pub id: String,
    /// The domain this anchor belongs to
    pub domain: Domain,
    /// The competency level
    pub level: CompetencyLevel,
    /// Observable behavior description
    pub description: String,
    /// How to validate demonstration
    pub assessment_criteria: Vec<String>,
    /// What evidence is acceptable
    pub evidence_types: Vec<EvidenceType>,
    /// Must achieve to advance to next level
    pub required_for_progression: bool,
}

/// Complete domain definition with behavioral anchors
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainDefinition {
    /// Domain identifier
    pub id: Domain,
    /// Human-readable name
    pub name: String,
    /// Domain description
    pub description: String,
    /// Thematic cluster (1-4)
    pub thematic_cluster: ThematicCluster,
    /// Behavioral anchors organized by level
    pub behavioral_anchors: BehavioralAnchorsByLevel,
}

/// Behavioral anchors organized by competency level
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BehavioralAnchorsByLevel {
    /// Level 1 anchors
    pub l1: Vec<BehavioralAnchor>,
    /// Level 2 anchors
    pub l2: Vec<BehavioralAnchor>,
    /// Level 3 anchors
    pub l3: Vec<BehavioralAnchor>,
    /// Level 4 anchors
    pub l4: Vec<BehavioralAnchor>,
    /// Level 5 anchors
    pub l5: Vec<BehavioralAnchor>,
    /// Level 5+ anchors
    #[serde(rename = "L5+")]
    pub l5_plus: Vec<BehavioralAnchor>,
    /// Level 5++ anchors
    #[serde(rename = "L5++")]
    pub l5_plus_plus: Vec<BehavioralAnchor>,
}

impl BehavioralAnchorsByLevel {
    /// Get anchors for a specific level
    #[must_use]
    pub fn get(&self, level: CompetencyLevel) -> &[BehavioralAnchor] {
        match level {
            CompetencyLevel::L1 => &self.l1,
            CompetencyLevel::L2 => &self.l2,
            CompetencyLevel::L3 => &self.l3,
            CompetencyLevel::L4 => &self.l4,
            CompetencyLevel::L5 => &self.l5,
            CompetencyLevel::L5Plus => &self.l5_plus,
            CompetencyLevel::L5PlusPlus => &self.l5_plus_plus,
        }
    }
}

/// User's progress in a specific domain
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DomainProgress {
    /// The domain being tracked
    pub domain: Domain,
    /// Current competency level
    pub current_level: CompetencyLevel,
    /// IDs of achieved behavioral anchors
    pub achieved_behavioral_anchors: Vec<String>,
    /// IDs of anchors in progress
    pub in_progress_anchors: Vec<String>,
    /// Last assessment date (ISO 8601)
    pub last_assessment_date: String,
    /// Supervisor/evaluator ID
    pub assessed_by: String,
    /// Next milestone target
    pub next_milestone: NextMilestone,
}

/// Next milestone target for domain progression
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NextMilestone {
    /// Target competency level
    pub target_level: CompetencyLevel,
    /// Required anchor IDs to achieve
    pub required_anchors: Vec<String>,
    /// Estimated completion date (ISO 8601)
    pub estimated_completion: String,
}

/// Record of an achieved behavioral anchor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AchievedBehavioralAnchor {
    /// The anchor ID
    pub anchor_id: String,
    /// Date achieved (ISO 8601)
    pub achieved_date: String,
    /// Portfolio artifact IDs as evidence
    pub evidence_artifacts: Vec<String>,
    /// Assessor's user ID
    pub assessor_id: String,
    /// Assessor's feedback
    pub assessor_feedback: String,
    /// Quality assurance validator (if any)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validated_by: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_competency_level_ordering() {
        assert!(CompetencyLevel::L1 < CompetencyLevel::L2);
        assert!(CompetencyLevel::L2 < CompetencyLevel::L3);
        assert!(CompetencyLevel::L5 < CompetencyLevel::L5Plus);
        assert!(CompetencyLevel::L5Plus < CompetencyLevel::L5PlusPlus);
    }

    #[test]
    fn test_competency_level_next() {
        assert_eq!(CompetencyLevel::L1.next(), Some(CompetencyLevel::L2));
        assert_eq!(
            CompetencyLevel::L5Plus.next(),
            Some(CompetencyLevel::L5PlusPlus)
        );
        assert_eq!(CompetencyLevel::L5PlusPlus.next(), None);
    }

    #[test]
    fn test_domain_cluster() {
        assert_eq!(
            Domain::Foundations.cluster(),
            ThematicCluster::FoundationalKnowledge
        );
        assert_eq!(
            Domain::SignalDetection.cluster(),
            ThematicCluster::CoreProcess
        );
        assert_eq!(
            Domain::BenefitRisk.cluster(),
            ThematicCluster::StrategicAssessment
        );
        assert_eq!(
            Domain::LeadershipEthics.cluster(),
            ThematicCluster::IntegrationLeadership
        );
    }

    #[test]
    fn test_thematic_cluster_domains() {
        assert_eq!(ThematicCluster::FoundationalKnowledge.domains().len(), 3);
        assert_eq!(ThematicCluster::CoreProcess.domains().len(), 6);
        assert_eq!(ThematicCluster::StrategicAssessment.domains().len(), 3);
        assert_eq!(ThematicCluster::IntegrationLeadership.domains().len(), 3);
    }

    #[test]
    fn test_domain_serialization() {
        let domain = Domain::SignalDetection;
        // INVARIANT: Domain enum always serializes successfully
        let json = serde_json::to_string(&domain).ok();
        assert!(json.is_some());
        assert_eq!(json.as_deref(), Some("\"Domain_7_Signal_Detection\""));

        // INVARIANT: Valid JSON always deserializes back
        let parsed: Option<Domain> = json.and_then(|j| serde_json::from_str(&j).ok());
        assert_eq!(parsed, Some(domain));
    }
}
