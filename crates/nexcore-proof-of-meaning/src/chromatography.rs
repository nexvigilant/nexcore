//! Chromatography — hierarchy binding by differential affinity.
//!
//! Chemistry: Separate components by affinity to a stationary phase.
//! Semantics: Bind atoms to hierarchy positions in the classification.

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::element::ElementClass;

/// A chromatographic band — one component bound to its hierarchy position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Band {
    /// The atom that bound to this position.
    pub atom_label: String,
    /// Where it bound in the hierarchy.
    pub bound_class: ElementClass,
    /// How strongly it bound — high = confident placement.
    pub binding_affinity: OrderedFloat<f64>,
    /// Bandwidth — narrow = precise, broad = ambiguous.
    pub bandwidth: OrderedFloat<f64>,
    /// Alternative binding sites with affinities.
    pub alternative_sites: Vec<(ElementClass, OrderedFloat<f64>)>,
}

/// Complete chromatographic separation result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chromatogram {
    pub input_expression: String,
    pub bands: Vec<Band>,
    pub resolution_scores: Vec<ResolutionScore>,
    pub quality: SeparationQuality,
}

/// Resolution between two adjacent bands.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionScore {
    pub band_a: String,
    pub band_b: String,
    /// > 1.5 = baseline resolved. 1.0-1.5 = partial. < 1.0 = co-eluting.
    pub resolution: OrderedFloat<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeparationQuality {
    /// All bands baseline-resolved.
    BaselineResolved,
    /// Most bands resolved, some co-elution.
    PartiallyResolved {
        co_eluting_pairs: Vec<(String, String)>,
    },
    /// Poor separation — many overlapping assignments.
    PoorResolution,
}

/// The chromatographic column — the separation engine.
pub struct Column {
    /// Available hierarchy positions (the "stationary phase").
    pub stationary_phases: Vec<ElementClass>,
}

impl Column {
    /// Create a standard PV chromatography column with all element classes.
    pub fn pv_standard() -> Self {
        Self {
            stationary_phases: vec![
                ElementClass::OrganSystem,
                ElementClass::Causality,
                ElementClass::Temporality,
                ElementClass::Severity,
                ElementClass::ObservationType,
                ElementClass::Modifier,
                ElementClass::Action,
                ElementClass::Outcome,
            ],
        }
    }

    /// Run chromatographic separation on an expression.
    pub fn separate(&self, expression: &str) -> Chromatogram {
        let tokens: Vec<&str> = expression.split_whitespace().collect();
        let mut bands = Vec::new();

        for token in &tokens {
            let mut best_class = ElementClass::Modifier;
            let mut best_affinity = 0.0_f64;
            let mut alternatives = Vec::new();

            for phase in &self.stationary_phases {
                let affinity = compute_affinity(token, phase);
                if affinity > best_affinity {
                    if best_affinity > 0.1 {
                        alternatives.push((best_class.clone(), OrderedFloat(best_affinity)));
                    }
                    best_class = phase.clone();
                    best_affinity = affinity;
                } else if affinity > 0.1 {
                    alternatives.push((phase.clone(), OrderedFloat(affinity)));
                }
            }

            let second_best = alternatives
                .iter()
                .map(|(_, a)| a.into_inner())
                .fold(0.0_f64, f64::max);
            let bandwidth = if best_affinity > 0.0 {
                1.0 - ((best_affinity - second_best) / best_affinity).min(1.0)
            } else {
                1.0
            };

            bands.push(Band {
                atom_label: token.to_string(),
                bound_class: best_class,
                binding_affinity: OrderedFloat(best_affinity),
                bandwidth: OrderedFloat(bandwidth),
                alternative_sites: alternatives,
            });
        }

        // Compute resolution between adjacent bands
        let mut resolution_scores = Vec::new();
        for window in bands.windows(2) {
            let (a, b) = (&window[0], &window[1]);
            let position_diff =
                (a.binding_affinity.into_inner() - b.binding_affinity.into_inner()).abs();
            let avg_bandwidth = (a.bandwidth.into_inner() + b.bandwidth.into_inner()) / 2.0;
            let resolution = if avg_bandwidth > 0.0 {
                position_diff / avg_bandwidth
            } else {
                f64::INFINITY
            };

            resolution_scores.push(ResolutionScore {
                band_a: a.atom_label.clone(),
                band_b: b.atom_label.clone(),
                resolution: OrderedFloat(resolution),
            });
        }

        let co_eluting: Vec<(String, String)> = resolution_scores
            .iter()
            .filter(|r| r.resolution.into_inner() < 1.0)
            .map(|r| (r.band_a.clone(), r.band_b.clone()))
            .collect();

        let quality = if co_eluting.is_empty() {
            SeparationQuality::BaselineResolved
        } else if co_eluting.len() <= 2 {
            SeparationQuality::PartiallyResolved {
                co_eluting_pairs: co_eluting,
            }
        } else {
            SeparationQuality::PoorResolution
        };

        Chromatogram {
            input_expression: expression.to_string(),
            bands,
            resolution_scores,
            quality,
        }
    }
}

/// Synonym-aware binding affinity between a token and a hierarchy position.
///
/// Uses curated synonym groups so that "heart" binds to OrganSystem
/// with the same strength as "cardiac", and "vaccination" binds to
/// Action just like "immunization".
fn compute_affinity(token: &str, class: &ElementClass) -> f64 {
    let registry = crate::synonymy::SynonymRegistry::pv_standard();
    let t = token.to_lowercase();

    // First: check if the synonym registry resolves this token to this class
    if let Some((canonical, resolved_class, similarity)) = registry.resolve(&t) {
        if &resolved_class == class {
            // Direct class match via synonym resolution
            // Canonical tokens get 0.95, synonyms get scaled by group similarity
            let is_canonical = canonical == t;
            return if is_canonical {
                0.95
            } else {
                0.95 * similarity
            };
        }
    }

    // Fallback: hardcoded affinities for tokens not in synonym groups
    // (preserves original behavior for edge cases)
    match class {
        ElementClass::OrganSystem => match t.as_str() {
            "cardiac" | "hepatic" | "renal" | "pulmonary" | "neurological" | "dermatologic"
            | "gastrointestinal" | "hematologic" => 0.95,
            _ => 0.05,
        },
        ElementClass::Causality => match t.as_str() {
            "related" | "unrelated" | "associated" | "attributed" | "causal" => 0.90,
            "suspected" | "possible" | "probable" | "definite" => 0.80,
            "coincidental" => 0.85,
            _ => 0.05,
        },
        ElementClass::Temporality => match t.as_str() {
            "acute" | "chronic" | "delayed" | "immediate" | "onset" => 0.90,
            "rapid" | "gradual" | "intermittent" | "persistent" => 0.80,
            "following" | "after" | "during" | "prior" => 0.70,
            _ => 0.05,
        },
        ElementClass::Severity => match t.as_str() {
            "mild" | "moderate" | "severe" | "life-threatening" | "fatal" => 0.95,
            "serious" | "non-serious" => 0.85,
            "significant" | "trivial" => 0.70,
            _ => 0.05,
        },
        ElementClass::ObservationType => match t.as_str() {
            "event" | "reaction" | "effect" | "finding" | "experience" => 0.90,
            "symptom" | "sign" | "diagnosis" | "condition" => 0.85,
            "observation" | "report" => 0.75,
            _ => 0.05,
        },
        ElementClass::Modifier => match t.as_str() {
            "unexpected" | "expected" | "unlisted" | "listed" => 0.90,
            "new" | "known" | "novel" | "established" => 0.80,
            "drug" | "adverse" => 0.40,
            _ => 0.15,
        },
        ElementClass::Action => match t.as_str() {
            "withdrawn" | "discontinued" | "reduced" | "interrupted" => 0.95,
            "hospitalized" | "treated" | "monitored" => 0.85,
            _ => 0.05,
        },
        ElementClass::Outcome => match t.as_str() {
            "recovered" | "recovering" | "resolved" | "fatal" | "died" => 0.95,
            "improved" | "worsened" | "unchanged" | "unknown" => 0.85,
            "sequelae" | "residual" => 0.80,
            _ => 0.05,
        },
    }
}
