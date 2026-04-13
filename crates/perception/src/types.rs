//! CDM (Common Data Model) types for the perception pipeline.
//!
//! All data flowing through the 4-layer pipeline is expressed in these types:
//! ingestion → normalization → fusion → world model.

use nexcore_chrono::DateTime;
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

// ── Newtype identifiers ────────────────────────────────────────────────────────

/// Opaque identifier for a data source (e.g. "faers", "pubmed").
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SourceId(pub String);

impl SourceId {
    /// Create a new source identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for SourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Opaque identifier for a fused entity in the world model.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(pub String);

impl EntityId {
    /// Create a new entity identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Opaque identifier for a raw / normalized record.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RecordId(pub String);

impl RecordId {
    /// Generate a fresh random record identifier.
    pub fn generate() -> Self {
        Self(NexId::v4().to_string())
    }

    /// Create from a known string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}

impl std::fmt::Display for RecordId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// CDM schema version tag for forward-compatibility checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SchemaVersion(pub u32);

impl SchemaVersion {
    /// Current schema version used by this build.
    pub const CURRENT: Self = Self(1);
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::CURRENT
    }
}

// ── Source health ──────────────────────────────────────────────────────────────

/// Health status reported by a source connector.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SourceHealth {
    /// Connector is reachable and streaming.
    Healthy,
    /// Connector is reachable but degraded (high latency, partial data).
    Degraded {
        /// Human-readable description of the degradation.
        reason: String,
    },
    /// Connector is unreachable or erroring.
    Unhealthy {
        /// Human-readable description of the failure.
        reason: String,
    },
}

// ── Layer 1: Raw ingestion types ───────────────────────────────────────────────

/// A raw record as received from a source connector before any normalization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRecord {
    /// Unique record identifier (assigned on ingestion).
    pub id: RecordId,
    /// The source that produced this record.
    pub source_id: SourceId,
    /// Wall-clock time the record was received by the pipeline.
    pub ingested_at: DateTime,
    /// Raw payload — source-specific JSON blob.
    pub payload: serde_json::Value,
    /// Schema version of the payload.
    pub schema_version: SchemaVersion,
}

impl RawRecord {
    /// Create a new raw record with a generated ID and current timestamp.
    pub fn new(source_id: SourceId, payload: serde_json::Value) -> Self {
        Self {
            id: RecordId::generate(),
            source_id,
            ingested_at: DateTime::now(),
            payload,
            schema_version: SchemaVersion::CURRENT,
        }
    }
}

// ── CDM adverse event ──────────────────────────────────────────────────────────

/// Seriousness classification of an adverse event per ICH E2A.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Seriousness {
    /// Serious adverse event (death, life-threatening, hospitalisation, etc.).
    Serious,
    /// Non-serious adverse event.
    NonSerious,
    /// Not yet classified.
    Unknown,
}

/// Outcome of the adverse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    /// Patient recovered / resolved.
    Recovered,
    /// Patient recovered with sequelae.
    RecoveredWithSequelae,
    /// Patient not yet recovered.
    NotRecovered,
    /// Patient died.
    Fatal,
    /// Outcome unknown.
    Unknown,
}

/// Type of reporter who submitted the adverse event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReporterType {
    /// Healthcare professional (physician, pharmacist, nurse, etc.).
    HealthcareProfessional,
    /// Patient or consumer.
    Patient,
    /// Manufacturer / marketing authorisation holder.
    Manufacturer,
    /// Regulatory authority.
    RegulatoryAuthority,
    /// Other / unknown.
    Other,
}

/// A CDM-normalised adverse event record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdmAdverseEvent {
    /// Preferred drug name (INN when available).
    pub drug_name: String,
    /// Preferred event term (MedDRA PT or equivalent).
    pub event_term: String,
    /// Onset date truncated to day precision for deduplication.
    pub onset_date_day: Option<String>,
    /// Country of the report (ISO 3166-1 alpha-2).
    pub country: Option<String>,
    /// Seriousness classification.
    pub seriousness: Seriousness,
    /// Patient outcome.
    pub outcome: Outcome,
    /// Reporter type.
    pub reporter_type: ReporterType,
    /// Patient age in years (if available).
    pub patient_age_years: Option<f32>,
    /// Patient sex (M / F / U).
    pub patient_sex: Option<String>,
}

/// An entity that has been extracted and identified from an adverse event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdmEntity {
    /// Stable entity identifier.
    pub id: EntityId,
    /// Drug name (canonical).
    pub drug_name: String,
    /// Event term (canonical).
    pub event_term: String,
    /// Combined drug–event pair key for deduplication.
    pub pair_key: String,
}

impl CdmEntity {
    /// Build an entity from a normalised adverse event.
    pub fn from_adverse_event(event: &CdmAdverseEvent) -> Self {
        let pair_key = format!(
            "{}|{}",
            event.drug_name.to_lowercase(),
            event.event_term.to_lowercase()
        );
        Self {
            id: EntityId::new(NexId::v4().to_string()),
            drug_name: event.drug_name.clone(),
            event_term: event.event_term.clone(),
            pair_key,
        }
    }
}

// ── Layer 2: Normalized record ─────────────────────────────────────────────────

/// A record after normalization — CDM-aligned, deduplicated, fingerprinted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedRecord {
    /// Unique record identifier (propagated from ingestion).
    pub id: RecordId,
    /// Source that produced this record.
    pub source_id: SourceId,
    /// Normalised adverse event data.
    pub event: CdmAdverseEvent,
    /// SHA-256 fingerprint for deduplication (`drug|event|date|country`).
    pub fingerprint: String,
    /// Normalizer confidence in this record (0.0–1.0).
    pub confidence: f64,
    /// Wall-clock time of normalization.
    pub normalized_at: DateTime,
}

// ── Layer 3: Fusion result ─────────────────────────────────────────────────────

/// Result produced by the fusion engine for a group of normalized records.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FusionResult {
    /// Only one record with this fingerprint — pass through directly.
    Singleton {
        /// The single normalised record.
        record: NormalizedRecord,
    },
    /// Multiple records merged into one high-confidence entity.
    Fused {
        /// Entity identifier for the merged result.
        entity_id: EntityId,
        /// Merged adverse event (highest-confidence source wins categoricals).
        event: CdmAdverseEvent,
        /// Weighted fused confidence score.
        fused_confidence: f64,
        /// Number of source records that contributed to this fusion.
        source_count: usize,
    },
    /// Conflict — records disagree beyond the arbitration threshold.
    Conflict {
        /// Entity identifier for the conflicting records.
        entity_id: EntityId,
        /// Conflict delta (max confidence spread across categorical disagreements).
        conflict_delta: f64,
        /// The records that could not be resolved.
        records: Vec<NormalizedRecord>,
    },
}

// ── Layer 4: World model types ─────────────────────────────────────────────────

/// Current state of a fused entity in the world model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityState {
    /// Stable entity identifier.
    pub entity_id: EntityId,
    /// Canonical adverse event data.
    pub event: CdmAdverseEvent,
    /// Confidence score at last update.
    pub confidence: f64,
    /// Whether this entity is marked uncertain (stale or in conflict).
    pub uncertain: bool,
    /// When this entity was first observed.
    pub first_seen: DateTime,
    /// When this entity was last updated.
    pub last_updated: DateTime,
    /// Number of contributing source records.
    pub source_count: usize,
}

/// Type of change emitted by the world model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// A new entity was inserted into the world model.
    Created,
    /// An existing entity was updated.
    Updated,
    /// A conflict was escalated to the arbitration queue.
    ConflictEscalated,
    /// An entity was marked uncertain due to staleness.
    MarkedUncertain,
}

/// A change event emitted by the world model on every state transition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeEvent {
    /// The entity affected by this change.
    pub entity_id: EntityId,
    /// Type of change.
    pub change_type: ChangeType,
    /// Snapshot of the entity state after the change (None for conflicts).
    pub state_snapshot: Option<EntityState>,
    /// Wall-clock time of the change.
    pub occurred_at: DateTime,
}

impl ChangeEvent {
    /// Build a change event from an entity state and change type.
    pub fn from_state(state: EntityState, change_type: ChangeType) -> Self {
        let entity_id = state.entity_id.clone();
        Self {
            entity_id,
            change_type,
            state_snapshot: Some(state),
            occurred_at: DateTime::now(),
        }
    }

    /// Build a conflict-escalated change event (no state snapshot).
    pub fn conflict(entity_id: EntityId) -> Self {
        Self {
            entity_id,
            change_type: ChangeType::ConflictEscalated,
            state_snapshot: None,
            occurred_at: DateTime::now(),
        }
    }
}
