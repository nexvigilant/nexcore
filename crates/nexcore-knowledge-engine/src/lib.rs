#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

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

pub use compiler::{CompileOptions, KnowledgeCompiler};
pub use compression::{
    CompressionMethod, CompressionResult, StructuralCompressor, token_similarity,
};
pub use concept_graph::{ConceptEdge, ConceptGraph, ConceptNode, ConceptRelation};
pub use error::{KnowledgeEngineError, Result};
pub use extraction::{ConceptExtractor, ExtractedConcept, ExtractedPrimitive, PrimitiveTier};
pub use ingest::{KnowledgeFragment, KnowledgeSource, RawKnowledge, ingest};
pub use knowledge_pack::{KnowledgePack, PackIndex, PackStats};
pub use scoring::{CompendiousScorer, ScoreResult};
pub use store::KnowledgeStore;

pub mod grounding;
pub mod query;
pub mod stats;

pub use query::{QueryEngine, QueryMode, QueryResponse, QueryResult};
pub use stats::{EngineStats, compute_stats, pack_stats};
