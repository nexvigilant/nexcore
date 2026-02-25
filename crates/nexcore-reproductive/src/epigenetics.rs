//! # Epigenetics — Heritable Modifications Without Genome Change
//!
//! Maps to memory persistence across Claude Code sessions. The "genome" (system
//! prompt, CLAUDE.md) stays the same, but "epigenetic marks" (implicit knowledge,
//! brain beliefs, user preferences) modify expression.
//!
//! ## Biological Mapping
//!
//! | Epigenetic Mechanism | Claude Code Analog |
//! |---------------------|-------------------|
//! | DNA Methylation | `implicit_set` — silences/activates specific behaviors |
//! | Histone Modification | Brain belief confidence — loosens/tightens expression |
//! | Chromatin Remodeling | Session restore — restructures available context |
//! | Genomic Imprinting | User-confirmed beliefs — parent-of-origin bias |
//! | Transgenerational | Memory files that persist across all future sessions |

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Type of epigenetic mark.
///
/// **T1 grounding**: Σ(Sum) + π(Persistence)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MarkType {
    /// Silences a behavior (equivalent to DNA methylation suppressing gene expression).
    Methylation,
    /// Modifies confidence/strength of a behavior (histone modification).
    HistoneModification,
    /// Restructures which context is available (chromatin remodeling).
    ChromatinRemodeling,
    /// Locks a preference from a specific source (genomic imprinting).
    Imprinting,
    /// Persists across all future sessions (transgenerational inheritance).
    Transgenerational,
}

/// An epigenetic mark on a behavioral "gene".
///
/// **T1 grounding**: π(Persistence) + ς(State) + →(Causality)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpigeneticMark {
    /// The behavior/capability being modified.
    pub gene: String,
    /// Type of modification.
    pub mark_type: MarkType,
    /// Strength of the mark (0.0 = no effect, 1.0 = fully expressed/suppressed).
    pub strength: f64,
    /// When this mark was established.
    pub established: DateTime,
    /// Source of the mark (user directive, learned behavior, system default).
    pub source: MarkSource,
    /// Number of sessions this mark has survived.
    pub generations_survived: u32,
}

/// Origin of an epigenetic mark.
///
/// **T1 grounding**: Σ(Sum) + λ(Location)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkSource {
    /// Explicit user instruction ("always use bun", "never auto-commit").
    UserDirective(String),
    /// Learned from repeated patterns.
    LearnedBehavior {
        pattern_id: String,
        occurrences: u32,
    },
    /// System default from CLAUDE.md or settings.
    SystemDefault,
    /// Inherited from a parent session's handoff.
    Inherited { parent_session: String },
}

/// Epigenome — the complete set of epigenetic marks for a session.
///
/// **T1 grounding**: ×(Product) + π(Persistence) + ς(State)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epigenome {
    /// All active marks.
    pub marks: Vec<EpigeneticMark>,
    /// Session generation (how many sessions since the first mark).
    pub generation: u32,
    /// Timestamp of last epigenome update.
    pub last_updated: DateTime,
}

impl Epigenome {
    /// Creates a new empty epigenome at generation 0.
    pub fn new() -> Self {
        Self {
            marks: Vec::new(),
            generation: 0,
            last_updated: DateTime::now(),
        }
    }

    /// Adds a mark to the epigenome.
    pub fn add_mark(&mut self, mark: EpigeneticMark) {
        self.marks.push(mark);
        self.last_updated = DateTime::now();
    }

    /// Returns all marks affecting a specific gene.
    pub fn marks_for_gene(&self, gene: &str) -> Vec<&EpigeneticMark> {
        self.marks.iter().filter(|m| m.gene == gene).collect()
    }

    /// Calculates the net expression level for a gene.
    ///
    /// Positive = activated, negative = suppressed, zero = baseline.
    /// Range: -1.0 (fully suppressed) to 1.0 (fully activated).
    pub fn expression_level(&self, gene: &str) -> f64 {
        let marks = self.marks_for_gene(gene);
        if marks.is_empty() {
            return 0.0; // Baseline expression
        }

        let mut level = 0.0_f64;
        for mark in &marks {
            match mark.mark_type {
                MarkType::Methylation => {
                    // Methylation suppresses
                    level -= mark.strength;
                }
                MarkType::HistoneModification => {
                    // Histone mods can activate or suppress based on strength sign
                    // Positive strength = activation, we model all as activation here
                    level += mark.strength * 0.5;
                }
                MarkType::Imprinting => {
                    // Imprinting strongly biases toward source's preference
                    level += mark.strength;
                }
                MarkType::ChromatinRemodeling | MarkType::Transgenerational => {
                    // These modify availability, counted as activation
                    level += mark.strength * 0.3;
                }
            }
        }

        level.clamp(-1.0, 1.0)
    }

    /// Advances the epigenome to the next generation.
    ///
    /// Marks decay unless reinforced:
    /// - Methylation decays by 10% per generation
    /// - HistoneModification decays by 20% per generation
    /// - Imprinting and Transgenerational do not decay
    /// - ChromatinRemodeling decays by 15% per generation
    ///
    /// Marks below 0.05 strength are removed (mark erasure).
    pub fn advance_generation(&mut self) {
        self.generation += 1;

        for mark in &mut self.marks {
            mark.generations_survived += 1;

            let decay_rate = match mark.mark_type {
                MarkType::Methylation => 0.10,
                MarkType::HistoneModification => 0.20,
                MarkType::ChromatinRemodeling => 0.15,
                MarkType::Imprinting | MarkType::Transgenerational => 0.0,
            };

            mark.strength *= 1.0 - decay_rate;
        }

        // Erase marks that have decayed below threshold
        self.marks.retain(|m| m.strength >= 0.05);
        self.last_updated = DateTime::now();
    }

    /// Returns the count of active marks by type.
    pub fn mark_counts(&self) -> EpigenomeSummary {
        let mut summary = EpigenomeSummary::default();
        for mark in &self.marks {
            match mark.mark_type {
                MarkType::Methylation => summary.methylation += 1,
                MarkType::HistoneModification => summary.histone += 1,
                MarkType::ChromatinRemodeling => summary.chromatin += 1,
                MarkType::Imprinting => summary.imprinting += 1,
                MarkType::Transgenerational => summary.transgenerational += 1,
            }
        }
        summary.total = self.marks.len();
        summary
    }

    /// Returns all transgenerational marks (persist forever).
    pub fn heritable_marks(&self) -> Vec<&EpigeneticMark> {
        self.marks
            .iter()
            .filter(|m| {
                matches!(
                    m.mark_type,
                    MarkType::Transgenerational | MarkType::Imprinting
                )
            })
            .collect()
    }
}

impl Default for Epigenome {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of epigenetic marks by type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpigenomeSummary {
    pub methylation: usize,
    pub histone: usize,
    pub chromatin: usize,
    pub imprinting: usize,
    pub transgenerational: usize,
    pub total: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_mark(gene: &str, mark_type: MarkType, strength: f64) -> EpigeneticMark {
        EpigeneticMark {
            gene: gene.into(),
            mark_type,
            strength,
            established: DateTime::now(),
            source: MarkSource::SystemDefault,
            generations_survived: 0,
        }
    }

    #[test]
    fn test_empty_epigenome() {
        let epi = Epigenome::new();
        assert_eq!(epi.generation, 0);
        assert!(epi.marks.is_empty());
        assert_eq!(epi.expression_level("any_gene"), 0.0);
    }

    #[test]
    fn test_methylation_suppresses() {
        let mut epi = Epigenome::new();
        epi.add_mark(make_mark("auto-commit", MarkType::Methylation, 0.8));
        assert!(epi.expression_level("auto-commit") < 0.0);
    }

    #[test]
    fn test_imprinting_activates() {
        let mut epi = Epigenome::new();
        epi.add_mark(make_mark("use-bun", MarkType::Imprinting, 0.9));
        assert!(epi.expression_level("use-bun") > 0.0);
    }

    #[test]
    fn test_generation_decay() {
        let mut epi = Epigenome::new();
        epi.add_mark(make_mark(
            "temp-behavior",
            MarkType::HistoneModification,
            0.3,
        ));

        // After enough generations, the mark should decay below threshold
        for _ in 0..20 {
            epi.advance_generation();
        }
        assert!(epi.marks.is_empty(), "mark should have decayed away");
    }

    #[test]
    fn test_transgenerational_persists() {
        let mut epi = Epigenome::new();
        epi.add_mark(make_mark("no-python", MarkType::Transgenerational, 1.0));

        for _ in 0..100 {
            epi.advance_generation();
        }

        assert_eq!(epi.marks.len(), 1, "transgenerational mark should persist");
        assert_eq!(epi.marks[0].strength, 1.0);
        assert_eq!(epi.marks[0].generations_survived, 100);
    }

    #[test]
    fn test_mark_counts() {
        let mut epi = Epigenome::new();
        epi.add_mark(make_mark("a", MarkType::Methylation, 0.5));
        epi.add_mark(make_mark("b", MarkType::Methylation, 0.5));
        epi.add_mark(make_mark("c", MarkType::Imprinting, 0.9));
        epi.add_mark(make_mark("d", MarkType::Transgenerational, 1.0));

        let counts = epi.mark_counts();
        assert_eq!(counts.methylation, 2);
        assert_eq!(counts.imprinting, 1);
        assert_eq!(counts.transgenerational, 1);
        assert_eq!(counts.total, 4);
    }

    #[test]
    fn test_heritable_marks() {
        let mut epi = Epigenome::new();
        epi.add_mark(make_mark("volatile", MarkType::HistoneModification, 0.5));
        epi.add_mark(make_mark("permanent", MarkType::Transgenerational, 1.0));
        epi.add_mark(make_mark("imprinted", MarkType::Imprinting, 0.8));

        let heritable = epi.heritable_marks();
        assert_eq!(heritable.len(), 2);
    }

    #[test]
    fn test_expression_clamping() {
        let mut epi = Epigenome::new();
        // Stack many activation marks
        for i in 0..10 {
            epi.add_mark(make_mark("overdrive", MarkType::Imprinting, 1.0));
            let _ = i;
        }
        let level = epi.expression_level("overdrive");
        assert!(level <= 1.0, "expression should be clamped to 1.0");
    }
}
