//! # NexVigilant Core — dna — DNA-Based Computation in Rust
//!
//! DNA is a quaternary encoding system (A, T, G, C) that maps cleanly to Rust's
//! type system. This crate makes the biological metaphor literal — DNA sequences
//! ARE programs.
//!
//! ## Capabilities
//!
//! | Capability | Module | Primitives |
//! |-----------|--------|------------|
//! | Core Types | `types` | ς, σ, μ, κ, ∃ |
//! | Data Storage | `storage` | σ, μ, π |
//! | Bio Ops | `ops` | σ, μ, →, ∂ |
//! | Codon VM | `vm` | σ, μ, ς, ∂, N, → |
//! | ISA | `isa` | μ, σ, ∂, κ |
//! | Lexicon | `lexicon` | N, σ, μ, κ, ∂, ∃, π |
//! | StateMind | `statemind` | ς, σ, μ, κ, N, λ, ∂, ∃, π |
//! | Assembler | `asm` | σ, μ, ∂, ς, →, ∃ |
//! | Disassembler | `disasm` | μ, σ, κ, ∃ |
//! | Program | `program` | σ, ∂, ς, μ |
//! | Pixel Tile | `tile` | σ, μ, ∂, λ |
//! | Voxel Cube | `voxel` | σ, μ, ∂, N, λ, κ, →, ∃ |
//! | Glyph IR | `glyph` | μ, ∂, κ, N, σ, ς, ρ, → |
//! | Transcoder | `transcode` | κ, μ, σ, ∂, → |
//! | Cortex | `cortex` | λ, N, κ, μ, ς, →, σ, ρ |
//! | String Theory | `string_theory` | ν, N, κ, σ, μ, Σ |
//! | Data Structures | `data` | σ, μ, ∂, ∃, κ, N, λ, π |
//! | PV Theory | `pv_theory` | →, κ, N, ∂, ς, ν |
//! | JSON AST | `lang::json` | μ, σ, ∂, κ, →, ∃ |
//! | Templates | `lang::templates` | σ, ρ, N, →, μ, ∂ |
//! | Diagnostics | `lang::diagnostic` | ∂, μ, →, σ |
//!
//! ## Quick Start
//!
//! ```rust
//! use nexcore_dna::prelude::*;
//!
//! // Parse a DNA strand
//! let strand = Strand::parse("ATGGAATAA").unwrap();
//!
//! // Translate to protein
//! let table = CodonTable::standard();
//! let protein = nexcore_dna::ops::translate(&strand, &table).unwrap();
//! assert_eq!(protein[0], AminoAcid::Met);
//! assert_eq!(protein[1], AminoAcid::Glu);
//!
//! // Encode data as DNA
//! let encoded = nexcore_dna::storage::encode_str("Hello, DNA!");
//! let decoded = nexcore_dna::storage::decode_str(&encoded).unwrap();
//! assert_eq!(decoded, "Hello, DNA!");
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod asm;
pub mod codon_table;
pub mod cortex;
pub mod data;
pub mod disasm;
pub mod error;
pub mod gene;
pub mod glyph;
pub mod grounding;
pub mod isa;
pub mod lang;
pub mod lexicon;
pub mod nexcore_encoding;
pub mod ops;
pub mod program;
pub mod pv_theory;
pub mod statemind;
pub mod storage;
pub mod string_theory;
pub mod tile;
pub mod transcode;
pub mod types;
pub mod vm;
pub mod voxel;

#[cfg(test)]
mod bench;

#[cfg(test)]
mod evaluation;

#[cfg(test)]
mod proofs;

/// Prelude: commonly used types for convenient imports.
pub mod prelude {
    pub use crate::asm::assemble;
    pub use crate::codon_table::CodonTable;
    pub use crate::cortex::{
        Cluster, ClusterResult, EvolutionConfig, EvolutionResult, GravityConfig, GravityResult,
        Organism, evolve, gravity_sim, kmeans,
    };
    pub use crate::data::{
        DnaArray, DnaFrame, DnaMap, DnaRecord, DnaType, DnaValue, excise_field, ligate_arrays,
        ligate_records, restrict, restrict_range, splice_field, transcribe_frame,
        transcribe_record, transcribe_value,
    };
    pub use crate::disasm::disassemble;
    pub use crate::error::{DnaError, Result};
    pub use crate::gene::{Gene, GeneAnnotation, Genome, Plasmid, crossover};
    pub use crate::glyph::{Glyph, GlyphPair};
    pub use crate::grounding::{GroundsTo, LexPrimitiva, PrimitiveComposition, Tier};
    pub use crate::isa::Instruction;
    pub use crate::lang::ast::{BinOp, Expr, Stmt};
    pub use crate::lang::compiler::{compile, compile_genome, compile_to_asm, eval};
    pub use crate::lang::diagnostic::{
        Diagnostic, ErrorCode, diagnose, diagnostic_to_json, diagnostics_to_json,
    };
    pub use crate::lang::json::{
        ast_to_json, ast_to_json_pretty, json_eval, json_to_ast, json_to_program, source_to_json,
    };
    pub use crate::lang::templates;
    pub use crate::lexicon::{Affinity, Lexicon, WordOre};
    pub use crate::nexcore_encoding::{
        NEXCORE_SIGNAL_GENOME_SOURCE, SignalDetectionProgram, encode_type_record,
        guardian_thresholds_record, nexcore_genome_strand, nexcore_signal_genome,
        signal_detection_programs, type_registry,
    };
    pub use crate::program::Program;
    pub use crate::pv_theory::{
        AlertLevel, CausalityCategory, CausalityScore, DrugEventSignal, DrugProfile, SafetyLevel,
        SafetyMargin, VigilanceState, assess_causality, detect_signal, monitor, profile_drug,
        safety_margin,
    };
    pub use crate::statemind::{Drift, MindPoint, StateMind};
    pub use crate::string_theory::{
        FrequencySpectrum, HarmonicMode, Resonance, StringEnergy, StringTension, resonance,
        spectrum, string_energy, tension, word_resonance, word_spectrum, word_tension,
    };
    pub use crate::tile::{Pixel, Tile};
    pub use crate::transcode::{Encoding, ProgramProfile, Recommendation, TranscodeResult};
    pub use crate::types::{AminoAcid, Codon, DoubleHelix, Nucleotide, Strand};
    pub use crate::vm::{CodonVM, HaltReason, VmConfig, VmResult};
    pub use crate::voxel::{VoxelCube, VoxelPos};
}
