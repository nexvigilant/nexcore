//! Incident types and signature matching.
//!
//! An incident is a recorded episode where the homeostasis system detected
//! anomalous conditions and took corrective action. Incidents carry enough
//! context to enable pattern matching against future events.
//!
//! ## T1 Grounding
//!
//! | Type | Primitives |
//! |------|------------|
//! | `Incident` | π (Persistence) + ς (State) + → (Causality) |
//! | `IncidentSignature` | κ (Comparison) + μ (Mapping) |
//! | `IncidentSeverity` | Σ (Sum) |

use chrono::{DateTime, Utc};
use nexcore_homeostasis_primitives::enums::{ActionType, HealthStatus, StormPhase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severity classification for incidents.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentSeverity {
    /// Minor deviation, self-corrected.
    Low,
    /// Significant deviation requiring active response.
    Medium,
    /// Storm-level event, circuit breakers engaged.
    High,
    /// Cascading failure, emergency shutdown invoked.
    Critical,
}

impl IncidentSeverity {
    /// Numeric weight for similarity scoring.
    #[must_use]
    pub fn weight(&self) -> f64 {
        match self {
            Self::Low => 0.25,
            Self::Medium => 0.50,
            Self::High => 0.75,
            Self::Critical => 1.0,
        }
    }
}

/// The compact signature of an incident — used for pattern matching.
///
/// Two incidents with similar signatures likely have similar root causes
/// and should trigger similar playbooks.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IncidentSignature {
    /// Storm phase at detection time.
    pub storm_phase: StormPhase,
    /// Severity classification.
    pub severity: IncidentSeverity,
    /// Peak risk score during the incident (0.0–1.0).
    pub peak_risk_score: f64,
    /// Peak proportionality ratio (response/threat).
    pub peak_proportionality: f64,
    /// Whether self-damage was detected.
    pub self_damage: bool,
    /// Names of affected subsystems.
    pub affected_systems: Vec<String>,
    /// Actions that were taken in response.
    pub actions_taken: Vec<ActionType>,
    /// Sensor types that triggered the incident.
    pub trigger_sensors: Vec<String>,
}

impl IncidentSignature {
    /// Compute similarity score between two signatures (0.0–1.0).
    ///
    /// Uses weighted feature comparison:
    /// - Storm phase match: 0.20
    /// - Severity proximity: 0.20
    /// - Risk score proximity: 0.15
    /// - Proportionality proximity: 0.15
    /// - Self-damage match: 0.10
    /// - Affected system overlap: 0.10
    /// - Action overlap: 0.10
    #[must_use]
    pub fn similarity(&self, other: &Self) -> f64 {
        let phase_score = if self.storm_phase == other.storm_phase {
            1.0
        } else {
            0.0
        };

        let severity_score = 1.0 - (self.severity.weight() - other.severity.weight()).abs();

        let risk_score = 1.0 - (self.peak_risk_score - other.peak_risk_score).abs();

        let prop_score = {
            let max = self.peak_proportionality.max(other.peak_proportionality);
            if max > 0.0 {
                1.0 - ((self.peak_proportionality - other.peak_proportionality).abs() / max)
                    .min(1.0)
            } else {
                1.0
            }
        };

        let damage_score = if self.self_damage == other.self_damage {
            1.0
        } else {
            0.0
        };

        let system_score = jaccard_similarity(&self.affected_systems, &other.affected_systems);

        let action_strs_self: Vec<String> =
            self.actions_taken.iter().map(|a| format!("{a:?}")).collect();
        let action_strs_other: Vec<String> = other
            .actions_taken
            .iter()
            .map(|a| format!("{a:?}"))
            .collect();
        let action_score = jaccard_similarity(&action_strs_self, &action_strs_other);

        phase_score * 0.20
            + severity_score * 0.20
            + risk_score * 0.15
            + prop_score * 0.15
            + damage_score * 0.10
            + system_score * 0.10
            + action_score * 0.10
    }
}

/// A fully recorded incident — the unit of immune memory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Incident {
    /// Unique identifier.
    pub id: String,
    /// When the incident was first detected.
    pub detected_at: DateTime<Utc>,
    /// When the incident was resolved (None if still active).
    pub resolved_at: Option<DateTime<Utc>>,
    /// Compact signature for matching.
    pub signature: IncidentSignature,
    /// Health status at detection time.
    pub initial_health: HealthStatus,
    /// Health status at resolution time.
    pub final_health: Option<HealthStatus>,
    /// Duration in seconds from detection to resolution.
    pub duration_secs: Option<f64>,
    /// Whether the response was effective (resolved without escalation).
    pub response_effective: bool,
    /// ID of the playbook that was applied (if any).
    pub playbook_applied: Option<String>,
    /// Free-form context tags.
    #[serde(default)]
    pub tags: HashMap<String, String>,
}

impl Incident {
    /// Create a new active incident.
    #[must_use]
    pub fn new(id: impl Into<String>, signature: IncidentSignature, health: HealthStatus) -> Self {
        Self {
            id: id.into(),
            detected_at: Utc::now(),
            resolved_at: None,
            signature,
            initial_health: health,
            final_health: None,
            duration_secs: None,
            response_effective: false,
            playbook_applied: None,
            tags: HashMap::new(),
        }
    }

    /// Mark the incident as resolved.
    pub fn resolve(&mut self, final_health: HealthStatus, effective: bool) {
        let now = Utc::now();
        self.resolved_at = Some(now);
        self.final_health = Some(final_health);
        self.response_effective = effective;
        self.duration_secs = Some((now - self.detected_at).num_milliseconds() as f64 / 1000.0);
    }

    /// Whether this incident is still active (unresolved).
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.resolved_at.is_none()
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Jaccard similarity between two string slices.
fn jaccard_similarity(a: &[String], b: &[String]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let set_a: std::collections::HashSet<&str> = a.iter().map(String::as_str).collect();
    let set_b: std::collections::HashSet<&str> = b.iter().map(String::as_str).collect();
    let intersection = set_a.intersection(&set_b).count();
    let union = set_a.union(&set_b).count();
    if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    fn sample_signature(phase: StormPhase, severity: IncidentSeverity) -> IncidentSignature {
        IncidentSignature {
            storm_phase: phase,
            severity,
            peak_risk_score: 0.7,
            peak_proportionality: 3.5,
            self_damage: false,
            affected_systems: vec!["api".into(), "db".into()],
            actions_taken: vec![ActionType::Dampen, ActionType::RateLimit],
            trigger_sensors: vec!["error_rate".into()],
        }
    }

    #[test]
    fn identical_signatures_have_perfect_similarity() {
        let sig = sample_signature(StormPhase::Active, IncidentSeverity::High);
        let score = sig.similarity(&sig);
        assert!((score - 1.0).abs() < f64::EPSILON, "score={score}");
    }

    #[test]
    fn different_signatures_have_lower_similarity() {
        let a = sample_signature(StormPhase::Active, IncidentSeverity::High);
        let b = IncidentSignature {
            storm_phase: StormPhase::Resolving,
            severity: IncidentSeverity::Low,
            peak_risk_score: 0.1,
            peak_proportionality: 1.0,
            self_damage: true,
            affected_systems: vec!["cache".into()],
            actions_taken: vec![ActionType::Alert],
            trigger_sensors: vec!["latency".into()],
        };
        let score = a.similarity(&b);
        assert!(score < 0.5, "should be dissimilar, got {score}");
    }

    #[test]
    fn incident_lifecycle() {
        let sig = sample_signature(StormPhase::Active, IncidentSeverity::Medium);
        let mut incident = Incident::new("inc-001", sig, HealthStatus::Warning);
        assert!(incident.is_active());
        assert!(incident.resolved_at.is_none());

        incident.resolve(HealthStatus::Healthy, true);
        assert!(!incident.is_active());
        assert!(incident.response_effective);
        assert!(incident.duration_secs.is_some());
    }

    #[test]
    fn severity_ordering() {
        assert!(IncidentSeverity::Low < IncidentSeverity::Medium);
        assert!(IncidentSeverity::Medium < IncidentSeverity::High);
        assert!(IncidentSeverity::High < IncidentSeverity::Critical);
    }
}
