// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Structural clustering of compounds by molecular similarity.
//!
//! Groups structurally similar drugs using agglomerative single-linkage
//! clustering on a pairwise similarity matrix.
//!
//! ## Similarity Engine
//!
//! **Current (T7 pending):** SMILES character bigram Jaccard similarity.
//! This correctly groups drugs with shared SMILES substrings (e.g., NSAIDs
//! sharing carboxylate `C(=O)O` patterns).
//!
//! **Planned (T7 complete):** Enable the `fingerprints` feature flag to
//! use Morgan/ECFP4 Tanimoto similarity from `nexcore-molcore`. Swap
//! `smiles_similarity` for `morgan_tanimoto` in `build_similarity_matrix`.
//!
//! ## Algorithm
//!
//! 1. Build N×N pairwise similarity matrix
//! 2. Single-linkage agglomerative clustering: merge two clusters if any
//!    member pair exceeds `threshold`
//! 3. Assign cluster IDs and collect shared adverse events
//!
//! ## Tier: T2-C (Σ + κ + ∂)
//! Sum (Σ) of comparisons (κ) partitioned by boundary (∂).

use crate::error::{RetrocastError, RetrocastResult};
use crate::types::{StructuralCluster, StructuredSignal};
use std::collections::{HashMap, HashSet};

/// Cluster compounds by structural similarity.
///
/// Uses SMILES character bigram Jaccard similarity until Morgan fingerprints
/// are available (Task 7 / `fingerprints` feature flag).
///
/// # Errors
/// Returns `RetrocastError::InvalidThreshold` if `threshold` ∉ [0.0, 1.0].
/// Returns `RetrocastError::ClusteringError` if compounds slice is empty.
pub fn cluster_by_similarity(
    compounds: &[StructuredSignal],
    threshold: f64,
) -> RetrocastResult<Vec<StructuralCluster>> {
    if !(0.0..=1.0).contains(&threshold) {
        return Err(RetrocastError::InvalidThreshold { value: threshold });
    }
    if compounds.is_empty() {
        return Err(RetrocastError::ClusteringError(
            "Cannot cluster empty compound list".to_string(),
        ));
    }

    let n = compounds.len();

    // Build pairwise similarity matrix
    let mut sim_matrix = vec![vec![0.0f64; n]; n];
    for i in 0..n {
        sim_matrix[i][i] = 1.0;
        for j in (i + 1)..n {
            let sim = structural_similarity(compounds[i].smiles(), compounds[j].smiles());
            sim_matrix[i][j] = sim;
            sim_matrix[j][i] = sim;
        }
    }

    // Single-linkage agglomerative clustering
    let cluster_ids = single_linkage_cluster(&sim_matrix, n, threshold);

    // Group compounds by cluster assignment
    let mut cluster_members: HashMap<usize, Vec<usize>> = HashMap::new();
    for (idx, &cid) in cluster_ids.iter().enumerate() {
        cluster_members.entry(cid).or_default().push(idx);
    }

    // Build StructuralCluster for each group
    let mut clusters: Vec<StructuralCluster> = cluster_members
        .into_iter()
        .map(|(cluster_id, member_indices)| {
            let members: Vec<String> = member_indices
                .iter()
                .map(|&i| compounds[i].signal.drug.clone())
                .collect();

            // Extract shared adverse events (events present in >50% of members)
            let shared_events = extract_shared_events(
                &member_indices
                    .iter()
                    .map(|&i| &compounds[i])
                    .collect::<Vec<_>>(),
            );

            // Extract common SMILES fragments (character bigrams appearing in all SMILES)
            let common_fragments = extract_common_fragments(
                &member_indices
                    .iter()
                    .filter_map(|&i| compounds[i].smiles())
                    .collect::<Vec<_>>(),
            );

            StructuralCluster {
                cluster_id,
                members,
                common_fragments,
                similarity_threshold: threshold,
                shared_events,
            }
        })
        .collect();

    // Sort by cluster_id for deterministic output
    clusters.sort_by_key(|c| c.cluster_id);

    Ok(clusters)
}

/// Single-linkage agglomerative clustering.
///
/// Returns a cluster ID assignment for each of the `n` items.
/// Two clusters merge if any pair of members has similarity ≥ threshold.
fn single_linkage_cluster(sim_matrix: &[Vec<f64>], n: usize, threshold: f64) -> Vec<usize> {
    // Initialize: each item is its own cluster
    let mut cluster_of: Vec<usize> = (0..n).collect();

    let mut changed = true;
    while changed {
        changed = false;
        for i in 0..n {
            for j in (i + 1)..n {
                if cluster_of[i] != cluster_of[j] && sim_matrix[i][j] >= threshold {
                    // Merge cluster_of[j] into cluster_of[i]
                    let old_id = cluster_of[j];
                    let new_id = cluster_of[i];
                    for slot in cluster_of.iter_mut().take(n) {
                        if *slot == old_id {
                            *slot = new_id;
                        }
                    }
                    changed = true;
                }
            }
        }
    }

    // Remap cluster IDs to 0-based sequential integers
    let mut id_map: HashMap<usize, usize> = HashMap::new();
    let mut next_id = 0usize;
    let mut result = vec![0usize; n];
    for i in 0..n {
        let mapped = *id_map.entry(cluster_of[i]).or_insert_with(|| {
            let id = next_id;
            next_id += 1;
            id
        });
        result[i] = mapped;
    }
    result
}

/// Compute structural similarity between two SMILES strings.
///
/// ## Current Implementation (T7 pending)
/// Character bigram Jaccard similarity — captures shared substructure tokens
/// such as `C(`, `=O`, `cO`, `c1`, etc.
///
/// ## Planned (T7 complete + `fingerprints` feature)
/// Replace with Morgan/ECFP4 Tanimoto from `nexcore_molcore::fingerprint`.
pub fn structural_similarity(smiles_a: Option<&str>, smiles_b: Option<&str>) -> f64 {
    match (smiles_a, smiles_b) {
        (Some(a), Some(b)) => smiles_bigram_jaccard(a, b),
        // No structure available — zero similarity
        _ => 0.0,
    }
}

/// Compute Jaccard similarity between two SMILES strings using character bigrams.
///
/// Bigrams capture paired-token patterns common in SMILES notation
/// (e.g., `C=`, `=O`, `c1`, `cc`, `(=`, etc.).
fn smiles_bigram_jaccard(a: &str, b: &str) -> f64 {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();

    let a_bigrams: HashSet<(char, char)> = a_chars.windows(2).map(|w| (w[0], w[1])).collect();
    let b_bigrams: HashSet<(char, char)> = b_chars.windows(2).map(|w| (w[0], w[1])).collect();

    if a_bigrams.is_empty() && b_bigrams.is_empty() {
        return 1.0; // Both empty — identical
    }

    let intersection = a_bigrams.intersection(&b_bigrams).count();
    let union = a_bigrams.union(&b_bigrams).count();

    if union == 0 {
        1.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Extract adverse events shared by ≥ 50% of cluster members.
fn extract_shared_events(members: &[&StructuredSignal]) -> Vec<String> {
    if members.is_empty() {
        return vec![];
    }

    let threshold = (members.len() as f64 * 0.5).ceil() as usize;
    let mut event_counts: HashMap<String, usize> = HashMap::new();

    for member in members {
        event_counts
            .entry(member.signal.event.clone())
            .and_modify(|c| *c += 1)
            .or_insert(1);
    }

    let mut shared: Vec<String> = event_counts
        .into_iter()
        .filter(|(_, count)| *count >= threshold)
        .map(|(event, _)| event)
        .collect();
    shared.sort();
    shared
}

/// Extract common SMILES fragments as character bigrams present in all structures.
///
/// Returns the bigrams (as two-char strings) found across all SMILES in the cluster.
fn extract_common_fragments(smiles_list: &[&str]) -> Vec<String> {
    if smiles_list.is_empty() {
        return vec![];
    }

    let bigram_sets: Vec<HashSet<String>> = smiles_list
        .iter()
        .map(|s| {
            let chars: Vec<char> = s.chars().collect();
            chars
                .windows(2)
                .map(|w| format!("{}{}", w[0], w[1]))
                .collect()
        })
        .collect();

    if bigram_sets.is_empty() {
        return vec![];
    }

    // Intersection across all sets
    let common = bigram_sets[0].clone();
    let mut result: Vec<String> = bigram_sets
        .iter()
        .skip(1)
        .fold(common, |acc, set| acc.intersection(set).cloned().collect())
        .into_iter()
        .collect();

    // Only keep chemically meaningful fragments (not single letters)
    result.retain(|f| {
        !f.chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_whitespace())
    });
    result.sort();
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SignalRecord;

    fn make_structured(drug: &str, smiles: Option<&str>, event: &str) -> StructuredSignal {
        use nexcore_compound_registry::types::{CompoundRecord, ResolutionSource};
        let compound = smiles.map(|s| {
            CompoundRecord::new(
                drug.to_string(),
                Some(s.to_string()),
                ResolutionSource::Manual,
            )
        });
        StructuredSignal {
            signal: SignalRecord {
                drug: drug.to_string(),
                event: event.to_string(),
                prr: 2.5,
                ror: 2.2,
                case_count: 30,
                ror_lci: Some(1.5),
                prr_chi_sq: Some(4.5),
            },
            compound,
            has_structure: smiles.is_some(),
        }
    }

    #[test]
    fn test_smiles_bigram_jaccard_identical() {
        let smiles = "CC(=O)Oc1ccccc1C(=O)O";
        let sim = smiles_bigram_jaccard(smiles, smiles);
        assert!(
            (sim - 1.0).abs() < f64::EPSILON,
            "Identical SMILES should have sim=1.0"
        );
    }

    #[test]
    fn test_smiles_bigram_jaccard_different() {
        let aspirin = "CC(=O)Oc1ccccc1C(=O)O";
        let caffeine = "Cn1cnc2c1c(=O)n(c(=O)n2C)C";
        let sim = smiles_bigram_jaccard(aspirin, caffeine);
        assert!(
            sim < 0.9,
            "Dissimilar molecules should have sim < 0.9, got {sim}"
        );
    }

    #[test]
    fn test_nsaid_similarity_higher_than_unrelated() {
        let aspirin = "CC(=O)Oc1ccccc1C(=O)O";
        let ibuprofen = "CC(C)Cc1ccc(cc1)C(C)C(=O)O";
        let caffeine = "Cn1cnc2c1c(=O)n(c(=O)n2C)C";

        let nsaid_sim = smiles_bigram_jaccard(aspirin, ibuprofen);
        let unrelated_sim = smiles_bigram_jaccard(aspirin, caffeine);

        // NSAIDs should be more similar to each other than to caffeine
        assert!(
            nsaid_sim > unrelated_sim,
            "NSAID similarity ({nsaid_sim:.3}) should exceed aspirin/caffeine ({unrelated_sim:.3})"
        );
    }

    #[test]
    fn test_cluster_threshold_zero_groups_all() {
        let compounds = vec![
            make_structured("aspirin", Some("CC(=O)Oc1ccccc1C(=O)O"), "bleeding"),
            make_structured("ibuprofen", Some("CC(C)Cc1ccc(cc1)C(C)C(=O)O"), "bleeding"),
            make_structured("caffeine", Some("Cn1cnc2c1c(=O)n(c(=O)n2C)C"), "arrhythmia"),
        ];

        let result = cluster_by_similarity(&compounds, 0.0);
        assert!(result.is_ok());
        if let Ok(clusters) = result {
            // threshold=0.0 means all pairs qualify — single cluster
            assert_eq!(
                clusters.len(),
                1,
                "All compounds should cluster at threshold=0"
            );
        }
    }

    #[test]
    fn test_cluster_threshold_one_each_alone() {
        let compounds = vec![
            make_structured("aspirin", Some("CC(=O)Oc1ccccc1C(=O)O"), "bleeding"),
            make_structured("caffeine", Some("Cn1cnc2c1c(=O)n(c(=O)n2C)C"), "arrhythmia"),
        ];

        let result = cluster_by_similarity(&compounds, 1.0);
        assert!(result.is_ok());
        if let Ok(clusters) = result {
            // threshold=1.0 means only identical SMILES merge — each alone
            assert_eq!(
                clusters.len(),
                2,
                "At threshold=1.0, different molecules form separate clusters"
            );
        }
    }

    #[test]
    fn test_cluster_invalid_threshold_returns_error() {
        let compounds = vec![make_structured(
            "aspirin",
            Some("CC(=O)Oc1ccccc1C(=O)O"),
            "bleeding",
        )];
        let result = cluster_by_similarity(&compounds, 1.5);
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(RetrocastError::InvalidThreshold { .. })
        ));
    }

    #[test]
    fn test_cluster_empty_returns_error() {
        let result = cluster_by_similarity(&[], 0.5);
        assert!(result.is_err());
        assert!(matches!(result, Err(RetrocastError::ClusteringError(_))));
    }

    #[test]
    fn test_cluster_without_smiles_forms_own_cluster() {
        let compounds = vec![
            make_structured("aspirin", Some("CC(=O)Oc1ccccc1C(=O)O"), "bleeding"),
            make_structured("unknown_drug", None, "rash"),
        ];

        let result = cluster_by_similarity(&compounds, 0.5);
        assert!(result.is_ok());
        if let Ok(clusters) = result {
            // Unknown drug has sim=0.0 with aspirin — should be in separate cluster
            assert_eq!(clusters.len(), 2);
        }
    }

    #[test]
    fn test_shared_events_extraction() {
        let sig1 = make_structured("aspirin", Some("CC(=O)Oc1ccccc1C(=O)O"), "GI bleeding");
        let sig2 = make_structured(
            "ibuprofen",
            Some("CC(C)Cc1ccc(cc1)C(C)C(=O)O"),
            "GI bleeding",
        );
        let sig3 = make_structured(
            "naproxen",
            Some("COc1ccc2cc(ccc2c1)C(C)C(=O)O"),
            "GI bleeding",
        );

        let members: Vec<&StructuredSignal> = vec![&sig1, &sig2, &sig3];
        let shared = extract_shared_events(&members);
        assert!(shared.contains(&"GI bleeding".to_string()));
    }
}
