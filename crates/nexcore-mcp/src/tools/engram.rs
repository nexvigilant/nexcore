//! Engram knowledge store MCP tools.
//!
//! Unified knowledge daemon — persistent memory with semantic search,
//! temporal decay, and multi-source ingestion. Consolidates MEMORY.md,
//! Brain artifacts, Lessons, and Implicit learning into a single
//! searchable store.

use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};

use nexcore_engram::prelude::*;
use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use crate::params::engram::{
    EngramBySourceParams, EngramDecayScoreParams, EngramFindDuplicatesParams, EngramIngestParams,
    EngramPeekParams, EngramSearchDecayParams, EngramSearchParams, EngramStatsParams,
};

// ── Store singleton ──────────────────────────────────────────────────────

static STORE: OnceLock<RwLock<EngramStore>> = OnceLock::new();

fn store_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".local/share/nexcore/engrams.json")
}

fn store() -> Result<&'static RwLock<EngramStore>, McpError> {
    Ok(STORE.get_or_init(|| {
        let path = store_path();
        let s = if path.exists() {
            EngramStore::load(&path).unwrap_or_else(|_| EngramStore::new())
        } else {
            EngramStore::new()
        };
        RwLock::new(s)
    }))
}

fn internal_err(msg: String) -> McpError {
    McpError {
        code: rmcp::model::ErrorCode::INTERNAL_ERROR,
        message: msg.into(),
        data: None,
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

fn ok_json(value: serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| format!("{value}")),
    )]))
}

fn err_result(msg: &str) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::error(vec![rmcp::model::Content::text(
        msg.to_string(),
    )]))
}

fn engram_to_json(e: &Engram) -> serde_json::Value {
    json!({
        "id": e.id,
        "title": e.title,
        "content": e.content,
        "source": format!("{:?}", e.source),
        "tags": e.tags,
        "created_at": e.created_at.to_rfc3339(),
        "last_accessed": e.last_accessed.to_rfc3339(),
        "access_count": e.access_count,
        "relevance": e.relevance,
    })
}

fn parse_source(s: &str) -> Option<EngramSource> {
    match s.to_lowercase().as_str() {
        "memory" => Some(EngramSource::Memory),
        "brain" => Some(EngramSource::Brain),
        "lesson" => Some(EngramSource::Lesson),
        "implicit" => Some(EngramSource::Implicit),
        "session" => Some(EngramSource::Session),
        _ => None,
    }
}

// ── Tools ────────────────────────────────────────────────────────────────

/// Search the engram store using TF-IDF ranking.
pub fn engram_search(p: EngramSearchParams) -> Result<CallToolResult, McpError> {
    let lock = store()?;
    let guard = lock
        .read()
        .map_err(|e| internal_err(["Store lock poisoned: ", &e.to_string()].concat()))?;
    let results = guard.search(&p.query);
    let limit = p.limit.unwrap_or(10);
    let items: Vec<serde_json::Value> = results
        .iter()
        .take(limit)
        .filter_map(|r| guard.peek(r.id).map(|e| (r, e)))
        .map(|(r, e)| {
            json!({
                "id": r.id,
                "score": r.score,
                "title": e.title,
                "source": format!("{:?}", e.source),
                "tags": e.tags,
                "content_preview": if e.content.len() > 200 { &e.content[..200] } else { &e.content },
            })
        })
        .collect();

    ok_json(json!({
        "query": p.query,
        "total_matches": results.len(),
        "returned": items.len(),
        "results": items,
    }))
}

/// Search with temporal decay — recent knowledge ranks higher.
pub fn engram_search_decay(p: EngramSearchDecayParams) -> Result<CallToolResult, McpError> {
    let lock = store()?;
    let guard = lock
        .read()
        .map_err(|e| internal_err(["Store lock poisoned: ", &e.to_string()].concat()))?;
    let results = guard.search_with_decay(&p.query);
    let limit = p.limit.unwrap_or(10);
    let items: Vec<serde_json::Value> = results
        .iter()
        .take(limit)
        .filter_map(|r| guard.peek(r.id).map(|e| (r, e)))
        .map(|(r, e)| {
            json!({
                "id": r.id,
                "score": r.score,
                "title": e.title,
                "source": format!("{:?}", e.source),
                "tags": e.tags,
            })
        })
        .collect();

    ok_json(json!({
        "query": p.query,
        "total_matches": results.len(),
        "returned": items.len(),
        "results": items,
    }))
}

/// Get a specific engram by ID (read-only, no access tracking).
pub fn engram_peek(p: EngramPeekParams) -> Result<CallToolResult, McpError> {
    let lock = store()?;
    let guard = lock
        .read()
        .map_err(|e| internal_err(["Store lock poisoned: ", &e.to_string()].concat()))?;
    match guard.peek(p.id) {
        Some(e) => ok_json(json!({
            "found": true,
            "engram": engram_to_json(e),
        })),
        None => ok_json(json!({
            "found": false,
            "id": p.id,
        })),
    }
}

/// Get store statistics (total, active, stale counts by layer).
pub fn engram_stats(_p: EngramStatsParams) -> Result<CallToolResult, McpError> {
    let lock = store()?;
    let guard = lock
        .read()
        .map_err(|e| internal_err(["Store lock poisoned: ", &e.to_string()].concat()))?;
    let stats = guard.stats();
    ok_json(json!({
        "total": stats.total,
        "active": stats.active,
        "stale": stats.stale,
        "by_source": {
            "memory": stats.memory_count,
            "brain": stats.brain_count,
            "lesson": stats.lesson_count,
            "implicit": stats.implicit_count,
            "session": stats.session_count,
        },
    }))
}

/// Find near-duplicate engrams by content similarity.
pub fn engram_find_duplicates(p: EngramFindDuplicatesParams) -> Result<CallToolResult, McpError> {
    let lock = store()?;
    let guard = lock
        .read()
        .map_err(|e| internal_err(["Store lock poisoned: ", &e.to_string()].concat()))?;
    let threshold = p.threshold.unwrap_or(0.3);
    let dupes = guard.find_duplicates(threshold);
    let items: Vec<serde_json::Value> = dupes
        .iter()
        .take(50)
        .map(|d| {
            json!({
                "id_a": d.id_a,
                "id_b": d.id_b,
                "similarity": d.similarity,
                "title_a": guard.peek(d.id_a).map(|e| e.title.as_str()),
                "title_b": guard.peek(d.id_b).map(|e| e.title.as_str()),
            })
        })
        .collect();

    ok_json(json!({
        "threshold": threshold,
        "duplicate_pairs": items.len(),
        "results": items,
    }))
}

/// Compute temporal decay score for an engram (pure computation, no store needed).
pub fn engram_decay_score(p: EngramDecayScoreParams) -> Result<CallToolResult, McpError> {
    let created_at = chrono::DateTime::parse_from_rfc3339(&p.created_at)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| internal_err(["Invalid created_at: ", &e.to_string()].concat()))?;
    let last_accessed = chrono::DateTime::parse_from_rfc3339(&p.last_accessed)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| internal_err(["Invalid last_accessed: ", &e.to_string()].concat()))?;

    let engram = Engram {
        id: 0,
        title: String::new(),
        content: String::new(),
        source: EngramSource::Session,
        tags: Vec::new(),
        created_at,
        last_accessed,
        access_count: p.access_count,
        relevance: 1.0,
    };
    let config = DecayConfig {
        half_life_days: p.half_life_days.unwrap_or(14.0),
        stale_threshold: p.stale_threshold.unwrap_or(0.1),
        ..DecayConfig::default()
    };
    let now = chrono::Utc::now();
    let score = decay_score(&engram, now, &config);
    let stale = is_stale(&engram, now, &config);

    ok_json(json!({
        "decay_score": score,
        "is_stale": stale,
        "half_life_days": config.half_life_days,
        "stale_threshold": config.stale_threshold,
        "age_days": (now - created_at).num_hours() as f64 / 24.0,
    }))
}

/// Ingest knowledge from a source file/directory into the store.
pub fn engram_ingest(p: EngramIngestParams) -> Result<CallToolResult, McpError> {
    let lock = store()?;
    let mut guard = lock
        .write()
        .map_err(|e| internal_err(["Store write lock poisoned: ", &e.to_string()].concat()))?;
    let path = Path::new(&p.path);

    let count = match p.source_type.as_str() {
        "memory_md" => nexcore_engram::ingest::ingest_memory_md(&mut guard, path),
        "brain_dir" => nexcore_engram::ingest::ingest_brain_dir(&mut guard, path),
        "lessons_jsonl" => nexcore_engram::ingest::ingest_lessons_jsonl(&mut guard, path),
        "implicit_json" => nexcore_engram::ingest::ingest_implicit_json(&mut guard, path),
        other => {
            return err_result(
                &[
                    "Unknown source type '",
                    other,
                    "'. Use: memory_md, brain_dir, lessons_jsonl, implicit_json",
                ]
                .concat(),
            );
        }
    };

    match count {
        Ok(n) => {
            // Auto-save after successful ingest
            let save_path = store_path();
            if let Some(parent) = save_path.parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    internal_err(["Save dir creation failed: ", &e.to_string()].concat())
                })?;
            }
            let saved = guard.save(&save_path).is_ok();
            ok_json(json!({
                "source_type": p.source_type,
                "path": p.path,
                "ingested": n,
                "total_engrams": guard.len(),
                "persisted": saved,
            }))
        }
        Err(e) => err_result(&["Ingest failed: ", &e.to_string()].concat()),
    }
}

/// List engrams filtered by source layer.
pub fn engram_by_source(p: EngramBySourceParams) -> Result<CallToolResult, McpError> {
    let source = match parse_source(&p.source) {
        Some(s) => s,
        None => {
            return err_result(
                &[
                    "Unknown source '",
                    &p.source,
                    "'. Use: memory, brain, lesson, implicit, session",
                ]
                .concat(),
            );
        }
    };
    let lock = store()?;
    let guard = lock
        .read()
        .map_err(|e| internal_err(["Store lock poisoned: ", &e.to_string()].concat()))?;
    let engrams = guard.by_source(&source);
    let items: Vec<serde_json::Value> = engrams
        .iter()
        .take(50)
        .map(|e| {
            json!({
                "id": e.id,
                "title": e.title,
                "tags": e.tags,
                "access_count": e.access_count,
                "created_at": e.created_at.to_rfc3339(),
            })
        })
        .collect();

    ok_json(json!({
        "source": p.source,
        "total": engrams.len(),
        "returned": items.len(),
        "results": items,
    }))
}
