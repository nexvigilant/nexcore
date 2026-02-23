// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Prelude — nexcore-lex-primitiva
//!
//! Convenience re-exports of the most frequently used types from the
//! Lex Primitiva system.
//!
//! Import everything from this module with:
//!
//! ```rust
//! use nexcore_lex_primitiva::prelude::*;
//! ```
//!
//! ## Included Types
//!
//! | Category | Types |
//! |----------|-------|
//! | Core primitives | [`LexPrimitiva`], [`PrimitiveComposition`] |
//! | Tier system | [`Tier`] |
//! | Grounding | [`GroundsTo`] |
//! | Bedrock | [`BedrockAtom`] |
//! | State | [`StateMode`] |
//! | Graph | [`DependencyGraph`], [`GroundingTrace`], [`MathFoundation`] |
//! | Constants | [`MathConstant`], [`PHI`], [`ALPHA_SIGNIFICANCE`], [`K_BOLTZMANN`], [`Trichotomy`] |
//! | Composition algebra | [`CompositionAlgebra`], [`CompositionBuilder`], [`CompositionOp`], [`CompositionResult`], [`CompositionScore`], [`SemanticValidation`] |
//! | Compound tracker | [`BasisSnapshot`], [`CompoundTracker`], [`ProjectionResult`] |
//! | Compound detector | [`Bottleneck`], [`ComponentAnalysis`], [`CompoundDetector`], [`DetectionResult`], [`GrowthPhase`] |
//! | Dossier | [`Dossier`], [`DossierGenerator`], [`DossierSubject`] |
//! | External grounding | [`ExternalAuthority`], [`GroundingStrength`] |
//! | Extraction | [`ExtractedPrimitive`], [`ExtractionResult`], [`PrimitiveExtractor`] |
//! | Grammar | [`CompressionMetrics`], [`Interaction`], [`InteractionGraph`], [`InteractionType`], [`Pattern`], [`PatternRegistry`] |
//! | Molecular weight | [`AtomicMass`], [`MolecularFormula`], [`MolecularWeight`], [`TransferClass`] |
//! | Semantic path | [`GroundingType`], [`SemanticGroundingPath`], [`SemanticRelation`], [`SemanticStep`] |
//! | Synthesizer | [`CompletionSuggestion`], [`PatternMatch`], [`RevSynthesizer`], [`SynthesisError`], [`SynthesisOpts`], [`SynthesisResult`] |
//! | Transfer | [`Domain`], [`TransferCalculator`], [`TransferResult`] |
//! | Validation | [`PrimitivaValidator`], [`ValidationReport`] |
//! | Weighted composition | [`WeightedBuilder`], [`WeightedComposition`] |
//! | Symbols | (all symbol constants via glob) |

// ── Core primitives ──────────────────────────────────────────────────────────

pub use crate::primitiva::{LexPrimitiva, PrimitiveComposition};

// ── Tier system ──────────────────────────────────────────────────────────────

pub use crate::tier::Tier;

// ── Grounding ────────────────────────────────────────────────────────────────

pub use crate::grounding::GroundsTo;

// ── Bedrock atoms ────────────────────────────────────────────────────────────

pub use crate::bedrock::BedrockAtom;

// ── State mode ───────────────────────────────────────────────────────────────

pub use crate::state_mode::StateMode;

// ── Dependency graph and mathematical foundations ────────────────────────────

pub use crate::graph::{DependencyGraph, GroundingTrace, MathFoundation};

// ── Mathematical constants ───────────────────────────────────────────────────

pub use crate::constants::{ALPHA_SIGNIFICANCE, K_BOLTZMANN, MathConstant, PHI, Trichotomy};

// ── Composition algebra ──────────────────────────────────────────────────────

pub use crate::composition::{
    CompositionAlgebra, CompositionBuilder, CompositionOp, CompositionResult, CompositionScore,
    SemanticValidation,
};

// ── Compound tracker ─────────────────────────────────────────────────────────

pub use crate::compound::{BasisSnapshot, CompoundTracker, ProjectionResult};

// ── Compound detector ────────────────────────────────────────────────────────

pub use crate::compound_detector::{
    Bottleneck, ComponentAnalysis, CompoundDetector, DetectionResult, GrowthPhase,
};

// ── Dossier ──────────────────────────────────────────────────────────────────

pub use crate::dossier::{Dossier, DossierGenerator, DossierSubject};

// ── External grounding ───────────────────────────────────────────────────────

pub use crate::external_grounding::{
    ExternalAuthority, GroundingStrength, external_authorities, grounding_strength,
    is_externally_grounded,
};

// ── Extraction ───────────────────────────────────────────────────────────────

pub use crate::extraction::{ExtractedPrimitive, ExtractionResult, PrimitiveExtractor};

// ── Grammar / interaction patterns ───────────────────────────────────────────

pub use crate::grammar::{
    CompressionMetrics, Interaction, InteractionGraph, InteractionType, Pattern, PatternRegistry,
};

// ── Molecular weight ─────────────────────────────────────────────────────────

pub use crate::molecular_weight::{
    AtomicMass, MolecularFormula, MolecularWeight, TransferClass,
};

// ── Semantic grounding path ──────────────────────────────────────────────────

pub use crate::semantic_path::{
    GroundingType, SemanticGroundingPath, SemanticRelation, SemanticStep,
    anti_triviality_argument, grounding_type, semantic_paths, validate_all_semantic_grounding,
};

// ── Synthesizer ──────────────────────────────────────────────────────────────

pub use crate::synthesizer::{
    CompletionSuggestion, PatternMatch, RevSynthesizer, SynthesisError, SynthesisOpts,
    SynthesisResult,
};

// ── Transfer calculator ──────────────────────────────────────────────────────

pub use crate::transfer::{Domain, TransferCalculator, TransferResult};

// ── Validation ───────────────────────────────────────────────────────────────

pub use crate::validate::{PrimitivaValidator, ValidationReport, validate_system};

// ── Weighted composition ─────────────────────────────────────────────────────

pub use crate::weighted::{WeightedBuilder, WeightedComposition};

// ── Complexity ───────────────────────────────────────────────────────────────

pub use crate::complexity::{
    ComplexityClass, MINIMAL_SET, comparison_report, derivation_complexity,
    practical_minimality_argument,
};

// ── Symbol constants ─────────────────────────────────────────────────────────

pub use crate::symbols::*;
