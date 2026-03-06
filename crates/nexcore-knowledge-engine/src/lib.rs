#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

//! nexcore-knowledge-engine — Knowledge compression, compilation, and query engine.
//!
//! Ingests raw knowledge, compresses it structurally, compiles it into indexed
//! "knowledge packs", and exposes query interfaces.
//!
//! Layer: Domain (depends on foundation, consumed by nexcore-mcp)
//!
//! Compendious Score: Cs = (I / E) × C × R

pub mod compiler;
pub mod compression;
pub mod concept_graph;
pub mod error;
pub mod extraction;
pub mod ingest;
pub mod knowledge_pack;
pub mod scoring;
pub mod store;

pub use compiler::{CompileOptions, CompressTextResult, KnowledgeCompiler};
pub use compression::{
    CompressionMethod, CompressionResult, StructuralCompressor, token_similarity,
};
pub use concept_graph::{ConceptEdge, ConceptGraph, ConceptNode, ConceptRelation};
pub use error::{KnowledgeEngineError, Result};

/// Opaque identifier for knowledge entities (fragments, packs).
///
/// Currently backed by `String` (NexId v4). Using a type alias allows future
/// migration to a newtype without changing call sites.
pub type KnowledgeId = String;

pub use extraction::{
    ConceptExtractor, DomainClassifier, ExtractedConcept, ExtractedPrimitive, PrimitiveTier,
};
pub use ingest::{KnowledgeFragment, KnowledgeSource, RawKnowledge, ingest};
pub use knowledge_pack::{KnowledgePack, PackIndex, PackStats};
pub use scoring::{CompendiousScorer, LimitingFactor, STOPWORDS, ScoreInterpretation, ScoreResult};
pub use store::KnowledgeStore;

pub mod grounding;
pub mod query;
pub mod stats;

pub use query::{QueryEngine, QueryMode, QueryResponse, QueryResult};
pub use stats::{EngineStats, compute_stats, pack_stats};
