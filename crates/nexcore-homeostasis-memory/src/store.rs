//! Incident memory store — the immune memory of the homeostasis system.
//!
//! Stores incidents and playbooks, supports similarity-based lookup,
//! and provides statistics for adaptive response tuning.
//!
//! ## T1 Grounding
//!
//! | Type | Primitives |
//! |------|------------|
//! | `MemoryStore` | π (Persistence) + μ (Mapping) + ς (State) |
//! | `MemoryStats` | N (Quantity) + Σ (Sum) |
//! | `SimilarIncident` | κ (Comparison) + N (Quantity) |

use crate::incident::{Incident, IncidentSeverity, IncidentSignature};
use crate::playbook::{Playbook, PlaybookMatch};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

/// Errors from memory store operations.
#[derive(Debug, Error)]
pub enum MemoryError {
    /// Incident with this ID already exists.
    #[error("incident '{0}' already exists")]
    DuplicateIncident(String),

    /// Playbook with this ID already exists.
    #[error("playbook '{0}' already exists")]
    DuplicatePlaybook(String),

    /// Referenced incident not found.
    #[error("incident '{0}' not found")]
    IncidentNotFound(String),

    /// Referenced playbook not found.
    #[error("playbook '{0}' not found")]
    PlaybookNotFound(String),

    /// Serialization/deserialization failure.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// A past incident matched by similarity search.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimilarIncident {
    /// The matched incident.
    pub incident: Incident,
    /// Similarity score (0.0–1.0).
    pub similarity: f64,
}

/// Aggregate statistics about stored incidents.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total incidents recorded.
    pub total_incidents: usize,
    /// Currently active (unresolved) incidents.
    pub active_incidents: usize,
    /// Incidents resolved successfully.
    pub successful_resolutions: usize,
    /// Incidents resolved unsuccessfully.
    pub failed_resolutions: usize,
    /// Average resolution time in seconds (resolved incidents only).
    pub avg_resolution_secs: f64,
    /// Count per severity level.
    pub by_severity: HashMap<String, usize>,
    /// Total playbooks registered.
    pub total_playbooks: usize,
    /// Enabled playbooks.
    pub enabled_playbooks: usize,
}

/// Configuration for the memory store.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Maximum incidents to retain (oldest evicted first).
    pub max_incidents: usize,
    /// Default similarity threshold for lookups.
    pub default_similarity_threshold: f64,
    /// Maximum similar incidents to return per query.
    pub max_similar_results: usize,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_incidents: 10_000,
            default_similarity_threshold: 0.60,
            max_similar_results: 10,
        }
    }
}

/// The incident memory store — biological immune memory for the homeostasis system.
///
/// Stores past incidents and response playbooks. Supports:
/// - Recording incidents (detection → resolution lifecycle)
/// - Similarity-based incident lookup (pattern matching against history)
/// - Playbook matching (find applicable response sequences)
/// - Statistics for adaptive tuning
///
/// ## Design Philosophy
///
/// Like biological immune memory, this store enables the system to:
/// - **Remember** past threats and responses
/// - **Recognize** similar patterns in new incidents
/// - **Recall** effective responses via playbooks
/// - **Refine** playbook success rates over time
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryStore {
    config: MemoryConfig,
    incidents: Vec<Incident>,
    playbooks: Vec<Playbook>,
}

impl MemoryStore {
    /// Create a new empty memory store.
    #[must_use]
    pub fn new(config: MemoryConfig) -> Self {
        Self {
            config,
            incidents: Vec::new(),
            playbooks: Vec::new(),
        }
    }

    /// Create a store with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(MemoryConfig::default())
    }

    // =========================================================================
    // Incident operations
    // =========================================================================

    /// Record a new incident.
    ///
    /// Returns an error if an incident with the same ID already exists.
    pub fn record_incident(&mut self, incident: Incident) -> Result<(), MemoryError> {
        if self.incidents.iter().any(|i| i.id == incident.id) {
            return Err(MemoryError::DuplicateIncident(incident.id));
        }
        self.incidents.push(incident);
        self.enforce_capacity();
        Ok(())
    }

    /// Resolve an active incident by ID.
    pub fn resolve_incident(
        &mut self,
        id: &str,
        final_health: nexcore_homeostasis_primitives::enums::HealthStatus,
        effective: bool,
    ) -> Result<(), MemoryError> {
        let incident = self
            .incidents
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or_else(|| MemoryError::IncidentNotFound(id.to_string()))?;
        incident.resolve(final_health, effective);
        Ok(())
    }

    /// Get an incident by ID.
    #[must_use]
    pub fn get_incident(&self, id: &str) -> Option<&Incident> {
        self.incidents.iter().find(|i| i.id == id)
    }

    /// Get all currently active incidents.
    #[must_use]
    pub fn active_incidents(&self) -> Vec<&Incident> {
        self.incidents.iter().filter(|i| i.is_active()).collect()
    }

    /// Find past incidents similar to the given signature.
    ///
    /// Returns up to `max_similar_results` incidents sorted by descending similarity,
    /// filtered by the given threshold (or the default if `None`).
    #[must_use]
    pub fn find_similar(
        &self,
        signature: &IncidentSignature,
        threshold: Option<f64>,
    ) -> Vec<SimilarIncident> {
        let threshold = threshold.unwrap_or(self.config.default_similarity_threshold);
        let mut matches: Vec<SimilarIncident> = self
            .incidents
            .iter()
            .filter(|i| !i.is_active()) // Only match against resolved incidents
            .map(|i| {
                let similarity = i.signature.similarity(signature);
                SimilarIncident {
                    incident: i.clone(),
                    similarity,
                }
            })
            .filter(|m| m.similarity >= threshold)
            .collect();

        // Sort descending by similarity, then by recency.
        matches.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.incident.detected_at.cmp(&a.incident.detected_at))
        });

        matches.truncate(self.config.max_similar_results);
        matches
    }

    // =========================================================================
    // Playbook operations
    // =========================================================================

    /// Register a new playbook.
    pub fn register_playbook(&mut self, playbook: Playbook) -> Result<(), MemoryError> {
        if self.playbooks.iter().any(|p| p.id == playbook.id) {
            return Err(MemoryError::DuplicatePlaybook(playbook.id));
        }
        self.playbooks.push(playbook);
        Ok(())
    }

    /// Find all playbooks that match the given incident signature.
    ///
    /// Returns matches sorted by descending similarity.
    #[must_use]
    pub fn match_playbooks(&self, signature: &IncidentSignature) -> Vec<PlaybookMatch> {
        let mut matches: Vec<PlaybookMatch> = self
            .playbooks
            .iter()
            .filter_map(|pb| pb.matches(signature))
            .collect();
        matches.sort_by(|a, b| {
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        matches
    }

    /// Get a playbook by ID.
    #[must_use]
    pub fn get_playbook(&self, id: &str) -> Option<&Playbook> {
        self.playbooks.iter().find(|p| p.id == id)
    }

    /// Get a mutable reference to a playbook by ID.
    pub fn get_playbook_mut(&mut self, id: &str) -> Option<&mut Playbook> {
        self.playbooks.iter_mut().find(|p| p.id == id)
    }

    /// Record an outcome for a playbook application.
    pub fn record_playbook_outcome(
        &mut self,
        playbook_id: &str,
        successful: bool,
    ) -> Result<(), MemoryError> {
        let pb = self
            .playbooks
            .iter_mut()
            .find(|p| p.id == playbook_id)
            .ok_or_else(|| MemoryError::PlaybookNotFound(playbook_id.to_string()))?;
        pb.record_outcome(successful);
        Ok(())
    }

    // =========================================================================
    // Statistics
    // =========================================================================

    /// Compute aggregate statistics.
    #[must_use]
    pub fn stats(&self) -> MemoryStats {
        let active = self.incidents.iter().filter(|i| i.is_active()).count();
        let resolved: Vec<&Incident> = self.incidents.iter().filter(|i| !i.is_active()).collect();
        let successful = resolved.iter().filter(|i| i.response_effective).count();
        let failed = resolved.len() - successful;

        let avg_resolution = if resolved.is_empty() {
            0.0
        } else {
            let total: f64 = resolved
                .iter()
                .filter_map(|i| i.duration_secs)
                .sum();
            let count = resolved.iter().filter(|i| i.duration_secs.is_some()).count();
            if count > 0 {
                total / count as f64
            } else {
                0.0
            }
        };

        let mut by_severity: HashMap<String, usize> = HashMap::new();
        for incident in &self.incidents {
            *by_severity
                .entry(format!("{:?}", incident.signature.severity))
                .or_insert(0) += 1;
        }

        MemoryStats {
            total_incidents: self.incidents.len(),
            active_incidents: active,
            successful_resolutions: successful,
            failed_resolutions: failed,
            avg_resolution_secs: avg_resolution,
            by_severity,
            total_playbooks: self.playbooks.len(),
            enabled_playbooks: self.playbooks.iter().filter(|p| p.enabled).count(),
        }
    }

    /// Export the store as JSON for persistence.
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::Serialization` if serialization fails.
    pub fn to_json(&self) -> Result<String, MemoryError> {
        serde_json::to_string_pretty(self).map_err(MemoryError::from)
    }

    /// Import a store from JSON.
    ///
    /// # Errors
    ///
    /// Returns `MemoryError::Serialization` if deserialization fails.
    pub fn from_json(json: &str) -> Result<Self, MemoryError> {
        serde_json::from_str(json).map_err(MemoryError::from)
    }

    // =========================================================================
    // Internal
    // =========================================================================

    /// Evict oldest resolved incidents if over capacity.
    fn enforce_capacity(&mut self) {
        if self.incidents.len() <= self.config.max_incidents {
            return;
        }
        // Only evict resolved incidents, oldest first.
        let to_remove = self.incidents.len() - self.config.max_incidents;
        let mut removed = 0;
        self.incidents.retain(|i| {
            if removed >= to_remove {
                return true;
            }
            if !i.is_active() {
                removed += 1;
                return false;
            }
            true
        });
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;
    use crate::incident::IncidentSeverity;
    use nexcore_homeostasis_primitives::enums::{ActionType, HealthStatus, StormPhase};

    fn make_signature(risk: f64, severity: IncidentSeverity) -> IncidentSignature {
        IncidentSignature {
            storm_phase: StormPhase::Active,
            severity,
            peak_risk_score: risk,
            peak_proportionality: 3.0,
            self_damage: false,
            affected_systems: vec!["api".into()],
            actions_taken: vec![ActionType::Dampen],
            trigger_sensors: vec!["error_rate".into()],
        }
    }

    fn resolved_incident(id: &str, risk: f64, severity: IncidentSeverity) -> Incident {
        let sig = make_signature(risk, severity);
        let mut inc = Incident::new(id, sig, HealthStatus::Warning);
        inc.resolve(HealthStatus::Healthy, true);
        inc
    }

    #[test]
    fn record_and_retrieve() {
        let mut store = MemoryStore::with_defaults();
        let inc = Incident::new(
            "inc-001",
            make_signature(0.8, IncidentSeverity::High),
            HealthStatus::Warning,
        );
        store.record_incident(inc).unwrap();
        assert!(store.get_incident("inc-001").is_some());
        assert_eq!(store.active_incidents().len(), 1);
    }

    #[test]
    fn duplicate_rejected() {
        let mut store = MemoryStore::with_defaults();
        let sig = make_signature(0.5, IncidentSeverity::Medium);
        store
            .record_incident(Incident::new("dup", sig.clone(), HealthStatus::Warning))
            .unwrap();
        let result = store.record_incident(Incident::new("dup", sig, HealthStatus::Warning));
        assert!(result.is_err());
    }

    #[test]
    fn resolve_and_stats() {
        let mut store = MemoryStore::with_defaults();
        store
            .record_incident(resolved_incident("r1", 0.7, IncidentSeverity::High))
            .unwrap();
        store
            .record_incident(resolved_incident("r2", 0.3, IncidentSeverity::Low))
            .unwrap();

        let stats = store.stats();
        assert_eq!(stats.total_incidents, 2);
        assert_eq!(stats.active_incidents, 0);
        assert_eq!(stats.successful_resolutions, 2);
    }

    #[test]
    fn similarity_search() {
        let mut store = MemoryStore::with_defaults();
        store
            .record_incident(resolved_incident("s1", 0.8, IncidentSeverity::High))
            .unwrap();
        store
            .record_incident(resolved_incident("s2", 0.2, IncidentSeverity::Low))
            .unwrap();

        let query = make_signature(0.75, IncidentSeverity::High);
        let results = store.find_similar(&query, Some(0.5));
        assert!(!results.is_empty());
        // First result should be the most similar (s1, risk=0.8)
        assert_eq!(results[0].incident.id, "s1");
    }

    #[test]
    fn capacity_enforcement() {
        let config = MemoryConfig {
            max_incidents: 3,
            ..Default::default()
        };
        let mut store = MemoryStore::new(config);
        for i in 0..5 {
            store
                .record_incident(resolved_incident(
                    &format!("cap-{i}"),
                    0.5,
                    IncidentSeverity::Medium,
                ))
                .unwrap();
        }
        assert!(store.incidents.len() <= 3);
    }

    #[test]
    fn json_roundtrip() {
        let mut store = MemoryStore::with_defaults();
        store
            .record_incident(resolved_incident("j1", 0.6, IncidentSeverity::Medium))
            .unwrap();
        let json = store.to_json().unwrap();
        let restored = MemoryStore::from_json(&json).unwrap();
        assert_eq!(restored.stats().total_incidents, 1);
    }

    #[test]
    fn playbook_matching() {
        use crate::playbook::{Playbook, PlaybookStep};

        let mut store = MemoryStore::with_defaults();
        let trigger = make_signature(0.8, IncidentSeverity::High);
        let pb = Playbook::new(
            "pb-api-storm",
            "API Storm Response",
            "Dampen when API escalates",
            IncidentSeverity::Medium,
            trigger.clone(),
            vec![PlaybookStep {
                order: 0,
                action: ActionType::Dampen,
                description: "Reduce concurrency".into(),
                delay_secs: 0.0,
                abort_on_failure: true,
            }],
        );
        store.register_playbook(pb).unwrap();

        let matches = store.match_playbooks(&trigger);
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].playbook_id, "pb-api-storm");
    }
}
