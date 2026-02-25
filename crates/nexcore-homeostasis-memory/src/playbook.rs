//! Response playbooks — pre-defined action sequences for known incident patterns.
//!
//! A playbook maps an [`IncidentSignature`] pattern to a sequence of recommended
//! actions. When a new incident matches a playbook's trigger signature above
//! a confidence threshold, the playbook is recommended for execution.
//!
//! ## T1 Grounding
//!
//! | Type | Primitives |
//! |------|------------|
//! | `Playbook` | σ (Sequence) + μ (Mapping) + π (Persistence) |
//! | `PlaybookStep` | → (Causality) + ς (State) |
//! | `PlaybookMatch` | κ (Comparison) + N (Quantity) |

use crate::incident::{IncidentSeverity, IncidentSignature};
use nexcore_chrono::DateTime;
use nexcore_homeostasis_primitives::enums::ActionType;
use serde::{Deserialize, Serialize};

/// A single step in a response playbook.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaybookStep {
    /// Order index (0-based).
    pub order: usize,
    /// Action to take.
    pub action: ActionType,
    /// Human-readable description of this step.
    pub description: String,
    /// Delay in seconds before executing this step (0 = immediate).
    pub delay_secs: f64,
    /// Whether to abort the playbook if this step fails.
    pub abort_on_failure: bool,
}

/// A response playbook — a named, reusable sequence of actions for a class of incidents.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Playbook {
    /// Unique identifier.
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// Description of when this playbook should be applied.
    pub description: String,
    /// Minimum severity this playbook applies to.
    pub min_severity: IncidentSeverity,
    /// Signature pattern this playbook matches against.
    pub trigger_pattern: IncidentSignature,
    /// Minimum similarity score (0.0–1.0) to trigger this playbook.
    pub match_threshold: f64,
    /// Ordered steps to execute.
    pub steps: Vec<PlaybookStep>,
    /// When this playbook was created.
    pub created_at: DateTime,
    /// How many times this playbook has been applied.
    pub application_count: u64,
    /// Success rate across all applications (0.0–1.0).
    pub success_rate: f64,
    /// Whether this playbook is active.
    pub enabled: bool,
}

impl Playbook {
    /// Create a new playbook with the given trigger pattern and steps.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
        min_severity: IncidentSeverity,
        trigger_pattern: IncidentSignature,
        steps: Vec<PlaybookStep>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: description.into(),
            min_severity,
            trigger_pattern,
            match_threshold: 0.70,
            steps,
            created_at: DateTime::now(),
            application_count: 0,
            success_rate: 0.0,
            enabled: true,
        }
    }

    /// Check whether this playbook matches the given incident signature.
    ///
    /// Returns `Some(PlaybookMatch)` if similarity >= threshold and severity >= min,
    /// otherwise `None`.
    #[must_use]
    pub fn matches(&self, signature: &IncidentSignature) -> Option<PlaybookMatch> {
        if !self.enabled {
            return None;
        }
        if signature.severity < self.min_severity {
            return None;
        }
        let similarity = self.trigger_pattern.similarity(signature);
        if similarity >= self.match_threshold {
            Some(PlaybookMatch {
                playbook_id: self.id.clone(),
                playbook_name: self.name.clone(),
                similarity,
                step_count: self.steps.len(),
            })
        } else {
            None
        }
    }

    /// Record an application outcome (updates success rate).
    pub fn record_outcome(&mut self, successful: bool) {
        let total = self.application_count as f64;
        let successes = self.success_rate * total;
        self.application_count += 1;
        let new_successes = if successful {
            successes + 1.0
        } else {
            successes
        };
        self.success_rate = new_successes / self.application_count as f64;
    }
}

/// Result of matching a playbook against an incident signature.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlaybookMatch {
    /// ID of the matched playbook.
    pub playbook_id: String,
    /// Name of the matched playbook.
    pub playbook_name: String,
    /// Similarity score that triggered the match.
    pub similarity: f64,
    /// Number of steps in the playbook.
    pub step_count: usize,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use nexcore_homeostasis_primitives::enums::StormPhase;

    fn test_signature() -> IncidentSignature {
        IncidentSignature {
            storm_phase: StormPhase::Active,
            severity: IncidentSeverity::High,
            peak_risk_score: 0.8,
            peak_proportionality: 4.0,
            self_damage: false,
            affected_systems: vec!["api".into()],
            actions_taken: vec![ActionType::Dampen],
            trigger_sensors: vec!["error_rate".into()],
        }
    }

    #[test]
    fn playbook_matches_similar_signature() {
        let trigger = test_signature();
        let pb = Playbook::new(
            "pb-001",
            "API Storm Response",
            "Dampen API when escalating",
            IncidentSeverity::Medium,
            trigger.clone(),
            vec![PlaybookStep {
                order: 0,
                action: ActionType::Dampen,
                description: "Reduce API concurrency".into(),
                delay_secs: 0.0,
                abort_on_failure: true,
            }],
        );

        let result = pb.matches(&trigger);
        assert!(result.is_some());
        let m = result.unwrap();
        assert!((m.similarity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn playbook_rejects_low_severity() {
        let trigger = test_signature();
        let pb = Playbook::new(
            "pb-002",
            "Critical Only",
            "Only for critical incidents",
            IncidentSeverity::Critical,
            trigger.clone(),
            vec![],
        );

        // High < Critical, should not match
        let result = pb.matches(&trigger);
        assert!(result.is_none());
    }

    #[test]
    fn playbook_rejects_disabled() {
        let trigger = test_signature();
        let mut pb = Playbook::new(
            "pb-003",
            "Disabled",
            "Disabled playbook",
            IncidentSeverity::Low,
            trigger.clone(),
            vec![],
        );
        pb.enabled = false;
        assert!(pb.matches(&trigger).is_none());
    }

    #[test]
    fn success_rate_tracking() {
        let trigger = test_signature();
        let mut pb = Playbook::new(
            "pb-004",
            "Track",
            "Success rate",
            IncidentSeverity::Low,
            trigger,
            vec![],
        );
        pb.record_outcome(true);
        pb.record_outcome(true);
        pb.record_outcome(false);
        assert_eq!(pb.application_count, 3);
        assert!((pb.success_rate - 2.0 / 3.0).abs() < 0.001);
    }
}
