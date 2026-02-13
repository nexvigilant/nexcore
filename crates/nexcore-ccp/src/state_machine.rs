//! 5-phase finite state machine for the Claude Care Process.
//!
//! # T1 Grounding
//! - σ (sequence): Phases progress in defined order
//! - ς (state): Each episode exists in exactly one phase
//! - ∂ (boundary): Transition guards enforce valid paths
//! - ρ (recursion): Any phase can loop back to Collect
//!
//! # Transition Rules
//! ```text
//! Collect ──→ Assess ──→ Plan ──→ Implement ──→ FollowUp
//!    ↑          │         │              │          │
//!    └──────────┴─────────┴──────────────┴──────────┘
//! ```
//! - Forward: any phase to its immediate next
//! - Backward: any phase can return to Collect (new information emerged)
//! - Skip forward: not allowed (must pass through each phase)

use serde::{Deserialize, Serialize};

use crate::error::CcpError;
use crate::types::Phase;

/// Record of a phase transition.
///
/// Tier: T2-C (composes Phase + timing + reason)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseTransition {
    /// Source phase.
    pub from: Phase,
    /// Target phase.
    pub to: Phase,
    /// Human-readable reason for the transition.
    pub reason: String,
    /// Epoch hours when transition occurred.
    pub timestamp: f64,
}

/// Check if a transition from `from` to `to` is valid.
///
/// Valid transitions:
/// 1. Forward to next phase (ordinal + 1)
/// 2. Backward to Collect from any phase (re-collection loop)
/// 3. Self-loop (staying in same phase) — not a transition
#[must_use]
pub fn can_transition(from: Phase, to: Phase) -> bool {
    if from == to {
        return false; // self-loop is not a transition
    }

    // Rule 1: Forward to next phase
    if let Some(next) = from.next() {
        if next == to {
            return true;
        }
    }

    // Rule 2: Backward to Collect from any phase
    if to == Phase::Collect && from != Phase::Collect {
        return true;
    }

    false
}

/// Execute a phase transition, returning a `PhaseTransition` record.
///
/// # Errors
/// Returns `CcpError::InvalidPhaseTransition` if the transition is not allowed.
pub fn execute_transition(
    from: Phase,
    to: Phase,
    reason: &str,
    timestamp: f64,
) -> Result<PhaseTransition, CcpError> {
    if !can_transition(from, to) {
        return Err(CcpError::InvalidPhaseTransition {
            from: from.to_string(),
            to: to.to_string(),
        });
    }

    Ok(PhaseTransition {
        from,
        to,
        reason: reason.to_string(),
        timestamp,
    })
}

/// Determine the optimal next phase based on readiness indicators.
///
/// Returns `None` if already at `FollowUp` (terminal phase).
#[must_use]
pub fn suggest_next_phase(current: Phase) -> Option<Phase> {
    current.next()
}

/// Count how many phases remain until FollowUp.
#[must_use]
pub fn phases_remaining(current: Phase) -> usize {
    Phase::FollowUp.ordinal() - current.ordinal()
}

/// Calculate completion percentage through the process.
#[must_use]
pub fn completion_percentage(current: Phase) -> f64 {
    current.ordinal() as f64 / Phase::FollowUp.ordinal() as f64 * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Forward transitions (valid) ──────────────────────────────────────

    #[test]
    fn forward_collect_to_assess() {
        assert!(can_transition(Phase::Collect, Phase::Assess));
    }

    #[test]
    fn forward_assess_to_plan() {
        assert!(can_transition(Phase::Assess, Phase::Plan));
    }

    #[test]
    fn forward_plan_to_implement() {
        assert!(can_transition(Phase::Plan, Phase::Implement));
    }

    #[test]
    fn forward_implement_to_followup() {
        assert!(can_transition(Phase::Implement, Phase::FollowUp));
    }

    // ── Backward to Collect (valid) ─────────────────────────────────────

    #[test]
    fn backward_assess_to_collect() {
        assert!(can_transition(Phase::Assess, Phase::Collect));
    }

    #[test]
    fn backward_plan_to_collect() {
        assert!(can_transition(Phase::Plan, Phase::Collect));
    }

    #[test]
    fn backward_implement_to_collect() {
        assert!(can_transition(Phase::Implement, Phase::Collect));
    }

    #[test]
    fn backward_followup_to_collect() {
        assert!(can_transition(Phase::FollowUp, Phase::Collect));
    }

    // ── Invalid transitions ─────────────────────────────────────────────

    #[test]
    fn skip_collect_to_plan() {
        assert!(!can_transition(Phase::Collect, Phase::Plan));
    }

    #[test]
    fn skip_collect_to_implement() {
        assert!(!can_transition(Phase::Collect, Phase::Implement));
    }

    #[test]
    fn skip_collect_to_followup() {
        assert!(!can_transition(Phase::Collect, Phase::FollowUp));
    }

    #[test]
    fn backward_plan_to_assess() {
        // Can only go back to Collect, not to Assess
        assert!(!can_transition(Phase::Plan, Phase::Assess));
    }

    #[test]
    fn self_loop_rejected() {
        assert!(!can_transition(Phase::Collect, Phase::Collect));
        assert!(!can_transition(Phase::FollowUp, Phase::FollowUp));
    }

    // ── Execute transition ──────────────────────────────────────────────

    #[test]
    fn execute_valid_transition() {
        let result = execute_transition(Phase::Collect, Phase::Assess, "context gathered", 1.0);
        assert!(result.is_ok());
        let pt = result.unwrap_or_else(|_| PhaseTransition {
            from: Phase::Collect,
            to: Phase::Collect,
            reason: String::new(),
            timestamp: 0.0,
        });
        assert_eq!(pt.from, Phase::Collect);
        assert_eq!(pt.to, Phase::Assess);
        assert_eq!(pt.reason, "context gathered");
    }

    #[test]
    fn execute_invalid_transition() {
        let result = execute_transition(Phase::Collect, Phase::FollowUp, "skip", 1.0);
        assert!(result.is_err());
    }

    // ── Helper functions ────────────────────────────────────────────────

    #[test]
    fn phases_remaining_from_collect() {
        assert_eq!(phases_remaining(Phase::Collect), 4);
    }

    #[test]
    fn phases_remaining_from_followup() {
        assert_eq!(phases_remaining(Phase::FollowUp), 0);
    }

    #[test]
    fn completion_percentage_values() {
        assert!((completion_percentage(Phase::Collect) - 0.0).abs() < f64::EPSILON);
        assert!((completion_percentage(Phase::FollowUp) - 100.0).abs() < f64::EPSILON);
        assert!((completion_percentage(Phase::Plan) - 50.0).abs() < f64::EPSILON);
    }
}
