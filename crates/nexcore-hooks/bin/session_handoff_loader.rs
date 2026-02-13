//! Session Handoff Loader - SessionStart Event
//!
//! Restores previous session context by loading the most recent handoff.
//! Uses tracking registry to find handoffs by ID (00001.md, 00002.md, etc.)
//!
//! **Phase 1 Enhanced Context Injection:**
//! - Injects FULL handoff content into Claude's context (not just summary)
//! - Loads associated plan content if referenced in handoff
//! - Provides complete session restoration for true continuity
//!
//! Actions:
//! - Find most recent handoff from tracking registry
//! - Inject FULL handoff content (was: summary only)
//! - Load and inject active plan content if present
//! - Initialize tracking for new session

use nexcore_hooks::paths::{handoffs_dir, plans_dir};
use nexcore_hooks::{exit_skip_prompt, exit_with_context, read_input};
use std::fs;
use std::path::PathBuf;

/// Find the most recent handoff file by modification time
///
/// O(n log n) where n = number of handoff files (due to sort)
fn find_most_recent_handoff() -> Option<(u32, PathBuf, String)> {
    let dir = handoffs_dir();
    if !dir.exists() {
        return None;
    }

    // Find all handoff files with their modification times - O(n)
    let mut handoffs: Vec<_> = fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .filter_map(|e| {
            let path = e.path();
            let name = path.file_stem()?.to_str()?;

            // Get modification time for sorting - O(1) per file
            let modified = e
                .metadata()
                .ok()?
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs_f64();

            // Try to parse tracking ID (5-digit format like 00001)
            let id: Option<u32> = if name.len() == 5 && name.chars().all(|c| c.is_ascii_digit()) {
                name.parse().ok()
            } else if name.starts_with("handoff_") {
                None // Legacy format - no tracking ID
            } else {
                return None; // Unknown format, skip
            };

            Some((id, path, modified))
        })
        .collect();

    // Sort by modification time descending - O(n log n)
    handoffs.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));

    // Return first (most recent) - O(1)
    handoffs.first().and_then(|(id, path, _)| {
        let content = fs::read_to_string(path).ok()?;
        Some((id.unwrap_or(0), path.clone(), content))
    })
}

/// Extract key information from handoff content
fn extract_key_info(content: &str) -> HandoffInfo {
    let mut info = HandoffInfo::default();
    info.is_final = content.contains("[FINAL]");

    let mut in_uncommitted = false;
    let mut in_blockers = false;
    let mut in_plan = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Reset section tracking on new headers
        if trimmed.starts_with("## ") {
            in_uncommitted = trimmed.contains("Uncommitted");
            in_blockers = trimmed.contains("Blocker")
                || trimmed.contains("Assumption")
                || trimmed.contains("Unverified");
            in_plan = trimmed.contains("Plan");
        }

        // Extract active plan path (new format)
        if in_plan && trimmed.starts_with("📄") {
            info.active_plan = trimmed
                .trim_start_matches("📄")
                .trim()
                .trim_start_matches("**")
                .trim_end_matches("**")
                .split_whitespace()
                .next()
                .map(String::from);
        }

        // Extract active plan path (old format)
        if trimmed.contains("**Path**:") {
            info.active_plan = trimmed.split(':').nth(1).map(|s| s.trim().to_string());
        }

        // Collect uncommitted changes (new format with backticks)
        if in_uncommitted && trimmed.starts_with("- `") {
            let file = trimmed
                .trim_start_matches("- `")
                .trim_end_matches('`')
                .to_string();
            if !file.contains("clean") {
                info.uncommitted.push(file);
            }
        }

        // Collect uncommitted changes (old format)
        if in_uncommitted && trimmed.starts_with("- ") && !trimmed.starts_with("- `") {
            let item = trimmed[2..].trim().to_string();
            if !item.contains("None") && !item.contains("clean") {
                info.uncommitted.push(item);
            }
        }

        // Collect blockers (unchecked items)
        if in_blockers && trimmed.starts_with("- [ ]") {
            info.blockers.push(trimmed[5..].trim().to_string());
        }
    }

    info
}

/// Structured handoff information
#[derive(Default)]
struct HandoffInfo {
    active_plan: Option<String>,
    uncommitted: Vec<String>,
    blockers: Vec<String>,
    is_final: bool,
}

/// Try to load plan content from path
fn load_plan_content(plan_path: &str) -> Option<String> {
    // Try direct path first
    if let Ok(content) = fs::read_to_string(plan_path) {
        return Some(content);
    }

    // Try relative to plans directory
    let plans = plans_dir();
    let relative_path = plans.join(plan_path.trim_start_matches("~/.claude/plans/"));
    if let Ok(content) = fs::read_to_string(&relative_path) {
        return Some(content);
    }

    // Try just the filename in plans directory
    if let Some(filename) = PathBuf::from(plan_path).file_name() {
        let filename_path = plans.join(filename);
        if let Ok(content) = fs::read_to_string(&filename_path) {
            return Some(content);
        }
    }

    None
}

/// Truncate handoff content if too large (preserve structure)
fn truncate_handoff(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= max_lines {
        return content.to_string();
    }

    // Keep header and first sections, truncate middle
    let header_end = lines
        .iter()
        .position(|l| l.starts_with("## "))
        .unwrap_or(10);
    let mut result = lines[..header_end.min(20)].join("\n");
    result.push_str("\n\n... [content truncated for context window] ...\n\n");

    // Include last section (usually Next Steps)
    let last_section_start = lines
        .iter()
        .rposition(|l| l.starts_with("## "))
        .unwrap_or(lines.len() - 10);
    result.push_str(&lines[last_section_start..].join("\n"));

    result
}

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_skip_prompt(),
    };

    // Find most recent handoff
    let (id, path, content) = match find_most_recent_handoff() {
        Some(h) => h,
        None => {
            // No previous handoff - fresh start
            exit_with_context(
                "🆕 **NEW SESSION** - No previous handoff found.\n\
                 \n\
                 Handoffs will be created automatically as you work.\n\
                 Consider documenting your plan early in ~/.claude/plans/",
            );
        }
    };

    let info = extract_key_info(&content);

    // Determine if ID is from new format (5-digit) or legacy
    let id_display = if id < 100000 {
        format!("#{:05}", id)
    } else {
        "".to_string() // Legacy format, don't show ID
    };

    // Build FULL context with actual handoff content
    let mut context = String::new();

    // ═══════════════════════════════════════════════════════════════════════════
    // HEADER
    // ═══════════════════════════════════════════════════════════════════════════
    context.push_str(
        "╔═══════════════════════════════════════════════════════════════════════════╗\n",
    );
    context.push_str(
        "║                    🔄 SESSION CONTEXT RESTORED                            ║\n",
    );
    context.push_str(
        "╚═══════════════════════════════════════════════════════════════════════════╝\n\n",
    );

    context.push_str(&format!(
        "**Handoff{}**: `{}`{}\n\n",
        if !id_display.is_empty() {
            format!(" {}", id_display)
        } else {
            String::new()
        },
        path.display(),
        if info.is_final {
            " (Final)"
        } else {
            " (In-Progress)"
        }
    ));

    // ═══════════════════════════════════════════════════════════════════════════
    // QUICK STATUS
    // ═══════════════════════════════════════════════════════════════════════════
    if !info.uncommitted.is_empty() || !info.blockers.is_empty() {
        context.push_str("### ⚠️ Attention Required\n");
        if !info.uncommitted.is_empty() {
            context.push_str(&format!(
                "- **{} uncommitted changes** to review\n",
                info.uncommitted.len()
            ));
        }
        if !info.blockers.is_empty() {
            context.push_str(&format!(
                "- **{} unresolved blockers** to address\n",
                info.blockers.len()
            ));
        }
        context.push('\n');
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FULL HANDOFF CONTENT (Phase 1 Enhancement)
    // ═══════════════════════════════════════════════════════════════════════════
    context.push_str("---\n\n");
    context.push_str("## 📋 Previous Session Handoff\n\n");

    // Inject full handoff content (truncated if necessary)
    let handoff_content = truncate_handoff(&content, 150);
    context.push_str(&handoff_content);
    context.push_str("\n\n");

    // ═══════════════════════════════════════════════════════════════════════════
    // ACTIVE PLAN CONTENT (Phase 1 Enhancement)
    // ═══════════════════════════════════════════════════════════════════════════
    if let Some(plan_path) = &info.active_plan {
        context.push_str("---\n\n");
        context.push_str("## 🎯 Active Plan\n\n");
        context.push_str(&format!("**Path**: `{}`\n\n", plan_path));

        if let Some(plan_content) = load_plan_content(plan_path) {
            // Truncate plan if very long
            let plan_display = if plan_content.lines().count() > 200 {
                truncate_handoff(&plan_content, 200)
            } else {
                plan_content
            };
            context.push_str(&plan_display);
            context.push('\n');
        } else {
            context.push_str("*Plan file not found. It may have been archived or moved.*\n");
        }
        context.push('\n');
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // INSTRUCTIONS
    // ═══════════════════════════════════════════════════════════════════════════
    context.push_str("---\n\n");
    context.push_str("**Claude**: You now have the full context from the previous session. ");
    if info.is_final {
        context.push_str(
            "The previous session was marked as complete. Start fresh or continue as needed.",
        );
    } else if !info.blockers.is_empty() {
        context.push_str("Address the blockers first, then continue with the plan.");
    } else if info.active_plan.is_some() {
        context.push_str("Continue executing the active plan from where it left off.");
    } else {
        context.push_str("Review the handoff and continue the previous work.");
    }
    context.push('\n');

    exit_with_context(&context);
}
