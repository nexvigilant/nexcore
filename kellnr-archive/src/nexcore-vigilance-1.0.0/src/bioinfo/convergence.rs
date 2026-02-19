//! Convergence scoring utilities for pathway overlap analysis.
//!
//! This module provides algorithms for measuring how biological entities
//! converge on shared pathways, using Jaccard similarity and related metrics.
//!
//! ## Key Concepts
//!
//! - **Jaccard Similarity**: J(A,B) = |A ∩ B| / |A ∪ B|
//! - **Convergence Interpretation**: Categorical labels for score ranges
//! - **Multi-Entity Convergence**: Finding pathway "hotspots" where multiple entities meet

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::bioinfo::error::{BioinfoError, BioinfoResult};

// =============================================================================
// Convergence Interpretation
// =============================================================================

/// Categorical interpretation of convergence scores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConvergenceLevel {
    /// Score < 0.1: Entities operate in distinct biological contexts.
    Minimal,
    /// Score 0.1-0.2: Limited pathway sharing, different primary mechanisms.
    Low,
    /// Score 0.2-0.4: Notable overlap worth investigating.
    Moderate,
    /// Score 0.4-0.6: Significant convergence, strong repositioning candidate.
    High,
    /// Score 0.6-0.8: Extensive pathway sharing, highly related activities.
    VeryHigh,
    /// Score >= 0.8: Near-complete overlap, functionally equivalent.
    NearIdentical,
}

impl ConvergenceLevel {
    /// Returns a detailed description of what this convergence level means.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::Minimal => {
                "Entities operate in largely distinct biological contexts with minimal pathway overlap."
            }
            Self::Low => {
                "Limited pathway sharing suggests different primary mechanisms with some peripheral connections."
            }
            Self::Moderate => {
                "Notable pathway overlap indicates shared biological processes worth investigating."
            }
            Self::High => {
                "Significant pathway convergence suggests related mechanisms - strong repositioning candidate."
            }
            Self::VeryHigh => {
                "Extensive pathway sharing indicates highly related biological activities."
            }
            Self::NearIdentical => {
                "Near-complete pathway overlap suggests functionally equivalent mechanisms."
            }
        }
    }
}

// =============================================================================
// Core Algorithms
// =============================================================================

/// Computes the Jaccard similarity coefficient between two sets.
///
/// The Jaccard index measures similarity between finite sets as:
/// J(A,B) = |A ∩ B| / |A ∪ B|
///
/// # Arguments
/// * `set_a` - First set of pathway IDs
/// * `set_b` - Second set of pathway IDs
///
/// # Returns
/// Jaccard index between 0.0 (no overlap) and 1.0 (identical sets)
///
/// # Examples
/// ```
/// use std::collections::HashSet;
/// use nexcore_vigilance::bioinfo::convergence::compute_jaccard;
///
/// let set_a: HashSet<&str> = ["hsa04110", "hsa04151", "hsa04010"].into_iter().collect();
/// let set_b: HashSet<&str> = ["hsa04110", "hsa04151", "hsa05200"].into_iter().collect();
///
/// let score = compute_jaccard(&set_a, &set_b);
/// assert!((score - 0.5).abs() < 0.001); // 2 shared / 4 total = 0.5
/// ```
#[must_use]
pub fn compute_jaccard<T: Eq + std::hash::Hash>(set_a: &HashSet<T>, set_b: &HashSet<T>) -> f64 {
    if set_a.is_empty() && set_b.is_empty() {
        return 0.0;
    }

    let intersection = set_a.intersection(set_b).count();
    let union = set_a.union(set_b).count();

    if union == 0 {
        0.0
    } else {
        intersection as f64 / union as f64
    }
}

/// Interprets a convergence score into a categorical level.
///
/// # Arguments
/// * `score` - Jaccard similarity score (0.0-1.0)
///
/// # Returns
/// Categorical interpretation of the convergence level
///
/// # Examples
/// ```
/// use nexcore_vigilance::bioinfo::convergence::{interpret_convergence, ConvergenceLevel};
///
/// assert_eq!(interpret_convergence(0.05), ConvergenceLevel::Minimal);
/// assert_eq!(interpret_convergence(0.35), ConvergenceLevel::Moderate);
/// assert_eq!(interpret_convergence(0.85), ConvergenceLevel::NearIdentical);
/// ```
#[must_use]
pub fn interpret_convergence(score: f64) -> ConvergenceLevel {
    if score < 0.1 {
        ConvergenceLevel::Minimal
    } else if score < 0.2 {
        ConvergenceLevel::Low
    } else if score < 0.4 {
        ConvergenceLevel::Moderate
    } else if score < 0.6 {
        ConvergenceLevel::High
    } else if score < 0.8 {
        ConvergenceLevel::VeryHigh
    } else {
        ConvergenceLevel::NearIdentical
    }
}

// =============================================================================
// Convergence Result Types
// =============================================================================

/// Entity information in convergence analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityInfo {
    /// Entity identifier.
    pub id: String,
    /// Number of pathways this entity participates in.
    pub pathway_count: usize,
    /// List of pathway IDs.
    pub pathways: Vec<String>,
    /// Optional entity type label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
}

/// Shared pathway details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedPathway {
    /// KEGG pathway ID.
    pub pathway_id: String,
    /// Pathway name (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Pathway class/category (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class: Option<String>,
    /// URL to pathway map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Analysis section of convergence result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceAnalysis {
    /// Details of shared pathways.
    pub shared_pathways: Vec<SharedPathway>,
    /// Count of shared pathways.
    pub shared_count: usize,
    /// Pathways unique to entity A.
    pub unique_to_a: Vec<String>,
    /// Pathways unique to entity B.
    pub unique_to_b: Vec<String>,
    /// Total unique pathways across both entities.
    pub total_unique_pathways: usize,
}

/// Convergence score details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceScore {
    /// Jaccard similarity score (0.0-1.0).
    pub score: f64,
    /// Categorical interpretation.
    pub interpretation: ConvergenceLevel,
    /// Human-readable description.
    pub description: String,
}

/// Complete convergence analysis result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvergenceResult {
    /// First entity information.
    pub entity_a: EntityInfo,
    /// Second entity information.
    pub entity_b: EntityInfo,
    /// Analysis details.
    pub analysis: ConvergenceAnalysis,
    /// Convergence score and interpretation.
    pub convergence: ConvergenceScore,
}

/// Computes a full convergence analysis between two entities.
///
/// # Arguments
/// * `entity_a` - First entity identifier
/// * `entity_b` - Second entity identifier
/// * `pathways_a` - Set of pathway IDs for entity A
/// * `pathways_b` - Set of pathway IDs for entity B
/// * `pathway_details` - Optional map of pathway_id -> (name, class) details
///
/// # Returns
/// Complete convergence analysis result
///
/// # Examples
/// ```
/// use std::collections::{HashSet, HashMap};
/// use nexcore_vigilance::bioinfo::convergence::compute_convergence_result;
///
/// let pathways_a: HashSet<String> = ["hsa04110", "hsa04151"].into_iter().map(String::from).collect();
/// let pathways_b: HashSet<String> = ["hsa04110", "hsa05200"].into_iter().map(String::from).collect();
///
/// let result = compute_convergence_result(
///     "CDK9",
///     "BRD4",
///     &pathways_a,
///     &pathways_b,
///     None,
/// );
///
/// assert_eq!(result.analysis.shared_count, 1);
/// assert_eq!(result.analysis.shared_pathways[0].pathway_id, "hsa04110");
/// ```
#[must_use]
pub fn compute_convergence_result(
    entity_a: &str,
    entity_b: &str,
    pathways_a: &HashSet<String>,
    pathways_b: &HashSet<String>,
    pathway_details: Option<&HashMap<String, (String, String)>>,
) -> ConvergenceResult {
    let shared: HashSet<&String> = pathways_a.intersection(pathways_b).collect();
    let unique_a: Vec<String> = pathways_a.difference(pathways_b).cloned().collect();
    let unique_b: Vec<String> = pathways_b.difference(pathways_a).cloned().collect();

    let score = compute_jaccard(pathways_a, pathways_b);
    let interpretation = interpret_convergence(score);

    // Build shared pathway details
    let mut shared_pathways: Vec<SharedPathway> = shared
        .iter()
        .map(|&pid| {
            let (name, class) = pathway_details
                .and_then(|details| details.get(pid))
                .map(|(n, c)| (Some(n.clone()), Some(c.clone())))
                .unwrap_or((None, None));

            SharedPathway {
                pathway_id: pid.clone(),
                name,
                class,
                url: Some(format!("https://www.kegg.jp/pathway/{pid}")),
            }
        })
        .collect();

    // Sort for deterministic output
    shared_pathways.sort_by(|a, b| a.pathway_id.cmp(&b.pathway_id));

    let mut sorted_pathways_a: Vec<String> = pathways_a.iter().cloned().collect();
    sorted_pathways_a.sort();

    let mut sorted_pathways_b: Vec<String> = pathways_b.iter().cloned().collect();
    sorted_pathways_b.sort();

    let mut sorted_unique_a = unique_a;
    sorted_unique_a.sort();

    let mut sorted_unique_b = unique_b;
    sorted_unique_b.sort();

    ConvergenceResult {
        entity_a: EntityInfo {
            id: entity_a.to_string(),
            pathway_count: pathways_a.len(),
            pathways: sorted_pathways_a,
            entity_type: None,
        },
        entity_b: EntityInfo {
            id: entity_b.to_string(),
            pathway_count: pathways_b.len(),
            pathways: sorted_pathways_b,
            entity_type: None,
        },
        analysis: ConvergenceAnalysis {
            shared_pathways,
            shared_count: shared.len(),
            unique_to_a: sorted_unique_a,
            unique_to_b: sorted_unique_b,
            total_unique_pathways: pathways_a.union(pathways_b).count(),
        },
        convergence: ConvergenceScore {
            score: (score * 10000.0).round() / 10000.0, // Round to 4 decimal places
            interpretation,
            description: interpretation.description().to_string(),
        },
    }
}

// =============================================================================
// Multi-Entity Convergence
// =============================================================================

/// Pathway hotspot in multi-entity convergence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathwayHotspot {
    /// KEGG pathway ID.
    pub pathway_id: String,
    /// Number of entities sharing this pathway.
    pub entity_count: usize,
    /// Fraction of entities sharing this pathway.
    pub fraction: f64,
}

/// Result of multi-entity convergence analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiEntityConvergence {
    /// List of entity IDs analyzed.
    pub entities: Vec<String>,
    /// Number of entities.
    pub entity_count: usize,
    /// Total unique pathways across all entities.
    pub total_unique_pathways: usize,
    /// Pathways shared by ALL entities.
    pub universal_pathways: Vec<String>,
    /// Count of universal pathways.
    pub universal_count: usize,
    /// Pathway hotspots (shared by 2+ entities), sorted by count.
    pub pathway_hotspots: Vec<PathwayHotspot>,
    /// Pairwise Jaccard scores (key format: "entity1_vs_entity2").
    pub pairwise_convergence: HashMap<String, f64>,
    /// Average pairwise convergence score.
    pub average_pairwise_convergence: f64,
}

/// Computes convergence analysis across multiple entities.
///
/// Useful for finding pathway "hotspots" where multiple drugs/genes converge.
///
/// # Arguments
/// * `entities` - Map of entity_id -> set of pathway IDs
///
/// # Returns
/// Multi-entity convergence analysis or error if fewer than 2 entities
///
/// # Examples
/// ```
/// use std::collections::{HashSet, HashMap};
/// use nexcore_vigilance::bioinfo::convergence::compute_multi_entity_convergence;
///
/// let mut entities: HashMap<String, HashSet<String>> = HashMap::new();
/// entities.insert("CDK9".into(), ["hsa04110", "hsa04151"].into_iter().map(String::from).collect());
/// entities.insert("BRD4".into(), ["hsa04110", "hsa05200"].into_iter().map(String::from).collect());
/// entities.insert("HEXIM1".into(), ["hsa04110", "hsa04151"].into_iter().map(String::from).collect());
///
/// let result = compute_multi_entity_convergence(&entities);
/// assert!(result.is_ok());
/// ```
pub fn compute_multi_entity_convergence(
    entities: &HashMap<String, HashSet<String>>,
) -> BioinfoResult<MultiEntityConvergence> {
    if entities.len() < 2 {
        return Err(BioinfoError::InsufficientEntities(entities.len()));
    }

    let entity_ids: Vec<String> = entities.keys().cloned().collect();

    // Collect all pathways - take ownership by iterating and cloning once
    let all_pathways: HashSet<String> = entities
        .values()
        .flat_map(|pathways| pathways.iter().cloned())
        .collect();

    // Find universal pathways (shared by ALL entities)
    let universal_pathways: HashSet<String> = if entities.is_empty() {
        HashSet::new()
    } else {
        let mut iter = entities.values();
        let first = iter.next().map_or_else(HashSet::new, Clone::clone);
        iter.fold(first, |acc, set| acc.intersection(set).cloned().collect())
    };

    // Count how many entities share each pathway
    // Use map + collect to avoid clone in loop
    let pathway_entity_count: HashMap<String, usize> = all_pathways
        .iter()
        .map(|pathway| {
            let count = entities
                .values()
                .filter(|paths| paths.contains(pathway))
                .count();
            (pathway.clone(), count)
        })
        .collect();

    // Build hotspots (pathways shared by 2+ entities)
    let mut hotspots: Vec<PathwayHotspot> = pathway_entity_count
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(pid, count)| PathwayHotspot {
            pathway_id: pid,
            entity_count: count,
            fraction: count as f64 / entities.len() as f64,
        })
        .collect();

    // Sort by entity_count descending
    hotspots.sort_by(|a, b| b.entity_count.cmp(&a.entity_count));

    // Compute pairwise Jaccard scores
    let mut pairwise: HashMap<String, f64> = HashMap::new();
    for (i, e1) in entity_ids.iter().enumerate() {
        for e2 in entity_ids.iter().skip(i + 1) {
            let key = format!("{e1}_vs_{e2}");
            let score = compute_jaccard(
                entities.get(e1).map_or(&HashSet::new(), |s| s),
                entities.get(e2).map_or(&HashSet::new(), |s| s),
            );
            pairwise.insert(key, (score * 10000.0).round() / 10000.0);
        }
    }

    // Compute average
    let avg = if pairwise.is_empty() {
        0.0
    } else {
        let sum: f64 = pairwise.values().sum();
        (sum / pairwise.len() as f64 * 10000.0).round() / 10000.0
    };

    let mut sorted_universal: Vec<String> = universal_pathways.into_iter().collect();
    sorted_universal.sort();

    Ok(MultiEntityConvergence {
        entities: entity_ids,
        entity_count: entities.len(),
        total_unique_pathways: all_pathways.len(),
        universal_pathways: sorted_universal,
        universal_count: entities
            .values()
            .next()
            .map(|first| {
                entities
                    .values()
                    .skip(1)
                    .fold(first.clone(), |acc, set| {
                        acc.intersection(set).cloned().collect()
                    })
                    .len()
            })
            .unwrap_or(0),
        pathway_hotspots: hotspots.into_iter().take(20).collect(),
        pairwise_convergence: pairwise,
        average_pairwise_convergence: avg,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_jaccard_identical() {
        let set: HashSet<&str> = ["a", "b", "c"].into_iter().collect();
        assert!((compute_jaccard(&set, &set) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_jaccard_disjoint() {
        let set_a: HashSet<&str> = ["a", "b"].into_iter().collect();
        let set_b: HashSet<&str> = ["c", "d"].into_iter().collect();
        assert!((compute_jaccard(&set_a, &set_b) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_jaccard_partial() {
        let set_a: HashSet<&str> = ["a", "b", "c"].into_iter().collect();
        let set_b: HashSet<&str> = ["b", "c", "d"].into_iter().collect();
        // Intersection: {b, c} = 2
        // Union: {a, b, c, d} = 4
        // Jaccard = 2/4 = 0.5
        assert!((compute_jaccard(&set_a, &set_b) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_compute_jaccard_empty() {
        let empty: HashSet<&str> = HashSet::new();
        assert!((compute_jaccard(&empty, &empty) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_interpret_convergence_levels() {
        assert_eq!(interpret_convergence(0.05), ConvergenceLevel::Minimal);
        assert_eq!(interpret_convergence(0.15), ConvergenceLevel::Low);
        assert_eq!(interpret_convergence(0.30), ConvergenceLevel::Moderate);
        assert_eq!(interpret_convergence(0.50), ConvergenceLevel::High);
        assert_eq!(interpret_convergence(0.70), ConvergenceLevel::VeryHigh);
        assert_eq!(interpret_convergence(0.90), ConvergenceLevel::NearIdentical);
    }

    #[test]
    fn test_convergence_level_description() {
        let level = ConvergenceLevel::High;
        assert!(level.description().contains("repositioning"));
    }

    #[test]
    fn test_compute_convergence_result() {
        let pathways_a: HashSet<String> = ["hsa04110", "hsa04151"]
            .into_iter()
            .map(String::from)
            .collect();
        let pathways_b: HashSet<String> = ["hsa04110", "hsa05200"]
            .into_iter()
            .map(String::from)
            .collect();

        let result = compute_convergence_result("Gene1", "Gene2", &pathways_a, &pathways_b, None);

        assert_eq!(result.entity_a.id, "Gene1");
        assert_eq!(result.entity_b.id, "Gene2");
        assert_eq!(result.analysis.shared_count, 1);
        assert_eq!(result.analysis.shared_pathways[0].pathway_id, "hsa04110");
        assert_eq!(result.analysis.total_unique_pathways, 3);
        // Jaccard = 1/3 ≈ 0.3333
        assert!((result.convergence.score - 0.3333).abs() < 0.001);
        assert_eq!(
            result.convergence.interpretation,
            ConvergenceLevel::Moderate
        );
    }

    #[test]
    fn test_compute_multi_entity_convergence() {
        let mut entities: HashMap<String, HashSet<String>> = HashMap::new();
        entities.insert(
            "E1".into(),
            ["p1", "p2", "p3"].into_iter().map(String::from).collect(),
        );
        entities.insert(
            "E2".into(),
            ["p1", "p2", "p4"].into_iter().map(String::from).collect(),
        );
        entities.insert(
            "E3".into(),
            ["p1", "p5"].into_iter().map(String::from).collect(),
        );

        let result = compute_multi_entity_convergence(&entities);
        assert!(result.is_ok());

        if let Ok(conv) = result {
            assert_eq!(conv.entity_count, 3);
            assert_eq!(conv.total_unique_pathways, 5);
            assert!(conv.universal_pathways.contains(&"p1".to_string()));
            assert!(!conv.pathway_hotspots.is_empty());
        }
    }

    #[test]
    fn test_multi_entity_insufficient() {
        let mut entities: HashMap<String, HashSet<String>> = HashMap::new();
        entities.insert("E1".into(), HashSet::new());

        let result = compute_multi_entity_convergence(&entities);
        assert!(matches!(result, Err(BioinfoError::InsufficientEntities(1))));
    }
}
