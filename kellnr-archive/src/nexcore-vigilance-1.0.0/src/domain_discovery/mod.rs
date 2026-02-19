//! # Domain Discovery Types
//!
//! Type-safe Rust representations of domain discovery artifacts.
//! Consolidated from YAML definitions in `~/.claude/knowledge/domains/`.
//!
//! ## Modules
//!
//! - [`primitives`] - Primitive extraction types (T1/T2/T3 tiers)
//! - [`grammar`] - Grammar production rules and terminals
//! - [`translation`] - Cross-domain translation mappings
//! - [`validation`] - L1-L5 validation results

pub mod grammar;
pub mod primitives;
pub mod translation;

// Re-exports for convenience
pub use grammar::{
    Grammar, GrammarCategory, NonTerminal, Production, ProductionId, Symbol, Terminal,
};
pub use primitives::{
    DomainCoverage, Primitive, PrimitiveId, PrimitiveTier, PrimitivesByTier, PrimitivesResult,
    TierCounts,
};
pub use translation::{
    FeasibilityScore, MappingType, NovelSynthesis, ReverseMapping, TransferMapping,
    TranslationRecord, UniversalMapping,
};
