//! Titration — measure semantic content against canonical standards.
//!
//! Chemistry: Add a known standard to an unknown until equivalence point.
//! Semantics: React an expression against canonical atoms to measure meaning.

use nexcore_id::NexId;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::element::Atom;

/// A single point on the titration curve.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitrationPoint {
    /// Which canonical atom was added at this step.
    pub titrant_atom: NexId,
    pub titrant_label: String,
    /// Volume of titrant added.
    pub volume_added: OrderedFloat<f64>,
    /// Current "pH" — residual unmatched meaning.
    pub residual_meaning: OrderedFloat<f64>,
    /// Rate of change — steep drop = sharp equivalence point.
    pub delta: OrderedFloat<f64>,
}

/// The complete titration curve for an expression.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitrationCurve {
    pub analyte: String,
    pub points: Vec<TitrationPoint>,
    pub equivalence_points: Vec<EquivalencePoint>,
    /// Residual after all titrants exhausted — unexplained meaning.
    pub residual: f64,
}

/// An equivalence point — sharp transition indicating a canonical match.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalencePoint {
    pub matched_atom: NexId,
    pub matched_label: String,
    /// Sharpness: 0.0 = broad (ambiguous), 1.0 = razor-sharp (exact).
    pub sharpness: OrderedFloat<f64>,
    /// How much of the analyte's meaning this atom accounts for.
    pub coverage: OrderedFloat<f64>,
}

/// The titration engine.
pub struct Titrator {
    /// Canonical atoms to use as titrants.
    pub titrants: Vec<Atom>,
    /// Minimum delta to detect an equivalence point.
    pub endpoint_threshold: f64,
}

impl Titrator {
    pub fn new(titrants: Vec<Atom>) -> Self {
        Self {
            titrants,
            endpoint_threshold: 0.1,
        }
    }

    /// Titrate an expression against all available canonical standards.
    pub fn titrate(&self, analyte: &str) -> TitrationCurve {
        let mut points = Vec::new();
        let mut equivalence_points = Vec::new();
        let mut residual_meaning = 1.0_f64;

        for titrant in &self.titrants {
            let similarity = compute_similarity(analyte, &titrant.label);

            if similarity > 0.1 {
                let volume_needed = similarity;
                let new_residual = (residual_meaning - similarity).max(0.0);
                let delta = residual_meaning - new_residual;

                points.push(TitrationPoint {
                    titrant_atom: titrant.id,
                    titrant_label: titrant.label.clone(),
                    volume_added: OrderedFloat(volume_needed),
                    residual_meaning: OrderedFloat(new_residual),
                    delta: OrderedFloat(delta),
                });

                if delta > self.endpoint_threshold {
                    let sharpness = (delta / volume_needed).min(1.0);
                    equivalence_points.push(EquivalencePoint {
                        matched_atom: titrant.id,
                        matched_label: titrant.label.clone(),
                        sharpness: OrderedFloat(sharpness),
                        coverage: OrderedFloat(delta),
                    });
                }

                residual_meaning = new_residual;
            }
        }

        TitrationCurve {
            analyte: analyte.to_string(),
            points,
            equivalence_points,
            residual: residual_meaning,
        }
    }
}

/// Prove equivalence between two expressions via titration comparison.
pub fn prove_equivalence(
    titrator: &Titrator,
    expression_a: &str,
    expression_b: &str,
) -> EquivalenceProof {
    let curve_a = titrator.titrate(expression_a);
    let curve_b = titrator.titrate(expression_b);

    let atoms_a: Vec<NexId> = curve_a
        .equivalence_points
        .iter()
        .map(|e| e.matched_atom)
        .collect();
    let atoms_b: Vec<NexId> = curve_b
        .equivalence_points
        .iter()
        .map(|e| e.matched_atom)
        .collect();

    let shared: Vec<NexId> = atoms_a
        .iter()
        .filter(|a| atoms_b.contains(a))
        .copied()
        .collect();
    let only_a: Vec<NexId> = atoms_a
        .iter()
        .filter(|a| !atoms_b.contains(a))
        .copied()
        .collect();
    let only_b: Vec<NexId> = atoms_b
        .iter()
        .filter(|b| !atoms_a.contains(b))
        .copied()
        .collect();

    let overlap = if atoms_a.is_empty() && atoms_b.is_empty() {
        0.0
    } else {
        shared.len() as f64 / (atoms_a.len().max(atoms_b.len())) as f64
    };

    EquivalenceProof {
        expression_a: expression_a.to_string(),
        expression_b: expression_b.to_string(),
        curve_a,
        curve_b,
        shared_atoms: shared.len(),
        unique_to_a: only_a.len(),
        unique_to_b: only_b.len(),
        equivalence_score: OrderedFloat(overlap),
        verdict: if overlap > 0.90 {
            EquivalenceVerdict::Equivalent
        } else if overlap > 0.60 {
            EquivalenceVerdict::PartialOverlap
        } else {
            EquivalenceVerdict::Distinct
        },
    }
}

/// The formal proof of equivalence between two expressions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquivalenceProof {
    pub expression_a: String,
    pub expression_b: String,
    pub curve_a: TitrationCurve,
    pub curve_b: TitrationCurve,
    pub shared_atoms: usize,
    pub unique_to_a: usize,
    pub unique_to_b: usize,
    pub equivalence_score: OrderedFloat<f64>,
    pub verdict: EquivalenceVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EquivalenceVerdict {
    /// Expressions are semantically equivalent (>90% shared).
    Equivalent,
    /// Expressions overlap but are not equivalent (60-90%).
    PartialOverlap,
    /// Expressions are distinct (<60% shared).
    Distinct,
}

/// Synonym-aware similarity computation.
///
/// Uses curated synonym groups to recognize that "event" ≈ "reaction",
/// "cardiac" ≈ "heart", "immunization" ≈ "vaccine", etc.
///
/// Phase 1: curated synonym groups from ICH/MedDRA/WHO-UMC.
/// Phase 2 (future): spectral vector cosine similarity from corpus embeddings.
fn compute_similarity(analyte: &str, titrant_label: &str) -> f64 {
    let registry = crate::synonymy::SynonymRegistry::pv_standard();
    let analyte_lower = analyte.to_lowercase();
    let label_lower = titrant_label.to_lowercase();

    // Direct substring containment (original heuristic, still valid)
    if analyte_lower.contains(&label_lower) {
        return 0.85;
    }

    // Synonym-aware word matching
    let analyte_words: Vec<&str> = analyte_lower.split_whitespace().collect();
    let label_words: Vec<&str> = label_lower.split_whitespace().collect();

    let mut total_similarity = 0.0_f64;
    let mut matched_count = 0_usize;

    for label_word in &label_words {
        let mut best_match = 0.0_f64;

        for analyte_word in &analyte_words {
            // Exact match
            if analyte_word == label_word {
                best_match = 1.0;
                break;
            }

            // Synonym match
            if let Some(sim) = registry.synonym_similarity(analyte_word, label_word) {
                best_match = best_match.max(sim);
            }
        }

        if best_match > 0.1 {
            total_similarity += best_match;
            matched_count += 1;
        }
    }

    if matched_count > 0 {
        (total_similarity / label_words.len().max(1) as f64) * 0.85
    } else {
        0.0
    }
}
