//! Layer 4 — World model.
//!
//! Maintains the current best-known state of every fused entity. Emits
//! [`ChangeEvent`] values on every state transition and marks entities
//! uncertain when they become stale.

use std::sync::Arc;

use nexcore_chrono::DateTime;

use crate::error::{PerceptionError, Result};
use crate::types::{ChangeEvent, ChangeType, EntityId, EntityState, FusionResult};

// ── State store trait ──────────────────────────────────────────────────────────

/// Pluggable storage backend for world-model entity states.
///
/// The default implementation is an in-memory `HashMap`; production use can
/// swap in a persistent backend without changing `WorldModel`.
pub trait StateStore: Send + Sync + 'static {
    /// Retrieve the current state of an entity, if present.
    fn get(&self, entity_id: &EntityId) -> Option<EntityState>;

    /// Insert or replace the state for an entity.
    fn upsert(&self, state: EntityState);

    /// Return all entity IDs whose `last_updated` is older than `max_age_secs`.
    fn find_stale(&self, max_age_secs: i64) -> Vec<EntityId>;

    /// Mark an entity as uncertain (stale flag). No-op if entity not found.
    fn mark_uncertain(&self, entity_id: &EntityId);

    /// Total number of entities in the store.
    fn len(&self) -> usize;

    /// True if the store is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ── In-memory state store ──────────────────────────────────────────────────────

/// Default in-memory implementation of [`StateStore`] backed by a `Mutex<HashMap>`.
#[derive(Debug, Default)]
pub struct InMemoryStateStore {
    inner: std::sync::Mutex<std::collections::HashMap<EntityId, EntityState>>,
}

impl InMemoryStateStore {
    /// Create a new empty in-memory store.
    pub fn new() -> Self {
        Self::default()
    }
}

impl StateStore for InMemoryStateStore {
    fn get(&self, entity_id: &EntityId) -> Option<EntityState> {
        self.inner
            .lock()
            .expect("state store mutex poisoned")
            .get(entity_id)
            .cloned()
    }

    fn upsert(&self, state: EntityState) {
        self.inner
            .lock()
            .expect("state store mutex poisoned")
            .insert(state.entity_id.clone(), state);
    }

    fn find_stale(&self, max_age_secs: i64) -> Vec<EntityId> {
        let now = DateTime::now();
        self.inner
            .lock()
            .expect("state store mutex poisoned")
            .values()
            .filter(|s| now.signed_duration_since(s.last_updated).num_seconds() > max_age_secs)
            .map(|s| s.entity_id.clone())
            .collect()
    }

    fn mark_uncertain(&self, entity_id: &EntityId) {
        let mut map = self.inner.lock().expect("state store mutex poisoned");
        if let Some(state) = map.get_mut(entity_id) {
            state.uncertain = true;
        }
    }

    fn len(&self) -> usize {
        self.inner.lock().expect("state store mutex poisoned").len()
    }
}

// ── World model ────────────────────────────────────────────────────────────────

/// Layer-4 world model — the live, queryable state of all known entities.
///
/// Updates are driven by fusion results; the model emits [`ChangeEvent`] values
/// that downstream consumers can subscribe to via `subscribe_changes`.
pub struct WorldModel {
    store: Arc<dyn StateStore>,
    /// Maximum entity age in seconds before it is considered stale.
    max_staleness_secs: i64,
    /// Broadcast channel for change events.
    change_tx: tokio::sync::broadcast::Sender<ChangeEvent>,
}

impl WorldModel {
    /// Create a world model backed by `store`.
    ///
    /// * `max_staleness_hours` — entities older than this are marked uncertain.
    /// * `channel_capacity` — broadcast channel buffer size for change events.
    pub fn new(
        store: Arc<dyn StateStore>,
        max_staleness_hours: f64,
        channel_capacity: usize,
    ) -> Self {
        let (change_tx, _) = tokio::sync::broadcast::channel(channel_capacity);
        Self {
            store,
            max_staleness_secs: (max_staleness_hours * 3600.0) as i64,
            change_tx,
        }
    }

    /// Subscribe to world-model change events.
    pub fn subscribe_changes(&self) -> tokio::sync::broadcast::Receiver<ChangeEvent> {
        self.change_tx.subscribe()
    }

    /// Apply a fusion result to the world model.
    ///
    /// Emits a [`ChangeEvent`] for each state transition. Returns the change
    /// event produced, or `None` for conflict results (already emitted via
    /// the arbitration queue).
    pub fn update(&self, result: FusionResult) -> Option<ChangeEvent> {
        match result {
            FusionResult::Singleton { record } => {
                let entity_id = EntityId::new(format!(
                    "entity-{}",
                    record.fingerprint.chars().take(8).collect::<String>()
                ));
                self.upsert_entity(entity_id, record.event, record.confidence, 1)
            }
            FusionResult::Fused {
                entity_id,
                event,
                fused_confidence,
                source_count,
            } => self.upsert_entity(entity_id, event, fused_confidence, source_count),
            FusionResult::Conflict { entity_id, .. } => {
                // Conflict is already on the arbitration queue — emit a
                // ConflictEscalated change event without touching stored state.
                let event = ChangeEvent::conflict(entity_id);
                let _ = self.change_tx.send(event.clone());
                None
            }
        }
    }

    /// Query the current state of an entity.
    ///
    /// If the entity is found but older than `max_staleness_secs`, its
    /// `uncertain` flag is set to `true` in the returned state (the store is
    /// also updated).
    pub fn query(&self, entity_id: &EntityId) -> Result<EntityState> {
        let mut state = self
            .store
            .get(entity_id)
            .ok_or_else(|| PerceptionError::NotFound(entity_id.to_string()))?;

        let now = DateTime::now();
        let age_secs = now.signed_duration_since(state.last_updated).num_seconds();
        if age_secs > self.max_staleness_secs {
            state.uncertain = true;
            self.store.mark_uncertain(entity_id);
        }

        Ok(state)
    }

    /// Sweep all entities, marking stale ones uncertain.
    ///
    /// Returns the number of entities newly marked uncertain.
    pub fn sweep_stale(&self) -> usize {
        let stale = self.store.find_stale(self.max_staleness_secs);
        let count = stale.len();
        for entity_id in &stale {
            self.store.mark_uncertain(entity_id);
            let event = ChangeEvent {
                entity_id: entity_id.clone(),
                change_type: ChangeType::MarkedUncertain,
                state_snapshot: self.store.get(entity_id),
                occurred_at: DateTime::now(),
            };
            let _ = self.change_tx.send(event);
        }
        count
    }

    /// Total number of entities in the world model.
    pub fn entity_count(&self) -> usize {
        self.store.len()
    }

    // ── Private ────────────────────────────────────────────────────────────────

    fn upsert_entity(
        &self,
        entity_id: EntityId,
        event: crate::types::CdmAdverseEvent,
        confidence: f64,
        source_count: usize,
    ) -> Option<ChangeEvent> {
        let now = DateTime::now();
        let (change_type, first_seen) = match self.store.get(&entity_id) {
            None => (ChangeType::Created, now),
            Some(existing) => (ChangeType::Updated, existing.first_seen),
        };

        let state = EntityState {
            entity_id: entity_id.clone(),
            event,
            confidence,
            uncertain: false,
            first_seen,
            last_updated: now,
            source_count,
        };

        self.store.upsert(state.clone());

        let change_event = ChangeEvent::from_state(state, change_type);
        let _ = self.change_tx.send(change_event.clone());
        Some(change_event)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CdmAdverseEvent, Outcome, ReporterType, Seriousness};
    use std::sync::Arc;

    fn make_event(drug: &str, event_term: &str) -> CdmAdverseEvent {
        CdmAdverseEvent {
            drug_name: drug.to_string(),
            event_term: event_term.to_string(),
            onset_date_day: None,
            country: None,
            seriousness: Seriousness::Unknown,
            outcome: Outcome::Unknown,
            reporter_type: ReporterType::Other,
            patient_age_years: None,
            patient_sex: None,
        }
    }

    fn make_world_model() -> WorldModel {
        WorldModel::new(
            Arc::new(InMemoryStateStore::new()) as Arc<dyn StateStore>,
            72.0,
            64,
        )
    }

    #[test]
    fn query_returns_not_found_for_missing_entity() {
        let wm = make_world_model();
        let result = wm.query(&EntityId::new("nonexistent"));
        assert!(matches!(result, Err(PerceptionError::NotFound(_))));
    }

    #[test]
    fn singleton_result_creates_entity() {
        use crate::types::{NormalizedRecord, RecordId, SourceId};
        use nexcore_chrono::DateTime;

        let wm = make_world_model();
        let event = make_event("metformin", "nausea");
        let fp = crate::normalizer::compute_fingerprint(&event);
        let record = NormalizedRecord {
            id: RecordId::generate(),
            source_id: SourceId::new("faers"),
            event,
            fingerprint: fp.clone(),
            confidence: 0.8,
            normalized_at: DateTime::now(),
        };

        let change = wm.update(FusionResult::Singleton { record });
        assert!(change.is_some());
        assert_eq!(change.unwrap().change_type, ChangeType::Created);
        assert_eq!(wm.entity_count(), 1);
    }

    #[test]
    fn fused_result_upserts_entity() {
        let wm = make_world_model();
        let entity_id = EntityId::new("entity-abc12345");
        let result = FusionResult::Fused {
            entity_id: entity_id.clone(),
            event: make_event("semaglutide", "pancreatitis"),
            fused_confidence: 0.87,
            source_count: 3,
        };

        let change = wm.update(result);
        assert!(change.is_some());

        let state = wm.query(&entity_id).expect("entity should exist");
        assert!((state.confidence - 0.87).abs() < f64::EPSILON);
        assert_eq!(state.source_count, 3);
        assert!(!state.uncertain);
    }

    #[test]
    fn conflict_result_emits_no_state_change() {
        use crate::types::{NormalizedRecord, RecordId, SourceId};
        use nexcore_chrono::DateTime;

        let wm = make_world_model();
        let entity_id = EntityId::new("conflict-abc12345");

        let make_record = |confidence: f64| {
            let ev = make_event("warfarin", "bleeding");
            let fp = crate::normalizer::compute_fingerprint(&ev);
            NormalizedRecord {
                id: RecordId::generate(),
                source_id: SourceId::new("test"),
                event: ev,
                fingerprint: fp,
                confidence,
                normalized_at: DateTime::now(),
            }
        };

        let result = FusionResult::Conflict {
            entity_id: entity_id.clone(),
            conflict_delta: 0.7,
            records: vec![make_record(0.9), make_record(0.2)],
        };

        let change = wm.update(result);
        assert!(change.is_none());
        assert_eq!(wm.entity_count(), 0);
    }

    #[test]
    fn sweep_stale_marks_old_entities_uncertain() {
        use crate::types::{EntityState, Outcome, Seriousness};
        use nexcore_chrono::Duration;

        let store = Arc::new(InMemoryStateStore::new());
        let wm = WorldModel::new(Arc::clone(&store) as Arc<dyn StateStore>, 1.0, 64); // 1 hour threshold

        // Inject a stale entity directly into the store.
        let entity_id = EntityId::new("stale-entity");
        let old_time = DateTime::now() - Duration::hours(3);
        store.upsert(EntityState {
            entity_id: entity_id.clone(),
            event: make_event("aspirin", "gi bleed"),
            confidence: 0.75,
            uncertain: false,
            first_seen: old_time,
            last_updated: old_time,
            source_count: 1,
        });

        let swept = wm.sweep_stale();
        assert_eq!(swept, 1);

        let state = wm.query(&entity_id).expect("entity should exist");
        assert!(state.uncertain);
    }
}
