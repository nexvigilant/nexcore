//! MCP tool definitions and handlers
//! Tier: T2-C (composes storage, extraction)

use crate::{extract, models, protocol::Response, storage};
use nexcore_chrono::DateTime;
use serde_json::{Value, json};

// ─────────────────────────────────────────────────────────────────────────────
// Tool Schema Builders (keep each small)
// ─────────────────────────────────────────────────────────────────────────────

fn schema_lesson_add() -> Value {
    json!({
        "name": "lesson_add",
        "description": "Add a new lesson with automatic primitive extraction",
        "inputSchema": input_schema_lesson_add()
    })
}

fn input_schema_lesson_add() -> Value {
    json!({
        "type": "object",
        "properties": {
            "title": { "type": "string", "description": "Lesson title" },
            "content": { "type": "string", "description": "Lesson content" },
            "context": { "type": "string", "description": "Context (hooks, skills, mcp)" },
            "tags": { "type": "array", "items": { "type": "string" } },
            "source": { "type": "string", "description": "Source of the lesson" }
        },
        "required": ["title", "content", "context"]
    })
}

fn schema_lesson_get() -> Value {
    json!({
        "name": "lesson_get",
        "description": "Get a lesson by ID",
        "inputSchema": {
            "type": "object",
            "properties": { "id": { "type": "integer" } },
            "required": ["id"]
        }
    })
}

fn schema_lesson_search() -> Value {
    json!({
        "name": "lesson_search",
        "description": "Search lessons by query string",
        "inputSchema": {
            "type": "object",
            "properties": { "query": { "type": "string" } },
            "required": ["query"]
        }
    })
}

fn schema_lesson_by_context() -> Value {
    json!({
        "name": "lesson_by_context",
        "description": "Filter lessons by context",
        "inputSchema": {
            "type": "object",
            "properties": { "context": { "type": "string" } },
            "required": ["context"]
        }
    })
}

fn schema_lesson_by_tag() -> Value {
    json!({
        "name": "lesson_by_tag",
        "description": "Filter lessons by tag",
        "inputSchema": {
            "type": "object",
            "properties": { "tag": { "type": "string" } },
            "required": ["tag"]
        }
    })
}

fn schema_primitives_summary() -> Value {
    json!({
        "name": "primitives_summary",
        "description": "Get summary of extracted primitives",
        "inputSchema": { "type": "object", "properties": {} }
    })
}

pub fn definitions() -> Value {
    json!({
        "tools": [
            schema_lesson_add(),
            schema_lesson_get(),
            schema_lesson_search(),
            schema_lesson_by_context(),
            schema_lesson_by_tag(),
            schema_primitives_summary()
        ]
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// Tool Handlers
// ─────────────────────────────────────────────────────────────────────────────

fn handle_add(params: &Value) -> Result<Value, String> {
    let title = params["title"].as_str().ok_or("Missing title")?;
    let content = params["content"].as_str().ok_or("Missing content")?;
    let context = params["context"].as_str().ok_or("Missing context")?;
    let tags = extract_tags(params);
    let source = params["source"].as_str().unwrap_or("").to_string();
    let primitives = extract::suggest_primitives(content);

    let lesson = models::Lesson {
        id: 0,
        title: title.into(),
        content: content.into(),
        context: context.into(),
        tags,
        primitives,
        created_at: DateTime::now(),
        source,
    };

    let mut db = storage::load();
    let id = db.add(lesson);
    storage::save(&db);

    Ok(json!({ "id": id, "message": format!("Lesson added with ID {}", id) }))
}

fn extract_tags(params: &Value) -> Vec<String> {
    params["tags"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

fn handle_get(params: &Value) -> Result<Value, String> {
    let id = params["id"].as_u64().ok_or("Missing or invalid id")?;
    let db = storage::load();
    db.get(id)
        .map(|l| serde_json::to_value(l).unwrap_or(json!(null)))
        .ok_or_else(|| format!("Lesson not found: {}", id))
}

fn handle_search(params: &Value) -> Result<Value, String> {
    let query = params["query"].as_str().ok_or("Missing query")?;
    let db = storage::load();
    Ok(serde_json::to_value(db.search(query)).unwrap_or(json!([])))
}

fn handle_by_context(params: &Value) -> Result<Value, String> {
    let ctx = params["context"].as_str().ok_or("Missing context")?;
    let db = storage::load();
    Ok(serde_json::to_value(db.by_context(ctx)).unwrap_or(json!([])))
}

fn handle_by_tag(params: &Value) -> Result<Value, String> {
    let tag = params["tag"].as_str().ok_or("Missing tag")?;
    let db = storage::load();
    Ok(serde_json::to_value(db.by_tag(tag)).unwrap_or(json!([])))
}

fn handle_summary() -> Result<Value, String> {
    let db = storage::load();
    let summary = db.primitives_summary();
    let formatted: Vec<Value> = summary
        .into_iter()
        .map(|(name, (tier, count))| json!({ "name": name, "tier": tier, "count": count }))
        .collect();
    Ok(json!({ "primitives": formatted }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Dispatch
// ─────────────────────────────────────────────────────────────────────────────

pub fn call(params: &Value, id: Value) -> Response {
    let name = match params["name"].as_str() {
        Some(n) => n,
        None => return Response::error(id, -32602, "Missing tool name"),
    };

    let args = &params["arguments"];
    let result = dispatch_tool(name, args);

    match result {
        Ok(content) => Response::success(id, wrap_content(content)),
        Err(e) => Response::error(id, -32000, &e),
    }
}

fn dispatch_tool(name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "lesson_add" => handle_add(args),
        "lesson_get" => handle_get(args),
        "lesson_search" => handle_search(args),
        "lesson_by_context" => handle_by_context(args),
        "lesson_by_tag" => handle_by_tag(args),
        "primitives_summary" => handle_summary(),
        _ => Err(format!("Unknown tool: {}", name)),
    }
}

fn wrap_content(content: Value) -> Value {
    json!({ "content": [{ "type": "text", "text": content.to_string() }] })
}
