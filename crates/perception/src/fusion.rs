//! Layer 3 — Fusion engine.
//!
//! Groups normalised records by fingerprint and resolves them into a single
//! [`FusionResult`]: singleton pass-through, weighted merge, or conflict
//! escalation.

use std::collections::HashMap;

use crate::error::Result;
use crate::registry::SourceRegistry;
use crate::types::{
    CdmAdverseEvent, EntityId, FusionResult, NormalizedRecord, Outcome, Seriousness,
};

// ── Arbitration queue trait ────────────────────────────────────────────────────

/// Queue for fusion results that could not be automatically resolved
/// (conflict delta exceeds `arbitration_threshold`).
pub trait ArbitrationQueue: Send + Sync + 'static {
    /// Push a conflicting fusion result onto the queue.
    fn push(&self, result: FusionResult);

    /// Drain all queued results (for testing or batch processing).
    fn drain(&self) -> Vec<FusionResult>;

    /// Number of results currently queued.
    fn len(&self) -> usize;

    /// True if the queue is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ── Conflict delta ─────────────────────────────────────────────────────────────

/// Compute the conflict delta for a group of normalised records.
///
/// For each pair of records, compare categorical fields (seriousness, outcome,
/// event_term). When categorical fields differ, the per-pair delta equals
/// `abs(a.confidence - b.confidence)`. Returns the maximum delta across all
/// pairs. Returns `0.0` for a group of fewer than two records.
pub fn compute_conflict_delta(records: &[NormalizedRecord]) -> f64 {
    if records.len() < 2 {
        return 0.0;
    }

    let mut max_delta: f64 = 0.0;

    for i in 0..records.len() {
        for j in (i + 1)..records.len() {
            let a = &records[i];
            let b = &records[j];

            let categoricals_differ = a.event.seriousness != b.event.seriousness
                || a.event.outcome != b.event.outcome
                || a.event.event_term.to_lowercase() != b.event.event_term.to_lowercase();

            if categoricals_differ {
                let delta = (a.confidence - b.confidence).abs();
                if delta > max_delta {
                    max_delta = delta;
                }
            }
        }
    }

    max_delta
}

// ── Merge ──────────────────────────────────────────────────────────────────────

/// Merge a group of normalised records into a single [`CdmAdverseEvent`] and
/// fused confidence score.
///
/// Categorical values are taken from the record with the highest confidence.
/// Fused confidence = sum(confidence × reliability_weight) / sum(weights).
pub fn merge(records: &[NormalizedRecord], registry: &SourceRegistry) -> (CdmAdverseEvent, f64) {
    // Pick the highest-confidence record for categorical values.
    let best = records
        .iter()
        .max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .expect("merge called with empty slice");

    // Weighted average confidence.
    let (weighted_sum, weight_sum) = records.iter().fold((0.0_f64, 0.0_f64), |(ws, wt), r| {
        let w = registry.reliability_weight(&r.source_id);
        (ws + r.confidence * w, wt + w)
    });

    let fused_confidence = if weight_sum > 0.0 {
        (weighted_sum / weight_sum).clamp(0.0, 1.0)
    } else {
        best.confidence
    };

    (best.event.clone(), fused_confidence)
}

// ── Fusion engine ──────────────────────────────────────────────────────────────

/// Groups incoming normalised records by fingerprint and resolves each group.
pub struct FusionEngine {
    /// Pending records grouped by fingerprint.
    pending: HashMap<String, Vec<NormalizedRecord>>,
    /// Conflict delta above which records are escalated to arbitration.
    arbitration_threshold: f64,
}

impl FusionEngine {
    /// Create a new fusion engine.
    pub fn new(arbitration_threshold: f64) -> Self {
        Self {
            pending: HashMap::new(),
            arbitration_threshold,
        }
    }

    /// Ingest a normalised record into the pending group for its fingerprint.
    pub fn ingest(&mut self, record: NormalizedRecord) {
        self.pending
            .entry(record.fingerprint.clone())
            .or_default()
            .push(record);
    }

    /// Resolve all pending groups, returning a list of [`FusionResult`] values.
    ///
    /// Groups that conflict beyond `arbitration_threshold` are also pushed onto
    /// `arbitration`. After this call the pending buffer is cleared.
    pub fn fuse(
        &mut self,
        registry: &SourceRegistry,
        arbitration: &dyn ArbitrationQueue,
    ) -> Vec<FusionResult> {
        let groups: Vec<Vec<NormalizedRecord>> = self.pending.drain().map(|(_, v)| v).collect();
        let mut results = Vec::with_capacity(groups.len());

        for group in groups {
            let result = self.resolve_group(group, registry);

            // Clone conflict results onto the arbitration queue before moving.
            if matches!(result, FusionResult::Conflict { .. }) {
                arbitration.push(result.clone());
            }

            results.push(result);
        }

        results
    }

    /// Resolve a single group of records with the same fingerprint.
    fn resolve_group(
        &self,
        records: Vec<NormalizedRecord>,
        registry: &SourceRegistry,
    ) -> FusionResult {
        if records.len() == 1 {
            return FusionResult::Singleton {
                record: records.into_iter().next().expect("len checked"),
            };
        }

        let delta = compute_conflict_delta(&records);

        if delta > self.arbitration_threshold {
            let entity_id = EntityId::new(format!(
                "conflict-{}",
                records[0].fingerprint.chars().take(8).collect::<String>()
            ));
            return FusionResult::Conflict {
                entity_id,
                conflict_delta: delta,
                records,
            };
        }

        let (event, fused_confidence) = merge(&records, registry);
        let source_count = records.len();
        let entity_id = EntityId::new(format!(
            "entity-{}",
            records[0].fingerprint.chars().take(8).collect::<String>()
        ));

        FusionResult::Fused {
            entity_id,
            event,
            fused_confidence,
            source_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{SourceConfig, SourceRegistry};
    use crate::types::{Outcome, RecordId, ReporterType, Seriousness, SourceId};
    use nexcore_chrono::DateTime;

    fn make_record(
        drug: &str,
        event: &str,
        seriousness: Seriousness,
        outcome: Outcome,
        confidence: f64,
        source: &str,
    ) -> NormalizedRecord {
        let ev = CdmAdverseEvent {
            drug_name: drug.to_string(),
            event_term: event.to_string(),
            onset_date_day: None,
            country: None,
            seriousness,
            outcome,
            reporter_type: ReporterType::Other,
            patient_age_years: None,
            patient_sex: None,
        };
        let fingerprint = crate::normalizer::compute_fingerprint(&ev);
        NormalizedRecord {
            id: RecordId::generate(),
            source_id: SourceId::new(source),
            event: ev,
            fingerprint,
            confidence,
            normalized_at: DateTime::now(),
        }
    }

    fn make_registry(sources: &[(&str, f64)]) -> SourceRegistry {
        SourceRegistry::from_configs(
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
        )
    }

    struct VecArbitration(std::sync::Mutex<Vec<FusionResult>>);
    impl VecArbitration {
        fn new() -> Self {
            Self(std::sync::Mutex::new(Vec::new()))
        }
    }
    impl ArbitrationQueue for VecArbitration {
        fn push(&self, result: FusionResult) {
            self.0.lock().expect("mutex poisoned").push(result);
        }
        fn drain(&self) -> Vec<FusionResult> {
            self.0.lock().expect("mutex poisoned").drain(..).collect()
        }
        fn len(&self) -> usize {
            self.0.lock().expect("mutex poisoned").len()
        }
    }

    #[test]
    fn singleton_group_returns_singleton() {
        let mut engine = FusionEngine::new(0.4);
        let registry = make_registry(&[("faers", 0.85)]);
        let arb = VecArbitration::new();

        let r = make_record(
            "metformin",
            "nausea",
            Seriousness::NonSerious,
            Outcome::Recovered,
            0.8,
            "faers",
        );
        engine.ingest(r);

        let results = engine.fuse(&registry, &arb);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], FusionResult::Singleton { .. }));
    }

    #[test]
    fn agreeing_records_produce_fused_result() {
        let mut engine = FusionEngine::new(0.4);
        let registry = make_registry(&[("faers", 0.85), ("pubmed", 0.75)]);
        let arb = VecArbitration::new();

        // Same drug/event/seriousness/outcome → no categorical conflict
        let r1 = make_record(
            "metformin",
            "nausea",
            Seriousness::NonSerious,
            Outcome::Recovered,
            0.8,
            "faers",
        );
        let r2 = make_record(
            "metformin",
            "nausea",
            Seriousness::NonSerious,
            Outcome::Recovered,
            0.75,
            "pubmed",
        );
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

    #[test]
    fn conflicting_records_escalate_to_arbitration() {
        let mut engine = FusionEngine::new(0.2);
        let registry = make_registry(&[("faers", 0.85), ("pubmed", 0.75)]);
        let arb = VecArbitration::new();

        // Seriousness differs → categorical conflict; confidence delta = 0.8 - 0.1 = 0.7 > 0.2
        let r1 = make_record(
            "semaglutide",
            "pancreatitis",
            Seriousness::Serious,
            Outcome::Recovered,
            0.8,
            "faers",
        );
        let r2 = make_record(
            "semaglutide",
            "pancreatitis",
            Seriousness::NonSerious,
            Outcome::Recovered,
            0.1,
            "pubmed",
        );
        engine.ingest(r1);
        engine.ingest(r2);

        let results = engine.fuse(&registry, &arb);
        assert_eq!(results.len(), 1);
        assert!(matches!(results[0], FusionResult::Conflict { .. }));
        assert_eq!(arb.len(), 1);
    }

    #[test]
    fn compute_conflict_delta_zero_for_single_record() {
        let r = make_record(
            "metformin",
            "nausea",
            Seriousness::NonSerious,
            Outcome::Recovered,
            0.8,
            "test",
        );
        assert!((compute_conflict_delta(&[r]) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn merge_uses_highest_confidence_categoricals() {
        let registry = make_registry(&[("faers", 0.85), ("pubmed", 0.75)]);
        let r1 = make_record(
            "metformin",
            "nausea",
            Seriousness::Serious,
            Outcome::Fatal,
            0.9,
            "faers",
        );
        let r2 = make_record(
            "metformin",
            "nausea",
            Seriousness::NonSerious,
            Outcome::Recovered,
            0.5,
            "pubmed",
        );
        let (event, _confidence) = merge(&[r1, r2], &registry);
        // r1 has highest confidence → its categoricals win
        assert_eq!(event.seriousness, Seriousness::Serious);
        assert_eq!(event.outcome, Outcome::Fatal);
    }

    #[test]
    fn fused_confidence_is_weighted_average() {
        let registry = make_registry(&[("faers", 0.8), ("pubmed", 0.4)]);
        let r1 = make_record(
            "metformin",
            "nausea",
            Seriousness::NonSerious,
            Outcome::Recovered,
            1.0,
            "faers",
        );
        let r2 = make_record(
            "metformin",
            "nausea",
            Seriousness::NonSerious,
            Outcome::Recovered,
            0.0,
            "pubmed",
        );
        let (_event, fused) = merge(&[r1, r2], &registry);
        // (1.0 * 0.8 + 0.0 * 0.4) / (0.8 + 0.4) = 0.8 / 1.2 ≈ 0.6667
        let expected = 0.8_f64 / 1.2_f64;
        assert!((fused - expected).abs() < 1e-9);
    }
}
