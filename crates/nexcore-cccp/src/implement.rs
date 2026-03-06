//! Phase 4: Implement — Execution tracking, status reporting, issue management.
//!
//! Algorithm: COMPOSE + TRANSFER
//! Templates: NV-COR-TRK-001 Tracker, NV-COR-RPT-001 Status Report, NV-COR-LOG-001 Issue Log
//!
//! Tracks intervention progress against the engagement plan.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use nexcore_vigilance::caba::DomainCategory;

/// Status of a single intervention.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionStatus {
    NotStarted,
    InProgress,
    Blocked,
    Completed,
    Deferred,
}

/// Tracked intervention with progress state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedIntervention {
    /// Intervention ID from the plan.
    pub intervention_id: String,
    /// Current status.
    pub status: InterventionStatus,
    /// Sessions spent so far.
    pub sessions_spent: u8,
    /// Action log entries.
    pub actions: Vec<ActionEntry>,
    /// Blocker description, if blocked.
    pub blocker: Option<String>,
}

/// An action taken during implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEntry {
    /// When the action was taken.
    pub date: DateTime<Utc>,
    /// What was done.
    pub description: String,
    /// Outcome.
    pub outcome: String,
}

/// Issue severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// An issue or risk identified during implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Issue identifier.
    pub id: String,
    /// Description.
    pub description: String,
    /// Severity.
    pub severity: IssueSeverity,
    /// Domains affected.
    pub domains: Vec<DomainCategory>,
    /// Root cause (primitive decomposition).
    pub root_cause: Option<String>,
    /// Resolution status.
    pub resolved: bool,
    /// Date identified.
    pub identified_at: DateTime<Utc>,
    /// Date resolved, if applicable.
    pub resolved_at: Option<DateTime<Utc>>,
}

/// The Phase 4 state — an implementation tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationTracker {
    /// Tracked interventions.
    pub interventions: Vec<TrackedIntervention>,
    /// Issues and risks.
    pub issues: Vec<Issue>,
}

impl ImplementationTracker {
    /// Create a tracker from a plan's intervention IDs.
    pub fn from_plan(intervention_ids: &[String]) -> Self {
        let interventions = intervention_ids
            .iter()
            .map(|id| TrackedIntervention {
                intervention_id: id.clone(),
                status: InterventionStatus::NotStarted,
                sessions_spent: 0,
                actions: Vec::new(),
                blocker: None,
            })
            .collect();

        Self {
            interventions,
            issues: Vec::new(),
        }
    }

    /// Fraction of interventions completed.
    pub fn completion_rate(&self) -> f64 {
        let total = self.interventions.len();
        if total == 0 {
            return 1.0;
        }
        let done = self
            .interventions
            .iter()
            .filter(|i| i.status == InterventionStatus::Completed)
            .count();
        done as f64 / total as f64
    }

    /// Interventions that are blocked.
    pub fn blocked(&self) -> Vec<&TrackedIntervention> {
        self.interventions
            .iter()
            .filter(|i| i.status == InterventionStatus::Blocked)
            .collect()
    }

    /// Open (unresolved) issues.
    pub fn open_issues(&self) -> Vec<&Issue> {
        self.issues.iter().filter(|i| !i.resolved).collect()
    }

    /// Total sessions spent across all interventions.
    pub fn total_sessions_spent(&self) -> u16 {
        self.interventions
            .iter()
            .map(|i| i.sessions_spent as u16)
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracker_completion() {
        let ids = vec!["INT-001".into(), "INT-002".into(), "INT-003".into()];
        let mut tracker = ImplementationTracker::from_plan(&ids);
        assert_eq!(tracker.completion_rate(), 0.0);

        tracker.interventions[0].status = InterventionStatus::Completed;
        let rate = tracker.completion_rate();
        assert!((rate - 1.0 / 3.0).abs() < 0.01);
    }
}
