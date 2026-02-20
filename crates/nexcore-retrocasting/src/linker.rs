// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! FAERS signal-to-structure linking pipeline.
//!
//! Resolves each drug name from a FAERS signal list to a `CompoundRecord`
//! via the compound registry, producing `StructuredSignal`s that carry
//! both the pharmacovigilance statistics and molecular identity.
//!
//! ## Algorithm
//!
//! For each `SignalRecord`:
//! 1. Attempt compound resolution (cache → PubChem → ChEMBL)
//! 2. Wrap result as `StructuredSignal` (resolution failure is non-fatal)
//! 3. Set `has_structure = true` if SMILES was obtained
//!
//! ## Tier: T3 (σ + → + μ + ∃)
//! Sequence (σ) of causal (→) mappings (μ) to existing (∃) structures.

use crate::error::{RetrocastError, RetrocastResult};
use crate::types::{SignalRecord, StructuredSignal};
use nexcore_compound_registry::{CacheStore, resolve};

/// Link a slice of FAERS signals to their compound structures.
///
/// Resolution failures are non-fatal: unresolvable drugs produce a
/// `StructuredSignal` with `compound: None, has_structure: false`.
///
/// # Errors
/// Returns `RetrocastError::EmptyInput` if `signals` is empty.
pub async fn link_signals_to_structures(
    signals: &[SignalRecord],
    store: &CacheStore,
    client: &reqwest::Client,
) -> RetrocastResult<Vec<StructuredSignal>> {
    if signals.is_empty() {
        return Err(RetrocastError::EmptyInput(
            "signals slice is empty".to_string(),
        ));
    }

    let mut structured = Vec::with_capacity(signals.len());

    for signal in signals {
        let compound_result = resolve(&signal.drug, store, client).await;

        let structured_signal = match compound_result {
            Ok(record) => {
                let has_structure = record.smiles.is_some();
                tracing::debug!(
                    drug = %signal.drug,
                    has_smiles = has_structure,
                    "Linked signal to compound"
                );
                StructuredSignal {
                    signal: signal.clone(),
                    compound: Some(record),
                    has_structure,
                }
            }
            Err(e) => {
                tracing::warn!(
                    drug = %signal.drug,
                    error = %e,
                    "Could not resolve compound — signal retained without structure"
                );
                StructuredSignal {
                    signal: signal.clone(),
                    compound: None,
                    has_structure: false,
                }
            }
        };

        structured.push(structured_signal);
    }

    Ok(structured)
}

/// Deduplicate a list of structured signals by drug name.
///
/// When multiple signals exist for the same drug, the one with the highest
/// PRR is retained as representative.
pub fn deduplicate_by_drug(signals: Vec<StructuredSignal>) -> Vec<StructuredSignal> {
    use std::collections::HashMap;

    let mut best: HashMap<String, StructuredSignal> = HashMap::new();

    for sig in signals {
        let drug = sig.signal.drug.clone();
        let entry = best.entry(drug).or_insert_with(|| sig.clone());
        if sig.signal.prr > entry.signal.prr {
            *entry = sig;
        }
    }

    let mut result: Vec<StructuredSignal> = best.into_values().collect();
    // Stable sort by drug name for deterministic output
    result.sort_by(|a, b| a.signal.drug.cmp(&b.signal.drug));
    result
}

/// Filter structured signals to only those with resolved structures.
///
/// Returns signals where `has_structure == true`, suitable for
/// structural clustering in subsequent pipeline stages.
pub fn filter_with_structure(signals: Vec<StructuredSignal>) -> Vec<StructuredSignal> {
    signals.into_iter().filter(|s| s.has_structure).collect()
}

/// Group structured signals by adverse event term.
///
/// Returns a map from MedDRA PT term → Vec of structured signals.
pub fn group_by_event(
    signals: &[StructuredSignal],
) -> std::collections::HashMap<String, Vec<&StructuredSignal>> {
    let mut map: std::collections::HashMap<String, Vec<&StructuredSignal>> =
        std::collections::HashMap::new();
    for sig in signals {
        map.entry(sig.signal.event.clone())
            .or_default()
            .push(sig);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SignalRecord;
    use nexcore_compound_registry::{CacheStore, types::{CompoundRecord, ResolutionSource}};

    fn make_signal(drug: &str, event: &str, prr: f64) -> SignalRecord {
        SignalRecord {
            drug: drug.to_string(),
            event: event.to_string(),
            prr,
            ror: prr * 0.9,
            case_count: 20,
            ror_lci: Some(prr * 0.6),
            prr_chi_sq: Some(4.0),
        }
    }

    fn make_compound(name: &str, smiles: Option<&str>) -> CompoundRecord {
        CompoundRecord {
            name: name.to_string(),
            smiles: smiles.map(|s| s.to_string()),
            inchi: None,
            inchi_key: None,
            cas_number: None,
            pubchem_cid: None,
            chembl_id: None,
            synonyms: vec![],
            source: ResolutionSource::LocalCache,
            resolved_at: chrono::Utc::now(),
        }
    }

    #[test]
    fn test_filter_with_structure_keeps_only_structured() {
        let sig_with = StructuredSignal {
            signal: make_signal("aspirin", "bleeding", 3.0),
            compound: Some(make_compound("aspirin", Some("CC(=O)Oc1ccccc1C(=O)O"))),
            has_structure: true,
        };
        let sig_without = StructuredSignal {
            signal: make_signal("unknown", "rash", 2.0),
            compound: None,
            has_structure: false,
        };

        let filtered = filter_with_structure(vec![sig_with, sig_without]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].signal.drug, "aspirin");
    }

    #[test]
    fn test_deduplicate_keeps_highest_prr() {
        let sig1 = StructuredSignal {
            signal: make_signal("aspirin", "bleeding", 2.5),
            compound: None,
            has_structure: false,
        };
        let sig2 = StructuredSignal {
            signal: make_signal("aspirin", "nausea", 4.0),
            compound: None,
            has_structure: false,
        };

        let deduped = deduplicate_by_drug(vec![sig1, sig2]);
        assert_eq!(deduped.len(), 1);
        // Should retain the signal with higher PRR
        assert!((deduped[0].signal.prr - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_group_by_event_groups_correctly() {
        let sig1 = StructuredSignal {
            signal: make_signal("aspirin", "bleeding", 3.0),
            compound: None,
            has_structure: false,
        };
        let sig2 = StructuredSignal {
            signal: make_signal("ibuprofen", "bleeding", 2.5),
            compound: None,
            has_structure: false,
        };
        let sig3 = StructuredSignal {
            signal: make_signal("aspirin", "nausea", 1.8),
            compound: None,
            has_structure: false,
        };

        let all_sigs = [sig1, sig2, sig3];
        let grouped = group_by_event(&all_sigs);
        let bleeding = grouped.get("bleeding");
        assert!(bleeding.is_some());
        assert_eq!(bleeding.map(|v| v.len()), Some(2));
    }

    #[tokio::test]
    async fn test_link_empty_signals_returns_error() {
        let store_result = CacheStore::new_in_memory();
        assert!(store_result.is_ok());
        if let Ok(store) = store_result {
            let client = reqwest::Client::new();
            let result = link_signals_to_structures(&[], &store, &client).await;
            assert!(result.is_err());
            assert!(matches!(result, Err(RetrocastError::EmptyInput(_))));
        }
    }

    #[tokio::test]
    async fn test_link_cached_compound_resolves_to_structure() {
        let store_result = CacheStore::new_in_memory();
        assert!(store_result.is_ok());
        if let Ok(store) = store_result {
            // Pre-seed cache with aspirin
            let record = make_compound("aspirin", Some("CC(=O)Oc1ccccc1C(=O)O"));
            assert!(store.put(&record).is_ok());

            let signals = vec![make_signal("aspirin", "bleeding", 3.5)];
            let client = reqwest::Client::new();
            let result = link_signals_to_structures(&signals, &store, &client).await;
            assert!(result.is_ok());
            if let Ok(linked) = result {
                assert_eq!(linked.len(), 1);
                assert!(linked[0].has_structure);
                assert!(linked[0].smiles().is_some());
            }
        }
    }

    /// Tests graceful handling when compound resolution fails (requires network).
    /// Gated behind `integration` feature to avoid slow/hanging CI runs.
    #[cfg(feature = "integration")]
    #[tokio::test]
    async fn test_link_unresolved_compound_produces_none_gracefully() {
        let store_result = CacheStore::new_in_memory();
        assert!(store_result.is_ok());
        if let Ok(store) = store_result {
            let signals = vec![make_signal(
                "zzzz_nonexistent_drug_99999",
                "headache",
                2.1,
            )];
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_millis(500))
                .build()
                .unwrap_or_default();

            let result = link_signals_to_structures(&signals, &store, &client).await;
            // Non-fatal: unresolvable compounds produce StructuredSignal with compound=None
            assert!(result.is_ok());
            if let Ok(linked) = result {
                assert_eq!(linked.len(), 1);
                assert!(!linked[0].has_structure);
                assert!(linked[0].compound.is_none());
            }
        }
    }
}
