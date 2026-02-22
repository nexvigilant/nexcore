//! Knowledge Engine Parameters (ingest, compress, compile, query, stats)

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for knowledge_ingest.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeIngestParams {
    /// Text to ingest as a knowledge fragment.
    pub text: String,
    /// Source type: free_text, brain_distillation, brain_artifact, implicit_knowledge, lesson, session_reflection.
    pub source_type: Option<String>,
    /// Domain classification (auto-detected if omitted).
    pub domain: Option<String>,
    /// Auto-classify domain from concepts (default: true).
    pub auto_classify: Option<bool>,
}

/// Parameters for knowledge_compress.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeCompressParams {
    /// Text to compress.
    pub text: String,
    /// Compression method: pattern, dedup, hierarchy, summary, or all (default: all).
    pub method: Option<String>,
    /// Target compendious score (aims to compress until this score is reached).
    pub target_cs: Option<f64>,
    /// Terms to preserve during compression.
    pub preserve: Option<Vec<String>>,
}

/// Parameters for knowledge_compile.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeCompileParams {
    /// Name for the knowledge pack.
    pub name: String,
    /// Raw text sources to include (optional, in addition to Brain sources).
    pub sources: Option<Vec<String>>,
    /// Include Brain distillation files (default: true).
    pub include_distillations: Option<bool>,
    /// Include Brain artifact files (default: false).
    pub include_artifacts: Option<bool>,
    /// Include implicit knowledge files (default: false).
    pub include_implicit: Option<bool>,
}

/// Parameters for knowledge_query.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeQueryParams {
    /// Search query text.
    pub query: String,
    /// Specific pack name to query (queries all packs if omitted).
    pub pack_name: Option<String>,
    /// Query mode: keyword (default), concept, or domain.
    pub mode: Option<String>,
    /// Filter results to a specific domain.
    pub domain: Option<String>,
    /// Maximum number of results (default: 10).
    pub limit: Option<usize>,
}

/// Parameters for knowledge_stats.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeStatsParams {
    /// Specific pack name (all packs if omitted).
    pub pack_name: Option<String>,
}

/// Parameters for knowledge_score_compendious.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeScoreCompendiousParams {
    /// Text to score for information density (Cs = I/E × C × R).
    pub text: String,
    /// Required terms that should appear in the text (affects completeness factor).
    #[serde(default)]
    pub required_terms: Vec<String>,
}

/// Parameters for knowledge_extract_primitives.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeExtractPrimitivesParams {
    /// Text to extract T1/T2/T3 primitives from via keyword heuristics.
    pub text: String,
}

/// Parameters for knowledge_extract_concepts.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct KnowledgeExtractConceptsParams {
    /// Text to extract significant concepts from with domain classification.
    pub text: String,
}
