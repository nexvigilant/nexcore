//! PreCompact Knowledge Extractor Hook
//!
//! PreCompact hook that extracts key learnings before context compaction.
//!
//! # Event
//! PreCompact
//!
//! # Purpose
//! Saves decisions, patterns, and insights to implicit knowledge store
//! so they persist across compaction events.
//!
//! # Output
//! Writes to ~/.claude/implicit/learnings.json
//!
//! # Exit Codes
//! - 0: Always (extraction is best-effort, non-blocking)

use nexcore_hooks::{HookInput, read_input};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let input: HookInput = match read_input() {
        Some(i) => i,
        None => std::process::exit(0),
    };

    // Read transcript to extract learnings
    if let Some(transcript_path) = &input.transcript_path {
        if let Ok(content) = fs::read_to_string(transcript_path) {
            let learnings = extract_learnings(&content);
            if !learnings.is_empty() {
                save_learnings(&learnings, &input.session_id);
                eprintln!(
                    "📚 Extracted {} learnings before compaction",
                    learnings.len()
                );
            }
        }
    }

    std::process::exit(0);
}

fn extract_learnings(content: &str) -> Vec<String> {
    let mut learnings = Vec::new();

    // Extract insights marked with ★
    for line in content.lines() {
        if line.contains("★ Insight")
            || line.contains("Key learning")
            || line.contains("Important:")
        {
            // Get the next few lines as context
            learnings.push(line.trim().to_string());
        }
    }

    // Extract decisions (patterns like "Decision:", "Chose:", "Selected:")
    for line in content.lines() {
        let lower = line.to_lowercase();
        if lower.contains("decision:") || lower.contains("chose ") || lower.contains("selected ") {
            if line.len() > 20 && line.len() < 500 {
                learnings.push(format!("Decision: {}", line.trim()));
            }
        }
    }

    // Extract error resolutions (learned fixes)
    let mut in_error_block = false;
    let mut error_context = String::new();
    for line in content.lines() {
        if line.contains("error[E") || line.contains("FAILED") {
            in_error_block = true;
            error_context = line.to_string();
        } else if in_error_block
            && (line.contains("Fixed") || line.contains("fixed") || line.contains("Solution:"))
        {
            learnings.push(format!(
                "Fix: {} -> {}",
                error_context.chars().take(100).collect::<String>(),
                line.trim()
            ));
            in_error_block = false;
        }
    }

    // Deduplicate
    learnings.sort();
    learnings.dedup();

    // Limit to most recent/relevant
    learnings.truncate(10);

    learnings
}

fn save_learnings(learnings: &[String], session_id: &str) {
    let implicit_dir = implicit_knowledge_path();

    if let Err(e) = fs::create_dir_all(&implicit_dir) {
        eprintln!("Warning: Could not create implicit dir: {}", e);
        return;
    }

    let learnings_file = implicit_dir.join("session_learnings.jsonl");

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&learnings_file);

    if let Ok(mut file) = file {
        for learning in learnings {
            let entry = serde_json::json!({
                "session_id": session_id,
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "learning": learning
            });
            if let Err(e) = writeln!(file, "{}", entry) {
                eprintln!("Warning: Could not write learning: {}", e);
            }
        }
    }
}

fn implicit_knowledge_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude").join("implicit")
}
