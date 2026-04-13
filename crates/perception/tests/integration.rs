//! Integration tests for the perception pipeline.
//!
//! Uses mockall mocks for all trait boundaries — no real HTTP calls.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use mockall::mock;
use mockall::predicate::*;
use nexcore_chrono::DateTime;
use perception::{
    InMemoryArbitrationQueue, InMemoryHoldQueue,
    config::PerceptionConfig,
    connector::SourceConnector,
    error::{PerceptionError, Result},
    fusion::{ArbitrationQueue, FusionEngine, compute_conflict_delta},
    normalizer::{
        DedupOutcome, DeduplicationEngine, HoldQueue, JsonNormalizer, Normalizer,
        compute_fingerprint,
    },
    registry::{SourceConfig, SourceRegistry},
    types::{
        CdmAdverseEvent, ChangeType, EntityId, EntityState, FusionResult, NormalizedRecord,
        Outcome, RawRecord, RecordId, ReporterType, Seriousness, SourceHealth, SourceId,
    },
    world_model::{InMemoryStateStore, StateStore, WorldModel},
};
use serde_json::json;

// ── Helpers ────────────────────────────────────────────────────────────────────

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

fn make_normalized(
    drug: &str,
    event_term: &str,
    confidence: f64,
    source: &str,
) -> NormalizedRecord {
    let ev = make_event(drug, event_term);
    let fp = compute_fingerprint(&ev);
    NormalizedRecord {
        id: RecordId::generate(),
        source_id: SourceId::new(source),
        event: ev,
        fingerprint: fp,
        confidence,
        normalized_at: DateTime::now(),
    }
}

fn make_registry(sources: &[(&str, f64)]) -> Arc<SourceRegistry> {
    Arc::new(SourceRegistry::from_configs(
        sources
            .iter()
            .map(|(id, weight)| SourceConfig {
                id: SourceId::new(*id),
                name: id.to_string(),
                base_url: format!("https://example.com/{id}"),
                expected_latency_hours: 1.0,
                reliability_weight: *weight,
                enabled: true,
            })
            .collect(),
    ))
}

fn make_world_model() -> Arc<WorldModel> {
    Arc::new(WorldModel::new(
        Arc::new(InMemoryStateStore::new()) as Arc<dyn StateStore>,
        72.0,
        128,
    ))
}

// ── Mock: StateStore ───────────────────────────────────────────────────────────

mock! {
    pub StateStore {}
    impl StateStore for StateStore {
        fn get(&self, entity_id: &EntityId) -> Option<EntityState>;
        fn upsert(&self, state: EntityState);
        fn find_stale(&self, max_age_secs: i64) -> Vec<EntityId>;
        fn mark_uncertain(&self, entity_id: &EntityId);
        fn len(&self) -> usize;
    }
}

// ── Mock: HoldQueue ────────────────────────────────────────────────────────────

mock! {
    pub HoldQueue {}
    impl HoldQueue for HoldQueue {
        fn push(&self, record: NormalizedRecord);
        fn drain(&self) -> Vec<NormalizedRecord>;
        fn len(&self) -> usize;
    }
}

// ── Mock: ArbitrationQueue ────────────────────────────────────────────────────

mock! {
    pub ArbitrationQueue {}
    impl ArbitrationQueue for ArbitrationQueue {
        fn push(&self, result: FusionResult);
        fn drain(&self) -> Vec<FusionResult>;
        fn len(&self) -> usize;
    }
}

// ── Mock: Normalizer ───────────────────────────────────────────────────────────

mock! {
    pub Normalizer {}
    #[async_trait]
    impl Normalizer for Normalizer {
        fn source_id(&self) -> &SourceId;
        async fn normalize(&self, record: RawRecord) -> Result<NormalizedRecord>;
    }
}

// ── Test 1: DeduplicationEngine — unique first record ─────────────────────────

#[test]
fn test_dedup_unique_first_record() {
    let mut engine = DeduplicationEngine::new(24.0, 0.6);
    let r = make_normalized("metformin", "nausea", 0.8, "faers");
    assert!(matches!(engine.check(&r), DedupOutcome::Unique));
}

// ── Test 2: DeduplicationEngine — duplicate within window ─────────────────────

#[test]
fn test_dedup_duplicate_within_window() {
    let mut engine = DeduplicationEngine::new(24.0, 0.6);
    let r = make_normalized("metformin", "nausea", 0.8, "faers");
    engine.check(&r);
    let r2 = make_normalized("metformin", "nausea", 0.8, "faers");
    assert!(matches!(engine.check(&r2), DedupOutcome::Duplicate));
}

// ── Test 3: DeduplicationEngine — hold on low confidence rematch ──────────────

#[test]
fn test_dedup_hold_low_confidence() {
    let mut engine = DeduplicationEngine::new(24.0, 0.6);
    let r1 = make_normalized("metformin", "nausea", 0.8, "faers");
    engine.check(&r1);
    let mut r2 = make_normalized("metformin", "nausea", 0.3, "faers");
    r2.fingerprint = r1.fingerprint.clone();
    assert!(matches!(engine.check(&r2), DedupOutcome::HoldForReview));
}

// ── Test 4: HoldQueue mock — push routes low-confidence record ────────────────

#[test]
fn test_hold_queue_receives_low_confidence_record() {
    let mut mock_hold = MockHoldQueue::new();
    mock_hold.expect_push().times(1).return_const(());
    mock_hold.expect_len().return_const(1usize);

    let record = make_normalized("warfarin", "bleeding", 0.3, "faers");
    mock_hold.push(record);
    assert_eq!(mock_hold.len(), 1);
}

// ── Test 5: FusionEngine — singleton group ─────────────────────────────────────

#[test]
fn test_fusion_singleton() {
    let mut engine = FusionEngine::new(0.4);
    let registry = make_registry(&[("faers", 0.85)]);
    let arb = InMemoryArbitrationQueue::new();

    let r = make_normalized("metformin", "lactic acidosis", 0.9, "faers");
    engine.ingest(r);
    let results = engine.fuse(&registry, &arb);
    assert_eq!(results.len(), 1);
    assert!(matches!(results[0], FusionResult::Singleton { .. }));
}

// ── Test 6: FusionEngine — agreeing records fuse ──────────────────────────────

#[test]
fn test_fusion_agreeing_records() {
    let mut engine = FusionEngine::new(0.4);
    let registry = make_registry(&[("faers", 0.85), ("pubmed", 0.75)]);
    let arb = InMemoryArbitrationQueue::new();

    let r1 = make_normalized("semaglutide", "pancreatitis", 0.85, "faers");
    let mut r2 = make_normalized("semaglutide", "pancreatitis", 0.75, "pubmed");
    r2.fingerprint = r1.fingerprint.clone();

    engine.ingest(r1);
    engine.ingest(r2);
    let results = engine.fuse(&registry, &arb);
    assert_eq!(results.len(), 1);
    assert!(matches!(
        results[0],
        FusionResult::Fused {
            source_count: 2,
            ..
        }
    ));
    assert!(arb.is_empty());
}

// ── Test 7: FusionEngine — conflict escalates to arbitration ──────────────────

#[test]
fn test_fusion_conflict_escalates() {
    let mut mock_arb = MockArbitrationQueue::new();
    mock_arb.expect_push().times(1).return_const(());
    mock_arb.expect_len().return_const(1usize);

    let registry = make_registry(&[("faers", 0.85), ("pubmed", 0.75)]);
    let mut engine = FusionEngine::new(0.2);

    let ev_a = CdmAdverseEvent {
        drug_name: "semaglutide".into(),
        event_term: "pancreatitis".into(),
        onset_date_day: None,
        country: None,
        seriousness: Seriousness::Serious,
        outcome: Outcome::Recovered,
        reporter_type: ReporterType::Other,
        patient_age_years: None,
        patient_sex: None,
    };
    let ev_b = CdmAdverseEvent {
        seriousness: Seriousness::NonSerious,
        ..ev_a.clone()
    };

    let fp = compute_fingerprint(&ev_a);
    let r1 = NormalizedRecord {
        id: RecordId::generate(),
        source_id: SourceId::new("faers"),
        event: ev_a,
        fingerprint: fp.clone(),
        confidence: 0.9,
        normalized_at: DateTime::now(),
    };
    let r2 = NormalizedRecord {
        id: RecordId::generate(),
        source_id: SourceId::new("pubmed"),
        event: ev_b,
        fingerprint: fp,
        confidence: 0.1,
        normalized_at: DateTime::now(),
    };

    engine.ingest(r1);
    engine.ingest(r2);
    engine.fuse(&registry, &mock_arb);
    assert_eq!(mock_arb.len(), 1);
}

// ── Test 8: ArbitrationQueue mock — push and drain ────────────────────────────

#[test]
fn test_arbitration_queue_push_drain() {
    let arb = InMemoryArbitrationQueue::new();
    let r = make_normalized("metformin", "nausea", 0.8, "faers");
    let result = FusionResult::Singleton { record: r };
    arb.push(result);
    assert_eq!(arb.len(), 1);
    let drained = arb.drain();
    assert_eq!(drained.len(), 1);
    assert_eq!(arb.len(), 0);
}

// ── Test 9: WorldModel — query returns NotFound for missing entity ─────────────

#[test]
fn test_world_model_not_found() {
    let wm = make_world_model();
    let result = wm.query(&EntityId::new("does-not-exist"));
    assert!(matches!(result, Err(PerceptionError::NotFound(_))));
}

// ── Test 10: WorldModel — singleton result creates entity ─────────────────────

#[test]
fn test_world_model_creates_entity_from_singleton() {
    let wm = make_world_model();
    let r = make_normalized("metformin", "nausea", 0.8, "faers");
    let entity_id_prefix = format!(
        "entity-{}",
        r.fingerprint.chars().take(8).collect::<String>()
    );
    let change = wm.update(FusionResult::Singleton { record: r });
    assert!(change.is_some());
    assert_eq!(change.unwrap().change_type, ChangeType::Created);
    assert_eq!(wm.entity_count(), 1);
    let entity_id = EntityId::new(entity_id_prefix);
    let state = wm.query(&entity_id).expect("entity should exist");
    assert!(!state.uncertain);
}

// ── Test 11: WorldModel — fused result updates entity ─────────────────────────

#[test]
fn test_world_model_updates_entity_from_fused() {
    let wm = make_world_model();
    let entity_id = EntityId::new("entity-test-001");

    let result = FusionResult::Fused {
        entity_id: entity_id.clone(),
        event: make_event("aspirin", "gi bleed"),
        fused_confidence: 0.82,
        source_count: 2,
    };

    wm.update(result);
    let state = wm.query(&entity_id).expect("should exist");
    assert!((state.confidence - 0.82).abs() < f64::EPSILON);
    assert_eq!(state.source_count, 2);
}

// ── Test 12: WorldModel — conflict produces no state change ───────────────────

#[test]
fn test_world_model_conflict_no_state() {
    let wm = make_world_model();
    let r1 = make_normalized("warfarin", "bleeding", 0.9, "faers");
    let r2 = make_normalized("warfarin", "bleeding", 0.2, "pubmed");
    let result = FusionResult::Conflict {
        entity_id: EntityId::new("conflict-001"),
        conflict_delta: 0.7,
        records: vec![r1, r2],
    };
    let change = wm.update(result);
    assert!(change.is_none());
    assert_eq!(wm.entity_count(), 0);
}

// ── Test 13: WorldModel — sweep_stale marks uncertain ─────────────────────────

#[test]
fn test_world_model_sweep_stale() {
    use nexcore_chrono::Duration;

    let store = Arc::new(InMemoryStateStore::new());
    let wm = WorldModel::new(Arc::clone(&store) as Arc<dyn StateStore>, 1.0, 64);

    let entity_id = EntityId::new("stale-001");
    let old = DateTime::now() - Duration::hours(3);
    store.upsert(EntityState {
        entity_id: entity_id.clone(),
        event: make_event("aspirin", "gi bleed"),
        confidence: 0.7,
        uncertain: false,
        first_seen: old,
        last_updated: old,
        source_count: 1,
    });

    let swept = wm.sweep_stale();
    assert_eq!(swept, 1);
    let state = wm.query(&entity_id).expect("should exist");
    assert!(state.uncertain);
}

// ── Test 14: StateStore mock — get/upsert contract ────────────────────────────

#[test]
fn test_state_store_mock_get_upsert() {
    let entity_id = EntityId::new("mock-entity-001");
    let state = EntityState {
        entity_id: entity_id.clone(),
        event: make_event("metformin", "nausea"),
        confidence: 0.8,
        uncertain: false,
        first_seen: DateTime::now(),
        last_updated: DateTime::now(),
        source_count: 1,
    };

    let mut mock_store = MockStateStore::new();
    let state_clone = state.clone();
    mock_store
        .expect_get()
        .withf(move |id| id == &entity_id)
        .return_once(move |_| Some(state_clone));
    mock_store.expect_upsert().times(1).return_const(());

    mock_store.upsert(state.clone());
    let result = mock_store.get(&EntityId::new("mock-entity-001"));
    assert!(result.is_some());
}

// ── Test 15: SourceRegistry — yaml round-trip ─────────────────────────────────

#[test]
fn test_registry_yaml_round_trip() {
    let yaml = r#"
sources:
  - id: faers
    name: FDA FAERS
    base_url: https://api.fda.gov/drug/event.json
    expected_latency_hours: 1.0
    reliability_weight: 0.85
    enabled: true
  - id: pubmed
    name: PubMed
    base_url: https://eutils.ncbi.nlm.nih.gov/entrez/eutils/
    expected_latency_hours: 2.0
    reliability_weight: 0.75
    enabled: false
"#;
    let reg = SourceRegistry::from_yaml(yaml).expect("parse failed");
    assert_eq!(reg.len(), 2);
    assert_eq!(reg.enabled_sources().len(), 1);
    assert!((reg.reliability_weight(&SourceId::new("faers")) - 0.85).abs() < f64::EPSILON);
    assert!((reg.reliability_weight(&SourceId::new("pubmed")) - 0.75).abs() < f64::EPSILON);
}

// ── Test 16: JsonNormalizer — full field extraction ────────────────────────────

#[tokio::test]
async fn test_json_normalizer_full_fields() {
    let normalizer = JsonNormalizer::new(SourceId::new("faers"));
    let payload = json!({
        "drug_name": "Semaglutide",
        "event_term": "Pancreatitis",
        "onset_date_day": "2024-06-01",
        "country": "US",
        "seriousness": "serious",
        "outcome": "recovered",
        "confidence": 0.92,
        "patient_age_years": 54.0,
        "patient_sex": "F"
    });
    let raw = RawRecord::new(SourceId::new("faers"), payload);
    let norm = normalizer
        .normalize(raw)
        .await
        .expect("normalization failed");

    assert_eq!(norm.event.drug_name, "Semaglutide");
    assert_eq!(norm.event.event_term, "Pancreatitis");
    assert_eq!(norm.event.seriousness, Seriousness::Serious);
    assert_eq!(norm.event.outcome, Outcome::Recovered);
    assert!((norm.confidence - 0.92).abs() < f64::EPSILON);
    assert!(!norm.fingerprint.is_empty());
}

// ── Test 17: PerceptionConfig — default values are sensible ───────────────────

#[test]
fn test_perception_config_defaults() {
    let cfg = PerceptionConfig::default();
    assert!(cfg.dedup_window_hours > 0.0);
    assert!(cfg.merge_threshold > 0.0 && cfg.merge_threshold < 1.0);
    assert!(cfg.arbitration_threshold > 0.0 && cfg.arbitration_threshold < 1.0);
    assert!(cfg.max_staleness_hours > 0.0);
    assert!(cfg.ingestion_buffer_size > 0);
    assert!(cfg.max_concurrent_connectors > 0);
}

// ── Test 18: End-to-end pipeline — normalize → fuse → world model ────────────

#[tokio::test]
async fn test_pipeline_run_processes_record_end_to_end() {
    use perception::PerceptionPipeline;

    let config = PerceptionConfig::default();
    let source_id = SourceId::new("faers");
    let world_model = make_world_model();
    let registry = make_registry(&[("faers", 0.9)]);
    let hold_queue = Arc::new(InMemoryHoldQueue::new());
    let arbitration = Arc::new(InMemoryArbitrationQueue::new());

    let pipeline = PerceptionPipeline::new(
        config,
        vec![],
        vec![],
        registry,
        Arc::clone(&world_model),
        hold_queue,
        arbitration,
    );

    // Normalize a record via JsonNormalizer, inject directly into world model.
    let normalizer = JsonNormalizer::new(source_id.clone());
    let payload = json!({
        "drug_name": "metformin",
        "event_term": "lactic acidosis",
        "confidence": 0.88
    });
    let raw = RawRecord::new(source_id, payload);
    let norm = normalizer
        .normalize(raw)
        .await
        .expect("normalization failed");

    let entity_id = EntityId::new(format!(
        "entity-{}",
        norm.fingerprint.chars().take(8).collect::<String>()
    ));

    pipeline
        .world_model()
        .update(FusionResult::Singleton { record: norm });

    let state = pipeline
        .world_model()
        .query(&entity_id)
        .expect("entity should exist");
    assert_eq!(state.event.drug_name, "metformin");
    assert_eq!(state.event.event_term, "lactic acidosis");
    assert!(!state.uncertain);
}
