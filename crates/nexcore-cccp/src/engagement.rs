//! Engagement lifecycle — the container for a CCCP consulting engagement.
//!
//! An engagement moves through five phases (Collect → Assess → Plan → Implement → Follow-Up).
//! Each phase produces typed deliverables that feed the next phase.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// The five phases of the CCCP.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// P1: Discovery — gather client context, map system boundaries.
    Collect,
    /// P2: Analysis — identify and prioritize gaps using DomainStateVector.
    Assess,
    /// P3: Strategy — design evidence-based engagement plan.
    Plan,
    /// P4: Execution — implement prioritized interventions.
    Implement,
    /// P5: Monitor — evaluate outcomes, detect drift.
    FollowUp,
}

impl Phase {
    /// Phase number (1-5).
    pub const fn number(&self) -> u8 {
        match self {
            Self::Collect => 1,
            Self::Assess => 2,
            Self::Plan => 3,
            Self::Implement => 4,
            Self::FollowUp => 5,
        }
    }

    /// Next phase in sequence, if any.
    pub const fn next(&self) -> Option<Phase> {
        match self {
            Self::Collect => Some(Self::Assess),
            Self::Assess => Some(Self::Plan),
            Self::Plan => Some(Self::Implement),
            Self::Implement => Some(Self::FollowUp),
            Self::FollowUp => None,
        }
    }

    /// Primary T1 algorithm for this phase.
    pub const fn primary_algorithm(&self) -> &'static str {
        match self {
            Self::Collect => "DEFINE + DECOMPOSE",
            Self::Assess => "COMPARE + PERSPECTIVE",
            Self::Plan => "COMPOSE + ENERGY",
            Self::Implement => "COMPOSE + TRANSFER",
            Self::FollowUp => "COMPARE + PERSPECTIVE",
        }
    }

    /// Typical session count for this phase.
    pub const fn typical_sessions(&self) -> &'static str {
        match self {
            Self::Collect => "2-3",
            Self::Assess => "1-2",
            Self::Plan => "1-2",
            Self::Implement => "ongoing",
            Self::FollowUp => "scheduled cadence",
        }
    }
}

/// Status of an engagement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngagementStatus {
    /// Not yet started.
    Pending,
    /// Currently active.
    Active,
    /// Paused (client request or blocker).
    Paused,
    /// Successfully completed through Phase 5.
    Completed,
    /// Terminated before completion.
    Terminated,
}

/// A CCCP consulting engagement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engagement {
    /// Unique engagement identifier (e.g., "ENG-2026-001").
    pub id: String,
    /// Client / principal name.
    pub client: String,
    /// Current phase.
    pub phase: Phase,
    /// Engagement status.
    pub status: EngagementStatus,
    /// When the engagement was created.
    pub created_at: DateTime<Utc>,
    /// When the engagement last transitioned phases.
    pub last_transition: DateTime<Utc>,
    /// Session log — chronological record of consulting sessions.
    pub sessions: Vec<SessionRecord>,
}

/// A single consulting session within an engagement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    /// Session date.
    pub date: DateTime<Utc>,
    /// Which phase this session was in.
    pub phase: Phase,
    /// Brief summary of what was accomplished.
    pub summary: String,
    /// Deliverables produced (template doc IDs).
    pub deliverables: Vec<String>,
}

impl Engagement {
    /// Create a new engagement in Phase 1 (Collect).
    pub fn new(id: impl Into<String>, client: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: id.into(),
            client: client.into(),
            phase: Phase::Collect,
            status: EngagementStatus::Active,
            created_at: now,
            last_transition: now,
            sessions: Vec::new(),
        }
    }

    /// Advance to the next phase. Returns the new phase, or None if already at FollowUp.
    pub fn advance(&mut self) -> Option<Phase> {
        if let Some(next) = self.phase.next() {
            self.phase = next;
            self.last_transition = Utc::now();
            Some(next)
        } else {
            None
        }
    }

    /// Record a consulting session.
    pub fn record_session(&mut self, summary: impl Into<String>, deliverables: Vec<String>) {
        self.sessions.push(SessionRecord {
            date: Utc::now(),
            phase: self.phase,
            summary: summary.into(),
            deliverables,
        });
    }

    /// Complete the engagement.
    pub fn complete(&mut self) {
        self.status = EngagementStatus::Completed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_sequence() {
        assert_eq!(Phase::Collect.next(), Some(Phase::Assess));
        assert_eq!(Phase::Assess.next(), Some(Phase::Plan));
        assert_eq!(Phase::Plan.next(), Some(Phase::Implement));
        assert_eq!(Phase::Implement.next(), Some(Phase::FollowUp));
        assert_eq!(Phase::FollowUp.next(), None);
    }

    #[test]
    fn test_phase_numbers() {
        assert_eq!(Phase::Collect.number(), 1);
        assert_eq!(Phase::FollowUp.number(), 5);
    }

    #[test]
    fn test_engagement_advance() {
        let mut eng = Engagement::new("ENG-2026-001", "Acme Pharma");
        assert_eq!(eng.phase, Phase::Collect);
        assert_eq!(eng.advance(), Some(Phase::Assess));
        assert_eq!(eng.phase, Phase::Assess);
    }

    #[test]
    fn test_engagement_session_recording() {
        let mut eng = Engagement::new("ENG-2026-001", "Test Client");
        eng.record_session("Initial intake", vec!["NV-COR-SOP-001".into()]);
        assert_eq!(eng.sessions.len(), 1);
        assert_eq!(eng.sessions[0].phase, Phase::Collect);
    }
}
