// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Alert correlation and confidence scoring.
//!
//! For each structural cluster, identifies adverse events consistently
//! associated with member drugs and extracts common structural fragments
//! as candidate structural alert patterns.
//!
//! ## Algorithm
//!
//! 1. Group `StructuredSignal`s by cluster membership
//! 2. For each cluster, find events present in ≥ 2 cluster members
//! 3. For qualifying events, compute:
//!    - Support count: how many drugs have this event + this fragment
//!    - Mean PRR: average disproportionality across supporters
//!    - Confidence: composite score from support density + mean PRR
//! 4. Emit `AlertCandidate` for fragment-event pairs exceeding min_confidence
//!
//! ## Tier: T3 (ν + κ + → + ∂)
//! Frequency (ν) comparisons (κ) causally mapped (→) to boundary (∂) alerts.

use crate::error::{RetrocastError, RetrocastResult};
use crate::types::{AlertCandidate, StructuralCluster, StructuredSignal};
use std::collections::HashMap;

/// Minimum number of supporting drugs required to propose an alert.
const MIN_SUPPORT: usize = 2;

/// Minimum PRR to consider a signal as contributing evidence.
const MIN_PRR: f64 = 2.0;

/// Correlate structural clusters with adverse event patterns to find alert candidates.
///
/// # Parameters
/// - `clusters`: structural clusters from `cluster_by_similarity`
/// - `signals`: all structured signals (used to look up per-drug AE evidence)
/// - `min_confidence`: minimum confidence threshold for emitting a candidate
///
/// # Errors
/// Returns `RetrocastError::CorrelationError` if both inputs are empty.
pub fn correlate_alerts(
    clusters: &[StructuralCluster],
    signals: &[StructuredSignal],
    min_confidence: f64,
) -> RetrocastResult<Vec<AlertCandidate>> {
    if clusters.is_empty() && signals.is_empty() {
        return Err(RetrocastError::CorrelationError(
            "Both clusters and signals are empty".to_string(),
        ));
    }

    // Build drug → signals index for fast lookup
    let mut drug_signals: HashMap<&str, Vec<&StructuredSignal>> = HashMap::new();
    for sig in signals {
        drug_signals
            .entry(sig.signal.drug.as_str())
            .or_default()
            .push(sig);
    }

    let mut candidates: Vec<AlertCandidate> = Vec::new();

    for cluster in clusters {
        if cluster.members.len() < MIN_SUPPORT {
            continue;
        }

        // Collect all events observed across cluster members with significant signals
        let mut event_supporters: HashMap<String, Vec<(String, f64)>> = HashMap::new();

        for drug_name in &cluster.members {
            if let Some(drug_sigs) = drug_signals.get(drug_name.as_str()) {
                for structured in drug_sigs {
                    if structured.signal.prr >= MIN_PRR && structured.signal.is_significant() {
                        event_supporters
                            .entry(structured.signal.event.clone())
                            .or_default()
                            .push((drug_name.clone(), structured.signal.prr));
                    }
                }
            }
        }

        // For each event with sufficient support, generate alert candidates
        for (event, supporters) in event_supporters {
            if supporters.len() < MIN_SUPPORT {
                continue;
            }

            let mean_prr =
                supporters.iter().map(|(_, prr)| prr).sum::<f64>() / supporters.len() as f64;

            let support_count = supporters.len();
            let supporting_drugs: Vec<String> = supporters.into_iter().map(|(d, _)| d).collect();

            // For each common fragment, create an alert candidate
            let fragments_to_emit: Vec<String> = if cluster.common_fragments.is_empty() {
                // Fall back to cluster-level representation
                vec![format!("cluster:{}", cluster.cluster_id)]
            } else {
                cluster.common_fragments.clone()
            };

            for fragment in fragments_to_emit {
                let confidence = compute_confidence(support_count, cluster.members.len(), mean_prr);

                if confidence >= min_confidence {
                    candidates.push(AlertCandidate {
                        fragment_smiles: fragment,
                        associated_events: vec![event.clone()],
                        confidence,
                        supporting_drugs: supporting_drugs.clone(),
                        mean_prr,
                        support_count,
                    });
                }
            }
        }
    }

    // Deduplicate by (fragment, event) — keep highest confidence
    let candidates = deduplicate_candidates(candidates);

    // Sort by confidence descending
    let mut sorted = candidates;
    sorted.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(sorted)
}

/// Compute a composite confidence score for an alert candidate.
///
/// ## Formula
///
/// ```text
/// density     = support_count / cluster_size           ∈ [0, 1]
/// prr_score   = min(mean_prr / 10.0, 1.0)             ∈ [0, 1]
/// confidence  = 0.6 * density + 0.4 * prr_score       ∈ [0, 1]
/// ```
///
/// Weights: 60% structural consistency, 40% signal strength.
fn compute_confidence(support_count: usize, cluster_size: usize, mean_prr: f64) -> f64 {
    let density = if cluster_size == 0 {
        0.0
    } else {
        support_count as f64 / cluster_size as f64
    };
    let prr_score = (mean_prr / 10.0).min(1.0);
    0.6 * density + 0.4 * prr_score
}

/// Deduplicate alert candidates, keeping highest confidence per (fragment, event).
fn deduplicate_candidates(candidates: Vec<AlertCandidate>) -> Vec<AlertCandidate> {
    let mut best: HashMap<(String, String), AlertCandidate> = HashMap::new();

    for candidate in candidates {
        let primary_event = candidate
            .associated_events
            .first()
            .cloned()
            .unwrap_or_default();
        let key = (candidate.fragment_smiles.clone(), primary_event);

        let entry = best.entry(key).or_insert_with(|| candidate.clone());
        if candidate.confidence > entry.confidence {
            *entry = candidate;
        }
    }

    best.into_values().collect()
}

/// Merge overlapping alert candidates that share the same fragment.
///
/// When the same structural fragment is linked to multiple adverse events,
/// this consolidates them into a single `AlertCandidate` with all associated events.
pub fn merge_by_fragment(candidates: Vec<AlertCandidate>) -> Vec<AlertCandidate> {
    let mut by_fragment: HashMap<String, AlertCandidate> = HashMap::new();

    for candidate in candidates {
        let entry = by_fragment
            .entry(candidate.fragment_smiles.clone())
            .or_insert_with(|| candidate.clone());

        // Merge events
        for event in &candidate.associated_events {
            if !entry.associated_events.contains(event) {
                entry.associated_events.push(event.clone());
            }
        }

        // Keep best confidence
        if candidate.confidence > entry.confidence {
            entry.confidence = candidate.confidence;
        }

        // Merge supporting drugs
        for drug in &candidate.supporting_drugs {
            if !entry.supporting_drugs.contains(drug) {
                entry.supporting_drugs.push(drug.clone());
            }
        }

        // Update support count
        entry.support_count = entry.supporting_drugs.len();
    }

    let mut merged: Vec<AlertCandidate> = by_fragment.into_values().collect();
    merged.sort_by(|a, b| {
        b.confidence
            .partial_cmp(&a.confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SignalRecord, StructuralCluster};
    use nexcore_compound_registry::types::{CompoundRecord, ResolutionSource};

    fn make_structured_signal(drug: &str, event: &str, prr: f64) -> StructuredSignal {
        let compound = CompoundRecord::new(
            drug.to_string(),
            Some(format!("C(=O)O{drug}")),
            ResolutionSource::Manual,
        );
        StructuredSignal {
            signal: SignalRecord {
                drug: drug.to_string(),
                event: event.to_string(),
                prr,
                ror: prr * 0.9,
                case_count: 40,
                ror_lci: Some(prr * 0.7),
                prr_chi_sq: Some(5.0),
            },
            compound: Some(compound),
            has_structure: true,
        }
    }

    fn make_cluster(
        id: usize,
        members: Vec<&str>,
        fragments: Vec<&str>,
        events: Vec<&str>,
    ) -> StructuralCluster {
        StructuralCluster {
            cluster_id: id,
            members: members.iter().map(|s| s.to_string()).collect(),
            common_fragments: fragments.iter().map(|s| s.to_string()).collect(),
            similarity_threshold: 0.5,
            shared_events: events.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_correlate_empty_inputs_returns_error() {
        let result = correlate_alerts(&[], &[], 0.3);
        assert!(result.is_err());
        assert!(matches!(result, Err(RetrocastError::CorrelationError(_))));
    }

    #[test]
    fn test_correlate_finds_shared_event_candidate() {
        let signals = vec![
            make_structured_signal("aspirin", "GI bleeding", 4.2),
            make_structured_signal("ibuprofen", "GI bleeding", 3.8),
            make_structured_signal("naproxen", "GI bleeding", 3.5),
        ];

        let cluster = make_cluster(
            0,
            vec!["aspirin", "ibuprofen", "naproxen"],
            vec!["C(", "=O", "OC"],
            vec!["GI bleeding"],
        );

        let result = correlate_alerts(&[cluster], &signals, 0.0);
        assert!(result.is_ok());
        if let Ok(candidates) = result {
            assert!(
                !candidates.is_empty(),
                "Expected at least one alert candidate"
            );
            let gi_candidate = candidates
                .iter()
                .find(|c| c.associated_events.iter().any(|e| e.contains("bleeding")));
            assert!(
                gi_candidate.is_some(),
                "Expected GI bleeding alert candidate"
            );
        }
    }

    #[test]
    fn test_correlate_below_min_support_no_candidate() {
        // Single-member cluster — below MIN_SUPPORT of 2
        let signals = vec![make_structured_signal("aspirin", "GI bleeding", 4.0)];
        let cluster = make_cluster(0, vec!["aspirin"], vec!["C(=O)"], vec!["GI bleeding"]);

        let result = correlate_alerts(&[cluster], &signals, 0.0);
        assert!(result.is_ok());
        if let Ok(candidates) = result {
            assert!(
                candidates.is_empty(),
                "Single-member cluster should produce no candidates"
            );
        }
    }

    #[test]
    fn test_confidence_score_formula() {
        // support=3, cluster=3, mean_prr=6.0
        // density = 1.0, prr_score = 0.6
        // confidence = 0.6*1.0 + 0.4*0.6 = 0.84
        let c = compute_confidence(3, 3, 6.0);
        assert!((c - 0.84).abs() < 0.001, "Expected 0.84, got {c}");
    }

    #[test]
    fn test_confidence_score_zero_cluster_size() {
        let c = compute_confidence(0, 0, 2.0);
        assert!(c < 1.0);
        assert!(c >= 0.0);
    }

    #[test]
    fn test_high_prr_capped_at_one_in_prr_score() {
        // mean_prr=100 — prr_score should cap at 1.0
        let c = compute_confidence(5, 5, 100.0);
        // density=1.0, prr_score=1.0 → confidence=1.0
        assert!(
            (c - 1.0).abs() < f64::EPSILON,
            "Should be capped at 1.0, got {c}"
        );
    }

    #[test]
    fn test_merge_by_fragment_consolidates_events() {
        let c1 = AlertCandidate {
            fragment_smiles: "C(=O)O".to_string(),
            associated_events: vec!["GI bleeding".to_string()],
            confidence: 0.7,
            supporting_drugs: vec!["aspirin".to_string()],
            mean_prr: 4.0,
            support_count: 1,
        };
        let c2 = AlertCandidate {
            fragment_smiles: "C(=O)O".to_string(),
            associated_events: vec!["peptic ulcer".to_string()],
            confidence: 0.65,
            supporting_drugs: vec!["ibuprofen".to_string()],
            mean_prr: 3.5,
            support_count: 1,
        };

        let merged = merge_by_fragment(vec![c1, c2]);
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].associated_events.len(), 2);
        assert!(merged[0].confidence >= 0.65);
    }

    #[test]
    fn test_correlate_min_confidence_filter() {
        let signals = vec![
            make_structured_signal("aspirin", "rash", 2.2),
            make_structured_signal("ibuprofen", "rash", 2.1),
        ];
        let cluster = make_cluster(0, vec!["aspirin", "ibuprofen"], vec!["C("], vec!["rash"]);

        // With min_confidence=0.99, no candidates should pass
        let strict = correlate_alerts(&[cluster.clone()], &signals, 0.99);
        assert!(strict.is_ok());
        if let Ok(candidates) = strict {
            assert!(
                candidates.is_empty(),
                "Strict threshold should filter all candidates"
            );
        }

        // With min_confidence=0.0, candidates should emerge
        let open = correlate_alerts(&[cluster], &signals, 0.0);
        assert!(open.is_ok());
        if let Ok(candidates) = open {
            assert!(
                !candidates.is_empty(),
                "Zero threshold should allow candidates"
            );
        }
    }
}
