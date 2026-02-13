//! Handoff Completion Reviewer - UserPromptSubmit Event
//!
//! Reviews all handoff documents after each user input to:
//! 1. Check completion status of todos/blockers
//! 2. Report which handoffs can be safely archived
//! 3. Suggest cleanup commands
//!
//! Runs on UserPromptSubmit to provide context about handoff hygiene.

use nexcore_hooks::{exit_skip_prompt, exit_with_context, read_input};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Get home directory
fn home_dir() -> PathBuf {
    dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
}

/// Get handoffs base directory
fn handoffs_base_dir() -> PathBuf {
    home_dir().join(".claude").join("handoffs")
}

/// Get plans base directory
fn plans_base_dir() -> PathBuf {
    home_dir().join(".claude").join("plans")
}

/// Get current timestamp
fn now() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Handoff completion status
#[derive(Debug)]
struct HandoffStatus {
    path: PathBuf,
    id: Option<u32>,
    is_final: bool,
    has_blockers: bool,
    blocker_count: usize,
    age_hours: f64,
    can_archive: bool,
}

/// Collect all markdown file paths from handoffs directory (flat list)
fn collect_handoff_paths() -> Vec<PathBuf> {
    let base = handoffs_base_dir();
    if !base.exists() {
        return Vec::new();
    }

    let mut paths = Vec::new();

    // First pass: collect root-level files and subdirectory paths
    let entries: Vec<_> = fs::read_dir(&base)
        .into_iter()
        .flatten()
        .flatten()
        .collect();

    // Collect root-level .md files
    paths.extend(
        entries
            .iter()
            .filter(|e| e.path().is_file())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .map(|e| e.path()),
    );

    // Collect subdirectory paths (excluding "archived")
    let subdirs: Vec<PathBuf> = entries
        .iter()
        .filter(|e| e.path().is_dir())
        .filter(|e| e.path().file_name().is_some_and(|n| n != "archived"))
        .map(|e| e.path())
        .collect();

    // Second pass: collect files from each subdirectory
    paths.extend(subdirs.iter().flat_map(|dir| {
        fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().is_file())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
            .map(|e| e.path())
    }));

    paths
}

/// Scan all handoff files and analyze each
fn scan_all_handoffs() -> Vec<HandoffStatus> {
    collect_handoff_paths()
        .into_iter()
        .filter_map(|p| analyze_handoff(&p))
        .collect()
}

/// Analyze a single handoff file
fn analyze_handoff(path: &PathBuf) -> Option<HandoffStatus> {
    let content = fs::read_to_string(path).ok()?;
    let metadata = fs::metadata(path).ok()?;
    let modified = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs_f64();

    // Extract ID from filename (e.g., "00142.md" -> 142)
    let id = path
        .file_stem()
        .and_then(|s| s.to_str())
        .and_then(|s| s.parse::<u32>().ok());

    // Check if marked [FINAL]
    let is_final = content.contains("[FINAL]");

    // Count unresolved blockers (unchecked checkboxes)
    let blocker_count = content.lines().filter(|l| l.starts_with("- [ ]")).count();
    let has_blockers = blocker_count > 0;

    // Calculate age in hours
    let age_hours = (now() - modified) / 3600.0;

    // Can archive if: FINAL + no blockers + older than 1 hour
    let can_archive = is_final && !has_blockers && age_hours > 1.0;

    Some(HandoffStatus {
        path: path.clone(),
        id,
        is_final,
        has_blockers,
        blocker_count,
        age_hours,
        can_archive,
    })
}

/// Check plans directory status
fn check_plans_status() -> (usize, usize) {
    let base = plans_base_dir();
    if !base.exists() {
        return (0, 0);
    }

    let entries: Vec<_> = fs::read_dir(&base)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter(|e| e.path().file_name().is_some_and(|n| n != "archived"))
        .collect();

    let mut total = 0;
    let mut empty_dirs = 0;

    entries.iter().for_each(|entry| {
        let has_plans = fs::read_dir(entry.path())
            .map(|e| {
                e.flatten()
                    .any(|f| f.path().extension().is_some_and(|ext| ext == "md"))
            })
            .unwrap_or(false);

        if has_plans {
            total += 1;
        } else {
            empty_dirs += 1;
        }
    });

    (total, empty_dirs)
}

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_skip_prompt(),
    };

    // Scan all handoffs
    let handoffs = scan_all_handoffs();

    // If no handoffs, skip
    if handoffs.is_empty() {
        exit_skip_prompt();
    }

    // Categorize
    let archivable: Vec<_> = handoffs.iter().filter(|h| h.can_archive).collect();
    let with_blockers: Vec<_> = handoffs.iter().filter(|h| h.has_blockers).collect();
    let in_progress: Vec<_> = handoffs.iter().filter(|h| !h.is_final).collect();

    // Only report if there are archivable handoffs or significant counts
    if archivable.is_empty() && with_blockers.is_empty() {
        exit_skip_prompt();
    }

    let (plan_count, empty_plan_dirs) = check_plans_status();

    let mut context = String::new();

    context.push_str("📋 **HANDOFF STATUS REVIEW**\n\n");

    // Summary
    context.push_str(&format!(
        "**Total Handoffs:** {} | **Archivable:** {} | **With Blockers:** {} | **In-Progress:** {}\n\n",
        handoffs.len(),
        archivable.len(),
        with_blockers.len(),
        in_progress.len()
    ));

    // Archivable handoffs
    if !archivable.is_empty() {
        context.push_str("### ✅ Ready for Archive\n");
        context.push_str("These handoffs are [FINAL], have no blockers, and are >1 hour old:\n");
        archivable.iter().for_each(|h| {
            let id_str = h.id.map(|i| format!("#{:05}", i)).unwrap_or_default();
            context.push_str(&format!(
                "- `{}` {} ({:.1}h old)\n",
                h.path.display(),
                id_str,
                h.age_hours
            ));
        });
        context.push('\n');

        // Cleanup command
        context.push_str("**Suggested cleanup:**\n");
        context.push_str("```bash\n");
        context.push_str("# Archive old handoffs\n");
        context.push_str("mkdir -p ~/.claude/handoffs/archived\n");
        archivable.iter().for_each(|h| {
            context.push_str(&format!(
                "mv \"{}\" ~/.claude/handoffs/archived/\n",
                h.path.display()
            ));
        });
        context.push_str("```\n\n");
    }

    // Handoffs with blockers
    if !with_blockers.is_empty() {
        context.push_str("### ⚠️ Handoffs with Unresolved Blockers\n");
        with_blockers.iter().for_each(|h| {
            let id_str = h.id.map(|i| format!("#{:05}", i)).unwrap_or_default();
            context.push_str(&format!(
                "- `{}` {} ({} blocker{})\n",
                h.path.display(),
                id_str,
                h.blocker_count,
                if h.blocker_count > 1 { "s" } else { "" }
            ));
        });
        context.push('\n');
    }

    // Plans status
    if plan_count > 0 || empty_plan_dirs > 0 {
        context.push_str(&format!(
            "**Plans:** {} active directories, {} empty (can delete)\n\n",
            plan_count, empty_plan_dirs
        ));
    }

    // Instructions
    context.push_str("---\n");
    context.push_str("*If the user confirms, run the cleanup commands above. ");
    context.push_str("Otherwise, continue with their request.*\n");

    exit_with_context(&context);
}
