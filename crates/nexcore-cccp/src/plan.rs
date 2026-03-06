//! Phase 3: Plan — Intervention composition and roadmap design.
//!
//! Algorithm: COMPOSE (P₁ × P₂ at ∂ via →) + ENERGY (N(→(Δς)))
//! Templates: NV-COR-SOP-003 Strategic Engagement Plan + Implementation Roadmap
//!
//! Takes the GapAnalysis from Phase 2 and composes a prioritized intervention plan.
//! Each intervention targets specific domain gaps and maps to EPAs/CPAs.

use serde::{Deserialize, Serialize};

use nexcore_vigilance::caba::{CPACategory, DomainCategory, EPACategory, ProficiencyLevel};

use crate::assess::DomainGap;

/// Priority level for an intervention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    /// Address immediately — blocks downstream work.
    Critical,
    /// Address in current phase — significant gap.
    High,
    /// Address when resources allow.
    Medium,
    /// Nice-to-have improvement.
    Low,
}

/// A single intervention — a concrete action to close a gap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intervention {
    /// Intervention identifier (e.g., "INT-001").
    pub id: String,
    /// What needs to be done.
    pub description: String,
    /// Which domains this intervention addresses.
    pub target_domains: Vec<DomainCategory>,
    /// Target proficiency after intervention.
    pub target_level: ProficiencyLevel,
    /// EPAs this intervention enables.
    pub enables_epas: Vec<EPACategory>,
    /// CPAs this intervention contributes to.
    pub contributes_to_cpas: Vec<CPACategory>,
    /// Priority.
    pub priority: Priority,
    /// Estimated effort in consulting sessions.
    pub estimated_sessions: u8,
    /// Dependencies — other intervention IDs that must complete first.
    pub depends_on: Vec<String>,
}

/// A milestone in the implementation roadmap.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    /// Milestone name.
    pub name: String,
    /// Interventions that must be complete for this milestone.
    pub intervention_ids: Vec<String>,
    /// Success criteria — how to verify the milestone is met.
    pub success_criteria: Vec<String>,
}

/// The complete Phase 3 output — an engagement plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementPlan {
    /// Confirmed scope boundaries.
    pub scope: Vec<DomainCategory>,
    /// Ordered list of interventions.
    pub interventions: Vec<Intervention>,
    /// Milestones.
    pub milestones: Vec<Milestone>,
    /// Total estimated sessions.
    pub total_estimated_sessions: u16,
}

impl EngagementPlan {
    /// Build a scaffold plan from priority gaps.
    /// Each gap becomes one intervention targeting that domain.
    /// This is a starting point — real plans require compound interventions,
    /// cross-domain dependencies, and consultant judgment to refine.
    pub fn from_gaps(gaps: &[&DomainGap]) -> Self {
        let interventions: Vec<Intervention> = gaps
            .iter()
            .enumerate()
            .map(|(i, gap)| {
                let priority = match gap.gap {
                    4.. => Priority::Critical,
                    3 => Priority::High,
                    2 => Priority::Medium,
                    _ => Priority::Low,
                };
                Intervention {
                    id: format!("INT-{:03}", i + 1),
                    description: format!(
                        "Develop {} capability from {} to {}",
                        gap.domain.as_str(),
                        gap.current.as_str(),
                        gap.desired.as_str(),
                    ),
                    target_domains: vec![gap.domain],
                    target_level: gap.desired,
                    enables_epas: Vec::new(),
                    contributes_to_cpas: Vec::new(),
                    priority,
                    estimated_sessions: gap.gap.unsigned_abs().min(4) + 1,
                    depends_on: Vec::new(),
                }
            })
            .collect();

        let scope: Vec<DomainCategory> = interventions
            .iter()
            .flat_map(|i| i.target_domains.iter().copied())
            .collect();

        let total: u16 = interventions
            .iter()
            .map(|i| i.estimated_sessions as u16)
            .sum();

        Self {
            scope,
            interventions,
            milestones: Vec::new(),
            total_estimated_sessions: total,
        }
    }

    /// Interventions sorted by priority (critical first).
    pub fn by_priority(&self) -> Vec<&Intervention> {
        let mut sorted: Vec<&Intervention> = self.interventions.iter().collect();
        sorted.sort_by_key(|i| i.priority);
        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assess::DomainGap;
    use nexcore_vigilance::caba::ProficiencyLevel;

    #[test]
    fn test_plan_from_gaps() {
        let gaps = vec![
            DomainGap {
                domain: DomainCategory::D05SignalDetection,
                current: ProficiencyLevel::L1Novice,
                desired: ProficiencyLevel::L4Proficient,
                gap: 3,
            },
            DomainGap {
                domain: DomainCategory::D04IcsrProcessing,
                current: ProficiencyLevel::L2AdvancedBeginner,
                desired: ProficiencyLevel::L3Competent,
                gap: 1,
            },
        ];
        let gap_refs: Vec<&DomainGap> = gaps.iter().collect();
        let plan = EngagementPlan::from_gaps(&gap_refs);

        assert_eq!(plan.interventions.len(), 2);
        assert_eq!(plan.interventions[0].priority, Priority::High);
        assert_eq!(plan.interventions[1].priority, Priority::Low);
    }
}
