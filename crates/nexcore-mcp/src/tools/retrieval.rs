//! Persistent Retrieval Pipeline — unified multi-source search with caching.
//!
//! Inspired by AI Engineering Bible Section 4 (RAG pipelines):
//! instead of fragmented search across brain, vigil, and filesystem,
//! provides a single retrieval API with merged ranking, caching, and metrics.
//!
//! # Architecture
//!
//! ```text
//! Query → Cache Check → [Brain, Qdrant, Filesystem, Implicit] → Merge → Rank → Return
//!              ↓                                                    ↓
//!         Cache Hit?                                           Cache Store
//!              ↓                                                    ↓
//!         Return cached                                      Update Metrics
//! ```
//!
//! # T1 Grounding: μ(Mapping) + σ(Sequence) + κ(Comparison) + π(Persistence) + ν(Frequency)
//! - μ: Query → ranked results mapping
//! - σ: Multi-source pipeline sequence
//! - κ: Relevance comparison and ranking
//! - π: Cache persistence and metrics tracking
//! - ν: Access frequency and freshness decay

use crate::params::{RetrievalIngestParams, RetrievalQueryParams, RetrievalStatsParams};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

// ============================================================================
// Persistent State
// ============================================================================

/// A single indexed document in the retrieval store.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RetrievalEntry {
    source_id: String,
    title: String,
    content: String,
    tags: Vec<String>,
    ingested_at: String,
    ttl_hours: u64,
    access_count: u64,
    last_accessed: String,
}

/// Cache entry for a recent query.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    query: String,
    results: Vec<serde_json::Value>,
    created_at: String,
    ttl_seconds: u64,
    hit_count: u64,
}

/// Persistent retrieval metrics.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RetrievalMetrics {
    total_queries: u64,
    cache_hits: u64,
    cache_misses: u64,
    source_hits: HashMap<String, u64>,
    query_log: Vec<QueryLogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryLogEntry {
    query: String,
    timestamp: String,
    result_count: usize,
    sources_searched: Vec<String>,
    cache_hit: bool,
    latency_ms: u64,
}

/// Full retrieval store state.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct RetrievalStore {
    entries: Vec<RetrievalEntry>,
    cache: Vec<CacheEntry>,
    metrics: RetrievalMetrics,
}

fn store_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    PathBuf::from(format!("{home}/.claude/retrieval"))
}

fn store_path() -> PathBuf {
    store_dir().join("store.json")
}

fn load_store() -> RetrievalStore {
    let path = store_path();
    if path.exists() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        RetrievalStore::default()
    }
}

fn save_store(store: &RetrievalStore) -> Result<(), McpError> {
    let dir = store_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| McpError::internal_error(format!("Cannot create retrieval dir: {e}"), None))?;
    let json = serde_json::to_string_pretty(store)
        .map_err(|e| McpError::internal_error(format!("Serialize error: {e}"), None))?;
    std::fs::write(store_path(), json)
        .map_err(|e| McpError::internal_error(format!("Write error: {e}"), None))?;
    Ok(())
}

fn now_iso() -> String {
    nexcore_chrono::DateTime::now().to_rfc3339()
}

fn parse_iso(s: &str) -> Option<nexcore_chrono::DateTime> {
    nexcore_chrono::DateTime::parse_from_rfc3339(s).ok()
}

fn hours_since(iso: &str) -> f64 {
    parse_iso(iso)
        .map(|dt| {
            let elapsed = nexcore_chrono::DateTime::now().signed_duration_since(dt);
            elapsed.num_minutes() as f64 / 60.0
        })
        .unwrap_or(999.0)
}

// ============================================================================
// Scoring Engine
// ============================================================================

/// Compute TF-IDF-style relevance score for content against query terms.
fn relevance_score(content: &str, query_terms: &[String]) -> f64 {
    if query_terms.is_empty() {
        return 0.0;
    }
    let content_lower = content.to_lowercase();
    let doc_len = content_lower.split_whitespace().count().max(1) as f64;
    let mut score = 0.0_f64;
    let mut terms_found = 0_usize;

    for term in query_terms {
        let tf_count = content_lower.matches(term.as_str()).count();
        if tf_count > 0 {
            terms_found += 1;
            // Normalized TF, capped to avoid domination by long docs
            let tf = (tf_count as f64 / doc_len).min(1.0);
            score += tf;
        }
    }

    // Coverage bonus: reward matching more query terms
    let coverage = terms_found as f64 / query_terms.len() as f64;
    score * (0.5 + 0.5 * coverage)
}

/// Freshness decay: exponential decay over hours.
/// Returns 1.0 for fresh content, decaying toward 0.1 over ttl_hours.
fn freshness_weight(hours_old: f64, ttl_hours: f64) -> f64 {
    if ttl_hours <= 0.0 {
        return 1.0;
    }
    let decay = (-2.3 * hours_old / ttl_hours).exp(); // e^(-2.3) ≈ 0.1 at TTL boundary
    0.1 + 0.9 * decay
}

/// Source weight multiplier (brain artifacts > ingested > filesystem).
fn source_weight(source: &str) -> f64 {
    match source {
        "brain" => 1.5,
        "ingested" => 1.3,
        "implicit" => 1.2,
        "qdrant" => 1.1,
        "filesystem" => 1.0,
        _ => 1.0,
    }
}

// ============================================================================
// Multi-Source Search
// ============================================================================

/// Search brain artifacts (sessions and their artifacts).
fn search_brain(query_terms: &[String], limit: usize) -> Vec<serde_json::Value> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    let sessions_dir = PathBuf::from(format!("{home}/.claude/brain/sessions"));
    if !sessions_dir.exists() {
        return Vec::new();
    }

    let mut results = Vec::new();
    let entries = match std::fs::read_dir(&sessions_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.flatten() {
        let session_path = entry.path();
        if !session_path.is_dir() {
            continue;
        }

        let session_id = session_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Search artifacts within session
        let artifacts = match std::fs::read_dir(&session_path) {
            Ok(e) => e,
            Err(_) => continue,
        };

        for artifact_entry in artifacts.flatten() {
            let artifact_path = artifact_entry.path();
            if !artifact_path.is_file() {
                continue;
            }

            let name = artifact_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            // Skip resolved versions and non-text files
            if name.contains(".resolved.") || name.starts_with('.') {
                continue;
            }

            let content = match std::fs::read_to_string(&artifact_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let score = relevance_score(&content, query_terms);
            if score <= 0.0 {
                continue;
            }

            // Name bonus
            let name_lower = name.to_lowercase();
            let name_bonus: f64 = query_terms
                .iter()
                .filter(|t| name_lower.contains(t.as_str()))
                .count() as f64
                * 0.5;

            let modified_hours = std::fs::metadata(&artifact_path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    t.elapsed()
                        .map(|d| d.as_secs_f64() / 3600.0)
                        .unwrap_or(999.0)
                })
                .unwrap_or(999.0);

            let freshness = freshness_weight(modified_hours, 168.0); // 7-day TTL for brain artifacts
            let composite = (score + name_bonus) * freshness * source_weight("brain");

            let preview = content.chars().take(200).collect::<String>();

            results.push(json!({
                "source": "brain",
                "source_id": format!("{session_id}/{name}"),
                "title": name,
                "relevance": (score * 1000.0).round() / 1000.0,
                "freshness": (freshness * 100.0).round() / 100.0,
                "composite_score": (composite * 1000.0).round() / 1000.0,
                "age_hours": (modified_hours * 10.0).round() / 10.0,
                "preview": preview,
            }));
        }
    }

    results.sort_by(|a, b| {
        let sa = a
            .get("composite_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let sb = b
            .get("composite_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        sb.total_cmp(&sa)
    });
    results.truncate(limit);
    results
}

/// Search the ingested retrieval store entries.
fn search_ingested(
    store: &RetrievalStore,
    query_terms: &[String],
    domain: Option<&str>,
    limit: usize,
) -> Vec<serde_json::Value> {
    let now_hours = 0.0_f64; // entries track their own age

    let mut results: Vec<serde_json::Value> = store
        .entries
        .iter()
        .filter(|e| {
            // Domain filter
            if let Some(d) = domain {
                if !e.tags.iter().any(|t| t == d) {
                    return false;
                }
            }
            true
        })
        .filter_map(|e| {
            let score = relevance_score(&e.content, query_terms);
            if score <= 0.0 {
                return None;
            }

            // Title bonus
            let title_bonus: f64 = query_terms
                .iter()
                .filter(|t| e.title.to_lowercase().contains(t.as_str()))
                .count() as f64
                * 0.5;

            let age = hours_since(&e.ingested_at);
            let freshness = freshness_weight(age, e.ttl_hours as f64);
            let composite = (score + title_bonus) * freshness * source_weight("ingested");

            let preview = e.content.chars().take(200).collect::<String>();

            Some(json!({
                "source": "ingested",
                "source_id": e.source_id,
                "title": e.title,
                "tags": e.tags,
                "relevance": (score * 1000.0).round() / 1000.0,
                "freshness": (freshness * 100.0).round() / 100.0,
                "composite_score": (composite * 1000.0).round() / 1000.0,
                "age_hours": (age * 10.0).round() / 10.0,
                "access_count": e.access_count,
                "preview": preview,
            }))
        })
        .collect();

    results.sort_by(|a, b| {
        let sa = a
            .get("composite_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let sb = b
            .get("composite_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        sb.total_cmp(&sa)
    });
    results.truncate(limit);
    results
}

/// Search implicit knowledge (preferences, patterns, corrections).
fn search_implicit(query_terms: &[String], limit: usize) -> Vec<serde_json::Value> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    let implicit_dir = PathBuf::from(format!("{home}/.claude/implicit"));
    if !implicit_dir.exists() {
        return Vec::new();
    }

    let files = ["preferences.json", "patterns.json", "corrections.json"];
    let mut results = Vec::new();

    for filename in &files {
        let path = implicit_dir.join(filename);
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Parse as JSON and search through entries
        let data: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let category = filename.trim_end_matches(".json");

        // Handle both array and object structures
        let entries: Vec<(&str, &serde_json::Value)> = if let Some(obj) = data.as_object() {
            obj.iter().map(|(k, v)| (k.as_str(), v)).collect()
        } else if let Some(arr) = data.as_array() {
            arr.iter().enumerate().map(|(i, v)| ("", v)).collect()
        } else {
            continue;
        };

        for (_key, value) in entries {
            let entry_str = value.to_string();
            let score = relevance_score(&entry_str, query_terms);
            if score <= 0.0 {
                continue;
            }

            let composite = score * source_weight("implicit");
            let preview = entry_str.chars().take(200).collect::<String>();

            results.push(json!({
                "source": "implicit",
                "source_id": format!("implicit/{category}"),
                "title": format!("{category} entry"),
                "relevance": (score * 1000.0).round() / 1000.0,
                "freshness": 1.0,
                "composite_score": (composite * 1000.0).round() / 1000.0,
                "preview": preview,
            }));
        }
    }

    results.sort_by(|a, b| {
        let sa = a
            .get("composite_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let sb = b
            .get("composite_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        sb.total_cmp(&sa)
    });
    results.truncate(limit);
    results
}

/// Search filesystem KSB (knowledge, skills, brain) — reuses vigil pattern.
fn search_filesystem(query_terms: &[String], limit: usize) -> Vec<serde_json::Value> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home".to_string());
    let search_roots = [
        format!("{home}/.claude/knowledge"),
        format!("{home}/.claude/skills"),
    ];

    let mut results = Vec::new();

    for root in &search_roots {
        let root_path = std::path::Path::new(root);
        if !root_path.exists() {
            continue;
        }
        collect_fs_matches(root_path, query_terms, &mut results, 3);
    }

    results.sort_by(|a, b| {
        let sa = a
            .get("composite_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let sb = b
            .get("composite_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        sb.total_cmp(&sa)
    });
    results.truncate(limit);
    results
}

fn collect_fs_matches(
    dir: &std::path::Path,
    query_terms: &[String],
    results: &mut Vec<serde_json::Value>,
    max_depth: usize,
) {
    if max_depth == 0 {
        return;
    }

    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if name.starts_with('.') {
            continue;
        }

        if path.is_dir() {
            collect_fs_matches(&path, query_terms, results, max_depth - 1);
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(
            ext,
            "md" | "yaml" | "yml" | "toml" | "json" | "txt" | "rs" | "sh"
        ) {
            continue;
        }

        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let score = relevance_score(&content, query_terms);
        if score <= 0.0 {
            continue;
        }

        let name_bonus: f64 = query_terms
            .iter()
            .filter(|t| name.to_lowercase().contains(t.as_str()))
            .count() as f64
            * 0.5;

        let modified_hours = std::fs::metadata(&path)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                t.elapsed()
                    .map(|d| d.as_secs_f64() / 3600.0)
                    .unwrap_or(999.0)
            })
            .unwrap_or(999.0);

        let freshness = freshness_weight(modified_hours, 720.0); // 30-day TTL for filesystem
        let composite = (score + name_bonus) * freshness * source_weight("filesystem");

        let preview = content.chars().take(200).collect::<String>();
        let path_str = path.to_string_lossy().to_string();

        results.push(json!({
            "source": "filesystem",
            "source_id": path_str,
            "title": name,
            "relevance": (score * 1000.0).round() / 1000.0,
            "freshness": (freshness * 100.0).round() / 100.0,
            "composite_score": (composite * 1000.0).round() / 1000.0,
            "age_hours": (modified_hours * 10.0).round() / 10.0,
            "preview": preview,
        }));
    }
}

// ============================================================================
// Tool Implementations
// ============================================================================

/// `retrieval_query` — Unified multi-source retrieval with caching and ranking.
///
/// Searches brain artifacts, ingested documents, implicit knowledge, and filesystem KSB.
/// Results are ranked by composite score: relevance × freshness × source_weight.
/// Recent queries are cached with configurable TTL.
pub fn retrieval_query(params: RetrievalQueryParams) -> Result<CallToolResult, McpError> {
    let start = std::time::Instant::now();
    let mut store = load_store();
    let limit = params.limit.unwrap_or(10);
    let min_relevance = params.min_relevance.unwrap_or(0.0);
    let use_cache = params.use_cache.unwrap_or(true);
    let source_filter = params.source.as_deref().unwrap_or("all");
    let domain = params.domain.as_deref();

    // 1. Check cache
    if use_cache {
        // Find cache hit and clone results to release the mutable borrow
        let cache_hit = store.cache.iter_mut().find_map(|c| {
            if c.query != params.query {
                return None;
            }
            let age_secs = hours_since(&c.created_at) * 3600.0;
            if age_secs >= c.ttl_seconds as f64 {
                return None;
            }
            c.hit_count += 1;
            Some((c.results.clone(), c.results.len()))
        });

        if let Some((cached_results, result_count)) = cache_hit {
            store.metrics.total_queries += 1;
            store.metrics.cache_hits += 1;

            let latency_ms = start.elapsed().as_millis() as u64;
            store.metrics.query_log.push(QueryLogEntry {
                query: params.query.clone(),
                timestamp: now_iso(),
                result_count,
                sources_searched: vec!["cache".to_string()],
                cache_hit: true,
                latency_ms,
            });

            // Trim query log
            if store.metrics.query_log.len() > 200 {
                let drain_count = store.metrics.query_log.len() - 200;
                store.metrics.query_log.drain(..drain_count);
            }

            let _ = save_store(&store);

            let result = json!({
                "status": "success",
                "source": "cache",
                "query": params.query,
                "cache_hit": true,
                "result_count": result_count,
                "latency_ms": latency_ms,
                "results": cached_results,
            });

            return Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
            )]));
        }
    }

    // 2. Tokenize query
    let query_terms: Vec<String> = params
        .query
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| s.len() >= 2)
        .map(|s| s.to_lowercase())
        .collect();

    if query_terms.is_empty() {
        return Err(McpError::invalid_params(
            "Query must contain at least one term (2+ chars)".to_string(),
            None,
        ));
    }

    // 3. Search each source
    let mut all_results: Vec<serde_json::Value> = Vec::new();
    let mut sources_searched = Vec::new();

    if source_filter == "all" || source_filter == "brain" {
        sources_searched.push("brain".to_string());
        let brain_results = search_brain(&query_terms, limit * 2);
        *store
            .metrics
            .source_hits
            .entry("brain".to_string())
            .or_insert(0) += brain_results.len() as u64;
        all_results.extend(brain_results);
    }

    if source_filter == "all" || source_filter == "ingested" {
        sources_searched.push("ingested".to_string());
        let ingested_results = search_ingested(&store, &query_terms, domain, limit * 2);
        *store
            .metrics
            .source_hits
            .entry("ingested".to_string())
            .or_insert(0) += ingested_results.len() as u64;
        all_results.extend(ingested_results);
    }

    if source_filter == "all" || source_filter == "implicit" {
        sources_searched.push("implicit".to_string());
        let implicit_results = search_implicit(&query_terms, limit * 2);
        *store
            .metrics
            .source_hits
            .entry("implicit".to_string())
            .or_insert(0) += implicit_results.len() as u64;
        all_results.extend(implicit_results);
    }

    if source_filter == "all" || source_filter == "filesystem" {
        sources_searched.push("filesystem".to_string());
        let fs_results = search_filesystem(&query_terms, limit * 2);
        *store
            .metrics
            .source_hits
            .entry("filesystem".to_string())
            .or_insert(0) += fs_results.len() as u64;
        all_results.extend(fs_results);
    }

    // 4. Filter by min_relevance
    if min_relevance > 0.0 {
        all_results.retain(|r| {
            r.get("composite_score")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0)
                >= min_relevance
        });
    }

    // 5. Sort by composite score and deduplicate by source_id
    all_results.sort_by(|a, b| {
        let sa = a
            .get("composite_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let sb = b
            .get("composite_score")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        sb.total_cmp(&sa)
    });

    // Deduplicate by source_id (keep highest-scored)
    let mut seen_sources = std::collections::HashSet::new();
    all_results.retain(|r| {
        let source_id = r.get("source_id").and_then(|v| v.as_str()).unwrap_or("");
        seen_sources.insert(source_id.to_string())
    });

    all_results.truncate(limit);

    // 6. Update access counts for ingested entries
    for result in &all_results {
        if result.get("source").and_then(|s| s.as_str()) == Some("ingested") {
            if let Some(sid) = result.get("source_id").and_then(|v| v.as_str()) {
                if let Some(entry) = store.entries.iter_mut().find(|e| e.source_id == sid) {
                    entry.access_count += 1;
                    entry.last_accessed = now_iso();
                }
            }
        }
    }

    // 7. Cache results
    if use_cache {
        // Remove stale cache entries
        store
            .cache
            .retain(|c| hours_since(&c.created_at) * 3600.0 < c.ttl_seconds as f64);

        // Cap cache size
        if store.cache.len() >= 50 {
            store.cache.remove(0);
        }

        store.cache.push(CacheEntry {
            query: params.query.clone(),
            results: all_results.clone(),
            created_at: now_iso(),
            ttl_seconds: 300,
            hit_count: 0,
        });
    }

    // 8. Update metrics
    let latency_ms = start.elapsed().as_millis() as u64;
    store.metrics.total_queries += 1;
    store.metrics.cache_misses += 1;
    store.metrics.query_log.push(QueryLogEntry {
        query: params.query.clone(),
        timestamp: now_iso(),
        result_count: all_results.len(),
        sources_searched: sources_searched.clone(),
        cache_hit: false,
        latency_ms,
    });

    if store.metrics.query_log.len() > 200 {
        let drain_count = store.metrics.query_log.len() - 200;
        store.metrics.query_log.drain(..drain_count);
    }

    let _ = save_store(&store);

    // 9. Source distribution summary
    let mut source_counts: HashMap<String, usize> = HashMap::new();
    for r in &all_results {
        if let Some(s) = r.get("source").and_then(|v| v.as_str()) {
            *source_counts.entry(s.to_string()).or_insert(0) += 1;
        }
    }

    let result = json!({
        "status": "success",
        "query": params.query,
        "query_terms": query_terms,
        "cache_hit": false,
        "sources_searched": sources_searched,
        "source_distribution": source_counts,
        "result_count": all_results.len(),
        "latency_ms": latency_ms,
        "results": all_results,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `retrieval_ingest` — Index content into the persistent retrieval store.
///
/// Stores content with metadata (tags, TTL, source) for later retrieval.
/// Deduplicates by source_id (updates existing entries).
pub fn retrieval_ingest(params: RetrievalIngestParams) -> Result<CallToolResult, McpError> {
    let mut store = load_store();

    let ttl = params.ttl_hours.unwrap_or(168); // 7 days default
    let title = params.title.unwrap_or_else(|| params.source_id.clone());
    let now = now_iso();

    // Check for existing entry with same source_id
    if let Some(existing) = store
        .entries
        .iter_mut()
        .find(|e| e.source_id == params.source_id)
    {
        existing.content = params.content.clone();
        existing.title = title.clone();
        existing.tags = params.tags.clone();
        existing.ttl_hours = ttl;
        existing.ingested_at = now.clone();

        save_store(&store)?;

        let result = json!({
            "status": "updated",
            "source_id": params.source_id,
            "title": title,
            "tags": params.tags,
            "ttl_hours": ttl,
            "total_entries": store.entries.len(),
        });

        return Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]));
    }

    // New entry
    store.entries.push(RetrievalEntry {
        source_id: params.source_id.clone(),
        title: title.clone(),
        content: params.content,
        tags: params.tags.clone(),
        ingested_at: now.clone(),
        ttl_hours: ttl,
        access_count: 0,
        last_accessed: now,
    });

    // Evict stale entries
    store
        .entries
        .retain(|e| hours_since(&e.ingested_at) < e.ttl_hours as f64);

    save_store(&store)?;

    let result = json!({
        "status": "ingested",
        "source_id": params.source_id,
        "title": title,
        "tags": params.tags,
        "ttl_hours": ttl,
        "total_entries": store.entries.len(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// `retrieval_stats` — Pipeline health, cache metrics, and query analytics.
pub fn retrieval_stats(params: RetrievalStatsParams) -> Result<CallToolResult, McpError> {
    let store = load_store();
    let include_top_queries = params.include_top_queries.unwrap_or(true);
    let include_freshness = params.include_freshness.unwrap_or(true);

    // Cache stats
    let active_cache = store
        .cache
        .iter()
        .filter(|c| hours_since(&c.created_at) * 3600.0 < c.ttl_seconds as f64)
        .count();
    let total_cache_hits: u64 = store.cache.iter().map(|c| c.hit_count).sum();

    let hit_rate = if store.metrics.total_queries > 0 {
        store.metrics.cache_hits as f64 / store.metrics.total_queries as f64
    } else {
        0.0
    };

    // Freshness breakdown
    let freshness = if include_freshness {
        let mut fresh = 0_usize; // < 1 hour
        let mut recent = 0_usize; // < 24 hours
        let mut moderate = 0_usize; // < 7 days
        let mut stale = 0_usize; // >= 7 days

        for entry in &store.entries {
            let age = hours_since(&entry.ingested_at);
            if age < 1.0 {
                fresh += 1;
            } else if age < 24.0 {
                recent += 1;
            } else if age < 168.0 {
                moderate += 1;
            } else {
                stale += 1;
            }
        }

        Some(json!({
            "fresh_lt_1h": fresh,
            "recent_lt_24h": recent,
            "moderate_lt_7d": moderate,
            "stale_gte_7d": stale,
        }))
    } else {
        None
    };

    // Top queries by frequency
    let top_queries = if include_top_queries {
        let mut query_freq: HashMap<String, usize> = HashMap::new();
        for log in &store.metrics.query_log {
            *query_freq.entry(log.query.clone()).or_insert(0) += 1;
        }
        let mut sorted: Vec<_> = query_freq.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        sorted.truncate(10);
        Some(
            sorted
                .into_iter()
                .map(|(q, count)| json!({"query": q, "count": count}))
                .collect::<Vec<_>>(),
        )
    } else {
        None
    };

    // Average latency
    let avg_latency = if store.metrics.query_log.is_empty() {
        0.0
    } else {
        let total: u64 = store.metrics.query_log.iter().map(|l| l.latency_ms).sum();
        total as f64 / store.metrics.query_log.len() as f64
    };

    let result = json!({
        "store": {
            "total_entries": store.entries.len(),
            "total_tags": store.entries.iter().flat_map(|e| &e.tags).collect::<std::collections::HashSet<_>>().len(),
        },
        "cache": {
            "active_entries": active_cache,
            "total_entries": store.cache.len(),
            "total_cache_hits": total_cache_hits,
        },
        "metrics": {
            "total_queries": store.metrics.total_queries,
            "cache_hits": store.metrics.cache_hits,
            "cache_misses": store.metrics.cache_misses,
            "hit_rate_pct": (hit_rate * 10000.0).round() / 100.0,
            "avg_latency_ms": (avg_latency * 10.0).round() / 10.0,
            "source_hits": store.metrics.source_hits,
        },
        "freshness": freshness,
        "top_queries": top_queries,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}
