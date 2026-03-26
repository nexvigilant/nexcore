//! Diagnostic Engine — 6-state FSM for prompt validation pipeline.
//!
//! States: Idle → Parsing → Evaluating → Reporting → Complete | Error
//!
//! The engine tracks the diagnostic lifecycle per session. State transitions
//! are validated: only legal transitions are permitted, and each transition
//! records its timestamp.

use serde::{Deserialize, Serialize};

/// The six states of the diagnostic pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DiagnosticState {
    /// No diagnostic activity. Waiting for prompt input.
    Idle,
    /// Extracting structural features (S1-S5) and foundation features (G1-G3) from prompt.
    Parsing,
    /// Scoring all 15 PDP gates (G1-G3, S1-S5, C1-C5) against parsed features.
    Evaluating,
    /// Writing evaluation results to telemetry and synthesizing autopsy record.
    Reporting,
    /// Diagnostic cycle complete. Autopsy record written with real gate scores.
    Complete,
    /// A state transition failed. Contains recovery context.
    Error,
}

impl DiagnosticState {
    /// Returns all legal successor states from the current state.
    pub fn successors(self) -> &'static [DiagnosticState] {
        use DiagnosticState::*;
        match self {
            Idle => &[Parsing, Error],
            Parsing => &[Evaluating, Error],
            Evaluating => &[Reporting, Error],
            Reporting => &[Complete, Error],
            Complete => &[Idle],       // cycle resets
            Error => &[Idle, Parsing], // recovery: restart or retry
        }
    }

    /// Check if transitioning to `target` is legal.
    pub fn can_transition_to(self, target: DiagnosticState) -> bool {
        self.successors().contains(&target)
    }
}

impl std::fmt::Display for DiagnosticState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "IDLE"),
            Self::Parsing => write!(f, "PARSING"),
            Self::Evaluating => write!(f, "EVALUATING"),
            Self::Reporting => write!(f, "REPORTING"),
            Self::Complete => write!(f, "COMPLETE"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// A timestamped diagnostic state transition record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticTransition {
    pub from: DiagnosticState,
    pub to: DiagnosticState,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_context: Option<String>,
}

/// Session-scoped diagnostic engine tracking the current pipeline state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticEngine {
    pub session_id: String,
    pub current_state: DiagnosticState,
    pub transitions: Vec<DiagnosticTransition>,
}

impl DiagnosticEngine {
    /// Create a new engine in IDLE state for a session.
    pub fn new(session_id: String) -> Self {
        Self {
            session_id,
            current_state: DiagnosticState::Idle,
            transitions: Vec::new(),
        }
    }

    /// Attempt a state transition. Returns Err if the transition is illegal.
    pub fn transition(&mut self, target: DiagnosticState, timestamp: String) -> Result<(), String> {
        if !self.current_state.can_transition_to(target) {
            return Err(format!(
                "illegal transition: {} → {}",
                self.current_state, target
            ));
        }
        self.transitions.push(DiagnosticTransition {
            from: self.current_state,
            to: target,
            timestamp,
            error_context: None,
        });
        self.current_state = target;
        Ok(())
    }

    /// Transition to ERROR state with context about what failed.
    pub fn fail(&mut self, context: String, timestamp: String) -> Result<(), String> {
        if !self.current_state.can_transition_to(DiagnosticState::Error) {
            return Err(format!(
                "cannot transition to ERROR from {}",
                self.current_state
            ));
        }
        self.transitions.push(DiagnosticTransition {
            from: self.current_state,
            to: DiagnosticState::Error,
            timestamp,
            error_context: Some(context),
        });
        self.current_state = DiagnosticState::Error;
        Ok(())
    }

    /// Check if the engine has completed a full cycle.
    pub fn is_complete(&self) -> bool {
        self.current_state == DiagnosticState::Complete
    }

    /// Count transitions in this session's diagnostic cycle.
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_full_cycle() {
        let mut engine = DiagnosticEngine::new("test-session".into());
        assert_eq!(engine.current_state, DiagnosticState::Idle);

        engine
            .transition(DiagnosticState::Parsing, "t1".into())
            .ok();
        engine
            .transition(DiagnosticState::Evaluating, "t2".into())
            .ok();
        engine
            .transition(DiagnosticState::Reporting, "t3".into())
            .ok();
        engine
            .transition(DiagnosticState::Complete, "t4".into())
            .ok();

        assert!(engine.is_complete());
        assert_eq!(engine.transition_count(), 4);
    }

    #[test]
    fn test_illegal_transition() {
        let mut engine = DiagnosticEngine::new("test".into());
        let result = engine.transition(DiagnosticState::Complete, "t1".into());
        assert!(result.is_err());
    }

    #[test]
    fn test_error_recovery() {
        let mut engine = DiagnosticEngine::new("test".into());
        engine
            .transition(DiagnosticState::Parsing, "t1".into())
            .ok();
        engine.fail("parse failed".into(), "t2".into()).ok();
        assert_eq!(engine.current_state, DiagnosticState::Error);

        // Recover to Idle
        engine.transition(DiagnosticState::Idle, "t3".into()).ok();
        assert_eq!(engine.current_state, DiagnosticState::Idle);
    }

    #[test]
    fn test_error_retry_parsing() {
        let mut engine = DiagnosticEngine::new("test".into());
        engine
            .transition(DiagnosticState::Parsing, "t1".into())
            .ok();
        engine.fail("transient error".into(), "t2".into()).ok();
        // Retry: ERROR → PARSING is legal
        engine
            .transition(DiagnosticState::Parsing, "t3".into())
            .ok();
        assert_eq!(engine.current_state, DiagnosticState::Parsing);
    }

    #[test]
    fn test_cycle_reset() {
        let mut engine = DiagnosticEngine::new("test".into());
        engine
            .transition(DiagnosticState::Parsing, "t1".into())
            .ok();
        engine
            .transition(DiagnosticState::Evaluating, "t2".into())
            .ok();
        engine
            .transition(DiagnosticState::Reporting, "t3".into())
            .ok();
        engine
            .transition(DiagnosticState::Complete, "t4".into())
            .ok();
        // COMPLETE → IDLE resets the cycle
        engine.transition(DiagnosticState::Idle, "t5".into()).ok();
        assert_eq!(engine.current_state, DiagnosticState::Idle);
    }
}
