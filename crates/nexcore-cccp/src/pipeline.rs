//! Typed CCCP pipeline — enforces phase transitions at compile time.
//!
//! Each phase transition consumes the prior phase's output and produces the next.
//! You cannot skip phases or call them out of order.
//!
//! ```text
//! Pipeline::new(client)
//!     .collect(system_map)      → Pipeline<Collected>
//!     .assess()                 → Pipeline<Assessed>
//!     .plan()                   → Pipeline<Planned>
//!     .begin_implementation()   → Pipeline<Implementing>
//!     .evaluate(final_state, objectives) → Pipeline<Evaluated>
//! ```

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use nexcore_vigilance::caba::{DomainStateVector, ProficiencyLevel};

use crate::assess::GapAnalysis;
use crate::collect::SystemMap;
use crate::engagement::Engagement;
use crate::follow_up::{ObjectiveEvaluation, OutcomeEvaluation};
use crate::implement::ImplementationTracker;
use crate::plan::EngagementPlan;

// ── Phase marker types (zero-sized) ──

/// Phase 1 complete — system map produced.
#[derive(Debug)]
pub struct Collected;
/// Phase 2 complete — gap analysis produced.
#[derive(Debug)]
pub struct Assessed;
/// Phase 3 complete — engagement plan produced.
#[derive(Debug)]
pub struct Planned;
/// Phase 4 in progress — tracking implementation.
#[derive(Debug)]
pub struct Implementing;
/// Phase 5 complete — outcome evaluated.
#[derive(Debug)]
pub struct Evaluated;

/// Typed CCCP pipeline that enforces phase ordering.
#[derive(Debug, Serialize, Deserialize)]
pub struct Pipeline<Phase> {
    pub engagement: Engagement,
    pub system_map: Option<SystemMap>,
    pub gap_analysis: Option<GapAnalysis>,
    pub plan: Option<EngagementPlan>,
    pub tracker: Option<ImplementationTracker>,
    pub evaluation: Option<OutcomeEvaluation>,
    #[serde(skip)]
    _phase: PhantomData<Phase>,
}

// ── Initial state (no phase marker needed, just construction) ──

impl Pipeline<()> {
    /// Start a new CCCP pipeline for a client.
    pub fn new(engagement_id: impl Into<String>, client: impl Into<String>) -> Pipeline<()> {
        Pipeline {
            engagement: Engagement::new(engagement_id, client),
            system_map: None,
            gap_analysis: None,
            plan: None,
            tracker: None,
            evaluation: None,
            _phase: PhantomData,
        }
    }

    /// Phase 1: Collect — provide the system map from intake.
    /// Transitions to `Pipeline<Collected>`.
    pub fn collect(mut self, system_map: SystemMap) -> Pipeline<Collected> {
        self.engagement.record_session(
            "Phase 1: System boundary mapping complete",
            vec!["NV-COR-SOP-001".into()],
        );
        Pipeline {
            engagement: self.engagement,
            system_map: Some(system_map),
            gap_analysis: None,
            plan: None,
            tracker: None,
            evaluation: None,
            _phase: PhantomData,
        }
    }
}

impl Pipeline<Collected> {
    /// Phase 2: Assess — compute gap analysis from current/desired state.
    /// Transitions to `Pipeline<Assessed>`.
    pub fn assess(
        mut self,
        current: DomainStateVector,
        desired: DomainStateVector,
    ) -> Pipeline<Assessed> {
        let analysis = GapAnalysis::compute(current, desired);
        self.engagement.advance();
        self.engagement.record_session(
            format!(
                "Phase 2: Gap analysis — {} domains with gaps, {:.0}% readiness",
                analysis.priority_gaps().len(),
                analysis.overall_readiness * 100.0
            ),
            vec!["NV-COR-SOP-002".into()],
        );
        Pipeline {
            engagement: self.engagement,
            system_map: self.system_map,
            gap_analysis: Some(analysis),
            plan: None,
            tracker: None,
            evaluation: None,
            _phase: PhantomData,
        }
    }
}

impl Pipeline<Assessed> {
    /// Access the gap analysis for inspection before planning.
    pub fn gap_analysis(&self) -> &GapAnalysis {
        self.gap_analysis
            .as_ref()
            .expect("Assessed pipeline always has gap_analysis")
    }

    /// Phase 3: Plan — generate engagement plan from gap analysis.
    /// Transitions to `Pipeline<Planned>`.
    pub fn plan(mut self) -> Pipeline<Planned> {
        let gaps = self.gap_analysis().priority_gaps();
        let gap_refs: Vec<&crate::assess::DomainGap> = gaps;
        let plan = EngagementPlan::from_gaps(&gap_refs);
        self.engagement.advance();
        self.engagement.record_session(
            format!(
                "Phase 3: Plan — {} interventions, est. {} sessions",
                plan.interventions.len(),
                plan.total_estimated_sessions
            ),
            vec!["NV-COR-SOP-003".into()],
        );
        Pipeline {
            engagement: self.engagement,
            system_map: self.system_map,
            gap_analysis: self.gap_analysis,
            plan: Some(plan),
            tracker: None,
            evaluation: None,
            _phase: PhantomData,
        }
    }
}

impl Pipeline<Planned> {
    /// Phase 4: Begin implementation — create tracker from plan.
    /// Transitions to `Pipeline<Implementing>`.
    pub fn begin_implementation(mut self) -> Pipeline<Implementing> {
        let ids: Vec<String> = self
            .plan
            .as_ref()
            .expect("Planned pipeline always has plan")
            .interventions
            .iter()
            .map(|i| i.id.clone())
            .collect();
        let tracker = ImplementationTracker::from_plan(&ids);
        self.engagement.advance();
        Pipeline {
            engagement: self.engagement,
            system_map: self.system_map,
            gap_analysis: self.gap_analysis,
            plan: self.plan,
            tracker: Some(tracker),
            evaluation: None,
            _phase: PhantomData,
        }
    }
}

impl Pipeline<Implementing> {
    /// Access the tracker for updates during implementation.
    pub fn tracker_mut(&mut self) -> &mut ImplementationTracker {
        self.tracker
            .as_mut()
            .expect("Implementing pipeline always has tracker")
    }

    /// Phase 5: Evaluate outcomes — compare final state to initial.
    /// Transitions to `Pipeline<Evaluated>`.
    pub fn evaluate(
        mut self,
        final_state: DomainStateVector,
        desired: DomainStateVector,
        objectives: Vec<ObjectiveEvaluation>,
    ) -> Pipeline<Evaluated> {
        let initial = self
            .gap_analysis
            .take()
            .expect("Implementing pipeline always has gap_analysis");
        let evaluation = OutcomeEvaluation::evaluate(initial, final_state, desired, objectives);
        self.engagement.advance();
        self.engagement.record_session(
            format!(
                "Phase 5: Evaluation — {:.0}% gap closure, disposition: {:?}",
                evaluation.gap_closure_rate * 100.0,
                evaluation.disposition
            ),
            vec!["NV-COR-EVL-001".into(), "NV-COR-CLO-001".into()],
        );
        self.engagement.complete();
        Pipeline {
            engagement: self.engagement,
            system_map: self.system_map,
            gap_analysis: None,
            plan: self.plan,
            tracker: self.tracker,
            evaluation: Some(evaluation),
            _phase: PhantomData,
        }
    }
}

impl Pipeline<Evaluated> {
    /// Access the final outcome evaluation.
    pub fn evaluation(&self) -> &OutcomeEvaluation {
        self.evaluation
            .as_ref()
            .expect("Evaluated pipeline always has evaluation")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collect::*;
    use crate::follow_up::Achievement;
    use nexcore_vigilance::caba::DomainCategory;

    #[test]
    fn test_full_pipeline_p1_through_p5() {
        // P1: Collect
        let system_map = SystemMap {
            context: OrganizationalContext {
                size: OrgSize::Mid,
                markets: vec!["US".into(), "EU".into()],
                product_types: vec!["small molecule".into()],
                product_count: Some(5),
                annual_icsr_volume: Some(8000),
            },
            subsystems: vec![
                Subsystem {
                    name: "Case Processing".into(),
                    domains: vec![DomainCategory::D04IcsrProcessing],
                    maturity_description: "Manual but functional".into(),
                    existence: ExistenceStatus::Exists,
                },
                Subsystem {
                    name: "Signal Detection".into(),
                    domains: vec![DomainCategory::D05SignalDetection],
                    maturity_description: "No formal process".into(),
                    existence: ExistenceStatus::Absent,
                },
            ],
            concerns: vec![Concern {
                description: "No signal detection capability".into(),
                domains: vec![DomainCategory::D05SignalDetection],
                priority: 1,
            }],
        };

        let pipeline = Pipeline::new("ENG-2026-001", "Acme Pharmaceuticals");

        // P1 → P2
        let pipeline = pipeline.collect(system_map);
        assert_eq!(pipeline.engagement.phase, crate::engagement::Phase::Collect);

        // P2: Assess
        let mut current = DomainStateVector::new([ProficiencyLevel::L2AdvancedBeginner; 15]);
        current.set(
            DomainCategory::D05SignalDetection,
            ProficiencyLevel::L1Novice,
        );
        let desired = DomainStateVector::new([ProficiencyLevel::L3Competent; 15]);

        let pipeline = pipeline.assess(current, desired.clone());
        assert_eq!(pipeline.engagement.phase, crate::engagement::Phase::Assess);
        assert!(pipeline.gap_analysis().priority_gaps().len() > 0);

        // P3: Plan
        let pipeline = pipeline.plan();
        assert_eq!(pipeline.engagement.phase, crate::engagement::Phase::Plan);

        // P4: Implement
        let mut pipeline = pipeline.begin_implementation();
        assert_eq!(
            pipeline.engagement.phase,
            crate::engagement::Phase::Implement
        );

        // Simulate completing all interventions
        for ti in pipeline.tracker_mut().interventions.iter_mut() {
            ti.status = crate::implement::InterventionStatus::Completed;
        }

        // P5: Evaluate
        let final_state = DomainStateVector::new([ProficiencyLevel::L3Competent; 15]);
        let pipeline = pipeline.evaluate(
            final_state,
            desired,
            vec![ObjectiveEvaluation {
                objective: "Establish signal detection".into(),
                achievement: Achievement::FullyAchieved,
                evidence: "Process documented, first signal detected".into(),
                domains: vec![DomainCategory::D05SignalDetection],
            }],
        );

        let eval = pipeline.evaluation();
        assert!(eval.gap_closure_rate > 0.9);
        assert_eq!(eval.disposition, crate::follow_up::Disposition::Close);
        assert_eq!(
            pipeline.engagement.status,
            crate::engagement::EngagementStatus::Completed
        );
        assert_eq!(pipeline.engagement.sessions.len(), 4); // P1 + P2 + P3 + P5
    }
}
