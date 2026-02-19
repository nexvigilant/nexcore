//! # PVST — Entity Lifecycles
//!
//! Pre-built finite state machines for PV domain entities:
//! cases, signals, workflows, and submissions. Each lifecycle
//! defines the legal states and transitions for its entity type.
//!
//! ## Primitive Grounding
//!
//! | Symbol | Role      | Weight |
//! |--------|-----------|--------|
//! | ς      | State     | 0.80 (dominant) |
//! | →      | Causality | 0.10   |
//! | ∂      | Boundary  | 0.05   |
//! | ∃      | Existence | 0.05   |
//!
//! Domain FSMs are ς-dominant — they define the legal existence
//! paths for each entity type.

use serde::{Deserialize, Serialize};

use super::state::{FsmState, StateId, StateMachine, TransitionDef};
use nexcore_lex_primitiva::{GroundsTo, LexPrimitiva, PrimitiveComposition};

// ═══════════════════════════════════════════════════════════
// CASE LIFECYCLE
// ═══════════════════════════════════════════════════════════

/// Case lifecycle states.
///
/// Received → Triaged → Assessed → Closed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CaseLifecycleState {
    /// Case has been received but not reviewed.
    Received,
    /// Case has been triaged for seriousness.
    Triaged,
    /// Case has been medically assessed.
    Assessed,
    /// Case is closed.
    Closed,
}

impl CaseLifecycleState {
    /// Returns the state ID for this lifecycle state.
    #[must_use]
    pub fn id(self) -> StateId {
        match self {
            Self::Received => StateId(1),
            Self::Triaged => StateId(2),
            Self::Assessed => StateId(3),
            Self::Closed => StateId(4),
        }
    }

    /// Returns the state name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Received => "received",
            Self::Triaged => "triaged",
            Self::Assessed => "assessed",
            Self::Closed => "closed",
        }
    }
}

/// Case lifecycle events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CaseEvent {
    /// Triage the case for seriousness.
    Triage,
    /// Medically assess the case.
    Assess,
    /// Close the case.
    Close,
}

impl CaseEvent {
    /// Returns the event name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Triage => "triage",
            Self::Assess => "assess",
            Self::Close => "close",
        }
    }
}

/// Builds a case lifecycle state machine.
#[must_use]
pub fn case_lifecycle(machine_id: u64, entity_id: u64, timestamp: u64) -> StateMachine {
    let mut fsm = StateMachine::new(
        machine_id,
        "case_lifecycle",
        CaseLifecycleState::Received.id(),
        entity_id,
        timestamp,
    );

    fsm.add_state(FsmState::new(1, "received").initial());
    fsm.add_state(FsmState::new(2, "triaged"));
    fsm.add_state(FsmState::new(3, "assessed"));
    fsm.add_state(FsmState::new(4, "closed").terminal());

    fsm.add_transition(
        TransitionDef::new(
            CaseLifecycleState::Received.id(),
            CaseEvent::Triage.name(),
            CaseLifecycleState::Triaged.id(),
        )
        .with_guard("has_required_fields"),
    );
    fsm.add_transition(
        TransitionDef::new(
            CaseLifecycleState::Triaged.id(),
            CaseEvent::Assess.name(),
            CaseLifecycleState::Assessed.id(),
        )
        .with_effect("notify_medical_reviewer"),
    );
    fsm.add_transition(
        TransitionDef::new(
            CaseLifecycleState::Assessed.id(),
            CaseEvent::Close.name(),
            CaseLifecycleState::Closed.id(),
        )
        .with_guard("assessment_complete"),
    );

    fsm
}

// ═══════════════════════════════════════════════════════════
// SIGNAL LIFECYCLE
// ═══════════════════════════════════════════════════════════

/// Signal lifecycle states.
///
/// Detected → Validated → Confirmed | Refuted
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SignalLifecycleState {
    /// Signal has been statistically detected.
    Detected,
    /// Signal has been clinically validated.
    Validated,
    /// Signal has been confirmed as real.
    Confirmed,
    /// Signal has been refuted as spurious.
    Refuted,
}

impl SignalLifecycleState {
    /// Returns the state ID.
    #[must_use]
    pub fn id(self) -> StateId {
        match self {
            Self::Detected => StateId(1),
            Self::Validated => StateId(2),
            Self::Confirmed => StateId(3),
            Self::Refuted => StateId(4),
        }
    }

    /// Returns the state name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Detected => "detected",
            Self::Validated => "validated",
            Self::Confirmed => "confirmed",
            Self::Refuted => "refuted",
        }
    }
}

/// Signal lifecycle events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalEvent {
    /// Clinically validate the signal.
    Validate,
    /// Confirm the signal as real.
    Confirm,
    /// Refute the signal as spurious.
    Refute,
}

impl SignalEvent {
    /// Returns the event name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Validate => "validate",
            Self::Confirm => "confirm",
            Self::Refute => "refute",
        }
    }
}

/// Builds a signal lifecycle state machine.
#[must_use]
pub fn signal_lifecycle(machine_id: u64, entity_id: u64, timestamp: u64) -> StateMachine {
    let mut fsm = StateMachine::new(
        machine_id,
        "signal_lifecycle",
        SignalLifecycleState::Detected.id(),
        entity_id,
        timestamp,
    );

    fsm.add_state(FsmState::new(1, "detected").initial());
    fsm.add_state(FsmState::new(2, "validated"));
    fsm.add_state(FsmState::new(3, "confirmed").terminal());
    fsm.add_state(FsmState::new(4, "refuted").terminal());

    fsm.add_transition(TransitionDef::new(
        SignalLifecycleState::Detected.id(),
        SignalEvent::Validate.name(),
        SignalLifecycleState::Validated.id(),
    ));
    fsm.add_transition(TransitionDef::new(
        SignalLifecycleState::Validated.id(),
        SignalEvent::Confirm.name(),
        SignalLifecycleState::Confirmed.id(),
    ));
    fsm.add_transition(TransitionDef::new(
        SignalLifecycleState::Validated.id(),
        SignalEvent::Refute.name(),
        SignalLifecycleState::Refuted.id(),
    ));

    fsm
}

// ═══════════════════════════════════════════════════════════
// WORKFLOW LIFECYCLE
// ═══════════════════════════════════════════════════════════

/// Workflow lifecycle states.
///
/// Pending → Running → Completed | Failed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WorkflowLifecycleState {
    /// Workflow is queued but not started.
    Pending,
    /// Workflow is actively executing.
    Running,
    /// Workflow completed successfully.
    Completed,
    /// Workflow failed.
    Failed,
}

impl WorkflowLifecycleState {
    /// Returns the state ID.
    #[must_use]
    pub fn id(self) -> StateId {
        match self {
            Self::Pending => StateId(1),
            Self::Running => StateId(2),
            Self::Completed => StateId(3),
            Self::Failed => StateId(4),
        }
    }

    /// Returns the state name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

/// Workflow lifecycle events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowEvent {
    /// Start executing the workflow.
    Start,
    /// Mark the workflow as completed.
    Complete,
    /// Mark the workflow as failed.
    Fail,
    /// Retry a failed workflow (goes back to running).
    Retry,
}

impl WorkflowEvent {
    /// Returns the event name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Start => "start",
            Self::Complete => "complete",
            Self::Fail => "fail",
            Self::Retry => "retry",
        }
    }
}

/// Builds a workflow lifecycle state machine.
#[must_use]
pub fn workflow_lifecycle(machine_id: u64, entity_id: u64, timestamp: u64) -> StateMachine {
    let mut fsm = StateMachine::new(
        machine_id,
        "workflow_lifecycle",
        WorkflowLifecycleState::Pending.id(),
        entity_id,
        timestamp,
    );

    fsm.add_state(FsmState::new(1, "pending").initial());
    fsm.add_state(FsmState::new(2, "running"));
    fsm.add_state(FsmState::new(3, "completed").terminal());
    fsm.add_state(FsmState::new(4, "failed"));

    fsm.add_transition(TransitionDef::new(
        WorkflowLifecycleState::Pending.id(),
        WorkflowEvent::Start.name(),
        WorkflowLifecycleState::Running.id(),
    ));
    fsm.add_transition(TransitionDef::new(
        WorkflowLifecycleState::Running.id(),
        WorkflowEvent::Complete.name(),
        WorkflowLifecycleState::Completed.id(),
    ));
    fsm.add_transition(TransitionDef::new(
        WorkflowLifecycleState::Running.id(),
        WorkflowEvent::Fail.name(),
        WorkflowLifecycleState::Failed.id(),
    ));
    fsm.add_transition(TransitionDef::new(
        WorkflowLifecycleState::Failed.id(),
        WorkflowEvent::Retry.name(),
        WorkflowLifecycleState::Running.id(),
    ));

    fsm
}

// ═══════════════════════════════════════════════════════════
// SUBMISSION LIFECYCLE
// ═══════════════════════════════════════════════════════════

/// Submission lifecycle states.
///
/// Draft → Validated → Signed → Sent → Acknowledged
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubmissionLifecycleState {
    /// Submission is being drafted.
    Draft,
    /// Submission has been validated.
    Validated,
    /// Submission has been digitally signed.
    Signed,
    /// Submission has been sent to authority.
    Sent,
    /// Submission has been acknowledged by authority.
    Acknowledged,
}

impl SubmissionLifecycleState {
    /// Returns the state ID.
    #[must_use]
    pub fn id(self) -> StateId {
        match self {
            Self::Draft => StateId(1),
            Self::Validated => StateId(2),
            Self::Signed => StateId(3),
            Self::Sent => StateId(4),
            Self::Acknowledged => StateId(5),
        }
    }

    /// Returns the state name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Validated => "validated",
            Self::Signed => "signed",
            Self::Sent => "sent",
            Self::Acknowledged => "acknowledged",
        }
    }
}

/// Submission lifecycle events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubmissionEvent {
    /// Validate the submission content.
    Validate,
    /// Digitally sign the submission.
    Sign,
    /// Send the submission to authority.
    Send,
    /// Authority acknowledged receipt.
    Acknowledge,
}

impl SubmissionEvent {
    /// Returns the event name.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Validate => "validate",
            Self::Sign => "sign",
            Self::Send => "send",
            Self::Acknowledge => "acknowledge",
        }
    }
}

/// Builds a submission lifecycle state machine.
#[must_use]
pub fn submission_lifecycle(machine_id: u64, entity_id: u64, timestamp: u64) -> StateMachine {
    let mut fsm = StateMachine::new(
        machine_id,
        "submission_lifecycle",
        SubmissionLifecycleState::Draft.id(),
        entity_id,
        timestamp,
    );

    fsm.add_state(FsmState::new(1, "draft").initial());
    fsm.add_state(FsmState::new(2, "validated"));
    fsm.add_state(FsmState::new(3, "signed"));
    fsm.add_state(FsmState::new(4, "sent"));
    fsm.add_state(FsmState::new(5, "acknowledged").terminal());

    fsm.add_transition(
        TransitionDef::new(
            SubmissionLifecycleState::Draft.id(),
            SubmissionEvent::Validate.name(),
            SubmissionLifecycleState::Validated.id(),
        )
        .with_guard("content_valid"),
    );
    fsm.add_transition(
        TransitionDef::new(
            SubmissionLifecycleState::Validated.id(),
            SubmissionEvent::Sign.name(),
            SubmissionLifecycleState::Signed.id(),
        )
        .with_guard("authorized_signer"),
    );
    fsm.add_transition(
        TransitionDef::new(
            SubmissionLifecycleState::Signed.id(),
            SubmissionEvent::Send.name(),
            SubmissionLifecycleState::Sent.id(),
        )
        .with_effect("notify_authority"),
    );
    fsm.add_transition(TransitionDef::new(
        SubmissionLifecycleState::Sent.id(),
        SubmissionEvent::Acknowledge.name(),
        SubmissionLifecycleState::Acknowledged.id(),
    ));

    fsm
}

/// GroundsTo for lifecycle FSMs as a category.
///
/// All lifecycle FSMs share the same primitive composition:
/// ς-dominant (state) with → (causality), ∂ (boundary), ∃ (existence).
///
/// Tier: T2-C (ς + → + ∂ + ∃)
pub struct LifecycleFsm;

impl GroundsTo for LifecycleFsm {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // ς — DOMINANT: entity lifecycle states
            LexPrimitiva::Causality, // → — transitions cause state change
            LexPrimitiva::Boundary,  // ∂ — guards constrain transitions
            LexPrimitiva::Existence, // ∃ — entity must exist to have state
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::GroundingTier;

    #[test]
    fn test_lifecycle_fsm_grounding() {
        let comp = LifecycleFsm::primitive_composition();
        assert_eq!(GroundingTier::classify(&comp), GroundingTier::T2Composite);
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(comp.unique().len(), 4);
    }

    // Case lifecycle tests

    #[test]
    fn test_case_lifecycle_structure() {
        let fsm = case_lifecycle(1, 100, 1000);
        assert_eq!(fsm.name, "case_lifecycle");
        assert_eq!(fsm.state_count(), 4);
        assert_eq!(fsm.transition_def_count(), 3);
        assert_eq!(fsm.current_state(), CaseLifecycleState::Received.id());
    }

    #[test]
    fn test_case_lifecycle_full_path() {
        let mut fsm = case_lifecycle(1, 100, 1000);

        let r1 = fsm.apply_transition(CaseEvent::Triage.name(), 2000);
        assert_eq!(r1, Some(CaseLifecycleState::Triaged.id()));

        let r2 = fsm.apply_transition(CaseEvent::Assess.name(), 3000);
        assert_eq!(r2, Some(CaseLifecycleState::Assessed.id()));

        let r3 = fsm.apply_transition(CaseEvent::Close.name(), 4000);
        assert_eq!(r3, Some(CaseLifecycleState::Closed.id()));

        assert!(fsm.is_terminal());
    }

    #[test]
    fn test_case_lifecycle_invalid_skip() {
        let mut fsm = case_lifecycle(1, 100, 1000);

        // Can't skip from received directly to assessed
        let result = fsm.apply_transition(CaseEvent::Assess.name(), 2000);
        assert_eq!(result, None);
        assert_eq!(fsm.current_state(), CaseLifecycleState::Received.id());
    }

    // Signal lifecycle tests

    #[test]
    fn test_signal_lifecycle_structure() {
        let fsm = signal_lifecycle(1, 200, 1000);
        assert_eq!(fsm.name, "signal_lifecycle");
        assert_eq!(fsm.state_count(), 4);
        assert_eq!(fsm.transition_def_count(), 3);
    }

    #[test]
    fn test_signal_lifecycle_confirm_path() {
        let mut fsm = signal_lifecycle(1, 200, 1000);

        fsm.apply_transition(SignalEvent::Validate.name(), 2000);
        assert_eq!(fsm.current_state(), SignalLifecycleState::Validated.id());

        fsm.apply_transition(SignalEvent::Confirm.name(), 3000);
        assert_eq!(fsm.current_state(), SignalLifecycleState::Confirmed.id());
        assert!(fsm.is_terminal());
    }

    #[test]
    fn test_signal_lifecycle_refute_path() {
        let mut fsm = signal_lifecycle(1, 200, 1000);

        fsm.apply_transition(SignalEvent::Validate.name(), 2000);
        fsm.apply_transition(SignalEvent::Refute.name(), 3000);
        assert_eq!(fsm.current_state(), SignalLifecycleState::Refuted.id());
        assert!(fsm.is_terminal());
    }

    #[test]
    fn test_signal_lifecycle_branching() {
        let fsm = signal_lifecycle(1, 200, 1000);
        // From detected, only validate is available
        assert!(fsm.can_transition(SignalEvent::Validate.name()));
        assert!(!fsm.can_transition(SignalEvent::Confirm.name()));
        assert!(!fsm.can_transition(SignalEvent::Refute.name()));
    }

    // Workflow lifecycle tests

    #[test]
    fn test_workflow_lifecycle_structure() {
        let fsm = workflow_lifecycle(1, 300, 1000);
        assert_eq!(fsm.name, "workflow_lifecycle");
        assert_eq!(fsm.state_count(), 4);
        assert_eq!(fsm.transition_def_count(), 4); // including retry
    }

    #[test]
    fn test_workflow_lifecycle_complete_path() {
        let mut fsm = workflow_lifecycle(1, 300, 1000);

        fsm.apply_transition(WorkflowEvent::Start.name(), 2000);
        assert_eq!(fsm.current_state(), WorkflowLifecycleState::Running.id());

        fsm.apply_transition(WorkflowEvent::Complete.name(), 3000);
        assert_eq!(fsm.current_state(), WorkflowLifecycleState::Completed.id());
        assert!(fsm.is_terminal());
    }

    #[test]
    fn test_workflow_lifecycle_fail_retry() {
        let mut fsm = workflow_lifecycle(1, 300, 1000);

        fsm.apply_transition(WorkflowEvent::Start.name(), 2000);
        fsm.apply_transition(WorkflowEvent::Fail.name(), 3000);
        assert_eq!(fsm.current_state(), WorkflowLifecycleState::Failed.id());
        assert!(!fsm.is_terminal()); // failed is not terminal — can retry

        fsm.apply_transition(WorkflowEvent::Retry.name(), 4000);
        assert_eq!(fsm.current_state(), WorkflowLifecycleState::Running.id());

        fsm.apply_transition(WorkflowEvent::Complete.name(), 5000);
        assert!(fsm.is_terminal());
    }

    // Submission lifecycle tests

    #[test]
    fn test_submission_lifecycle_structure() {
        let fsm = submission_lifecycle(1, 400, 1000);
        assert_eq!(fsm.name, "submission_lifecycle");
        assert_eq!(fsm.state_count(), 5);
        assert_eq!(fsm.transition_def_count(), 4);
    }

    #[test]
    fn test_submission_lifecycle_full_path() {
        let mut fsm = submission_lifecycle(1, 400, 1000);

        fsm.apply_transition(SubmissionEvent::Validate.name(), 2000);
        assert_eq!(
            fsm.current_state(),
            SubmissionLifecycleState::Validated.id()
        );

        fsm.apply_transition(SubmissionEvent::Sign.name(), 3000);
        assert_eq!(fsm.current_state(), SubmissionLifecycleState::Signed.id());

        fsm.apply_transition(SubmissionEvent::Send.name(), 4000);
        assert_eq!(fsm.current_state(), SubmissionLifecycleState::Sent.id());

        fsm.apply_transition(SubmissionEvent::Acknowledge.name(), 5000);
        assert_eq!(
            fsm.current_state(),
            SubmissionLifecycleState::Acknowledged.id()
        );
        assert!(fsm.is_terminal());
    }
}
