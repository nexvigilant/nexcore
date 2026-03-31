//! Primitive distance — symmetric difference metric on T1 compositions.
//!
//! The distance d(A, B) = |A △ B| counts primitives present in one composition
//! but not the other. This is a proper metric (non-negative, symmetric,
//! satisfies triangle inequality, d(A,A) = 0).
//!
//! Used for nearest-neighbor concept search and cross-domain transfer scoring.
//!
//! Source: `primitives.ipynb` Part II — Metric Space section.

use std::collections::BTreeSet;

use crate::primitiva::{LexPrimitiva, PrimitiveComposition};

/// Result of a distance computation between two compositions.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DistanceResult {
    /// Symmetric difference distance |A △ B|.
    pub distance: usize,
    /// Maximum possible distance (|A ∪ B|).
    pub max_distance: usize,
    /// Normalized similarity (1.0 - distance/max_distance). 1.0 = identical.
    pub similarity: f64,
    /// Primitives in A but not B.
    pub only_in_a: Vec<String>,
    /// Primitives in B but not A.
    pub only_in_b: Vec<String>,
    /// Primitives shared by both.
    pub shared: Vec<String>,
}

/// Compute the symmetric difference distance between two primitive compositions.
///
/// d(A, B) = |A △ B| = |A \ B| + |B \ A|
///
/// This is Algorithm 2 (COMPARE) from the Lex Primitiva proof notebook.
#[must_use]
pub fn distance(a: &PrimitiveComposition, b: &PrimitiveComposition) -> DistanceResult {
    let set_a = a.unique();
    let set_b = b.unique();

    let only_a: BTreeSet<_> = set_a.difference(&set_b).copied().collect();
    let only_b: BTreeSet<_> = set_b.difference(&set_a).copied().collect();
    let shared: BTreeSet<_> = set_a.intersection(&set_b).copied().collect();

    let d = only_a.len() + only_b.len();
    let union_size = set_a.union(&set_b).count();
    let similarity = if union_size == 0 {
        1.0
    } else {
        1.0 - (d as f64 / union_size as f64)
    };

    DistanceResult {
        distance: d,
        max_distance: union_size,
        similarity,
        only_in_a: only_a.iter().map(|p| p.name().to_string()).collect(),
        only_in_b: only_b.iter().map(|p| p.name().to_string()).collect(),
        shared: shared.iter().map(|p| p.name().to_string()).collect(),
    }
}

/// A named concept with its T1 primitive composition for nearest-neighbor search.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConceptEntry {
    /// Concept name.
    pub name: String,
    /// T1 primitive composition.
    pub primitives: Vec<String>,
}

/// Result of a nearest-neighbor search.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NeighborResult {
    /// Concept name.
    pub name: String,
    /// Distance from query.
    pub distance: usize,
    /// Similarity score (0.0-1.0).
    pub similarity: f64,
    /// Primitives shared with query.
    pub shared: Vec<String>,
    /// Primitives unique to this concept (not in query).
    pub unique: Vec<String>,
}

/// Built-in concept catalog for nearest-neighbor search.
///
/// Each entry maps a concept name to its T1 primitive composition.
/// Concepts span multiple domains to enable cross-domain transfer.
#[must_use]
pub fn builtin_catalog() -> Vec<ConceptEntry> {
    vec![
        // Computing
        concept(
            "Transformer",
            &[
                "Sequence",
                "Mapping",
                "Comparison",
                "Quantity",
                "Boundary",
                "Causality",
                "Recursion",
                "State",
                "Location",
                "Nothing",
                "Frequency",
                "Existence",
                "Persistence",
                "Irreversibility",
                "Sum",
            ],
        ),
        concept(
            "Convolution",
            &[
                "Sequence",
                "Mapping",
                "Comparison",
                "Quantity",
                "Boundary",
                "Causality",
            ],
        ),
        concept(
            "RNN/LSTM",
            &[
                "Sequence",
                "Mapping",
                "State",
                "Recursion",
                "Boundary",
                "Causality",
                "Persistence",
            ],
        ),
        concept(
            "Hash Table",
            &["Comparison", "Mapping", "Location", "Boundary", "Existence"],
        ),
        concept(
            "Graph Neural Network",
            &[
                "Comparison",
                "Mapping",
                "Sequence",
                "Recursion",
                "Quantity",
                "Sum",
                "Location",
            ],
        ),
        concept(
            "Database Query",
            &[
                "Comparison",
                "Mapping",
                "Sequence",
                "Boundary",
                "Location",
                "Sum",
                "Quantity",
            ],
        ),
        concept(
            "Compiler",
            &[
                "Mapping",
                "Sequence",
                "Boundary",
                "Causality",
                "State",
                "Location",
            ],
        ),
        concept(
            "Operating System",
            &[
                "Boundary",
                "State",
                "Sequence",
                "Causality",
                "Persistence",
                "Location",
                "Quantity",
            ],
        ),
        // Biology
        concept(
            "Immune System (T-cell)",
            &[
                "Comparison",
                "Mapping",
                "Recursion",
                "Boundary",
                "Existence",
                "Nothing",
                "Sum",
                "Causality",
            ],
        ),
        concept(
            "DNA Replication",
            &[
                "Sequence",
                "Mapping",
                "Persistence",
                "Recursion",
                "Boundary",
                "Existence",
            ],
        ),
        concept(
            "Enzyme (Catalysis)",
            &["Mapping", "Boundary", "Comparison", "Causality", "Quantity"],
        ),
        concept(
            "Neuron",
            &[
                "Mapping",
                "Boundary",
                "State",
                "Causality",
                "Frequency",
                "Sum",
            ],
        ),
        // Pharmacovigilance
        concept(
            "Signal Detection (PV)",
            &[
                "Comparison",
                "Quantity",
                "Boundary",
                "Causality",
                "Existence",
                "Sum",
                "Frequency",
                "State",
            ],
        ),
        concept(
            "Causality Assessment",
            &[
                "Comparison",
                "Causality",
                "Boundary",
                "Sequence",
                "State",
                "Existence",
            ],
        ),
        concept(
            "ICSR Processing",
            &[
                "Boundary",
                "State",
                "Sequence",
                "Persistence",
                "Existence",
                "Location",
            ],
        ),
        concept(
            "Expedited Reporting",
            &[
                "Boundary",
                "Causality",
                "Sequence",
                "Quantity",
                "Irreversibility",
            ],
        ),
        concept(
            "Benefit-Risk",
            &["Comparison", "Quantity", "Boundary", "Mapping", "Sum"],
        ),
        // Economics
        concept(
            "Market Auction",
            &[
                "Comparison",
                "Quantity",
                "Boundary",
                "Causality",
                "Sum",
                "State",
                "Frequency",
            ],
        ),
        concept(
            "Insurance",
            &["Boundary", "Quantity", "Frequency", "Persistence", "Sum"],
        ),
        concept(
            "Supply Chain",
            &[
                "Sequence",
                "Location",
                "State",
                "Causality",
                "Quantity",
                "Persistence",
            ],
        ),
        // Physics
        concept(
            "Wave Function",
            &[
                "Mapping",
                "Frequency",
                "Quantity",
                "Boundary",
                "Sum",
                "State",
            ],
        ),
        concept(
            "Conservation Law",
            &["Boundary", "State", "Nothing", "Existence", "Causality"],
        ),
        concept(
            "Entropy",
            &[
                "Quantity",
                "State",
                "Irreversibility",
                "Boundary",
                "Frequency",
            ],
        ),
    ]
}

/// Resolve a primitive name (case-insensitive) to a `LexPrimitiva`.
fn resolve_primitive(name: &str) -> Option<LexPrimitiva> {
    let lower = name.to_lowercase();
    // Try symbol first, then match by name
    LexPrimitiva::from_symbol(name).or_else(|| {
        LexPrimitiva::all()
            .into_iter()
            .find(|p| p.name().to_lowercase() == lower)
    })
}

fn concept(name: &str, primitives: &[&str]) -> ConceptEntry {
    ConceptEntry {
        name: name.to_string(),
        primitives: primitives.iter().map(|s| s.to_string()).collect(),
    }
}

/// Find the k nearest neighbors to a query composition from the built-in catalog.
///
/// Returns results sorted by distance (ascending), then by name.
#[must_use]
pub fn nearest_neighbors(query: &PrimitiveComposition, k: usize) -> Vec<NeighborResult> {
    nearest_neighbors_from(query, &builtin_catalog(), k)
}

/// Find the k nearest neighbors from a custom catalog.
#[must_use]
pub fn nearest_neighbors_from(
    query: &PrimitiveComposition,
    catalog: &[ConceptEntry],
    k: usize,
) -> Vec<NeighborResult> {
    let mut results: Vec<NeighborResult> = catalog
        .iter()
        .filter_map(|entry| {
            let primitives: Vec<LexPrimitiva> = entry
                .primitives
                .iter()
                .filter_map(|name| resolve_primitive(name))
                .collect();

            if primitives.is_empty() {
                return None;
            }

            let comp = PrimitiveComposition::new(primitives);
            let d = distance(query, &comp);

            Some(NeighborResult {
                name: entry.name.clone(),
                distance: d.distance,
                similarity: d.similarity,
                shared: d.shared,
                unique: d.only_in_b,
            })
        })
        .collect();

    results.sort_by(|a, b| a.distance.cmp(&b.distance).then(a.name.cmp(&b.name)));
    results.truncate(k);
    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitiva::LexPrimitiva;

    #[test]
    fn test_distance_identical() {
        let comp =
            PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Boundary]);
        let d = distance(&comp, &comp);
        assert_eq!(d.distance, 0);
        assert!((d.similarity - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_distance_disjoint() {
        let a = PrimitiveComposition::new(vec![LexPrimitiva::Comparison]);
        let b = PrimitiveComposition::new(vec![LexPrimitiva::Mapping]);
        let d = distance(&a, &b);
        assert_eq!(d.distance, 2);
        assert_eq!(d.shared.len(), 0);
    }

    #[test]
    fn test_distance_overlap() {
        let a = PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
        ]);
        let b = PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Boundary,
            LexPrimitiva::Mapping,
        ]);
        let d = distance(&a, &b);
        assert_eq!(d.distance, 2); // Causality + Mapping
        assert_eq!(d.shared.len(), 2); // Comparison + Boundary
    }

    #[test]
    fn test_nearest_neighbors() {
        // Query: something comparison-heavy
        let query = PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::Causality,
            LexPrimitiva::Existence,
            LexPrimitiva::Sum,
            LexPrimitiva::Frequency,
            LexPrimitiva::State,
        ]);
        let results = nearest_neighbors(&query, 3);
        assert!(!results.is_empty());
        // Signal Detection should be very close (same composition)
        assert_eq!(results[0].name, "Signal Detection (PV)");
        assert_eq!(results[0].distance, 0);
    }

    #[test]
    fn test_triangle_inequality() {
        let a = PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Boundary]);
        let b = PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Mapping]);
        let c = PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Causality]);
        let d_ab = distance(&a, &b).distance;
        let d_bc = distance(&b, &c).distance;
        let d_ac = distance(&a, &c).distance;
        assert!(d_ac <= d_ab + d_bc, "Triangle inequality violated");
    }
}
