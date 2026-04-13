//! Layer 2 — Normalization and deduplication.
//!
//! Normalizes raw records into CDM-aligned [`NormalizedRecord`] values and
//! detects duplicates using SHA-256 fingerprinting.

use std::collections::HashMap;

use async_trait::async_trait;
use nexcore_chrono::DateTime;
use sha2::{Digest, Sha256};

use crate::error::{PerceptionError, Result};
use crate::types::{
    CdmAdverseEvent, NormalizedRecord, Outcome, RawRecord, RecordId, ReporterType, Seriousness,
    SourceId,
};

// ── Normalizer trait ───────────────────────────────────────────────────────────

/// Converts a [`RawRecord`] into a [`NormalizedRecord`].
///
/// Implementors are responsible for extracting and validating CDM fields from
/// the source-specific payload.
#[async_trait]
pub trait Normalizer: Send + Sync + 'static {
    /// The source this normalizer handles.
    fn source_id(&self) -> &SourceId;

    /// Normalize a raw record into CDM form.
    async fn normalize(&self, record: RawRecord) -> Result<NormalizedRecord>;
}

// ── Hold queue trait ───────────────────────────────────────────────────────────

/// Queue for records that could not be automatically resolved during
/// deduplication (fingerprint match but confidence below `merge_threshold`).
pub trait HoldQueue: Send + Sync + 'static {
    /// Push a record onto the hold queue for manual review.
    fn push(&self, record: NormalizedRecord);

    /// Drain all held records (for testing or batch processing).
    fn drain(&self) -> Vec<NormalizedRecord>;

    /// Number of records currently held.
    fn len(&self) -> usize;

    /// True if the queue is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ── Fingerprint ────────────────────────────────────────────────────────────────

/// Compute the SHA-256 deduplication fingerprint for an adverse event.
///
/// Fingerprint = SHA-256(`drug_name|event_term|onset_date_day|country`)
/// All fields are lower-cased; missing optional fields become empty strings.
pub fn compute_fingerprint(event: &CdmAdverseEvent) -> String {
    let raw = format!(
        "{}|{}|{}|{}",
        event.drug_name.to_lowercase(),
        event.event_term.to_lowercase(),
        event.onset_date_day.as_deref().unwrap_or(""),
        event.country.as_deref().unwrap_or(""),
    );
    let digest = Sha256::digest(raw.as_bytes());
    format!("{digest:x}")
}

// ── Deduplication engine ───────────────────────────────────────────────────────

/// Entry stored in the dedup cache.
#[derive(Debug)]
struct FingerprintEntry {
    /// Confidence of the first record seen with this fingerprint.
    confidence: f64,
    /// When this fingerprint was first seen.
    first_seen: DateTime,
}

/// Stateful deduplication engine using SHA-256 fingerprints.
///
/// Three cases on `check`:
/// 1. **No match** → unique record, pass through.
/// 2. **Match within window AND confidence ≥ merge_threshold** → duplicate, drop.
/// 3. **Match within window AND confidence < merge_threshold** → route to hold queue.
#[derive(Debug)]
pub struct DeduplicationEngine {
    /// `fingerprint hex → entry`
    cache: HashMap<String, FingerprintEntry>,
    /// Deduplication window in seconds.
    window_secs: i64,
    /// Minimum confidence to consider a fingerprint match a true duplicate.
    merge_threshold: f64,
}

/// Outcome of a deduplication check.
#[derive(Debug)]
pub enum DedupOutcome {
    /// Record is unique — pass to fusion.
    Unique,
    /// Record is a duplicate within the window — drop.
    Duplicate,
    /// Fingerprint matched but confidence is below threshold — route to hold.
    HoldForReview,
}

impl DeduplicationEngine {
    /// Create a new deduplication engine.
    ///
    /// * `dedup_window_hours` — window within which fingerprint matches are deduplicated.
    /// * `merge_threshold` — confidence floor for treating a match as a duplicate.
    pub fn new(dedup_window_hours: f64, merge_threshold: f64) -> Self {
        Self {
            cache: HashMap::new(),
            window_secs: (dedup_window_hours * 3600.0) as i64,
            merge_threshold,
        }
    }

    /// Check a normalised record for duplication.
    ///
    /// Side effect: registers the fingerprint on first encounter.
    pub fn check(&mut self, record: &NormalizedRecord) -> DedupOutcome {
        let now = DateTime::now();

        match self.cache.get(&record.fingerprint) {
            None => {
                // First time we've seen this fingerprint — register it.
                self.cache.insert(
                    record.fingerprint.clone(),
                    FingerprintEntry {
                        confidence: record.confidence,
                        first_seen: now,
                    },
                );
                DedupOutcome::Unique
            }
            Some(entry) => {
                let age_secs = now.signed_duration_since(entry.first_seen).num_seconds();
                if age_secs > self.window_secs {
                    // Outside window — treat as a new record, update cache.
                    self.cache.insert(
                        record.fingerprint.clone(),
                        FingerprintEntry {
                            confidence: record.confidence,
                            first_seen: now,
                        },
                    );
                    DedupOutcome::Unique
                } else if record.confidence >= self.merge_threshold {
                    DedupOutcome::Duplicate
                } else {
                    DedupOutcome::HoldForReview
                }
            }
        }
    }

    /// Evict fingerprints older than the dedup window to prevent unbounded growth.
    pub fn evict_stale(&mut self) {
        let now = DateTime::now();
        self.cache.retain(|_, entry| {
            now.signed_duration_since(entry.first_seen).num_seconds() <= self.window_secs
        });
    }

    /// Number of fingerprints currently in the cache.
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

// ── Default normalizer (passthrough for testing) ───────────────────────────────

/// A minimal normalizer that extracts CDM fields from a flat JSON payload.
///
/// Expected payload fields:
/// - `drug_name` (string, required)
/// - `event_term` (string, required)
/// - `onset_date_day` (string, optional — YYYY-MM-DD)
/// - `country` (string, optional — ISO 3166-1 alpha-2)
/// - `seriousness` (string, optional — "serious" | "non_serious")
/// - `outcome` (string, optional — "recovered" | "fatal" | …)
/// - `confidence` (f64, optional — defaults to 0.7)
pub struct JsonNormalizer {
    source_id: SourceId,
}

impl JsonNormalizer {
    /// Create a JSON normalizer for the given source.
    pub fn new(source_id: SourceId) -> Self {
        Self { source_id }
    }
}

#[async_trait]
impl Normalizer for JsonNormalizer {
    fn source_id(&self) -> &SourceId {
        &self.source_id
    }

    async fn normalize(&self, record: RawRecord) -> Result<NormalizedRecord> {
        let p = &record.payload;

        let drug_name = p["drug_name"]
            .as_str()
            .ok_or_else(|| PerceptionError::MissingField("drug_name".into()))?
            .to_string();

        let event_term = p["event_term"]
            .as_str()
            .ok_or_else(|| PerceptionError::MissingField("event_term".into()))?
            .to_string();

        let onset_date_day = p["onset_date_day"].as_str().map(String::from);
        let country = p["country"].as_str().map(String::from);

        let seriousness = match p["seriousness"].as_str().unwrap_or("unknown") {
            "serious" => Seriousness::Serious,
            "non_serious" => Seriousness::NonSerious,
            _ => Seriousness::Unknown,
        };

        let outcome = match p["outcome"].as_str().unwrap_or("unknown") {
            "recovered" => Outcome::Recovered,
            "recovered_with_sequelae" => Outcome::RecoveredWithSequelae,
            "not_recovered" => Outcome::NotRecovered,
            "fatal" => Outcome::Fatal,
            _ => Outcome::Unknown,
        };

        let confidence = p["confidence"].as_f64().unwrap_or(0.7).clamp(0.0, 1.0);

        let event = CdmAdverseEvent {
            drug_name,
            event_term,
            onset_date_day,
            country,
            seriousness,
            outcome,
            reporter_type: ReporterType::Other,
            patient_age_years: p["patient_age_years"].as_f64().map(|v| v as f32),
            patient_sex: p["patient_sex"].as_str().map(String::from),
        };

        let fingerprint = compute_fingerprint(&event);

        Ok(NormalizedRecord {
            id: RecordId::new(record.id.to_string()),
            source_id: record.source_id,
            event,
            fingerprint,
            confidence,
            normalized_at: DateTime::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_event(
        drug: &str,
        event: &str,
        date: Option<&str>,
        country: Option<&str>,
    ) -> CdmAdverseEvent {
        CdmAdverseEvent {
            drug_name: drug.to_string(),
            event_term: event.to_string(),
            onset_date_day: date.map(String::from),
            country: country.map(String::from),
            seriousness: Seriousness::Unknown,
            outcome: Outcome::Unknown,
            reporter_type: ReporterType::Other,
            patient_age_years: None,
            patient_sex: None,
        }
    }

    #[test]
    fn fingerprint_is_deterministic() {
        let e = make_event(
            "metformin",
            "lactic acidosis",
            Some("2024-01-01"),
            Some("US"),
        );
        assert_eq!(compute_fingerprint(&e), compute_fingerprint(&e));
    }

    #[test]
    fn fingerprint_differs_on_drug_change() {
        let e1 = make_event("metformin", "lactic acidosis", None, None);
        let e2 = make_event("semaglutide", "lactic acidosis", None, None);
        assert_ne!(compute_fingerprint(&e1), compute_fingerprint(&e2));
    }

    #[test]
    fn fingerprint_is_case_insensitive() {
        let e1 = make_event("Metformin", "Lactic Acidosis", None, None);
        let e2 = make_event("metformin", "lactic acidosis", None, None);
        assert_eq!(compute_fingerprint(&e1), compute_fingerprint(&e2));
    }

    #[test]
    fn dedup_engine_unique_on_first_encounter() {
        let mut engine = DeduplicationEngine::new(24.0, 0.6);
        let record = NormalizedRecord {
            id: RecordId::generate(),
            source_id: SourceId::new("test"),
            event: make_event("metformin", "nausea", None, None),
            fingerprint: "abc123".to_string(),
            confidence: 0.8,
            normalized_at: DateTime::now(),
        };
        assert!(matches!(engine.check(&record), DedupOutcome::Unique));
    }

    #[test]
    fn dedup_engine_duplicate_on_high_confidence_rematch() {
        let mut engine = DeduplicationEngine::new(24.0, 0.6);
        let fp = "dedup_fp_001".to_string();
        let make_record = || NormalizedRecord {
            id: RecordId::generate(),
            source_id: SourceId::new("test"),
            event: make_event("metformin", "nausea", None, None),
            fingerprint: fp.clone(),
            confidence: 0.8,
            normalized_at: DateTime::now(),
        };
        engine.check(&make_record()); // first → Unique
        assert!(matches!(
            engine.check(&make_record()),
            DedupOutcome::Duplicate
        ));
    }

    #[test]
    fn dedup_engine_hold_on_low_confidence_rematch() {
        let mut engine = DeduplicationEngine::new(24.0, 0.6);
        let fp = "dedup_fp_002".to_string();
        // First record at high confidence — registers in cache
        let first = NormalizedRecord {
            id: RecordId::generate(),
            source_id: SourceId::new("test"),
            event: make_event("metformin", "nausea", None, None),
            fingerprint: fp.clone(),
            confidence: 0.8,
            normalized_at: DateTime::now(),
        };
        engine.check(&first);
        // Second record same fingerprint but low confidence → HoldForReview
        let second = NormalizedRecord {
            id: RecordId::generate(),
            source_id: SourceId::new("test"),
            event: make_event("metformin", "nausea", None, None),
            fingerprint: fp.clone(),
            confidence: 0.3,
            normalized_at: DateTime::now(),
        };
        assert!(matches!(engine.check(&second), DedupOutcome::HoldForReview));
    }

    #[tokio::test]
    async fn json_normalizer_extracts_required_fields() {
        let normalizer = JsonNormalizer::new(SourceId::new("test"));
        let payload = json!({
            "drug_name": "Metformin",
            "event_term": "Lactic Acidosis",
            "onset_date_day": "2024-01-15",
            "country": "US",
            "confidence": 0.9
        });
        let raw = RawRecord::new(SourceId::new("test"), payload);
        let normalized = normalizer
            .normalize(raw)
            .await
            .expect("normalization failed");
        assert_eq!(normalized.event.drug_name, "Metformin");
        assert_eq!(normalized.event.event_term, "Lactic Acidosis");
        assert!((normalized.confidence - 0.9).abs() < f64::EPSILON);
        assert!(!normalized.fingerprint.is_empty());
    }

    #[tokio::test]
    async fn json_normalizer_returns_error_on_missing_drug() {
        let normalizer = JsonNormalizer::new(SourceId::new("test"));
        let payload = json!({ "event_term": "Lactic Acidosis" });
        let raw = RawRecord::new(SourceId::new("test"), payload);
        let result = normalizer.normalize(raw).await;
        assert!(matches!(result, Err(PerceptionError::MissingField(_))));
    }
}
