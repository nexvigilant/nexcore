//! # I.R.O.N.M.A.N. Reactor — Chemistry x Primitives
//!
//! 15 chemistry operations mapped to 15 T1 primitives, composed into
//! 7 reactor phases. Each operation is a cognitive instruction that
//! transforms input through a specific primitive lens.
//!
//! ## The Periodic Table of Cognition
//!
//! | Chemistry Op | Primitive | Phase |
//! |-------------|-----------|-------|
//! | REDUCTION | Causality | I |
//! | FISSION | Sum | I |
//! | ACTIVATION_ENERGY | Boundary (if-then) | R |
//! | PRECIPITATION | Quantity (threshold) | R |
//! | CATALYSIS | Mapping (mechanism) | O |
//! | RESONANCE | Frequency | O |
//! | MEMBRANE_TRANSPORT | Boundary | N |
//! | ENTROPY | Location | N |
//! | CHIRALITY | Comparison (identity) | N |
//! | FUSION | Existence | M |
//! | POLYMERIZATION | Sequence | M |
//! | TITRATION | Quantity (amount) | A |
//! | OXIDATION | Causality (effect) | A |
//! | HALF_LIFE | Persistence (duration) | N2 |
//! | SUBLIMATION | State (changes) | N2 |

use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};

// ─── Chemistry Operations ──────────────────────────────────────────────────

/// One of the 15 chemistry operations in the reactor.
///
/// Tier: T2-P | Dominant: Σ (Sum) — 15-variant alternation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChemOp {
    Reduction,
    Oxidation,
    Catalysis,
    Polymerization,
    HalfLife,
    Resonance,
    Precipitation,
    Titration,
    MembraneTransport,
    Chirality,
    Sublimation,
    Fusion,
    Fission,
    Entropy,
    ActivationEnergy,
}

/// The 7 I.R.O.N.M.A.N. reactor phases.
///
/// Tier: T2-P | Dominant: σ (Sequence) — ordered pipeline
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IronmanPhase {
    /// I — Identify the Ore: strip to root cause + classify
    Identify,
    /// R — React with Constraints: define triggers + decision gates
    React,
    /// O — Optimize the Pathway: find catalysts + match cadence
    Optimize,
    /// N — Navigate Boundaries: define interfaces + fight entropy + check chirality
    Navigate,
    /// M — Manufacture the Prototype: fuse + chain into pipeline
    Manufacture,
    /// A — Assay the Results: measure precisely + expose to reality
    Assay,
    /// N2 — Negotiate the Half-Life: determine decay + plan phase transitions
    Negotiate,
}

/// Static descriptor for a chemistry operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChemistryOperation {
    pub op: ChemOp,
    pub name: &'static str,
    pub primitive: LexPrimitiva,
    pub cognitive_instruction: &'static str,
    pub phase: IronmanPhase,
}

/// Output from applying a chemistry operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactorOutput {
    pub op: ChemOp,
    pub phase: IronmanPhase,
    pub input: String,
    pub result: String,
    pub primitive: LexPrimitiva,
}

/// Output from running a full I.R.O.N.M.A.N. phase.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseOutput {
    pub phase: IronmanPhase,
    pub phase_name: &'static str,
    pub operations: Vec<ReactorOutput>,
    pub synthesis: String,
}

// ─── Static Operation Table ────────────────────────────────────────────────

impl ChemOp {
    /// All 15 operations.
    pub const ALL: [ChemOp; 15] = [
        Self::Reduction,
        Self::Oxidation,
        Self::Catalysis,
        Self::Polymerization,
        Self::HalfLife,
        Self::Resonance,
        Self::Precipitation,
        Self::Titration,
        Self::MembraneTransport,
        Self::Chirality,
        Self::Sublimation,
        Self::Fusion,
        Self::Fission,
        Self::Entropy,
        Self::ActivationEnergy,
    ];

    /// Get the full descriptor for this operation.
    pub fn descriptor(self) -> ChemistryOperation {
        match self {
            Self::Reduction => ChemistryOperation {
                op: self,
                name: "REDUCTION",
                primitive: LexPrimitiva::Causality,
                cognitive_instruction: "Strip to root cause. Remove noise. Find the naked truth.",
                phase: IronmanPhase::Identify,
            },
            Self::Fission => ChemistryOperation {
                op: self,
                name: "FISSION",
                primitive: LexPrimitiva::Sum,
                cognitive_instruction: "Split heavy into classified lighter parts. Taxonomy through controlled decomposition.",
                phase: IronmanPhase::Identify,
            },
            Self::ActivationEnergy => ChemistryOperation {
                op: self,
                name: "ACTIVATION_ENERGY",
                primitive: LexPrimitiva::Boundary,
                cognitive_instruction: "Define the trigger threshold. Below it = inert. Above it = chain reaction.",
                phase: IronmanPhase::React,
            },
            Self::Precipitation => ChemistryOperation {
                op: self,
                name: "PRECIPITATION",
                primitive: LexPrimitiva::Quantity,
                cognitive_instruction: "Supersaturate, then crystallize. Decisions precipitate at the saturation point.",
                phase: IronmanPhase::React,
            },
            Self::Catalysis => ChemistryOperation {
                op: self,
                name: "CATALYSIS",
                primitive: LexPrimitiva::Mapping,
                cognitive_instruction: "Find the accelerant that lowers activation energy without being consumed.",
                phase: IronmanPhase::Optimize,
            },
            Self::Resonance => ChemistryOperation {
                op: self,
                name: "RESONANCE",
                primitive: LexPrimitiva::Frequency,
                cognitive_instruction: "Find the natural frequency. Match iteration cadence to the problem's harmonic.",
                phase: IronmanPhase::Optimize,
            },
            Self::MembraneTransport => ChemistryOperation {
                op: self,
                name: "MEMBRANE_TRANSPORT",
                primitive: LexPrimitiva::Boundary,
                cognitive_instruction: "Semi-permeable. Define what crosses and what doesn't. Every interface is a membrane.",
                phase: IronmanPhase::Navigate,
            },
            Self::Entropy => ChemistryOperation {
                op: self,
                name: "ENTROPY",
                primitive: LexPrimitiva::Location,
                cognitive_instruction: "Place things where entropy is locally minimized. Fight decay with structure.",
                phase: IronmanPhase::Navigate,
            },
            Self::Chirality => ChemistryOperation {
                op: self,
                name: "CHIRALITY",
                primitive: LexPrimitiva::Comparison,
                cognitive_instruction: "Mirror images are NOT identical. Same components, different handedness = different behavior.",
                phase: IronmanPhase::Navigate,
            },
            Self::Fusion => ChemistryOperation {
                op: self,
                name: "FUSION",
                primitive: LexPrimitiva::Existence,
                cognitive_instruction: "Combine lighter elements into heavier ones. Release massive energy. Create what didn't exist.",
                phase: IronmanPhase::Manufacture,
            },
            Self::Polymerization => ChemistryOperation {
                op: self,
                name: "POLYMERIZATION",
                primitive: LexPrimitiva::Sequence,
                cognitive_instruction: "Chain monomers into polymers. Single steps become pipelines. Order creates emergent strength.",
                phase: IronmanPhase::Manufacture,
            },
            Self::Titration => ChemistryOperation {
                op: self,
                name: "TITRATION",
                primitive: LexPrimitiva::Quantity,
                cognitive_instruction: "Add reagent drop by drop until the indicator changes. Measure the exact dose.",
                phase: IronmanPhase::Assay,
            },
            Self::Oxidation => ChemistryOperation {
                op: self,
                name: "OXIDATION",
                primitive: LexPrimitiva::Causality,
                cognitive_instruction: "Expose outcomes to air. What corrodes? What burns bright? What is inert?",
                phase: IronmanPhase::Assay,
            },
            Self::HalfLife => ChemistryOperation {
                op: self,
                name: "HALF_LIFE",
                primitive: LexPrimitiva::Persistence,
                cognitive_instruction: "Every solution decays. What is the half-life? When does it lose half its value?",
                phase: IronmanPhase::Negotiate,
            },
            Self::Sublimation => ChemistryOperation {
                op: self,
                name: "SUBLIMATION",
                primitive: LexPrimitiva::State,
                cognitive_instruction: "Solid to gas. Skip the liquid phase. Transform state without the intermediate mess.",
                phase: IronmanPhase::Negotiate,
            },
        }
    }

    /// Get the T1 primitive for this operation.
    pub fn primitive(self) -> LexPrimitiva {
        self.descriptor().primitive
    }

    /// Get the I.R.O.N.M.A.N. phase this operation belongs to.
    pub fn phase(self) -> IronmanPhase {
        self.descriptor().phase
    }
}

// ─── Phase Composition ─────────────────────────────────────────────────────

impl IronmanPhase {
    /// All 7 phases in order.
    pub const ALL: [IronmanPhase; 7] = [
        Self::Identify,
        Self::React,
        Self::Optimize,
        Self::Navigate,
        Self::Manufacture,
        Self::Assay,
        Self::Negotiate,
    ];

    /// Phase letter in the mnemonic.
    pub fn letter(self) -> char {
        match self {
            Self::Identify => 'I',
            Self::React => 'R',
            Self::Optimize => 'O',
            Self::Navigate => 'N',
            Self::Manufacture => 'M',
            Self::Assay => 'A',
            Self::Negotiate => 'N',
        }
    }

    /// Human-readable phase name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Identify => "Identify the Ore",
            Self::React => "React with Constraints",
            Self::Optimize => "Optimize the Pathway",
            Self::Navigate => "Navigate Boundaries",
            Self::Manufacture => "Manufacture the Prototype",
            Self::Assay => "Assay the Results",
            Self::Negotiate => "Negotiate the Half-Life",
        }
    }

    /// Which chemistry operations compose this phase.
    pub fn operations(self) -> &'static [ChemOp] {
        match self {
            Self::Identify => &[ChemOp::Reduction, ChemOp::Fission],
            Self::React => &[ChemOp::ActivationEnergy, ChemOp::Precipitation],
            Self::Optimize => &[ChemOp::Catalysis, ChemOp::Resonance],
            Self::Navigate => &[ChemOp::MembraneTransport, ChemOp::Entropy, ChemOp::Chirality],
            Self::Manufacture => &[ChemOp::Fusion, ChemOp::Polymerization],
            Self::Assay => &[ChemOp::Titration, ChemOp::Oxidation],
            Self::Negotiate => &[ChemOp::HalfLife, ChemOp::Sublimation],
        }
    }

    /// Apply this phase to an input concept, producing structured output.
    pub fn apply(self, input: &str) -> PhaseOutput {
        let operations: Vec<ReactorOutput> = self
            .operations()
            .iter()
            .map(|op| {
                let desc = op.descriptor();
                ReactorOutput {
                    op: *op,
                    phase: self,
                    input: input.to_string(),
                    result: format!(
                        "[{}] {}: Apply to '{}' — {}",
                        desc.name,
                        desc.primitive,
                        input,
                        desc.cognitive_instruction,
                    ),
                    primitive: desc.primitive,
                }
            })
            .collect();

        let synthesis = format!(
            "Phase {} ({}) applied {} operations to '{}'",
            self.letter(),
            self.name(),
            operations.len(),
            input,
        );

        PhaseOutput {
            phase: self,
            phase_name: self.name(),
            operations,
            synthesis,
        }
    }
}

/// Run the full I.R.O.N.M.A.N. pipeline on an input.
pub fn run_full_pipeline(input: &str) -> Vec<PhaseOutput> {
    IronmanPhase::ALL.iter().map(|phase| phase.apply(input)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fifteen_operations() {
        assert_eq!(ChemOp::ALL.len(), 15);
    }

    #[test]
    fn seven_phases() {
        assert_eq!(IronmanPhase::ALL.len(), 7);
    }

    #[test]
    fn every_op_has_a_phase() {
        for op in &ChemOp::ALL {
            let _ = op.phase(); // should not panic
        }
    }

    #[test]
    fn phases_spell_ironman() {
        let letters: String = IronmanPhase::ALL.iter().map(|p| p.letter()).collect();
        assert_eq!(letters, "IRONMAN");
    }

    #[test]
    fn all_ops_covered_by_phases() {
        let mut covered: Vec<ChemOp> = Vec::new();
        for phase in &IronmanPhase::ALL {
            for op in phase.operations() {
                covered.push(*op);
            }
        }
        assert_eq!(covered.len(), 15);
    }

    #[test]
    fn phase_apply_produces_output() {
        let output = IronmanPhase::Identify.apply("test concept");
        assert_eq!(output.operations.len(), 2); // REDUCTION + FISSION
        assert_eq!(output.phase, IronmanPhase::Identify);
    }

    #[test]
    fn full_pipeline_produces_seven_phases() {
        let outputs = run_full_pipeline("test");
        assert_eq!(outputs.len(), 7);
    }
}
