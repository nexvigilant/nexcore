//! `IncidentMemory` — the adaptive learning layer wrapping [`crate::store::MemoryStore`].
//!
//! Extends the base store with automatic pattern extraction and playbook
//! generation when incidents resolve. Implements the "adaptive immune memory"
//! — learning from each resolved incident to improve future responses.
//!
//! ## Relation to `MemoryStore`
//!
//! [`crate::store::MemoryStore`] handles raw CRUD for incidents and playbooks.
//! `IncidentMemory` wraps it and adds the **learning cycle**:
//!
//! ```text
//! resolve_incident()
//!   └─ store.resolve_incident()        — mark resolved
//!   └─ extract_patterns()              — find similar history
//!       └─ create_or_update_playbook() — register or update playbook
//! ```
//!
//! ## T1 Grounding
//!
//! | Type | Primitives |
//! |------|------------|
//! | `IncidentMemory` | π (Persistence) + μ (Mapping) + → (Causality) + ς (State) |

use crate::incident::{Incident, IncidentSeverity, IncidentSignature};
use crate::playbook::{Playbook, PlaybookStep};
use crate::store::{MemoryConfig, MemoryError, MemoryStats, MemoryStore, SimilarIncident};
use nexcore_chrono::DateTime;
use nexcore_homeostasis_primitives::enums::{ActionType, HealthStatus};
use serde_json::Value;
use std::collections::HashMap;

/// The adaptive incident memory system — learning wrapper over [`MemoryStore`].
///
/// On each resolved incident, `IncidentMemory` searches for similar past
/// episodes. When the count meets the `playbook_creation_threshold`, it either
/// creates a new playbook (from common successful actions) or records a new
/// outcome against an existing one.
///
/// # Example
///
/// ```
/// use nexcore_homeostasis_memory::memory::IncidentMemory;
/// use nexcore_homeostasis_memory::incident::{IncidentSeverity, IncidentSignature};
/// use nexcore_homeostasis_primitives::enums::{ActionType, HealthStatus, StormPhase};
///
/// let mut memory = IncidentMemory::with_defaults();
/// let sig = IncidentSignature {
///     storm_phase: StormPhase::Active,
///     severity: IncidentSeverity::High,
///     peak_risk_score: 0.8,
///     peak_proportionality: 3.5,
///     self_damage: false,
///     affected_systems: vec!["api".to_string()],
///     actions_taken: vec![ActionType::Dampen],
///     trigger_sensors: vec!["error_rate".to_string()],
/// };
/// let id = memory.create_incident(sig, IncidentSeverity::High);
/// assert!(!id.is_empty());
/// ```
#[derive(Debug)]
pub struct IncidentMemory {
    store: MemoryStore,
    /// Whether pattern extraction fires on incident resolution.
    pattern_extraction_enabled: bool,
    /// Minimum similar incidents required before a playbook is created.
    playbook_creation_threshold: usize,
    /// Monotonic counter for deterministic ID generation.
    incident_counter: usize,
}

impl IncidentMemory {
    /// Create with explicit configuration.
    #[must_use]
    pub fn new(
        config: MemoryConfig,
        pattern_extraction_enabled: bool,
        playbook_creation_threshold: usize,
    ) -> Self {
        Self {
            store: MemoryStore::new(config),
            pattern_extraction_enabled,
            playbook_creation_threshold,
            incident_counter: 0,
        }
    }

    /// Create with defaults: 10 000 incident capacity, extraction enabled, threshold = 3.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(MemoryConfig::default(), true, 3)
    }

    // =========================================================================
    // Incident operations
    // =========================================================================

    /// Create and store a new active incident.
    ///
    /// Returns the generated incident ID.
    pub fn create_incident(
        &mut self,
        signature: IncidentSignature,
        severity: IncidentSeverity,
    ) -> String {
        let id = format!(
            "inc-{}-{:04}",
            DateTime::now().format("%Y%m%d%H%M%S").unwrap_or_default(),
            self.incident_counter,
        );
        self.incident_counter += 1;
        let health = severity_to_health(severity);
        let incident = Incident::new(id.clone(), signature, health);
        // Monotonic counter guarantees ID uniqueness — duplicate is structurally impossible.
        if let Err(_) = self.store.record_incident(incident) {}
        id
    }

    /// Get a stored incident by ID.
    #[must_use]
    pub fn get_incident(&self, id: &str) -> Option<&Incident> {
        self.store.get_incident(id)
    }

    /// Resolve an active incident, then trigger pattern extraction.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError::IncidentNotFound`] if `id` is unknown.
    pub fn resolve_incident(
        &mut self,
        id: &str,
        final_health: HealthStatus,
        effective: bool,
    ) -> Result<(), MemoryError> {
        self.store.resolve_incident(id, final_health, effective)?;
        if self.pattern_extraction_enabled {
            self.extract_patterns(id);
        }
        Ok(())
    }

    /// Find resolved incidents with signatures similar to the given one.
    #[must_use]
    pub fn find_similar_incidents(
        &self,
        signature: &IncidentSignature,
        threshold: Option<f64>,
    ) -> Vec<SimilarIncident> {
        self.store.find_similar(signature, threshold)
    }

    /// Return the ID of the best-matching playbook for this signature, if any.
    #[must_use]
    pub fn get_playbook_for_incident(&self, signature: &IncidentSignature) -> Option<String> {
        self.store
            .match_playbooks(signature)
            .into_iter()
            .next()
            .map(|m| m.playbook_id)
    }

    // =========================================================================
    // Statistics and insights
    // =========================================================================

    /// Aggregate statistics from the underlying store.
    #[must_use]
    pub fn stats(&self) -> MemoryStats {
        self.store.stats()
    }

    /// High-level insights from incident history.
    ///
    /// Returns a JSON object with keys:
    /// - `top_affected_systems` — up to 5 most frequently impacted systems
    /// - `avg_resolution_time_by_severity` — mean resolution seconds per severity level
    /// - `resolution_efficiency` — `{ successful, failed }` counts
    /// - `total_playbooks` / `enabled_playbooks`
    #[must_use]
    pub fn insights(&self) -> Value {
        let all = self.store.all_incidents();

        // Tally incidents per affected system.
        let mut system_counts: HashMap<String, usize> = HashMap::new();
        for inc in all {
            for sys in &inc.signature.affected_systems {
                *system_counts.entry(sys.clone()).or_insert(0) += 1;
            }
        }
        let mut top_systems: Vec<(String, usize)> = system_counts.into_iter().collect();
        top_systems.sort_by(|a, b| b.1.cmp(&a.1));
        top_systems.truncate(5);

        // Average resolution time grouped by severity.
        let mut resolution_by_severity: HashMap<String, Vec<f64>> = HashMap::new();
        for inc in all {
            if let Some(duration) = inc.duration_secs {
                let key = format!("{:?}", inc.signature.severity);
                resolution_by_severity
                    .entry(key)
                    .or_default()
                    .push(duration);
            }
        }
        let avg_times: HashMap<String, f64> = resolution_by_severity
            .iter()
            .map(|(k, v)| {
                let avg = if v.is_empty() {
                    0.0
                } else {
                    v.iter().sum::<f64>() / v.len() as f64
                };
                (k.clone(), avg)
            })
            .collect();

        let stats = self.store.stats();

        serde_json::json!({
            "top_affected_systems": top_systems,
            "avg_resolution_time_by_severity": avg_times,
            "resolution_efficiency": {
                "successful": stats.successful_resolutions,
                "failed": stats.failed_resolutions,
            },
            "total_playbooks": stats.total_playbooks,
            "enabled_playbooks": stats.enabled_playbooks,
        })
    }

    // =========================================================================
    // Private: learning cycle
    // =========================================================================

    /// Extract patterns from a recently-resolved incident.
    ///
    /// Looks up similar historical incidents. When the count reaches
    /// `playbook_creation_threshold`, delegates to `create_or_update_playbook`.
    fn extract_patterns(&mut self, incident_id: &str) {
        // Clone so we release the immutable borrow before taking a mutable one.
        let incident = match self.store.get_incident(incident_id) {
            Some(i) => i.clone(),
            None => return,
        };

        if incident.is_active() {
            return;
        }

        // Find similar resolved incidents, excluding the current one.
        let similar: Vec<SimilarIncident> = self
            .store
            .find_similar(&incident.signature, None)
            .into_iter()
            .filter(|s| s.incident.id != incident_id)
            .collect();

        if similar.len() >= self.playbook_creation_threshold {
            self.create_or_update_playbook(incident, similar);
        }
    }

    /// Create a new playbook from similar incident patterns, or record an
    /// outcome against an existing one if the signature already has a playbook.
    fn create_or_update_playbook(&mut self, incident: Incident, similar: Vec<SimilarIncident>) {
        let playbook_id = signature_playbook_id(&incident.signature);

        if self.store.get_playbook(&playbook_id).is_some() {
            // Playbook exists — record this resolution as a usage outcome.
            // Playbook ID was just confirmed present; error is impossible.
            if let Err(_) = self
                .store
                .record_playbook_outcome(&playbook_id, incident.response_effective)
            {}
            return;
        }

        // Require at least 2 successful similar incidents to build from.
        let successful_similar: Vec<&SimilarIncident> = similar
            .iter()
            .filter(|s| s.incident.response_effective)
            .collect();
        if successful_similar.len() < 2 {
            return;
        }

        // Count how often each action appeared across successful incidents.
        let mut action_counts: HashMap<ActionType, usize> = HashMap::new();
        for s in &successful_similar {
            for action in &s.incident.signature.actions_taken {
                *action_counts.entry(*action).or_insert(0) += 1;
            }
        }

        // Only include actions that appeared in >= 2 incidents, most common first.
        let mut ranked: Vec<(ActionType, usize)> = action_counts
            .into_iter()
            .filter(|(_, count)| *count >= 2)
            .collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1));

        let steps: Vec<PlaybookStep> = ranked
            .iter()
            .enumerate()
            .map(|(order, (action, _))| PlaybookStep {
                order,
                action: *action,
                description: format!("Apply {action:?}"),
                delay_secs: 0.0,
                abort_on_failure: false,
            })
            .collect();

        // Confidence grows with more evidence, capped at 0.80.
        let confidence = (0.5_f64 + similar.len() as f64 * 0.05).min(0.8);

        let name = format!(
            "Auto: {:?}/{:?}",
            incident.signature.storm_phase, incident.signature.severity
        );

        let mut playbook = Playbook::new(
            playbook_id,
            name,
            "Auto-generated from resolved incident patterns",
            incident.signature.severity,
            incident.signature,
            steps,
        );
        // Relax match threshold proportionally to confidence level.
        playbook.match_threshold = (1.0 - confidence).max(0.60);

        // Non-existence was verified above; duplicate error is impossible.
        if let Err(_) = self.store.register_playbook(playbook) {}
    }
}

// =============================================================================
// Helpers
// =============================================================================

/// Map `IncidentSeverity` to an approximate initial `HealthStatus`.
fn severity_to_health(severity: IncidentSeverity) -> HealthStatus {
    match severity {
        IncidentSeverity::Low => HealthStatus::Elevated,
        IncidentSeverity::Medium => HealthStatus::Warning,
        IncidentSeverity::High => HealthStatus::Critical,
        IncidentSeverity::Critical => HealthStatus::Emergency,
    }
}

/// Deterministic playbook ID derived from the incident signature.
///
/// Incidents with the same storm phase, severity, and affected systems
/// share the same playbook key.
fn signature_playbook_id(sig: &IncidentSignature) -> String {
    let mut systems = sig.affected_systems.clone();
    systems.sort_unstable();
    format!(
        "auto-pb-{:?}-{:?}-{}",
        sig.storm_phase,
        sig.severity,
        systems.join(",")
    )
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use nexcore_homeostasis_primitives::enums::StormPhase;

    fn make_sig(severity: IncidentSeverity) -> IncidentSignature {
        IncidentSignature {
            storm_phase: StormPhase::Active,
            severity,
            peak_risk_score: 0.75,
            peak_proportionality: 3.5,
            self_damage: false,
            affected_systems: vec!["api".into()],
            actions_taken: vec![ActionType::Dampen, ActionType::RateLimit],
            trigger_sensors: vec!["error_rate".into()],
        }
    }

    fn make_sig_system(system: &str, severity: IncidentSeverity) -> IncidentSignature {
        IncidentSignature {
            storm_phase: StormPhase::Active,
            severity,
            peak_risk_score: 0.75,
            peak_proportionality: 3.5,
            self_damage: false,
            affected_systems: vec![system.into()],
            actions_taken: vec![ActionType::Dampen],
            trigger_sensors: vec!["error_rate".into()],
        }
    }

    // ── Basic CRUD ─────────────────────────────────────────────────────────

    #[test]
    fn with_defaults_creates_empty_memory() {
        let memory = IncidentMemory::with_defaults();
        let stats = memory.stats();
        assert_eq!(stats.total_incidents, 0);
        assert_eq!(stats.total_playbooks, 0);
        assert_eq!(stats.active_incidents, 0);
    }

    #[test]
    fn create_incident_returns_id() {
        let mut memory = IncidentMemory::with_defaults();
        let id =
            memory.create_incident(make_sig(IncidentSeverity::Medium), IncidentSeverity::Medium);
        assert!(!id.is_empty());
        assert!(id.starts_with("inc-"));
    }

    #[test]
    fn create_incident_stores_it() {
        let mut memory = IncidentMemory::with_defaults();
        let id = memory.create_incident(make_sig(IncidentSeverity::High), IncidentSeverity::High);
        assert!(memory.get_incident(&id).is_some());
        assert_eq!(memory.stats().total_incidents, 1);
        assert_eq!(memory.stats().active_incidents, 1);
    }

    #[test]
    fn get_nonexistent_incident_returns_none() {
        let memory = IncidentMemory::with_defaults();
        assert!(memory.get_incident("nonexistent").is_none());
    }

    #[test]
    fn create_incident_ids_are_unique() {
        let mut memory = IncidentMemory::with_defaults();
        let id1 = memory.create_incident(make_sig(IncidentSeverity::Low), IncidentSeverity::Low);
        let id2 = memory.create_incident(make_sig(IncidentSeverity::Low), IncidentSeverity::Low);
        assert_ne!(id1, id2);
    }

    // ── Resolution lifecycle ───────────────────────────────────────────────

    #[test]
    fn resolve_incident_marks_resolved() {
        let mut memory = IncidentMemory::with_defaults();
        let id = memory.create_incident(make_sig(IncidentSeverity::High), IncidentSeverity::High);
        memory
            .resolve_incident(&id, HealthStatus::Healthy, true)
            .unwrap();
        let inc = memory.get_incident(&id).unwrap();
        assert!(!inc.is_active());
        assert!(inc.response_effective);
        assert!(inc.duration_secs.is_some());
    }

    #[test]
    fn resolve_nonexistent_incident_errors() {
        let mut memory = IncidentMemory::with_defaults();
        let result = memory.resolve_incident("nonexistent", HealthStatus::Healthy, true);
        assert!(result.is_err());
    }

    // ── Similarity search ──────────────────────────────────────────────────

    #[test]
    fn find_similar_incidents_returns_matches() {
        let mut memory = IncidentMemory::with_defaults();
        let sig = make_sig(IncidentSeverity::High);

        let id1 = memory.create_incident(sig.clone(), IncidentSeverity::High);
        memory
            .resolve_incident(&id1, HealthStatus::Healthy, true)
            .unwrap();
        let id2 = memory.create_incident(sig.clone(), IncidentSeverity::High);
        memory
            .resolve_incident(&id2, HealthStatus::Healthy, true)
            .unwrap();

        let results = memory.find_similar_incidents(&sig, Some(0.9));
        assert!(results.len() >= 2);
    }

    #[test]
    fn find_similar_excludes_active_incidents() {
        let mut memory = IncidentMemory::with_defaults();
        let sig = make_sig(IncidentSeverity::Medium);

        // Active incident — should not appear in similarity results.
        let _active_id = memory.create_incident(sig.clone(), IncidentSeverity::Medium);

        // Resolved incident — should appear.
        let id2 = memory.create_incident(sig.clone(), IncidentSeverity::Medium);
        memory
            .resolve_incident(&id2, HealthStatus::Healthy, true)
            .unwrap();

        let results = memory.find_similar_incidents(&sig, Some(0.9));
        assert_eq!(results.len(), 1);
    }

    // ── Playbook generation ────────────────────────────────────────────────

    #[test]
    fn no_playbook_created_below_threshold() {
        // threshold = 3, only 2 resolved incidents → insufficient history
        let mut memory = IncidentMemory::new(MemoryConfig::default(), true, 3);
        let sig = make_sig(IncidentSeverity::High);

        for _ in 0..2 {
            let id = memory.create_incident(sig.clone(), IncidentSeverity::High);
            memory
                .resolve_incident(&id, HealthStatus::Healthy, true)
                .unwrap();
        }

        assert_eq!(memory.stats().total_playbooks, 0);
    }

    #[test]
    fn playbook_created_at_threshold() {
        // threshold = 3; 4th resolution finds 3 similar → triggers creation
        let mut memory = IncidentMemory::new(MemoryConfig::default(), true, 3);
        let sig = make_sig(IncidentSeverity::High);

        for _ in 0..3 {
            let id = memory.create_incident(sig.clone(), IncidentSeverity::High);
            memory
                .resolve_incident(&id, HealthStatus::Healthy, true)
                .unwrap();
        }
        // No playbook yet (3rd resolution finds 2 similar, below threshold).
        assert_eq!(memory.stats().total_playbooks, 0);

        // 4th resolution finds 3 similar → playbook created.
        let id4 = memory.create_incident(sig.clone(), IncidentSeverity::High);
        memory
            .resolve_incident(&id4, HealthStatus::Healthy, true)
            .unwrap();
        assert!(memory.stats().total_playbooks > 0);
    }

    #[test]
    fn get_playbook_for_incident_after_creation() {
        let mut memory = IncidentMemory::new(MemoryConfig::default(), true, 3);
        let sig = make_sig(IncidentSeverity::High);

        for _ in 0..4 {
            let id = memory.create_incident(sig.clone(), IncidentSeverity::High);
            memory
                .resolve_incident(&id, HealthStatus::Healthy, true)
                .unwrap();
        }

        let playbook_id = memory.get_playbook_for_incident(&sig);
        assert!(playbook_id.is_some());
    }

    #[test]
    fn get_playbook_returns_none_when_no_match() {
        let memory = IncidentMemory::with_defaults();
        let sig = make_sig(IncidentSeverity::Critical);
        assert!(memory.get_playbook_for_incident(&sig).is_none());
    }

    #[test]
    fn pattern_extraction_disabled_no_playbooks() {
        // Even with threshold=1 and many incidents, extraction must be a no-op.
        let mut memory = IncidentMemory::new(MemoryConfig::default(), false, 1);
        let sig = make_sig(IncidentSeverity::High);

        for _ in 0..5 {
            let id = memory.create_incident(sig.clone(), IncidentSeverity::High);
            memory
                .resolve_incident(&id, HealthStatus::Healthy, true)
                .unwrap();
        }

        assert_eq!(memory.stats().total_playbooks, 0);
    }

    #[test]
    fn playbook_outcome_recorded_on_subsequent_resolution() {
        // threshold = 2; 3rd resolution creates playbook; 4th records outcome.
        let mut memory = IncidentMemory::new(MemoryConfig::default(), true, 2);
        let sig = make_sig(IncidentSeverity::High);

        for _ in 0..3 {
            let id = memory.create_incident(sig.clone(), IncidentSeverity::High);
            memory
                .resolve_incident(&id, HealthStatus::Healthy, true)
                .unwrap();
        }
        assert!(memory.stats().total_playbooks > 0);

        // 4th resolution updates the existing playbook, not creates a second one.
        let id = memory.create_incident(sig.clone(), IncidentSeverity::High);
        memory
            .resolve_incident(&id, HealthStatus::Healthy, true)
            .unwrap();
        assert_eq!(memory.stats().total_playbooks, 1);
    }

    // ── Statistics and insights ───────────────────────────────────────────

    #[test]
    fn stats_reflect_resolutions() {
        let mut memory = IncidentMemory::with_defaults();

        let id1 = memory.create_incident(make_sig(IncidentSeverity::High), IncidentSeverity::High);
        memory
            .resolve_incident(&id1, HealthStatus::Healthy, true)
            .unwrap();

        let id2 =
            memory.create_incident(make_sig(IncidentSeverity::Medium), IncidentSeverity::Medium);
        memory
            .resolve_incident(&id2, HealthStatus::Warning, false)
            .unwrap();

        let _id3 = memory.create_incident(make_sig(IncidentSeverity::Low), IncidentSeverity::Low);

        let stats = memory.stats();
        assert_eq!(stats.total_incidents, 3);
        assert_eq!(stats.active_incidents, 1);
        assert_eq!(stats.successful_resolutions, 1);
        assert_eq!(stats.failed_resolutions, 1);
    }

    #[test]
    fn insights_returns_valid_json_object() {
        let mut memory = IncidentMemory::with_defaults();
        let id =
            memory.create_incident(make_sig(IncidentSeverity::Medium), IncidentSeverity::Medium);
        memory
            .resolve_incident(&id, HealthStatus::Healthy, true)
            .unwrap();

        let insights = memory.insights();
        assert!(insights.is_object());
        assert!(insights.get("top_affected_systems").is_some());
        assert!(insights.get("avg_resolution_time_by_severity").is_some());
        assert!(insights.get("resolution_efficiency").is_some());
        assert!(insights.get("total_playbooks").is_some());
    }

    #[test]
    fn insights_top_affected_systems_sorted_by_frequency() {
        let mut memory = IncidentMemory::with_defaults();

        // 3× "api", 1× "db"
        for _ in 0..3 {
            let id = memory.create_incident(
                make_sig_system("api", IncidentSeverity::Medium),
                IncidentSeverity::Medium,
            );
            memory
                .resolve_incident(&id, HealthStatus::Healthy, true)
                .unwrap();
        }
        let id = memory.create_incident(
            make_sig_system("db", IncidentSeverity::Low),
            IncidentSeverity::Low,
        );
        memory
            .resolve_incident(&id, HealthStatus::Healthy, true)
            .unwrap();

        let insights = memory.insights();
        let top = insights["top_affected_systems"].as_array().unwrap();
        assert!(!top.is_empty());
        // Most frequent system appears first.
        assert_eq!(top[0][0].as_str().unwrap(), "api");
    }
}
