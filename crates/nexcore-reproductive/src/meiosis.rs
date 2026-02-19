//! # Meiosis — Sexual Reproduction
//!
//! Combines two parent sources (artifacts, branches, configs) into recombined
//! offspring with traits from both. This is the genetic recombination engine.
//!
//! ## Biological Mapping
//!
//! | Phase | Biological | Claude Code |
//! |-------|-----------|-------------|
//! | Prophase I | Homologous pairing | Identify shared interfaces/types |
//! | Crossing Over | Chromatid exchange | Merge selected modules from each parent |
//! | Metaphase I | Alignment | Dependency resolution |
//! | Anaphase I | Separation | Feature selection (which parent's version) |
//! | Telophase I | Division | First candidate assembly |
//! | Meiosis II | Haploid division | Final artifact with half each parent's traits |

use serde::{Deserialize, Serialize};

/// A parent source contributing genetic material.
///
/// **T1 grounding**: ∃(Existence) + ×(Product) + λ(Location)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parent {
    /// Identifier (branch name, artifact ID, session ID).
    pub id: String,
    /// Traits (features, modules, capabilities) this parent carries.
    pub traits: Vec<String>,
    /// Fitness score of the parent (0.0 to 1.0).
    pub fitness: f64,
}

/// Which parent's version of a trait to select.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraitSelection {
    /// Use parent A's version.
    ParentA,
    /// Use parent B's version.
    ParentB,
    /// Merge both (crossing over).
    Crossover,
    /// Neither — novel mutation.
    Novel,
}

/// A single recombination decision for one trait.
///
/// **T1 grounding**: κ(Comparison) + Σ(Sum) + →(Causality)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecombinationEvent {
    /// The trait being decided on.
    pub trait_name: String,
    /// Which parent's version to use.
    pub selection: TraitSelection,
    /// Reason for the selection.
    pub rationale: String,
}

/// Phase of meiotic division.
///
/// **T1 grounding**: Σ(Sum) + σ(Sequence)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MeioticPhase {
    /// Identify shared interfaces between parents.
    ProphaseI,
    /// Exchange modules/traits between parents.
    CrossingOver,
    /// Resolve dependencies in combined set.
    MetaphaseI,
    /// Select which parent's version of each trait to keep.
    AnaphaseI,
    /// Assemble first candidate.
    TelophaseI,
    /// Final refinement into haploid offspring.
    MeiosisII,
    /// Complete — offspring is ready.
    Complete,
}

impl MeioticPhase {
    /// Advances to the next phase. Returns `None` if already complete.
    pub fn next(self) -> Option<Self> {
        match self {
            Self::ProphaseI => Some(Self::CrossingOver),
            Self::CrossingOver => Some(Self::MetaphaseI),
            Self::MetaphaseI => Some(Self::AnaphaseI),
            Self::AnaphaseI => Some(Self::TelophaseI),
            Self::TelophaseI => Some(Self::MeiosisII),
            Self::MeiosisII => Some(Self::Complete),
            Self::Complete => None,
        }
    }
}

/// The meiotic recombination process.
///
/// **T1 grounding**: →(Causality) + μ(Mapping) + σ(Sequence) + ×(Product)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meiosis {
    /// First parent.
    pub parent_a: Parent,
    /// Second parent.
    pub parent_b: Parent,
    /// Current phase of meiotic division.
    pub phase: MeioticPhase,
    /// Recombination decisions made so far.
    pub recombinations: Vec<RecombinationEvent>,
    /// Shared traits found between parents.
    pub homologous_traits: Vec<String>,
    /// Traits unique to parent A.
    pub unique_a: Vec<String>,
    /// Traits unique to parent B.
    pub unique_b: Vec<String>,
}

impl Meiosis {
    /// Initiates meiosis between two parents.
    pub fn new(parent_a: Parent, parent_b: Parent) -> Self {
        let homologous: Vec<String> = parent_a
            .traits
            .iter()
            .filter(|t| parent_b.traits.contains(t))
            .cloned()
            .collect();

        let unique_a: Vec<String> = parent_a
            .traits
            .iter()
            .filter(|t| !parent_b.traits.contains(t))
            .cloned()
            .collect();

        let unique_b: Vec<String> = parent_b
            .traits
            .iter()
            .filter(|t| !parent_a.traits.contains(t))
            .cloned()
            .collect();

        Self {
            parent_a,
            parent_b,
            phase: MeioticPhase::ProphaseI,
            recombinations: Vec::new(),
            homologous_traits: homologous,
            unique_a,
            unique_b,
        }
    }

    /// Advances to the next meiotic phase.
    /// Returns `false` if already complete.
    pub fn advance(&mut self) -> bool {
        match self.phase.next() {
            Some(next) => {
                self.phase = next;
                true
            }
            None => false,
        }
    }

    /// Records a recombination decision.
    pub fn recombine(&mut self, event: RecombinationEvent) {
        self.recombinations.push(event);
    }

    /// Returns the offspring trait set after meiosis completes.
    /// Requires phase == Complete.
    pub fn offspring_traits(&self) -> Option<Vec<String>> {
        if self.phase != MeioticPhase::Complete {
            return None;
        }

        let mut traits = Vec::new();

        for event in &self.recombinations {
            match event.selection {
                TraitSelection::ParentA | TraitSelection::ParentB | TraitSelection::Novel => {
                    traits.push(event.trait_name.clone());
                }
                TraitSelection::Crossover => {
                    // Crossover produces a merged trait
                    traits.push(format!("{}:merged", event.trait_name));
                }
            }
        }

        Some(traits)
    }

    /// Calculates the genetic diversity of the offspring.
    /// Higher values indicate more novel combinations.
    /// Range: 0.0 (clone of one parent) to 1.0 (maximum recombination).
    pub fn genetic_diversity(&self) -> f64 {
        if self.recombinations.is_empty() {
            return 0.0;
        }

        let crossovers = self
            .recombinations
            .iter()
            .filter(|r| r.selection == TraitSelection::Crossover)
            .count();
        let novels = self
            .recombinations
            .iter()
            .filter(|r| r.selection == TraitSelection::Novel)
            .count();

        let diversity_events = crossovers + (novels * 2); // Novels count double
        let total = self.recombinations.len();

        (diversity_events as f64 / total as f64).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_parents() -> (Parent, Parent) {
        let a = Parent {
            id: "branch-a".into(),
            traits: vec![
                "error-handling".into(),
                "logging".into(),
                "auth".into(),
                "caching".into(),
            ],
            fitness: 0.9,
        };
        let b = Parent {
            id: "branch-b".into(),
            traits: vec![
                "error-handling".into(),
                "logging".into(),
                "telemetry".into(),
                "rate-limiting".into(),
            ],
            fitness: 0.85,
        };
        (a, b)
    }

    #[test]
    fn test_meiosis_initialization() {
        let (a, b) = test_parents();
        let m = Meiosis::new(a, b);

        assert_eq!(m.phase, MeioticPhase::ProphaseI);
        assert_eq!(m.homologous_traits, vec!["error-handling", "logging"]);
        assert_eq!(m.unique_a, vec!["auth", "caching"]);
        assert_eq!(m.unique_b, vec!["telemetry", "rate-limiting"]);
    }

    #[test]
    fn test_meiotic_phase_progression() {
        let (a, b) = test_parents();
        let mut m = Meiosis::new(a, b);

        assert!(m.advance()); // ProphaseI -> CrossingOver
        assert_eq!(m.phase, MeioticPhase::CrossingOver);
        assert!(m.advance()); // -> MetaphaseI
        assert!(m.advance()); // -> AnaphaseI
        assert!(m.advance()); // -> TelophaseI
        assert!(m.advance()); // -> MeiosisII
        assert!(m.advance()); // -> Complete
        assert_eq!(m.phase, MeioticPhase::Complete);
        assert!(!m.advance()); // Can't advance past Complete
    }

    #[test]
    fn test_offspring_traits_requires_complete() {
        let (a, b) = test_parents();
        let m = Meiosis::new(a, b);
        assert!(m.offspring_traits().is_none());
    }

    #[test]
    fn test_offspring_traits_after_recombination() {
        let (a, b) = test_parents();
        let mut m = Meiosis::new(a, b);

        m.recombine(RecombinationEvent {
            trait_name: "error-handling".into(),
            selection: TraitSelection::Crossover,
            rationale: "Both parents have good error handling".into(),
        });
        m.recombine(RecombinationEvent {
            trait_name: "auth".into(),
            selection: TraitSelection::ParentA,
            rationale: "Only parent A has auth".into(),
        });
        m.recombine(RecombinationEvent {
            trait_name: "telemetry".into(),
            selection: TraitSelection::ParentB,
            rationale: "Only parent B has telemetry".into(),
        });

        // Advance to complete
        while m.advance() {}

        let traits = m.offspring_traits().expect("should have traits");
        assert_eq!(traits.len(), 3);
        assert!(traits.contains(&"error-handling:merged".to_string()));
        assert!(traits.contains(&"auth".to_string()));
        assert!(traits.contains(&"telemetry".to_string()));
    }

    #[test]
    fn test_genetic_diversity() {
        let (a, b) = test_parents();
        let mut m = Meiosis::new(a, b);

        // All from one parent — zero diversity
        m.recombine(RecombinationEvent {
            trait_name: "x".into(),
            selection: TraitSelection::ParentA,
            rationale: "".into(),
        });
        assert_eq!(m.genetic_diversity(), 0.0);

        // Add a crossover — diversity increases
        m.recombine(RecombinationEvent {
            trait_name: "y".into(),
            selection: TraitSelection::Crossover,
            rationale: "".into(),
        });
        assert!(m.genetic_diversity() > 0.0);

        // Add a novel — diversity increases more
        m.recombine(RecombinationEvent {
            trait_name: "z".into(),
            selection: TraitSelection::Novel,
            rationale: "".into(),
        });
        assert_eq!(m.genetic_diversity(), 1.0); // (1 + 2) / 3 = 1.0
    }
}
