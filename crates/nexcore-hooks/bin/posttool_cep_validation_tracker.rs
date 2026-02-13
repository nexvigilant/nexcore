//! CEP Validation Tracker (PostToolUse:Edit|Write)
//!
//! Tracks extraction quality and suggests improvements when metrics fall short.
//! Monitors coverage, minimality, and independence in CEP-related code.
//!
//! Patent: NV-2026-001, NV-2026-002

use nexcore_hooks::{exit_success_auto, exit_success_auto_with, exit_warn, read_input};
use std::fs;
use std::path::Path;

/// Tracking file for CEP metrics
const CEP_METRICS_FILE: &str = ".claude/cep_metrics.json";

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    let file_path = input.get_file_path().unwrap_or("");

    // Only track CEP-related files
    let is_cep_file = file_path.contains("cep")
        || file_path.contains("primitive")
        || file_path.contains("extraction")
        || file_path.contains("domain_discovery");

    if !is_cep_file {
        exit_success_auto();
    }

    // Track the modification
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let metrics_path = Path::new(&home).join(CEP_METRICS_FILE);

    // Ensure parent directory exists (best-effort)
    if let Some(parent) = metrics_path.parent() {
        // Intentionally ignore: best-effort directory creation
        #[allow(unused_must_use)]
        {
            fs::create_dir_all(parent);
        }
    }

    // Load existing metrics or create new
    let mut metrics: serde_json::Value = fs::read_to_string(&metrics_path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| {
            serde_json::json!({
                "files_modified": 0,
                "last_modified": null,
                "extraction_count": 0,
                "validation_count": 0
            })
        });

    // Update metrics
    if let Some(obj) = metrics.as_object_mut() {
        let count = obj
            .get("files_modified")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        obj.insert("files_modified".to_string(), serde_json::json!(count + 1));
        obj.insert(
            "last_modified".to_string(),
            serde_json::json!(chrono::Utc::now().to_rfc3339()),
        );

        // Track specific operations
        let content = input.get_written_content().unwrap_or("");
        if content.contains("extract") || content.contains("decompose") {
            let extraction_count = obj
                .get("extraction_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            obj.insert(
                "extraction_count".to_string(),
                serde_json::json!(extraction_count + 1),
            );
        }
        if content.contains("validate") || content.contains("coverage") {
            let validation_count = obj
                .get("validation_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            obj.insert(
                "validation_count".to_string(),
                serde_json::json!(validation_count + 1),
            );
        }
    }

    // Save updated metrics (best-effort)
    if let Ok(json) = serde_json::to_string_pretty(&metrics) {
        // Intentionally ignore: best-effort metrics save
        #[allow(unused_must_use)]
        {
            fs::write(&metrics_path, json);
        }
    }

    // Check for common issues and suggest improvements
    let content = input.get_written_content().unwrap_or("");
    let mut suggestions = Vec::new();

    // Check for missing validation after extraction
    if content.contains("extract") && !content.contains("validate") {
        suggestions.push("Consider adding validation after extraction (coverage ≥0.95)");
    }

    // Check for missing feedback loop
    if content.contains("deploy") && !content.contains("improve") && !content.contains("feedback") {
        suggestions.push("CEP Stage 8 (IMPROVE) creates feedback loop for continuous refinement");
    }

    // Check for missing tier classification
    if content.contains("primitive")
        && !content.contains("T1")
        && !content.contains("T2")
        && !content.contains("T3")
    {
        suggestions.push(
            "Classify primitives by tier: T1 (universal), T2 (cross-domain), T3 (domain-specific)",
        );
    }

    if !suggestions.is_empty() {
        let msg = format!("CEP suggestions:\n• {}", suggestions.join("\n• "));
        exit_warn(&msg);
    }

    exit_success_auto_with("CEP metrics tracked");
}
