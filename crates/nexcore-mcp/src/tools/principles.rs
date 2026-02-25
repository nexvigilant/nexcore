//! Principles knowledge base tools
//!
//! Search and retrieve principles from ~/.claude/knowledge/principles/

use crate::params::{PrinciplesGetParams, PrinciplesListParams, PrinciplesSearchParams};
use crate::tooling::{ReadOutcome, ScanLimiter, ScanLimits, read_limited_file, snippet_for};
use nexcore_fs::dirs;
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::fs;
use std::path::PathBuf;

/// Get the principles directory path
fn principles_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude/knowledge/principles")
}

/// List all available principles
pub fn list_principles(_params: PrinciplesListParams) -> Result<CallToolResult, McpError> {
    let dir = principles_dir();
    let limits = ScanLimits::from_env();
    let mut limiter = ScanLimiter::new(limits.max_hits);
    let mut notices = Vec::new();

    let mut principles = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if !limiter.allow() {
                break;
            }
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") {
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_default();

                // Read first few lines to get title and description
                let read_outcome = read_limited_file(&path, limits).unwrap_or(ReadOutcome {
                    content: String::new(),
                    notice: None,
                });
                if let Some(notice) = read_outcome.notice {
                    notices.push(notice);
                }
                if !read_outcome.content.is_empty() {
                    let lines: Vec<&str> = read_outcome.content.lines().take(10).collect();
                    let title = lines
                        .iter()
                        .find(|l| l.starts_with("# "))
                        .map(|l| l.trim_start_matches("# ").to_string())
                        .unwrap_or_else(|| name.clone());

                    principles.push(json!({
                        "id": name,
                        "title": title,
                        "path": path.to_string_lossy(),
                    }));
                }
            }
        }
    }

    if let Some(notice) = limiter.notice(limits) {
        notices.push(notice);
    }

    let result = json!({
        "count": principles.len(),
        "principles": principles,
    });

    let mut result = result;
    if !notices.is_empty() {
        if let Ok(value) = serde_json::to_value(notices) {
            result["scan_notice"] = value;
        }
    }

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Get a specific principle by name
pub fn get_principle(params: PrinciplesGetParams) -> Result<CallToolResult, McpError> {
    let dir = principles_dir();
    let file_path = dir.join(format!("{}.md", params.name));
    let limits = ScanLimits::from_env();
    let mut notices = Vec::new();

    if !file_path.exists() {
        // Try fuzzy match
        let mut best_match: Option<(String, usize)> = None;
        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().is_some_and(|e| e == "md") {
                    let name = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();

                    // Simple substring match
                    if name.to_lowercase().contains(&params.name.to_lowercase()) {
                        let dist = name.len().abs_diff(params.name.len());
                        let is_better = best_match
                            .as_ref()
                            .map(|(_, best_dist)| dist < *best_dist)
                            .unwrap_or(true);
                        if is_better {
                            best_match = Some((name, dist));
                        }
                    }
                }
            }
        }

        if let Some((matched_name, _)) = best_match {
            let matched_path = dir.join(format!("{}.md", matched_name));
            let read_outcome = read_limited_file(&matched_path, limits).unwrap_or(ReadOutcome {
                content: String::new(),
                notice: None,
            });
            if let Some(notice) = read_outcome.notice {
                notices.push(notice);
            }
            if !read_outcome.content.is_empty() {
                let mut body = format!(
                    "# Found: {} (fuzzy match for '{}')\n\n{}",
                    matched_name, params.name, read_outcome.content
                );
                if !notices.is_empty() {
                    if let Ok(value) = serde_json::to_string_pretty(&notices) {
                        body.push_str("\n\n---\nscan_notice:\n");
                        body.push_str(&value);
                    }
                }
                return Ok(CallToolResult::success(vec![Content::text(body)]));
            }
        }

        let mut result = json!({
            "error": format!("Principle '{}' not found", params.name),
            "hint": "Use principles_list to see available principles",
        });
        if !notices.is_empty() {
            if let Ok(value) = serde_json::to_value(&notices) {
                result["scan_notice"] = value;
            }
        }
        return Ok(CallToolResult::success(vec![Content::text(
            result.to_string(),
        )]));
    }

    let read_outcome = read_limited_file(&file_path, limits).unwrap_or(ReadOutcome {
        content: String::new(),
        notice: None,
    });
    if let Some(notice) = read_outcome.notice {
        notices.push(notice);
    }
    if !read_outcome.content.is_empty() {
        let mut body = read_outcome.content;
        if !notices.is_empty() {
            if let Ok(value) = serde_json::to_string_pretty(&notices) {
                body.push_str("\n\n---\nscan_notice:\n");
                body.push_str(&value);
            }
        }
        return Ok(CallToolResult::success(vec![Content::text(body)]));
    }

    let mut result = json!({
        "error": "Failed to read principle",
    });
    if !notices.is_empty() {
        if let Ok(value) = serde_json::to_value(&notices) {
            result["scan_notice"] = value;
        }
    }
    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}

/// Search principles by keyword
pub fn search_principles(params: PrinciplesSearchParams) -> Result<CallToolResult, McpError> {
    let dir = principles_dir();
    let query = params.query.to_lowercase();
    let limits = ScanLimits::from_env();
    let limit = params.limit.unwrap_or(10).min(limits.max_hits);
    let mut notices = Vec::new();
    let mut limiter = ScanLimiter::new(limits.max_hits);

    let mut results = Vec::new();

    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            if !limiter.allow() {
                break;
            }
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") {
                let read_outcome = read_limited_file(&path, limits).unwrap_or(ReadOutcome {
                    content: String::new(),
                    notice: None,
                });
                if let Some(notice) = read_outcome.notice {
                    notices.push(notice);
                }
                if !read_outcome.content.is_empty() {
                    let content_lower = read_outcome.content.to_lowercase();

                    if content_lower.contains(&query) {
                        let name = path
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();

                        // Extract matching sections
                        let mut sections = Vec::new();
                        let lines: Vec<&str> = read_outcome.content.lines().collect();

                        for (i, line) in lines.iter().enumerate() {
                            if line.to_lowercase().contains(&query) {
                                // Get context: heading + surrounding lines
                                let mut heading = String::new();
                                for j in (0..i).rev() {
                                    if lines[j].starts_with('#') {
                                        heading = lines[j].to_string();
                                        break;
                                    }
                                }

                                // Get 2 lines before and after for context
                                let start = i.saturating_sub(2);
                                let end = (i + 3).min(lines.len());
                                let context: Vec<&str> = lines[start..end].to_vec();
                                let context_text = snippet_for(&context.join("\n"), limits).text;

                                sections.push(json!({
                                    "heading": heading,
                                    "line_number": i + 1,
                                    "context": context_text,
                                }));

                                if sections.len() >= 5 {
                                    break;
                                }
                            }
                        }

                        results.push(json!({
                            "principle": name,
                            "path": path.to_string_lossy(),
                            "match_count": content_lower.matches(&query).count(),
                            "sections": sections,
                        }));
                    }
                }
            }
        }
    }

    // Sort by match count descending
    results.sort_by(|a, b| {
        let count_a = a.get("match_count").and_then(|v| v.as_u64()).unwrap_or(0);
        let count_b = b.get("match_count").and_then(|v| v.as_u64()).unwrap_or(0);
        count_b.cmp(&count_a)
    });

    results.truncate(limit);

    if let Some(notice) = limiter.notice(limits) {
        notices.push(notice);
    }

    let mut result = json!({
        "query": params.query,
        "count": results.len(),
        "results": results,
    });

    if !notices.is_empty() {
        if let Ok(value) = serde_json::to_value(notices) {
            result["scan_notice"] = value;
        }
    }

    Ok(CallToolResult::success(vec![Content::text(
        result.to_string(),
    )]))
}
