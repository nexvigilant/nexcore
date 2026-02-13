//! Session Continuity Prompt Hook
//!
//! SessionStart hook that ensures work continuity across sessions.
//!
//! # Event
//! SessionStart
//!
//! # Purpose
//! Reviews existing handoffs/plans and prompts user to:
//! 1. Resume previous work (load handoff + plan)
//! 2. Verify completion and archive old artifacts
//! 3. Start fresh on something new
//!
//! # Scoping
//! Plans/handoffs are isolated per working directory.
//!
//! # Exit Codes
//! - 0: Context injected or no previous work found

use nexcore_hooks::paths::{handoffs_dir, plans_dir};
use nexcore_hooks::{exit_skip_session, exit_with_session_context, read_input};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn archived_dir() -> PathBuf {
    plans_dir().join("archived")
}

/// Get current timestamp
fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Check if within last N hours
fn is_recent(modified: f64, hours: f64) -> bool {
    (now() - modified) < (hours * 3600.0)
}

/// Extract tracking ID from filename
fn extract_id(filename: &str) -> Option<u32> {
    let stem = filename.strip_suffix(".md")?;
    let first_part = stem.split('_').next()?;
    if first_part.len() == 5 && first_part.chars().all(|c| c.is_ascii_digit()) {
        first_part.parse().ok()
    } else {
        None
    }
}

/// Extract title from markdown content
fn extract_title(content: &str) -> String {
    content
        .lines()
        .find(|l| l.starts_with('#'))
        .map(|l| l.trim_start_matches('#').trim().to_string())
        .unwrap_or_else(|| "Untitled".to_string())
}

/// Handoff summary
struct HandoffSummary {
    id: Option<u32>,
    path: PathBuf,
    title: String,
    modified: f64,
    uncommitted_count: usize,
    blocker_count: usize,
    is_final: bool,
}

/// Plan summary
struct PlanSummary {
    id: Option<u32>,
    path: PathBuf,
    title: String,
    modified: f64,
}

fn find_recent_handoffs(hours: f64) -> Vec<HandoffSummary> {
    let dir = handoffs_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut handoffs: Vec<_> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .filter_map(|e| {
            let path = e.path();
            let modified = e
                .metadata()
                .ok()?
                .modified()
                .ok()?
                .duration_since(UNIX_EPOCH)
                .ok()?
                .as_secs_f64();

            if !is_recent(modified, hours) {
                return None;
            }

            let filename = path.file_name()?.to_str()?;
            let id = extract_id(filename);
            let content = fs::read_to_string(&path).ok()?;
            let title = extract_title(&content);

            // Count uncommitted and blockers
            let uncommitted_count = content
                .lines()
                .filter(|l| {
                    l.starts_with("- `M ") || l.starts_with("- `D ") || l.starts_with("- `?")
                })
                .count();
            let blocker_count = content.lines().filter(|l| l.starts_with("- [ ]")).count();
            let is_final = content.contains("[FINAL]");

            Some(HandoffSummary {
                id,
                path,
                title,
                modified,
                uncommitted_count,
                blocker_count,
                is_final,
            })
        })
        .collect();

    handoffs.sort_by(|a, b| {
        b.modified
            .partial_cmp(&a.modified)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    handoffs
}

fn find_recent_plans(hours: f64) -> Vec<PlanSummary> {
    let dir = plans_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut plans: Vec<_> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .filter(|e| e.path().is_file()) // Skip directories
        .filter_map(|e| {
            let path = e.path();
            let modified = e
                .metadata()
                .ok()?
                .modified()
                .ok()?
                .duration_since(UNIX_EPOCH)
                .ok()?
                .as_secs_f64();

            if !is_recent(modified, hours) {
                return None;
            }

            let filename = path.file_name()?.to_str()?;
            let id = extract_id(filename);
            let content = fs::read_to_string(&path).ok()?;
            let title = extract_title(&content);

            Some(PlanSummary {
                id,
                path,
                title,
                modified,
            })
        })
        .collect();

    plans.sort_by(|a, b| {
        b.modified
            .partial_cmp(&a.modified)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    plans
}

fn format_age(modified: f64) -> String {
    let age_secs = now() - modified;
    let hours = (age_secs / 3600.0) as u64;
    let mins = ((age_secs % 3600.0) / 60.0) as u64;

    if hours > 0 {
        format!("{}h {}m ago", hours, mins)
    } else {
        format!("{}m ago", mins)
    }
}

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_skip_session(),
    };

    // Find recent artifacts (last 48 hours for broader context)
    let handoffs = find_recent_handoffs(48.0);
    let plans = find_recent_plans(48.0);

    // If nothing recent, skip
    if handoffs.is_empty() && plans.is_empty() {
        exit_skip_session();
    }

    let mut context = String::new();

    // Header
    context.push_str("═══════════════════════════════════════════════════════════════\n");
    context.push_str("                    📋 SESSION CONTINUITY CHECK\n");
    context.push_str("═══════════════════════════════════════════════════════════════\n\n");

    // Most recent handoff
    if let Some(h) = handoffs.first() {
        context.push_str("## Most Recent Handoff\n");
        if let Some(id) = h.id {
            context.push_str(&format!("📄 **#{}** - {}\n", format!("{:05}", id), h.title));
        } else {
            context.push_str(&format!("📄 {}\n", h.title));
        }
        context.push_str(&format!("   Path: `{}`\n", h.path.display()));
        context.push_str(&format!("   Age: {}\n", format_age(h.modified)));
        context.push_str(&format!(
            "   Status: {}\n",
            if h.is_final {
                "✅ Final"
            } else {
                "🔄 In-Progress"
            }
        ));

        if h.uncommitted_count > 0 {
            context.push_str(&format!(
                "   ⚠️ {} uncommitted changes\n",
                h.uncommitted_count
            ));
        }
        if h.blocker_count > 0 {
            context.push_str(&format!("   🚧 {} unresolved blockers\n", h.blocker_count));
        }
        context.push('\n');
    }

    // Most recent plan
    if let Some(p) = plans.first() {
        context.push_str("## Most Recent Plan\n");
        if let Some(id) = p.id {
            context.push_str(&format!("📋 **#{}** - {}\n", format!("{:05}", id), p.title));
        } else {
            context.push_str(&format!("📋 {}\n", p.title));
        }
        context.push_str(&format!("   Path: `{}`\n", p.path.display()));
        context.push_str(&format!("   Age: {}\n", format_age(p.modified)));
        context.push('\n');
    }

    // Other recent items
    if handoffs.len() > 1 || plans.len() > 1 {
        context.push_str("## Other Recent Items\n");
        for h in handoffs.iter().skip(1).take(2) {
            let id_str = h.id.map(|id| format!("#{:05}", id)).unwrap_or_default();
            context.push_str(&format!(
                "  • Handoff {} {} ({})\n",
                id_str,
                h.title,
                format_age(h.modified)
            ));
        }
        for p in plans.iter().skip(1).take(2) {
            let id_str = p.id.map(|id| format!("#{:05}", id)).unwrap_or_default();
            context.push_str(&format!(
                "  • Plan {} {} ({})\n",
                id_str,
                p.title,
                format_age(p.modified)
            ));
        }
        context.push('\n');
    }

    // Instructions for Claude
    context.push_str("═══════════════════════════════════════════════════════════════\n");
    context.push_str("                      🤖 CLAUDE INSTRUCTIONS\n");
    context.push_str("═══════════════════════════════════════════════════════════════\n\n");

    context.push_str("**YOU MUST ASK THE USER** which option they prefer:\n\n");

    context.push_str("**Option 1: Resume** 🔄\n");
    context.push_str("  - Read the handoff and plan files above\n");
    context.push_str("  - Continue where the previous session left off\n");
    context.push_str("  - Address any uncommitted changes or blockers\n\n");

    context.push_str("**Option 2: Complete & Archive** ✅\n");
    context.push_str("  - Review if the previous work is actually complete\n");
    context.push_str("  - If complete: archive the plan to ~/.claude/plans/archived/\n");
    context.push_str("  - Delete or archive old handoffs\n");
    context.push_str("  - Start fresh with a clean slate\n\n");

    context.push_str("**Option 3: Something New** 🆕\n");
    context.push_str("  - Keep existing artifacts but don't load them\n");
    context.push_str("  - Start working on an unrelated task\n");
    context.push_str("  - Previous work remains for later\n\n");

    context.push_str("**Ask the user now**: \"I found recent session artifacts. ");
    context.push_str("Would you like to (1) Resume previous work, ");
    context.push_str("(2) Verify completion and archive, or ");
    context.push_str("(3) Work on something new?\"\n");

    exit_with_session_context(&context);
}
