//! Involutionary Engine — bidirectional computation on DNA-encoded helix states.
//!
//! An involution is a function that is its own inverse: f(f(x)) = x.
//! The complement operation on codons is an involution: complement(complement(c)) = c.
//!
//! This engine exploits involutions to enable:
//! 1. **Reversible computation** — every transform can be undone
//! 2. **Dual perspective** — every state has a shadow (what it ISN'T)
//! 3. **Conservation verification** — if T(complement(x)) ≠ complement(T(x)),
//!    the transform T violates conservation
//! 4. **Fixed-point detection** — codons where complement(c) = c reveal equilibrium
//!
//! The engine operates on strands and produces new strands. All operations
//! preserve the involution property. Non-involutory transforms are detected
//! and flagged as conservation violations.

use crate::dna::{Codon, Nucleotide, Strand};
use crate::{ConservationInput, ExistenceClass, conservation};

/// Result of an involutionary operation.
#[derive(Debug, Clone)]
pub struct InvolutionResult {
    /// The output strand.
    pub output: Strand,
    /// Whether the operation preserved the involution property.
    pub involution_holds: bool,
    /// Conservation violation details (empty if none).
    pub violations: Vec<String>,
    /// Fixed points found (codons where complement = self).
    pub fixed_points: Vec<usize>,
}

/// A transform that can be applied to a strand.
#[derive(Debug, Clone, Copy)]
pub enum Transform {
    /// Complement: A↔T, G↔C. The fundamental involution.
    Complement,
    /// Strengthen: raise each nucleotide by one level (A→T, T→G, G→C, C→C).
    Strengthen,
    /// Weaken: lower each nucleotide by one level (C→G, G→T, T→A, A→A).
    Weaken,
    /// Focus boundary: set ∂ to max of all three primitives.
    FocusBoundary,
    /// Focus state: set ς to max of all three primitives.
    FocusState,
    /// Focus void: set ∅ to max of all three primitives.
    FocusVoid,
    /// Equilibrate: set all three to the median value.
    Equilibrate,
    /// Reverse: reverse codon order in the strand.
    Reverse,
}

fn shift_up(n: Nucleotide) -> Nucleotide {
    match n {
        Nucleotide::A => Nucleotide::T,
        Nucleotide::T => Nucleotide::G,
        Nucleotide::G => Nucleotide::C,
        Nucleotide::C => Nucleotide::C, // ceiling
    }
}

fn shift_down(n: Nucleotide) -> Nucleotide {
    match n {
        Nucleotide::A => Nucleotide::A, // floor
        Nucleotide::T => Nucleotide::A,
        Nucleotide::G => Nucleotide::T,
        Nucleotide::C => Nucleotide::G,
    }
}

fn median_nuc(a: Nucleotide, b: Nucleotide, c: Nucleotide) -> Nucleotide {
    let mut vals = [a as u8, b as u8, c as u8];
    vals.sort_unstable();
    match vals[1] {
        0 => Nucleotide::A,
        1 => Nucleotide::T,
        2 => Nucleotide::G,
        _ => Nucleotide::C,
    }
}

fn max_nuc(a: Nucleotide, b: Nucleotide, c: Nucleotide) -> Nucleotide {
    a.max(b).max(c)
}

/// Apply a transform to a single codon.
fn transform_codon(codon: Codon, t: Transform) -> Codon {
    match t {
        Transform::Complement => codon.complement(),
        Transform::Strengthen => Codon {
            boundary: shift_up(codon.boundary),
            state: shift_up(codon.state),
            void: shift_up(codon.void),
        },
        Transform::Weaken => Codon {
            boundary: shift_down(codon.boundary),
            state: shift_down(codon.state),
            void: shift_down(codon.void),
        },
        Transform::FocusBoundary => {
            let m = max_nuc(codon.boundary, codon.state, codon.void);
            Codon {
                boundary: m,
                state: codon.state,
                void: codon.void,
            }
        }
        Transform::FocusState => {
            let m = max_nuc(codon.boundary, codon.state, codon.void);
            Codon {
                boundary: codon.boundary,
                state: m,
                void: codon.void,
            }
        }
        Transform::FocusVoid => {
            let m = max_nuc(codon.boundary, codon.state, codon.void);
            Codon {
                boundary: codon.boundary,
                state: codon.state,
                void: m,
            }
        }
        Transform::Equilibrate => {
            let med = median_nuc(codon.boundary, codon.state, codon.void);
            Codon {
                boundary: med,
                state: med,
                void: med,
            }
        }
        Transform::Reverse => codon, // reverse operates on strands, not codons
    }
}

/// The Involutionary Engine.
pub struct Engine;

impl Engine {
    /// Apply a transform to a strand.
    pub fn apply(strand: &Strand, t: Transform) -> InvolutionResult {
        let output = match t {
            Transform::Reverse => Strand {
                codons: strand.codons.iter().rev().copied().collect(),
            },
            _ => Strand {
                codons: strand
                    .codons
                    .iter()
                    .map(|c| transform_codon(*c, t))
                    .collect(),
            },
        };

        // Check involution: T(T(x)) should equal x for true involutions
        let double_applied = match t {
            Transform::Reverse => Strand {
                codons: output.codons.iter().rev().copied().collect(),
            },
            _ => Strand {
                codons: output
                    .codons
                    .iter()
                    .map(|c| transform_codon(*c, t))
                    .collect(),
            },
        };

        let involution_holds = strand.codons.len() == double_applied.codons.len()
            && strand
                .codons
                .iter()
                .zip(double_applied.codons.iter())
                .all(|(a, b)| a == b);

        // Check conservation: T(complement(x)) should equal complement(T(x))
        let mut violations = Vec::new();
        let complement_then_transform = match t {
            Transform::Reverse => Strand {
                codons: strand.complement().codons.iter().rev().copied().collect(),
            },
            _ => Strand {
                codons: strand
                    .complement()
                    .codons
                    .iter()
                    .map(|c| transform_codon(*c, t))
                    .collect(),
            },
        };
        let transform_then_complement = output.complement();

        for (i, (a, b)) in complement_then_transform
            .codons
            .iter()
            .zip(transform_then_complement.codons.iter())
            .enumerate()
        {
            if a != b {
                violations.push(format!(
                    "Codon {}: T(comp(x))={} ≠ comp(T(x))={}",
                    i,
                    a.as_str(),
                    b.as_str()
                ));
            }
        }

        // Find fixed points: codons where complement = self (impossible for standard
        // nucleotides, but codons where the TRANSLATION is the same)
        let fixed_points: Vec<usize> = output
            .codons
            .iter()
            .enumerate()
            .filter(|(_, c)| c.translate() == c.complement().translate())
            .map(|(i, _)| i)
            .collect();

        InvolutionResult {
            output,
            involution_holds,
            violations,
            fixed_points,
        }
    }

    /// Apply a sequence of transforms, checking conservation at each step.
    pub fn pipeline(strand: &Strand, transforms: &[Transform]) -> Vec<InvolutionResult> {
        let mut current = strand.clone();
        let mut results = Vec::new();
        for &t in transforms {
            let result = Self::apply(&current, t);
            current = result.output.clone();
            results.push(result);
        }
        results
    }

    /// Find the transform that maximizes strand health.
    pub fn optimize(strand: &Strand) -> (Transform, Strand, f64) {
        let transforms = [
            Transform::Strengthen,
            Transform::FocusBoundary,
            Transform::FocusState,
            Transform::FocusVoid,
            Transform::Equilibrate,
        ];

        let mut best_transform = Transform::Strengthen;
        let mut best_strand = strand.clone();
        let mut best_health = strand.health();

        for &t in &transforms {
            let result = Self::apply(strand, t);
            let h = result.output.health();
            if h > best_health {
                best_health = h;
                best_strand = result.output;
                best_transform = t;
            }
        }

        (best_transform, best_strand, best_health)
    }

    /// Diagnose a strand: what's wrong, what transform fixes it.
    pub fn diagnose(strand: &Strand) -> Diagnosis {
        let health = strand.health();
        let stop_count = strand.stop_count();
        let total = strand.codons.len();

        // Find which primitive is weakest across the strand
        let (mut a_boundary, mut a_state, mut a_void) = (0usize, 0usize, 0usize);
        for codon in &strand.codons {
            if codon.boundary == Nucleotide::A {
                a_boundary += 1;
            }
            if codon.state == Nucleotide::A {
                a_state += 1;
            }
            if codon.void == Nucleotide::A {
                a_void += 1;
            }
        }

        let weakest_primitive = if a_state >= a_boundary && a_state >= a_void {
            "ς (state)"
        } else if a_boundary >= a_void {
            "∂ (boundary)"
        } else {
            "∅ (void)"
        };

        let recommended = if a_state >= a_boundary && a_state >= a_void {
            Transform::FocusState
        } else if a_boundary >= a_void {
            Transform::FocusBoundary
        } else {
            Transform::FocusVoid
        };

        let (opt_transform, opt_strand, opt_health) = Self::optimize(strand);

        Diagnosis {
            health,
            stop_count,
            total_codons: total,
            weakest_primitive: weakest_primitive.to_string(),
            a_counts: (a_boundary, a_state, a_void),
            recommended_transform: recommended,
            optimal_transform: opt_transform,
            optimal_health: opt_health,
            dna_before: strand.as_dna(),
            dna_after: opt_strand.as_dna(),
        }
    }
}

/// Diagnosis of a DNA strand.
#[derive(Debug, Clone)]
pub struct Diagnosis {
    /// Current health (0.0-1.0).
    pub health: f64,
    /// Number of stop codons.
    pub stop_count: usize,
    /// Total codons.
    pub total_codons: usize,
    /// Which primitive has the most A (collapsing) nucleotides.
    pub weakest_primitive: String,
    /// Count of A nucleotides per position (boundary, state, void).
    pub a_counts: (usize, usize, usize),
    /// Recommended transform based on weakest primitive.
    pub recommended_transform: Transform,
    /// Optimal transform (maximizes health).
    pub optimal_transform: Transform,
    /// Health after optimal transform.
    pub optimal_health: f64,
    /// DNA string before transform.
    pub dna_before: String,
    /// DNA string after optimal transform.
    pub dna_after: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ConservationInput;

    fn nexvigilant_portfolio() -> Strand {
        Strand::encode(&[
            ConservationInput {
                boundary: 0.95,
                state: 0.85,
                void: 0.7,
            }, // Station
            ConservationInput {
                boundary: 0.6,
                state: 0.4,
                void: 0.3,
            }, // Nucleus
            ConservationInput {
                boundary: 0.9,
                state: 0.9,
                void: 0.8,
            }, // Micrograms
            ConservationInput {
                boundary: 0.85,
                state: 0.7,
                void: 0.6,
            }, // NexCore
            ConservationInput {
                boundary: 0.3,
                state: 0.15,
                void: 0.2,
            }, // Academy
        ])
    }

    #[test]
    fn complement_is_involution() {
        let strand = nexvigilant_portfolio();
        let result = Engine::apply(&strand, Transform::Complement);
        assert!(result.involution_holds);
        assert!(result.violations.is_empty());
    }

    #[test]
    fn reverse_is_involution() {
        let strand = nexvigilant_portfolio();
        let result = Engine::apply(&strand, Transform::Reverse);
        assert!(result.involution_holds);
    }

    #[test]
    fn strengthen_is_not_involution() {
        let strand = nexvigilant_portfolio();
        let result = Engine::apply(&strand, Transform::Strengthen);
        assert!(!result.involution_holds); // strengthen is lossy at ceiling
    }

    #[test]
    fn strengthen_removes_stop_codons() {
        let strand = nexvigilant_portfolio();
        assert_eq!(strand.stop_count(), 1); // Academy is TAA
        let result = Engine::apply(&strand, Transform::Strengthen);
        assert_eq!(result.output.stop_count(), 0); // A→T, no more stops
        assert_eq!(result.output.health(), 1.0);
    }

    #[test]
    fn diagnose_finds_weakest() {
        let strand = nexvigilant_portfolio();
        let diag = Engine::diagnose(&strand);
        assert_eq!(diag.stop_count, 1);
        assert_eq!(diag.health, 0.8);
        // Academy has A in state and void, so state and void tie
        // The engine should find the optimal transform
        assert!(diag.optimal_health > diag.health);
    }

    #[test]
    fn pipeline_chains_transforms() {
        let strand = nexvigilant_portfolio();
        let results = Engine::pipeline(
            &strand,
            &[
                Transform::Strengthen,
                Transform::Complement,
                Transform::Complement, // should undo the complement
            ],
        );
        assert_eq!(results.len(), 3);
        // After strengthen + complement + complement, should equal just strengthen
        let strengthened = Engine::apply(&strand, Transform::Strengthen);
        assert_eq!(results[2].output.as_dna(), strengthened.output.as_dna());
    }

    #[test]
    fn equilibrate_nexvigilant() {
        let strand = nexvigilant_portfolio();
        let result = Engine::apply(&strand, Transform::Equilibrate);
        // Each codon gets median of its three nucleotides
        // Academy TAA → median(T,A,A) = A → AAA (still stop, but equilibrated)
        // Station CCG → median(C,C,G) = C → CCC
        for codon in &result.output.codons {
            assert_eq!(codon.boundary, codon.state);
            assert_eq!(codon.state, codon.void);
        }
    }

    #[test]
    fn focus_state_fixes_academy() {
        let strand = nexvigilant_portfolio();
        let result = Engine::apply(&strand, Transform::FocusState);
        // Academy: T A A → FocusState sets ς to max(T,A,A) = T → T T A
        // Still has A in void, so still stop. But ς is no longer A.
        let academy = result.output.codons[4];
        assert_eq!(academy.state, Nucleotide::T); // promoted from A
    }

    #[test]
    fn conservation_check_on_strengthen() {
        let strand = nexvigilant_portfolio();
        let result = Engine::apply(&strand, Transform::Strengthen);
        // Strengthen doesn't commute with complement (it's not linear)
        // So we expect conservation violations
        assert!(!result.violations.is_empty());
    }
}
