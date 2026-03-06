//! Phase 5: Follow-Up — Outcome evaluation and transition/closure.
//!
//! Algorithm: COMPARE (κ) + PERSPECTIVE (∂₂(ς))
//! Templates: NV-COR-EVL-001 Outcome Evaluation, NV-COR-CLO-001 Transition/Closure
//!
//! Re-runs the gap analysis to measure delta, evaluates objective achievement,
//! and produces the transition package.

use serde::{Deserialize, Serialize};

use nexcore_vigilance::caba::{DomainCategory, DomainStateVector};

use crate::assess::GapAnalysis;

/// Achievement level for an engagement objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Achievement {
    /// 4 — Fully achieved, evidence demonstrates sustained capability.
    FullyAchieved,
    /// 3 — Substantially achieved with minor residual gaps.
    SubstantiallyAchieved,
    /// 2 — Partially achieved, significant work remains.
    PartiallyAchieved,
    /// 1 — Not achieved, capability not established.
    NotAchieved,
}

impl Achievement {
    pub const fn score(&self) -> u8 {
        match self {
            Self::FullyAchieved => 4,
            Self::SubstantiallyAchieved => 3,
            Self::PartiallyAchieved => 2,
            Self::NotAchieved => 1,
        }
    }
}

/// Evaluation of a single objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveEvaluation {
    /// Objective description.
    pub objective: String,
    /// Achievement level.
    pub achievement: Achievement,
    /// Supporting evidence.
    pub evidence: String,
    /// Domains involved.
    pub domains: Vec<DomainCategory>,
}

/// Disposition recommendation after evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    /// Close engagement — objectives met, client self-sustaining.
    Close,
    /// Continue engagement — objectives partially met, more work needed.
    Continue,
    /// Transition to new engagement — scope change required.
    TransitionNew,
    /// Pause — external blocker, resume when resolved.
    Pause,
}

/// The complete Phase 5 output — outcome evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutcomeEvaluation {
    /// Initial gap analysis (from Phase 2).
    pub initial_analysis: GapAnalysis,
    /// Final gap analysis (re-run at follow-up).
    pub final_analysis: GapAnalysis,
    /// Per-objective evaluations.
    pub objectives: Vec<ObjectiveEvaluation>,
    /// Overall gap closure (0.0 to 1.0).
    pub gap_closure_rate: f64,
    /// Disposition recommendation.
    pub disposition: Disposition,
    /// Residual gaps that remain.
    pub residual_gaps: Vec<DomainCategory>,
}

impl OutcomeEvaluation {
    /// Compute outcome evaluation by comparing initial and final state.
    pub fn evaluate(
        initial: GapAnalysis,
        final_state: DomainStateVector,
        desired: DomainStateVector,
        objectives: Vec<ObjectiveEvaluation>,
    ) -> Self {
        let final_analysis = GapAnalysis::compute(final_state, desired.clone());

        // Gap closure: how much of the original gap was closed
        let initial_gaps: i32 = initial
            .domain_gaps
            .iter()
            .map(|g| g.gap.max(0) as i32)
            .sum();
        let final_gaps: i32 = final_analysis
            .domain_gaps
            .iter()
            .map(|g| g.gap.max(0) as i32)
            .sum();
        let gap_closure_rate = if initial_gaps > 0 {
            1.0 - (final_gaps as f64 / initial_gaps as f64)
        } else {
            1.0
        };

        // Residual gaps
        let residual_gaps: Vec<DomainCategory> = final_analysis
            .domain_gaps
            .iter()
            .filter(|g| g.gap > 0)
            .map(|g| g.domain)
            .collect();

        // Auto-determine disposition
        let avg_achievement = if objectives.is_empty() {
            0.0
        } else {
            objectives
                .iter()
                .map(|o| o.achievement.score() as f64)
                .sum::<f64>()
                / objectives.len() as f64
        };

        let disposition = if avg_achievement >= 3.5 && gap_closure_rate >= 0.8 {
            Disposition::Close
        } else if avg_achievement >= 2.5 {
            Disposition::Continue
        } else if gap_closure_rate < 0.2 {
            Disposition::Pause
        } else {
            Disposition::TransitionNew
        };

        Self {
            initial_analysis: initial,
            final_analysis,
            objectives,
            gap_closure_rate,
            disposition,
            residual_gaps,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_vigilance::caba::ProficiencyLevel;

    #[test]
    fn test_full_closure_recommends_close() {
        let state = DomainStateVector::new([ProficiencyLevel::L3Competent; 15]);
        let initial = GapAnalysis::compute(
            DomainStateVector::new([ProficiencyLevel::L1Novice; 15]),
            state.clone(),
        );

        let eval = OutcomeEvaluation::evaluate(
            initial,
            state.clone(),
            state,
            vec![ObjectiveEvaluation {
                objective: "Establish signal detection".into(),
                achievement: Achievement::FullyAchieved,
                evidence: "Formal process documented and validated".into(),
                domains: vec![DomainCategory::D05SignalDetection],
            }],
        );

        assert_eq!(eval.disposition, Disposition::Close);
        assert!(eval.gap_closure_rate > 0.99);
        assert!(eval.residual_gaps.is_empty());
    }
}
