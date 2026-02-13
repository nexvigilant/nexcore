//! Chemistry-based gate for improvement actions.
//!
//! Uses real thermodynamic and kinetic equations to determine whether
//! capability improvements should proceed based on:
//! - Arrhenius kinetics (complexity vs urgency)
//! - Gibbs free energy (effort vs quality gain)
//! - Staleness decay (time since last update)
//!
//! # Exit Codes
//! - 0: Allow (Proceed/Schedule recommendation)
//! - 1: Warn (Defer recommendation)
//! - 2: Block (Skip recommendation)

use nexcore_hooks::{
    chemistry_scoring::{ChemistryScore, Recommendation},
    exit_block, exit_success_auto_with, exit_warn, get_file_path, read_input,
};
use std::fs;
use std::path::Path;
use std::time::SystemTime;

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => return, // No input, skip silently
    };

    // Only gate Write and Edit tools
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if !matches!(tool_name, "Write" | "Edit") {
        // Not a write operation, allow
        exit_success_auto_with("non-write tool");
    }

    // Get tool_input, defaulting to empty object if None
    let tool_input = match &input.tool_input {
        Some(v) => v,
        None => exit_success_auto_with("no tool input"),
    };

    let file_path = match get_file_path(tool_input) {
        Some(p) => p,
        None => exit_success_auto_with("no file path"),
    };

    // Calculate chemistry score based on file characteristics
    let score = calculate_score_for_file(&file_path, tool_input);

    match score.recommendation {
        Recommendation::Proceed => {
            exit_success_auto_with(&format!(
                "PROCEED (rate={:.3}, ΔG={:.1}, stale={:.0}%)",
                score.kinetics.rate,
                score.thermodynamics.delta_g,
                (1.0 - score.staleness.relevance) * 100.0
            ));
        }
        Recommendation::Schedule => {
            exit_success_auto_with(&format!(
                "SCHEDULE (rate={:.3}, ΔG={:.1})",
                score.kinetics.rate, score.thermodynamics.delta_g
            ));
        }
        Recommendation::Defer => {
            exit_warn(&format!(
                "DEFER: {} (rate={:.3}, ΔG={:.1})",
                score.kinetics.interpretation(),
                score.kinetics.rate,
                score.thermodynamics.delta_g
            ));
        }
        Recommendation::Skip => {
            exit_block(&format!(
                "SKIP: High barrier (Ea={:.0}), unfavorable thermodynamics (ΔG={:.1})",
                score.kinetics.activation_energy, score.thermodynamics.delta_g
            ));
        }
    }
}

/// Calculate chemistry score for a file modification.
fn calculate_score_for_file(file_path: &str, tool_input: &serde_json::Value) -> ChemistryScore {
    let path = Path::new(file_path);

    // Estimate complexity from content size
    let content_len = tool_input
        .get("content")
        .or_else(|| tool_input.get("new_string"))
        .and_then(|v| v.as_str())
        .map(|s| s.len())
        .unwrap_or(0);

    // Complexity: lines changed (rough estimate: 50 chars per line)
    let lines_changed = (content_len / 50).min(100) as f64;
    let complexity = lines_changed.max(1.0);

    // Urgency: based on file type
    let urgency = if file_path.contains("test") {
        5.0 // Tests are medium urgency
    } else if file_path.ends_with(".rs") {
        7.0 // Rust source is higher urgency
    } else if file_path.ends_with(".md") {
        3.0 // Documentation is lower urgency
    } else {
        5.0 // Default medium
    };

    // Effort: based on complexity
    let effort = complexity * 0.5;

    // Quality gain: assume positive for now (could be enhanced with static analysis)
    let quality_gain = 0.6;

    // Days since update: from file mtime
    let days_since_update = get_days_since_modified(path);

    ChemistryScore::calculate(complexity, urgency, effort, quality_gain, days_since_update)
}

/// Get days since file was last modified.
fn get_days_since_modified(path: &Path) -> f64 {
    let metadata = match fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return 0.0, // New file
    };

    let modified = match metadata.modified() {
        Ok(t) => t,
        Err(_) => return 30.0, // Assume stale if can't read
    };

    let now = SystemTime::now();
    let duration = now.duration_since(modified).unwrap_or_default();
    duration.as_secs_f64() / 86400.0 // Convert to days
}
