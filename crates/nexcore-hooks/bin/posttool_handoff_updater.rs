//! Incremental Handoff Updater - PostToolUse:Edit|Write Event
//!
//! Updates the session handoff document after every file write.
//! This ensures context is preserved even if the session crashes.
//!
//! Uses the tracking registry to maintain consistent IDs:
//! - Handoff filename: 00001.md, 00002.md, etc.
//! - Updated incrementally with each file modification

use nexcore_hooks::paths::handoffs_dir;
use nexcore_hooks::state::{SessionState, now};
use nexcore_hooks::tracking::TrackingRegistry;
use nexcore_hooks::{exit_success_auto, read_input};
use std::fs;
use std::process::Command;

fn get_uncommitted_count() -> usize {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .filter(|l| !l.trim().is_empty())
                .count()
        })
        .unwrap_or(0)
}

fn get_modified_files() -> Vec<String> {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .take(20) // Limit to 20 files
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn format_duration(secs: f64) -> String {
    let mins = (secs / 60.0) as u64;
    let hours = mins / 60;
    let mins = mins % 60;
    if hours > 0 {
        format!("{}h {}m", hours, mins)
    } else {
        format!("{}m", mins)
    }
}

fn generate_handoff_content(
    tracking_id: u32,
    session_id: &str,
    state: &SessionState,
    modified_file: Option<&str>,
) -> String {
    let mut content = String::new();
    let duration = now() - state.session_start;
    let uncommitted = get_uncommitted_count();
    let modified_files = get_modified_files();

    content.push_str(&format!(
        "# Handoff #{:05}\n\
         Session: {}\n\
         Last Updated: {}\n\n",
        tracking_id,
        session_id,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Session metrics
    content.push_str("## Session Metrics\n");
    content.push_str(&format!("- **Duration**: {}\n", format_duration(duration)));
    content.push_str(&format!(
        "- **Files touched**: {}\n",
        state.files_since_verification
    ));
    content.push_str(&format!(
        "- **Lines written**: {}\n",
        state.lines_since_verification
    ));
    content.push_str(&format!("- **Uncommitted changes**: {}\n\n", uncommitted));

    // Last action
    if let Some(file) = modified_file {
        content.push_str("## Last Action\n");
        content.push_str(&format!("Modified: `{}`\n\n", file));
    }

    // Uncommitted files
    if !modified_files.is_empty() {
        content.push_str("## Working Tree\n");
        for f in &modified_files {
            content.push_str(&format!("- {}\n", f));
        }
        if uncommitted > 20 {
            content.push_str(&format!("- ...and {} more\n", uncommitted - 20));
        }
        content.push_str("\n");
    }

    // Unverified assumptions
    let unverified: Vec<_> = state
        .assumptions
        .iter()
        .filter(|a| a.status == "assumed")
        .collect();

    if !unverified.is_empty() {
        content.push_str("## Unverified Assumptions\n");
        for a in unverified.iter().take(5) {
            content.push_str(&format!("- [ ] {} ({})\n", a.assumption, a.confidence));
        }
        if unverified.len() > 5 {
            content.push_str(&format!("- ...and {} more\n", unverified.len() - 5));
        }
        content.push_str("\n");
    }

    // Requirements
    if state.requirements_verified {
        content.push_str("## Requirements\n");
        content.push_str("✅ Requirements verified\n");
        if !state.explicit_requirements.is_empty() {
            for req in state.explicit_requirements.iter().take(3) {
                content.push_str(&format!("- {}\n", req));
            }
        }
        content.push_str("\n");
    }

    content.push_str("---\n");
    content.push_str("*Auto-updated by incremental_handoff_updater*\n");

    content
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only process Edit/Write tool completions
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "Edit" && tool_name != "Write" {
        exit_success_auto();
    }

    // Get the file that was modified
    let modified_file = input.get_file_path();

    // Skip if modifying our own handoff files (avoid recursion)
    if let Some(path) = &modified_file {
        if path.contains("/handoffs/") || path.contains("/tracking_registry") {
            exit_success_auto();
        }
    }

    let session_id = input.session_id.clone();
    let state = SessionState::load();

    // Load registry and get/create handoff ID
    let mut registry = TrackingRegistry::load();
    let handoff_id = registry.get_or_create_handoff_id(&session_id);

    // Generate handoff content
    let content =
        generate_handoff_content(handoff_id, &session_id, &state, modified_file.as_deref());

    // Write handoff file
    let dir = handoffs_dir();
    let _ = fs::create_dir_all(&dir);
    let handoff_path = dir.join(format!("{:05}.md", handoff_id));
    let _ = fs::write(&handoff_path, &content);

    // Update registry
    registry.update_artifact(handoff_id);
    let _ = registry.save();

    exit_success_auto();
}
