//! GroundsTo trait and LexPrimitiva — zero-dep inline definitions.
//!
//! These are the Lex Primitiva types defined locally to avoid any external
//! dependency. The types are API-compatible with `nexcore-lex-primitiva`
//! for future integration.

use crate::asm::Token;
use crate::codon_table::CodonTable;
use crate::cortex::{
    Cluster, ClusterResult, EvolutionConfig, EvolutionResult, GravityConfig, Organism, Particle,
};
use crate::data::{DnaArray, DnaFrame, DnaMap, DnaRecord, DnaType, DnaValue};
use crate::disasm::DisasmOptions;
use crate::gene::{Gene, Genome, Plasmid};
use crate::glyph::{Glyph, GlyphPair};
use crate::isa::Instruction;
use crate::lang::diagnostic::{Diagnostic, ErrorCode};
use crate::lang::json::JsonValue;
use crate::lang::templates::Template;
use crate::lexicon::{Affinity, Lexicon, WordOre};
use crate::program::Program;
use crate::pv_theory::{
    AlertLevel, CausalityCategory, CausalityScore, DrugProfile, SafetyLevel, SafetyMargin, Signal,
    VigilanceState,
};
use crate::statemind::{Drift, MindPoint, StateMind};
use crate::string_theory::{
    FrequencySpectrum, HarmonicMode, Resonance, StringEnergy, StringTension,
};
use crate::tile::{Pixel, Tile};
use crate::types::{AminoAcid, Codon, DoubleHelix, Nucleotide, Strand};
use crate::vm::CodonVM;
use crate::voxel::{VoxelCube, VoxelPos};

// ---------------------------------------------------------------------------
// Lex Primitiva (inline, zero-dep)
// ---------------------------------------------------------------------------

/// The 16 irreducible Lex Primitiva symbols.
///
/// Tier: T1 — the quarks of computation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LexPrimitiva {
    /// σ — Ordered succession
    Sequence,
    /// μ — Transformation A → B
    Mapping,
    /// ς — Context at a point in time
    State,
    /// ρ — Self-reference via indirection
    Recursion,
    /// ∅ — Meaningful absence
    Void,
    /// ∂ — Delimiters and limits
    Boundary,
    /// ν — Rate of occurrence
    Frequency,
    /// ∃ — Instantiation of being
    Existence,
    /// π — Continuity across time
    Persistence,
    /// → — Cause and consequence
    Causality,
    /// κ — Predicate matching
    Comparison,
    /// N — Numerical magnitude
    Quantity,
    /// λ — Positional context
    Location,
    /// ∝ — One-way state transition
    Irreversibility,
    /// Σ — Exclusive disjunction
    Sum,
    /// × — Conjunctive combination
    Product,
}

impl LexPrimitiva {
    /// Mathematical symbol for this primitive.
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            Self::Sequence => "σ",
            Self::Mapping => "μ",
            Self::State => "ς",
            Self::Recursion => "ρ",
            Self::Void => "∅",
            Self::Boundary => "∂",
            Self::Frequency => "ν",
            Self::Existence => "∃",
            Self::Persistence => "π",
            Self::Causality => "→",
            Self::Comparison => "κ",
            Self::Quantity => "N",
            Self::Location => "λ",
            Self::Irreversibility => "∝",
            Self::Sum => "Σ",
            Self::Product => "×",
        }
    }
}

/// Primitive composition: which T1 primitives compose a type.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimitiveComposition {
    /// The primitives that compose this type.
    pub primitives: Vec<LexPrimitiva>,
    /// The dominant primitive (primary characteristic).
    pub dominant: Option<LexPrimitiva>,
    /// Confidence in this grounding (0.0-1.0).
    pub confidence: f64,
}

impl PrimitiveComposition {
    /// Create a new composition. First primitive becomes dominant, confidence 1.0.
    #[must_use]
    pub fn new(primitives: Vec<LexPrimitiva>) -> Self {
        let dominant = primitives.first().copied();
        Self {
            primitives,
            dominant,
            confidence: 1.0,
        }
    }

    /// Set dominant primitive and confidence (builder pattern).
    #[must_use]
    pub fn with_dominant(mut self, dominant: LexPrimitiva, confidence: f64) -> Self {
        self.dominant = Some(dominant);
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// True if exactly one primitive (pure T1).
    #[must_use]
    pub fn is_pure(&self) -> bool {
        self.primitives.len() == 1
    }

    /// Tier classification based on primitive count.
    #[must_use]
    pub fn tier(&self) -> &'static str {
        match self.primitives.len() {
            1 => "T1",
            2..=3 => "T2-P",
            4..=5 => "T2-C",
            _ => "T3",
        }
    }
}

/// Trait for types that ground to T1 primitives.
pub trait GroundsTo {
    /// Returns the primitive composition that grounds this type.
    fn primitive_composition() -> PrimitiveComposition;

    /// Returns the dominant primitive (convenience method).
    fn dominant_primitive() -> Option<LexPrimitiva> {
        Self::primitive_composition().dominant
    }

    /// Returns true if this type is purely one primitive.
    fn is_pure_primitive() -> bool {
        Self::primitive_composition().is_pure()
    }
}

// ---------------------------------------------------------------------------
// Nucleotide → T1 (ς State)
// ---------------------------------------------------------------------------

impl GroundsTo for Nucleotide {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

// ---------------------------------------------------------------------------
// AminoAcid → T1 (ς State)
// ---------------------------------------------------------------------------

impl GroundsTo for AminoAcid {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

// ---------------------------------------------------------------------------
// Codon → T2-P (σ Sequence + ς State)
// ---------------------------------------------------------------------------

impl GroundsTo for Codon {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Strand → T2-P (σ Sequence + ∃ Existence)
// ---------------------------------------------------------------------------

impl GroundsTo for Strand {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Existence])
            .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

// ---------------------------------------------------------------------------
// CodonTable → T2-P (μ Mapping + ∂ Boundary)
// ---------------------------------------------------------------------------

impl GroundsTo for CodonTable {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

// ---------------------------------------------------------------------------
// DoubleHelix → T2-C (σ + κ + ∃ + μ)
// ---------------------------------------------------------------------------

impl GroundsTo for DoubleHelix {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Existence,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.70)
    }
}

// ---------------------------------------------------------------------------
// CodonVM → T3 (σ + μ + ς + ∂ + N + →)
// ---------------------------------------------------------------------------

impl GroundsTo for CodonVM {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.60)
    }
}

// ---------------------------------------------------------------------------
// Instruction → T2-P (μ Mapping + ∂ Boundary)
// ---------------------------------------------------------------------------

impl GroundsTo for Instruction {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

// ---------------------------------------------------------------------------
// Program → T2-C (σ Sequence + ∂ Boundary + ς State + μ Mapping)
// ---------------------------------------------------------------------------

impl GroundsTo for Program {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.75)
    }
}

// ---------------------------------------------------------------------------
// Token → T2-P (∂ Boundary + ∃ Existence)
// ---------------------------------------------------------------------------

impl GroundsTo for Token {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Existence])
            .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// DisasmOptions → T2-P (∂ Boundary + κ Comparison)
// ---------------------------------------------------------------------------

impl GroundsTo for DisasmOptions {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

// --- Phase 5: Gene type system ---

/// Gene: T3 (σ + μ + ∂ + ρ + → + ς)
/// Dominant: → Causality (a gene causes a protein/computation)
impl GroundsTo for Gene {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::Recursion,
            LexPrimitiva::Causality,
            LexPrimitiva::State,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.70)
    }
}

/// Genome: T3 (σ + μ + ∂ + ∃ + Σ + ς + →)
/// Dominant: σ Sequence (a genome is an ordered collection of genes)
impl GroundsTo for Genome {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
            LexPrimitiva::Sum,
            LexPrimitiva::State,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.65)
    }
}

/// Plasmid: T2-C (σ + ∂ + μ + ∃)
/// Dominant: σ Sequence (portable ordered snippet)
impl GroundsTo for Plasmid {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Mapping,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.75)
    }
}

// --- Phase 7a: Tile + Voxel type system ---

/// Pixel: T2-P (μ Mapping + ∂ Boundary)
/// Dominant: μ Mapping (encodes instruction → RGBA)
impl GroundsTo for Pixel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Mapping, 0.90)
    }
}

/// Tile: T2-C (σ Sequence + μ Mapping + ∂ Boundary + λ Location)
/// Dominant: σ Sequence (ordered 8×8 grid of encoded instructions)
impl GroundsTo for Tile {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.75)
    }
}

/// VoxelPos: T2-P (λ Location + N Quantity)
/// Dominant: λ Location (3D position in chemical space)
impl GroundsTo for VoxelPos {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Location, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

/// VoxelCube: T3 (σ + μ + ∂ + N + λ + κ + → + ∃)
/// Dominant: λ Location (maps instructions to 3D chemical space)
impl GroundsTo for VoxelCube {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
            LexPrimitiva::Location,
            LexPrimitiva::Comparison,
            LexPrimitiva::Causality,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Location, 0.65)
    }
}

// --- Phase 7b: Glyph IR type system ---

/// Glyph: T1 (μ Mapping)
/// Dominant: μ Mapping (a glyph maps to a hardware unit / T1 primitive)
impl GroundsTo for Glyph {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::Mapping, 1.0)
    }
}

/// GlyphPair: T2-C (μ Mapping + ∂ Boundary + κ Comparison + N Quantity)
/// Dominant: μ Mapping (pair encodes family→variant instruction semantics)
/// - μ: maps codons to instructions via family+variant
/// - ∂: P0 delineates hardware unit boundaries
/// - κ: Hamming distance enables error comparison
/// - N: 6-bit numeric encoding (3 bits × 2 positions)
impl GroundsTo for GlyphPair {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// --- Phase 8: Lexicon type system ---

/// WordOre: T2-C (N Quantity + σ Sequence + μ Mapping + κ Comparison)
/// Dominant: N Quantity (word mining extracts numerical properties)
impl GroundsTo for WordOre {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// Affinity: T2-P (κ Comparison + N Quantity)
/// Dominant: κ Comparison (measures distance between words)
impl GroundsTo for Affinity {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// Lexicon: T3 (σ + μ + κ + N + ∂ + ∃ + π)
/// Dominant: μ Mapping (maps words to mathematical properties)
impl GroundsTo for Lexicon {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.65)
    }
}

// --- Phase 8b: StateMind type system ---

/// MindPoint: T2-P (N Quantity + λ Location)
/// Dominant: λ Location (positional context in 3D space)
impl GroundsTo for MindPoint {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Location])
            .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

/// Drift: T2-P (κ Comparison + N Quantity)
/// Dominant: κ Comparison (measures directional change between points)
impl GroundsTo for Drift {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Comparison, 0.90)
    }
}

/// StateMind: T3 (ς + σ + μ + κ + N + λ + ∂ + ∃ + π)
/// Dominant: ς State (the mind's state evolves as words are ingested)
impl GroundsTo for StateMind {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Location,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::State, 0.60)
    }
}

// --- Phase 9: Cortex type system ---

/// Cluster: T2-C (λ Location + N Quantity + κ Comparison + μ Mapping)
/// Dominant: λ Location (spatial grouping in 3D mind-space)
impl GroundsTo for Cluster {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Location, 0.80)
    }
}

/// ClusterResult: T3 (σ + μ + κ + N + λ + ∂ + ∃)
/// Dominant: σ Sequence (iterative convergence process)
impl GroundsTo for ClusterResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Location,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.60)
    }
}

/// Particle: T2-C (λ Location + N Quantity + ς State + → Causality)
/// Dominant: ς State (particle state evolves under forces)
impl GroundsTo for Particle {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Location,
            LexPrimitiva::Quantity,
            LexPrimitiva::State,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::State, 0.75)
    }
}

/// GravityConfig: T2-P (∂ Boundary + N Quantity)
/// Dominant: ∂ Boundary (defines simulation limits)
impl GroundsTo for GravityConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Boundary, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Boundary, 0.90)
    }
}

/// Organism: T2-C (σ Sequence + κ Comparison + N Quantity + λ Location)
/// Dominant: N Quantity (fitness is the primary measure)
impl GroundsTo for Organism {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// EvolutionConfig: T2-C (N Quantity + ∂ Boundary + ν Frequency + κ Comparison)
/// Dominant: ∂ Boundary (defines GA parameter limits)
impl GroundsTo for EvolutionConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::Frequency,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
    }
}

/// EvolutionResult: T3 (σ + μ + κ + N + λ + → + ∃)
/// Dominant: σ Sequence (generational progression)
impl GroundsTo for EvolutionResult {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Location,
            LexPrimitiva::Causality,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.60)
    }
}

// --- Phase 10: String Theory type system ---

/// StringTension: T2-C (N Quantity + ν Frequency + σ Sequence + μ Mapping)
/// Dominant: N Quantity (tension is a measurable force)
impl GroundsTo for StringTension {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

/// HarmonicMode: T2-P (ν Frequency + N Quantity)
/// Dominant: ν Frequency (oscillation period and amplitude)
impl GroundsTo for HarmonicMode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Frequency, LexPrimitiva::Quantity])
            .with_dominant(LexPrimitiva::Frequency, 0.90)
    }
}

/// FrequencySpectrum: T2-C (ν Frequency + N Quantity + σ Sequence + κ Comparison)
/// Dominant: ν Frequency (spectral analysis of periodic structure)
impl GroundsTo for FrequencySpectrum {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Frequency,
            LexPrimitiva::Quantity,
            LexPrimitiva::Sequence,
            LexPrimitiva::Comparison,
        ])
        .with_dominant(LexPrimitiva::Frequency, 0.75)
    }
}

/// Resonance: T2-C (κ Comparison + ν Frequency + N Quantity + μ Mapping)
/// Dominant: κ Comparison (spectral overlap measurement)
impl GroundsTo for Resonance {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Frequency,
            LexPrimitiva::Quantity,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.75)
    }
}

/// StringEnergy: T2-C (N Quantity + ν Frequency + Σ Sum + μ Mapping)
/// Dominant: N Quantity (measurable energy values)
impl GroundsTo for StringEnergy {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Sum,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

// --- Phase 10b: PV Theory type system ---

/// CausalityCategory: T1 (ς State)
/// Dominant: ς State (discrete classification)
impl GroundsTo for CausalityCategory {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

/// SafetyLevel: T1 (ς State)
/// Dominant: ς State (discrete safety classification)
impl GroundsTo for SafetyLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

/// AlertLevel: T1 (ς State)
/// Dominant: ς State (discrete alert classification)
impl GroundsTo for AlertLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

/// DrugProfile: T3 (→ + κ + N + ν + σ + μ + ∂)
/// Dominant: → Causality (drugs cause effects)
impl GroundsTo for DrugProfile {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.65)
    }
}

/// Signal: T2-C (→ Causality + κ Comparison + N Quantity + ν Frequency)
/// Dominant: → Causality (signal implies causal link)
impl GroundsTo for Signal {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

/// SafetyMargin: T2-C (∂ Boundary + N Quantity + κ Comparison + → Causality)
/// Dominant: ∂ Boundary (distance from safety boundary)
impl GroundsTo for SafetyMargin {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Quantity,
            LexPrimitiva::Comparison,
            LexPrimitiva::Causality,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.80)
    }
}

/// CausalityScore: T2-C (→ Causality + κ Comparison + N Quantity + ∂ Boundary)
/// Dominant: → Causality (assessing causal relationship)
impl GroundsTo for CausalityScore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Causality, 0.80)
    }
}

/// VigilanceState: T3 (ς + → + κ + N + σ + ∂ + ν)
/// Dominant: ς State (monitoring state evolves with each signal)
impl GroundsTo for VigilanceState {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Causality,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Frequency,
        ])
        .with_dominant(LexPrimitiva::State, 0.60)
    }
}

// --- Phase 11: Data type system ---

/// DnaType: T1 (κ Comparison)
/// Dominant: κ Comparison (type discriminant for tagged values)
impl GroundsTo for DnaType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Comparison, 1.0)
    }
}

/// DnaValue: T2-P (∃ Existence + ς State)
/// Dominant: ∃ Existence (instantiation of a typed value)
impl GroundsTo for DnaValue {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Existence, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Existence, 0.90)
    }
}

/// DnaArray: T2-C (σ Sequence + N Quantity + ∂ Boundary)
/// Dominant: σ Sequence (ordered homogeneous collection)
impl GroundsTo for DnaArray {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// DnaRecord: T2-C (μ Mapping + λ Location + σ Sequence)
/// Dominant: μ Mapping (named field access)
impl GroundsTo for DnaRecord {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Location,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// DnaMap: T2-C (μ Mapping + κ Comparison + ∃ Existence)
/// Dominant: μ Mapping (key-value association)
impl GroundsTo for DnaMap {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Comparison,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.75)
    }
}

/// DnaFrame: T3 (σ + μ + N + ∂ + λ)
/// Dominant: σ Sequence (tabular rows in order)
impl GroundsTo for DnaFrame {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
            LexPrimitiva::Boundary,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.60)
    }
}

// ---------------------------------------------------------------------------
// JsonValue → T3 (μ Mapping + σ Sequence + ∂ Boundary + κ Comparison + → Causality + ∃ Existence)
// ---------------------------------------------------------------------------

impl GroundsTo for JsonValue {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Boundary,
            LexPrimitiva::Comparison,
            LexPrimitiva::Causality,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.55)
    }
}

// ---------------------------------------------------------------------------
// Template → T3 (σ Sequence + ρ Recursion + N Quantity + → Causality + μ Mapping + ∂ Boundary)
// ---------------------------------------------------------------------------

impl GroundsTo for Template {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,
            LexPrimitiva::Recursion,
            LexPrimitiva::Quantity,
            LexPrimitiva::Causality,
            LexPrimitiva::Mapping,
            LexPrimitiva::Boundary,
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.55)
    }
}

// ---------------------------------------------------------------------------
// ErrorCode → T2-P (κ Comparison + ∂ Boundary)
// ---------------------------------------------------------------------------

impl GroundsTo for ErrorCode {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::Boundary])
            .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Diagnostic → T2-C (∂ Boundary + μ Mapping + → Causality + σ Sequence)
// ---------------------------------------------------------------------------

impl GroundsTo for Diagnostic {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary,
            LexPrimitiva::Mapping,
            LexPrimitiva::Causality,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.75)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Nucleotide;

    #[test]
    fn nucleotide_is_t1() {
        let comp = Nucleotide::primitive_composition();
        assert_eq!(comp.tier(), "T1");
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!((comp.confidence - 1.0).abs() < f64::EPSILON);
        assert!(Nucleotide::is_pure_primitive());
    }

    #[test]
    fn amino_acid_is_t1() {
        let comp = AminoAcid::primitive_composition();
        assert_eq!(comp.tier(), "T1");
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn codon_is_t2p() {
        let comp = Codon::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn strand_is_t2p() {
        let comp = Strand::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn codon_table_is_t2p() {
        let comp = CodonTable::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn double_helix_is_t2c() {
        let comp = DoubleHelix::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn codon_vm_is_t3() {
        let comp = CodonVM::primitive_composition();
        assert_eq!(comp.tier(), "T3");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!((comp.confidence - 0.60).abs() < f64::EPSILON);
    }

    #[test]
    fn instruction_is_t2p() {
        let comp = Instruction::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn program_is_t2c() {
        let comp = Program::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn token_is_t2p() {
        let comp = Token::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn disasm_options_is_t2p() {
        let comp = DisasmOptions::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
    }

    #[test]
    fn lex_primitiva_symbols() {
        assert_eq!(LexPrimitiva::Sequence.symbol(), "σ");
        assert_eq!(LexPrimitiva::State.symbol(), "ς");
        assert_eq!(LexPrimitiva::Causality.symbol(), "→");
        assert_eq!(LexPrimitiva::Boundary.symbol(), "∂");
    }

    #[test]
    fn gene_is_t3() {
        let comp = Gene::primitive_composition();
        assert_eq!(comp.tier(), "T3");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
    }

    #[test]
    fn genome_is_t3() {
        let comp = Genome::primitive_composition();
        assert_eq!(comp.tier(), "T3");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    #[test]
    fn plasmid_is_t2c() {
        let comp = Plasmid::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
    }

    // --- Phase 7a: Tile + Voxel grounding tests ---

    #[test]
    fn pixel_is_t2p() {
        let comp = Pixel::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn tile_is_t2c() {
        let comp = Tile::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn voxel_pos_is_t2p() {
        let comp = VoxelPos::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
        assert!((comp.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn voxel_cube_is_t3() {
        let comp = VoxelCube::primitive_composition();
        assert_eq!(comp.tier(), "T3");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
        assert!((comp.confidence - 0.65).abs() < f64::EPSILON);
    }

    // --- Phase 7b: Glyph IR grounding tests ---

    #[test]
    fn glyph_is_t1() {
        let comp = Glyph::primitive_composition();
        assert_eq!(comp.tier(), "T1");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 1.0).abs() < f64::EPSILON);
        assert!(Glyph::is_pure_primitive());
    }

    #[test]
    fn glyph_pair_is_t2c() {
        let comp = GlyphPair::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
        assert!(!GlyphPair::is_pure_primitive());
    }

    // --- Phase 8: Lexicon grounding tests ---

    #[test]
    fn wordore_is_t2c() {
        let comp = WordOre::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
        assert!(!WordOre::is_pure_primitive());
    }

    #[test]
    fn affinity_is_t2p() {
        let comp = Affinity::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!((comp.confidence - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn lexicon_is_t3() {
        let comp = Lexicon::primitive_composition();
        assert_eq!(comp.tier(), "T3");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 0.65).abs() < f64::EPSILON);
    }

    #[test]
    fn wordore_primitives_count() {
        let comp = WordOre::primitive_composition();
        assert_eq!(comp.primitives.len(), 4);
    }

    #[test]
    fn lexicon_primitives_count() {
        let comp = Lexicon::primitive_composition();
        assert_eq!(comp.primitives.len(), 7);
    }

    // --- Phase 8b: StateMind grounding tests ---

    #[test]
    fn mindpoint_is_t2p() {
        let comp = MindPoint::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
        assert!((comp.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn drift_is_t2p() {
        let comp = Drift::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!((comp.confidence - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn statemind_is_t3() {
        let comp = StateMind::primitive_composition();
        assert_eq!(comp.tier(), "T3");
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!((comp.confidence - 0.60).abs() < f64::EPSILON);
    }

    #[test]
    fn mindpoint_primitives_count() {
        let comp = MindPoint::primitive_composition();
        assert_eq!(comp.primitives.len(), 2);
        assert!(!MindPoint::is_pure_primitive());
    }

    #[test]
    fn statemind_primitives_count() {
        let comp = StateMind::primitive_composition();
        assert_eq!(comp.primitives.len(), 9);
        assert!(!StateMind::is_pure_primitive());
    }

    // --- Phase 9: Cortex grounding tests ---

    #[test]
    fn cluster_is_t2c() {
        let comp = Cluster::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn cluster_result_is_t3() {
        let comp = ClusterResult::primitive_composition();
        assert_eq!(comp.tier(), "T3");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.60).abs() < f64::EPSILON);
    }

    #[test]
    fn particle_is_t2c() {
        let comp = Particle::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!((comp.confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn organism_is_t2c() {
        let comp = Organism::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn evolution_result_is_t3() {
        let comp = EvolutionResult::primitive_composition();
        assert_eq!(comp.tier(), "T3");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.60).abs() < f64::EPSILON);
    }

    // --- Phase 10: String Theory grounding tests ---

    #[test]
    fn string_tension_is_t2c() {
        let comp = StringTension::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn harmonic_mode_is_t2p() {
        let comp = HarmonicMode::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Frequency));
        assert!((comp.confidence - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn frequency_spectrum_is_t2c() {
        let comp = FrequencySpectrum::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Frequency));
        assert!((comp.confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn resonance_is_t2c() {
        let comp = Resonance::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!((comp.confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn string_energy_is_t2c() {
        let comp = StringEnergy::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
    }

    // --- Phase 10b: PV Theory grounding tests ---

    #[test]
    fn causality_category_is_t1() {
        let comp = CausalityCategory::primitive_composition();
        assert_eq!(comp.tier(), "T1");
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!(CausalityCategory::is_pure_primitive());
    }

    #[test]
    fn safety_level_is_t1() {
        let comp = SafetyLevel::primitive_composition();
        assert_eq!(comp.tier(), "T1");
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn alert_level_is_t1() {
        let comp = AlertLevel::primitive_composition();
        assert_eq!(comp.tier(), "T1");
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
    }

    #[test]
    fn drug_profile_is_t3() {
        let comp = DrugProfile::primitive_composition();
        assert_eq!(comp.tier(), "T3");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!((comp.confidence - 0.65).abs() < f64::EPSILON);
    }

    #[test]
    fn signal_is_t2c() {
        let comp = Signal::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn safety_margin_is_t2c() {
        let comp = SafetyMargin::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn causality_score_is_t2c() {
        let comp = CausalityScore::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Causality));
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn vigilance_state_is_t3() {
        let comp = VigilanceState::primitive_composition();
        assert_eq!(comp.tier(), "T3");
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!((comp.confidence - 0.60).abs() < f64::EPSILON);
    }

    // --- Phase 11: Data grounding tests ---

    #[test]
    fn dna_type_is_t1() {
        let comp = DnaType::primitive_composition();
        assert_eq!(comp.tier(), "T1");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!((comp.confidence - 1.0).abs() < f64::EPSILON);
        assert!(DnaType::is_pure_primitive());
    }

    #[test]
    fn dna_value_is_t2p() {
        let comp = DnaValue::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Existence));
        assert!((comp.confidence - 0.90).abs() < f64::EPSILON);
    }

    #[test]
    fn dna_array_is_t2p() {
        let comp = DnaArray::primitive_composition();
        assert_eq!(comp.tier(), "T2-P"); // 3 primitives → T2-P
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn dna_record_is_t2p() {
        let comp = DnaRecord::primitive_composition();
        assert_eq!(comp.tier(), "T2-P"); // 3 primitives → T2-P
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 0.80).abs() < f64::EPSILON);
    }

    #[test]
    fn dna_map_is_t2p() {
        let comp = DnaMap::primitive_composition();
        assert_eq!(comp.tier(), "T2-P"); // 3 primitives → T2-P
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn dna_frame_is_t2c() {
        let comp = DnaFrame::primitive_composition();
        assert_eq!(comp.tier(), "T2-C"); // 5 primitives → T2-C
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.60).abs() < f64::EPSILON);
    }

    // --- Phase 13: AI Interface grounding tests ---

    #[test]
    fn json_value_is_t3() {
        let comp = JsonValue::primitive_composition();
        assert_eq!(comp.tier(), "T3"); // 6 primitives → T3
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert!((comp.confidence - 0.55).abs() < f64::EPSILON);
    }

    #[test]
    fn template_is_t3() {
        let comp = Template::primitive_composition();
        assert_eq!(comp.tier(), "T3"); // 6 primitives → T3
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert!((comp.confidence - 0.55).abs() < f64::EPSILON);
    }

    // --- Phase 14: Diagnostic grounding tests ---

    #[test]
    fn error_code_is_t2p() {
        let comp = ErrorCode::primitive_composition();
        assert_eq!(comp.tier(), "T2-P");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert!((comp.confidence - 0.85).abs() < f64::EPSILON);
    }

    #[test]
    fn diagnostic_is_t2c() {
        let comp = Diagnostic::primitive_composition();
        assert_eq!(comp.tier(), "T2-C");
        assert_eq!(comp.dominant, Some(LexPrimitiva::Boundary));
        assert!((comp.confidence - 0.75).abs() < f64::EPSILON);
    }
}
