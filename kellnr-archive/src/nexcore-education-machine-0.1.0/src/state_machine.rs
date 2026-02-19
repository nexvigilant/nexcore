//! 5-phase finite state machine for the education process.
//!
//! # T1 Grounding
//! - σ (sequence): Phases progress in defined order
//! - ς (state): Each learner exists in exactly one phase
//! - ∂ (boundary): Transition guards enforce valid paths
//! - ρ (recursion): Any phase can loop back to Discover
//!
//! # Transition Rules
//! ```text
//! Discover -> Extract -> Practice -> Assess -> Master
//!    ^           |           |           |        |
//!    +-----------+-----------+-----------+--------+  (backward to Discover)
//! ```
//! - Forward: any phase to its immediate next
//! - Backward: any phase can return to Discover (new information emerged)
//! - Skip forward: not allowed (must pass through each phase)

use serde::{Deserialize, Serialize};

use crate::error::EduError;
use crate::types::LearningPhase;

/// Record of a phase transition.
///
/// Tier: T2-C (composes LearningPhase + timing + reason)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhaseTransition {
    /// Source phase.
    pub from: LearningPhase,
    /// Target phase.
    pub to: LearningPhase,
    /// Human-readable reason for the transition.
    pub reason: String,
    /// Epoch seconds when transition occurred.
    pub timestamp: f64,
}

/// Check if a transition from `from` to `to` is valid.
///
/// Valid transitions:
/// 1. Forward to next phase (ordinal + 1)
/// 2. Backward to Discover from any phase (re-discovery loop)
/// 3. Self-loop (staying in same phase) — not a transition
#[must_use]
pub fn can_transition(from: LearningPhase, to: LearningPhase) -> bool {
    if from == to {
        return false; // self-loop is not a transition
    }

    // Rule 1: Forward to next phase
    if let Some(next) = from.next() {
        if next == to {
            return true;
        }
    }

    // Rule 2: Backward to Discover from any phase
    if to == LearningPhase::Discover && from != LearningPhase::Discover {
        return true;
    }

    false
}

/// Execute a phase transition, returning a `PhaseTransition` record.
///
/// # Errors
/// Returns `EduError::InvalidPhaseTransition` if the transition is not allowed.
pub fn execute_transition(
    from: LearningPhase,
    to: LearningPhase,
    reason: &str,
    timestamp: f64,
) -> Result<PhaseTransition, EduError> {
    if !can_transition(from, to) {
        return Err(EduError::InvalidPhaseTransition {
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

/// Determine the optimal next phase.
///
/// Returns `None` if already at Master (terminal phase).
#[must_use]
pub fn suggest_next_phase(current: LearningPhase) -> Option<LearningPhase> {
    current.next()
}

/// Count how many phases remain until Master.
#[must_use]
pub fn phases_remaining(current: LearningPhase) -> usize {
    LearningPhase::Master.ordinal() - current.ordinal()
}

/// Calculate completion percentage through the process.
#[must_use]
pub fn completion_percentage(current: LearningPhase) -> f64 {
    let master_ord = LearningPhase::Master.ordinal();
    if master_ord == 0 {
        return 100.0;
    }
    current.ordinal() as f64 / master_ord as f64 * 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Forward transitions (valid) ──────────────────────────────────
    #[test]
    fn forward_discover_to_extract() {
        assert!(can_transition(
            LearningPhase::Discover,
            LearningPhase::Extract
        ));
    }

    #[test]
    fn forward_extract_to_practice() {
        assert!(can_transition(
            LearningPhase::Extract,
            LearningPhase::Practice
        ));
    }

    #[test]
    fn forward_practice_to_assess() {
        assert!(can_transition(
            LearningPhase::Practice,
            LearningPhase::Assess
        ));
    }

    #[test]
    fn forward_assess_to_master() {
        assert!(can_transition(LearningPhase::Assess, LearningPhase::Master));
    }

    // ── Backward to Discover (valid) ─────────────────────────────────
    #[test]
    fn backward_extract_to_discover() {
        assert!(can_transition(
            LearningPhase::Extract,
            LearningPhase::Discover
        ));
    }

    #[test]
    fn backward_practice_to_discover() {
        assert!(can_transition(
            LearningPhase::Practice,
            LearningPhase::Discover
        ));
    }

    #[test]
    fn backward_assess_to_discover() {
        assert!(can_transition(
            LearningPhase::Assess,
            LearningPhase::Discover
        ));
    }

    #[test]
    fn backward_master_to_discover() {
        assert!(can_transition(
            LearningPhase::Master,
            LearningPhase::Discover
        ));
    }

    // ── Invalid transitions ─────────────────────────────────────────
    #[test]
    fn skip_discover_to_practice() {
        assert!(!can_transition(
            LearningPhase::Discover,
            LearningPhase::Practice
        ));
    }

    #[test]
    fn skip_discover_to_master() {
        assert!(!can_transition(
            LearningPhase::Discover,
            LearningPhase::Master
        ));
    }

    #[test]
    fn backward_practice_to_extract() {
        // Can only go back to Discover, not to Extract
        assert!(!can_transition(
            LearningPhase::Practice,
            LearningPhase::Extract
        ));
    }

    #[test]
    fn self_loop_rejected() {
        assert!(!can_transition(
            LearningPhase::Discover,
            LearningPhase::Discover
        ));
        assert!(!can_transition(
            LearningPhase::Master,
            LearningPhase::Master
        ));
    }

    // ── Execute transition ──────────────────────────────────────────
    #[test]
    fn execute_valid_transition() {
        let result = execute_transition(
            LearningPhase::Discover,
            LearningPhase::Extract,
            "concepts identified",
            1000.0,
        );
        assert!(result.is_ok());
        if let Ok(pt) = result {
            assert_eq!(pt.from, LearningPhase::Discover);
            assert_eq!(pt.to, LearningPhase::Extract);
            assert_eq!(pt.reason, "concepts identified");
        }
    }

    #[test]
    fn execute_invalid_transition() {
        let result = execute_transition(
            LearningPhase::Discover,
            LearningPhase::Master,
            "skip",
            1000.0,
        );
        assert!(result.is_err());
    }

    // ── Helper functions ────────────────────────────────────────────
    #[test]
    fn phases_remaining_from_discover() {
        assert_eq!(phases_remaining(LearningPhase::Discover), 4);
    }

    #[test]
    fn phases_remaining_from_master() {
        assert_eq!(phases_remaining(LearningPhase::Master), 0);
    }

    #[test]
    fn completion_percentage_values() {
        assert!((completion_percentage(LearningPhase::Discover) - 0.0).abs() < f64::EPSILON);
        assert!((completion_percentage(LearningPhase::Master) - 100.0).abs() < f64::EPSILON);
        assert!((completion_percentage(LearningPhase::Practice) - 50.0).abs() < f64::EPSILON);
    }
}
