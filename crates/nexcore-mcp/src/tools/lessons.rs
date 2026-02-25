//! Lessons Learned — persistent lesson storage with primitive extraction.
//!
//! Consolidated from `lessons-mcp` satellite MCP server.
//! 6 tools: lesson_add, lesson_get, lesson_search, lesson_by_context, lesson_by_tag, primitives_summary.
//!
//! Tier: T3 (π Persistence + μ Mapping + σ Sequence + ∃ Existence)

use crate::params::{
    LessonAddParams, LessonByContextParams, LessonByTagParams, LessonGetParams, LessonSearchParams,
};
use nexcore_chrono::DateTime;
use nexcore_fs::dirs;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

// ============================================================================
// Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum PrimitiveTier {
    T1,
    T2P,
    T2C,
    T3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExtractedPrimitive {
    name: String,
    tier: PrimitiveTier,
    description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Lesson {
    id: u64,
    title: String,
    content: String,
    context: String,
    tags: Vec<String>,
    primitives: Vec<ExtractedPrimitive>,
    created_at: DateTime,
    #[serde(default)]
    source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct LessonsDb {
    lessons: Vec<Lesson>,
    next_id: u64,
}

// ============================================================================
// Storage
// ============================================================================

static DB_LOCK: Mutex<()> = Mutex::new(());

fn data_path() -> PathBuf {
    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("nexvigilant")
        .join("lessons-mcp");
    fs::create_dir_all(&data_dir).ok();
    data_dir.join("lessons.json")
}

fn load_db() -> LessonsDb {
    let path = data_path();
    if !path.exists() {
        return LessonsDb::default();
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_db(db: &LessonsDb) {
    if let Ok(content) = serde_json::to_string_pretty(db) {
        fs::write(data_path(), content).ok();
    }
}

// ============================================================================
// Primitive Extraction (keyword heuristic)
// ============================================================================

fn suggest_primitives(text: &str) -> Vec<ExtractedPrimitive> {
    let lower = text.to_lowercase();
    let mut prims = Vec::new();

    // T1
    if lower.contains("sequence") || lower.contains("iterate") || lower.contains("loop") {
        prims.push(prim("Sequence (σ)", PrimitiveTier::T1, "Ordered iteration"));
    }
    if lower.contains("map") || lower.contains("lookup") || lower.contains("key") {
        prims.push(prim(
            "Mapping (μ)",
            PrimitiveTier::T1,
            "Key-value association",
        ));
    }
    if lower.contains("state") || lower.contains("mutate") || lower.contains("update") {
        prims.push(prim("State (ς)", PrimitiveTier::T1, "State mutation"));
    }
    if lower.contains("recursive") || lower.contains("tree") || lower.contains("traverse") {
        prims.push(prim(
            "Recursion (ρ)",
            PrimitiveTier::T1,
            "Recursive structure",
        ));
    }
    if lower.contains("filter") || lower.contains("guard") || lower.contains("validate") {
        prims.push(prim(
            "Existence (∃)",
            PrimitiveTier::T1,
            "Validation/filtering",
        ));
    }

    // T2-P
    if lower.contains("exit code") || lower.contains("allow") || lower.contains("block") {
        prims.push(prim(
            "DecisionGate",
            PrimitiveTier::T2P,
            "Hook decision pattern",
        ));
    }
    if lower.contains("timeout") || lower.contains("limit") || lower.contains("threshold") {
        prims.push(prim("Threshold", PrimitiveTier::T2P, "Limit enforcement"));
    }
    if lower.contains("transform") || lower.contains("convert") || lower.contains("parse") {
        prims.push(prim("Transform", PrimitiveTier::T2P, "Data transformation"));
    }

    // T2-C
    if lower.contains("pipeline") || lower.contains("chain") || lower.contains("compose") {
        prims.push(prim("Pipeline", PrimitiveTier::T2C, "Composable pipeline"));
    }

    // T3
    if lower.contains("pretooluse") || lower.contains("posttooluse") {
        prims.push(prim(
            "ToolInterceptor",
            PrimitiveTier::T3,
            "Tool lifecycle interception",
        ));
    }
    if lower.contains("sessionstart") || lower.contains("stop") {
        prims.push(prim(
            "SessionLifecycle",
            PrimitiveTier::T3,
            "Session boundary handling",
        ));
    }

    prims
}

fn prim(name: &str, tier: PrimitiveTier, desc: &str) -> ExtractedPrimitive {
    ExtractedPrimitive {
        name: name.into(),
        tier,
        description: desc.into(),
    }
}

// ============================================================================
// Tools
// ============================================================================

/// Add a new lesson with automatic primitive extraction.
pub fn lesson_add(params: LessonAddParams) -> Result<CallToolResult, McpError> {
    let _lock = DB_LOCK
        .lock()
        .map_err(|e| McpError::internal_error(format!("DB lock failed: {e}"), None))?;
    let mut db = load_db();

    let combined = format!("{} {}", params.title, params.content);
    let primitives = suggest_primitives(&combined);

    let lesson = Lesson {
        id: 0, // set by add()
        title: params.title,
        content: params.content,
        context: params.context,
        tags: params.tags.unwrap_or_default(),
        primitives,
        created_at: DateTime::now(),
        source: params.source.unwrap_or_default(),
    };

    let id = db.next_id;
    db.next_id += 1;
    let mut lesson = lesson;
    lesson.id = id;
    db.lessons.push(lesson);
    save_db(&db);

    Ok(CallToolResult::success(vec![Content::text(
        json!({"id": id, "message": format!("Lesson #{id} added")}).to_string(),
    )]))
}

/// Get a lesson by ID.
pub fn lesson_get(params: LessonGetParams) -> Result<CallToolResult, McpError> {
    let db = load_db();
    match db.lessons.iter().find(|l| l.id == params.id) {
        Some(lesson) => Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(lesson).unwrap_or_default(),
        )])),
        None => Ok(CallToolResult::success(vec![Content::text(format!(
            "Lesson #{} not found",
            params.id
        ))])),
    }
}

/// Search lessons by query string.
pub fn lesson_search(params: LessonSearchParams) -> Result<CallToolResult, McpError> {
    let db = load_db();
    let q = params.query.to_lowercase();
    let results: Vec<_> = db
        .lessons
        .iter()
        .filter(|l| l.title.to_lowercase().contains(&q) || l.content.to_lowercase().contains(&q))
        .collect();
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&results).unwrap_or_default(),
    )]))
}

/// Filter lessons by context.
pub fn lesson_by_context(params: LessonByContextParams) -> Result<CallToolResult, McpError> {
    let db = load_db();
    let results: Vec<_> = db
        .lessons
        .iter()
        .filter(|l| l.context.eq_ignore_ascii_case(&params.context))
        .collect();
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&results).unwrap_or_default(),
    )]))
}

/// Filter lessons by tag.
pub fn lesson_by_tag(params: LessonByTagParams) -> Result<CallToolResult, McpError> {
    let db = load_db();
    let t = params.tag.to_lowercase();
    let results: Vec<_> = db
        .lessons
        .iter()
        .filter(|l| l.tags.iter().any(|tag| tag.to_lowercase() == t))
        .collect();
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&results).unwrap_or_default(),
    )]))
}

/// Get summary of extracted primitives.
pub fn primitives_summary() -> Result<CallToolResult, McpError> {
    let db = load_db();
    let mut map: HashMap<String, (PrimitiveTier, usize)> = HashMap::new();
    for p in db.lessons.iter().flat_map(|l| &l.primitives) {
        map.entry(p.name.clone())
            .and_modify(|(_, c)| *c += 1)
            .or_insert((p.tier.clone(), 1));
    }

    let primitives: Vec<_> = map
        .into_iter()
        .map(|(name, (tier, count))| json!({"name": name, "tier": tier, "count": count}))
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({"primitives": primitives}).to_string(),
    )]))
}
