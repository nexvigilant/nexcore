//! Knowledge Engine tools — ingest, compress, compile, query, stats.
//!
//! 5 MCP tools for structured knowledge management.
//! Highway Class: II (<100ms) for ingest/compress/query/stats, III (<500ms) for compile.

use crate::params::knowledge_engine::{
    KnowledgeCompileParams, KnowledgeCompressParams, KnowledgeExtractConceptsParams,
    KnowledgeExtractPrimitivesParams, KnowledgeIngestParams, KnowledgeQueryParams,
    KnowledgeScoreCompendiousParams, KnowledgeStatsParams,
};
use nexcore_chrono::DateTime;
use nexcore_knowledge_engine::extraction::ConceptExtractor;
use nexcore_knowledge_engine::scoring::CompendiousScorer;
use nexcore_knowledge_engine::{
    CompileOptions, KnowledgeCompiler, KnowledgeSource, KnowledgeStore, QueryEngine, QueryMode,
    RawKnowledge,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;

fn parse_source_type(s: &str) -> KnowledgeSource {
    match s {
        "brain_distillation" => KnowledgeSource::BrainDistillation,
        "brain_artifact" => KnowledgeSource::BrainArtifact,
        "implicit_knowledge" => KnowledgeSource::ImplicitKnowledge,
        "lesson" => KnowledgeSource::Lesson,
        "session_reflection" => KnowledgeSource::SessionReflection,
        _ => KnowledgeSource::FreeText,
    }
}

fn parse_query_mode(s: &str) -> QueryMode {
    match s {
        "concept" => QueryMode::Concept,
        "domain" => QueryMode::Domain,
        _ => QueryMode::Keyword,
    }
}

fn store() -> Result<KnowledgeStore, McpError> {
    KnowledgeStore::default_location().map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Store error: {e}").into(),
        data: None,
    })
}

/// Ingest text as a knowledge fragment.
pub fn ingest(params: KnowledgeIngestParams) -> Result<CallToolResult, McpError> {
    let source_type = params
        .source_type
        .as_deref()
        .map(parse_source_type)
        .unwrap_or(KnowledgeSource::FreeText);

    let frag = KnowledgeCompiler::ingest_single(&params.text, source_type, params.domain).map_err(
        |e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Ingest error: {e}").into(),
            data: None,
        },
    )?;

    let result = json!({
        "fragment_id": frag.id,
        "domain": frag.domain,
        "concepts": frag.concepts.iter().map(|c| &c.term).collect::<Vec<_>>(),
        "primitives": frag.primitives.iter().map(|p| json!({
            "name": p.name,
            "tier": p.tier.to_string(),
        })).collect::<Vec<_>>(),
        "compendious_score": format!("{:.3}", frag.score.compendious_score),
        "interpretation": frag.score.interpretation,
        "expression_cost": frag.score.expression_cost,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Compress text and return before/after scores.
pub fn compress(params: KnowledgeCompressParams) -> Result<CallToolResult, McpError> {
    let (original_score, compressed_score, compressed_text, ratio) =
        KnowledgeCompiler::compress_text(&params.text);

    let result = json!({
        "original_score": format!("{:.3}", original_score.compendious_score),
        "compressed_score": format!("{:.3}", compressed_score.compendious_score),
        "compressed_text": compressed_text,
        "compression_ratio": format!("{:.1}%", ratio * 100.0),
        "original_words": original_score.expression_cost,
        "compressed_words": compressed_score.expression_cost,
        "original_interpretation": original_score.interpretation,
        "compressed_interpretation": compressed_score.interpretation,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Compile knowledge from sources into a pack.
pub fn compile(params: KnowledgeCompileParams) -> Result<CallToolResult, McpError> {
    let knowledge_store = store()?;
    let compiler = KnowledgeCompiler::new(knowledge_store);

    let sources: Vec<RawKnowledge> = params
        .sources
        .unwrap_or_default()
        .into_iter()
        .map(|text| RawKnowledge {
            text,
            source: KnowledgeSource::FreeText,
            domain: None,
            timestamp: DateTime::now(),
        })
        .collect();

    let options = CompileOptions {
        name: params.name,
        include_distillations: params.include_distillations.unwrap_or(true),
        include_artifacts: params.include_artifacts.unwrap_or(false),
        include_implicit: params.include_implicit.unwrap_or(false),
        sources,
    };

    let pack = compiler.compile(options).map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Compile error: {e}").into(),
        data: None,
    })?;

    let result = json!({
        "pack_id": pack.id,
        "name": pack.name,
        "version": pack.version,
        "fragment_count": pack.stats.fragment_count,
        "concept_count": pack.stats.concept_count,
        "compression_ratio": format!("{:.1}%", pack.stats.compression_ratio * 100.0),
        "avg_score": format!("{:.3}", pack.stats.avg_compendious_score),
        "domains": pack.stats.domains,
        "total_words": pack.stats.total_words,
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Query knowledge packs.
pub fn query(params: KnowledgeQueryParams) -> Result<CallToolResult, McpError> {
    let knowledge_store = store()?;
    let engine = QueryEngine::new(knowledge_store);

    let mode = params
        .mode
        .as_deref()
        .map(parse_query_mode)
        .unwrap_or_default();
    let limit = params.limit.unwrap_or(10);

    let responses = engine
        .query(
            &params.query,
            params.pack_name.as_deref(),
            mode,
            params.domain.as_deref(),
            limit,
        )
        .map_err(|e| McpError {
            code: rmcp::model::ErrorCode::INTERNAL_ERROR,
            message: format!("Query error: {e}").into(),
            data: None,
        })?;

    let results: Vec<serde_json::Value> = responses
        .iter()
        .flat_map(|r| {
            r.results.iter().map(|qr| {
                json!({
                    "content": qr.content,
                    "concepts": qr.concepts,
                    "domain": qr.domain,
                    "relevance": format!("{:.3}", qr.relevance),
                })
            })
        })
        .collect();

    let total: usize = responses.iter().map(|r| r.total_matches).sum();

    let result = json!({
        "results": results,
        "total_matches": total,
        "packs_searched": responses.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Get knowledge engine statistics.
pub fn stats(params: KnowledgeStatsParams) -> Result<CallToolResult, McpError> {
    let knowledge_store = store()?;

    let engine_stats = if let Some(ref name) = params.pack_name {
        nexcore_knowledge_engine::pack_stats(&knowledge_store, name)
    } else {
        nexcore_knowledge_engine::compute_stats(&knowledge_store)
    }
    .map_err(|e| McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: format!("Stats error: {e}").into(),
        data: None,
    })?;

    let packs: Vec<serde_json::Value> = engine_stats
        .packs
        .iter()
        .map(|p| {
            json!({
                "name": p.name,
                "version": p.version,
                "fragments": p.fragment_count,
                "concepts": p.concept_count,
                "avg_score": format!("{:.3}", p.avg_score),
            })
        })
        .collect();

    let result = json!({
        "packs": packs,
        "total_packs": engine_stats.total_packs,
        "total_fragments": engine_stats.total_fragments,
        "total_concepts": engine_stats.total_concepts,
        "avg_score": format!("{:.3}", engine_stats.avg_score),
    });

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Score text for information density using the Compendious Score (Cs = I/E × C × R).
///
/// Returns the score breakdown: density, completeness, readability, interpretation,
/// and the limiting factor dragging the score down.
pub fn score_compendious(
    params: KnowledgeScoreCompendiousParams,
) -> Result<CallToolResult, McpError> {
    let result = CompendiousScorer::score(&params.text, &params.required_terms);

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "compendious_score": format!("{:.3}", result.compendious_score),
            "information_content": format!("{:.1}", result.information_content),
            "expression_cost": result.expression_cost,
            "completeness": format!("{:.3}", result.completeness),
            "readability": format!("{:.3}", result.readability),
            "interpretation": result.interpretation,
            "limiting_factor": result.limiting_factor,
        })
        .to_string(),
    )]))
}

/// Extract T1/T2/T3 primitives from text using keyword heuristics.
///
/// Scans for primitive indicators (cause, transform, boundary, etc.) and
/// returns classified primitives with tier and description.
pub fn extract_primitives(
    params: KnowledgeExtractPrimitivesParams,
) -> Result<CallToolResult, McpError> {
    let prims = ConceptExtractor::extract_primitives(&params.text);

    let prims_json: Vec<serde_json::Value> = prims
        .iter()
        .map(|p| {
            json!({
                "name": p.name,
                "tier": p.tier.to_string(),
                "description": p.description,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "primitive_count": prims.len(),
            "primitives": prims_json,
        })
        .to_string(),
    )]))
}

/// Extract significant concepts from text with domain classification.
///
/// Returns terms sorted by frequency, each optionally classified into a domain
/// (pv, rust, claude-code, chemistry, physics, regulatory).
pub fn extract_concepts(
    params: KnowledgeExtractConceptsParams,
) -> Result<CallToolResult, McpError> {
    let concepts = ConceptExtractor::extract_concepts(&params.text);

    let concepts_json: Vec<serde_json::Value> = concepts
        .iter()
        .map(|c| {
            json!({
                "term": c.term,
                "domain": c.domain,
                "frequency": c.frequency,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "concept_count": concepts.len(),
            "concepts": concepts_json,
        })
        .to_string(),
    )]))
}
