//! Plan Loader - SessionStart Event
//!
//! Detects and surfaces existing plan files on session start.
//! Supports both tracked (00001_name.md) and legacy (name.md) formats.
//!
//! Actions:
//! - Check ~/.claude/plans/ for recent plans
//! - If plan exists from last 24h, inject context with plan path
//! - Surface plan summary and tracking ID if available

use nexcore_hooks::paths::plans_dir;
use nexcore_hooks::{exit_skip_prompt, exit_with_context, read_input};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Check if a timestamp is within the last N hours
fn is_recent(modified_secs: f64, hours: f64) -> bool {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0);
    (now - modified_secs) < (hours * 3600.0)
}

/// Extract tracking ID from filename if present (e.g., "00042_my-plan.md" -> Some(42))
fn extract_tracking_id(filename: &str) -> Option<u32> {
    let stem = filename.strip_suffix(".md")?;
    let parts: Vec<&str> = stem.splitn(2, '_').collect();
    if !parts.is_empty() && parts[0].len() == 5 && parts[0].chars().all(|c| c.is_ascii_digit()) {
        parts[0].parse().ok()
    } else {
        None
    }
}

/// Extract first heading or summary from plan content
fn extract_summary(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            return trimmed.trim_start_matches('#').trim().to_string();
        }
    }
    content
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(|l| {
            if l.len() > 80 {
                format!("{}...", &l[..77])
            } else {
                l.to_string()
            }
        })
        .unwrap_or_else(|| "Untitled plan".to_string())
}

/// Extract objectives section if present
fn extract_objectives(content: &str) -> Vec<String> {
    let mut in_objectives = false;
    let mut objectives = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        let lower = trimmed.to_lowercase();
        if lower.contains("objective") && trimmed.starts_with('#') {
            in_objectives = true;
            continue;
        }
        if in_objectives {
            if trimmed.starts_with('#') {
                break;
            }
            if trimmed.starts_with('-') || trimmed.starts_with('*') {
                objectives.push(trimmed[1..].trim().to_string());
            }
        }
    }

    objectives.into_iter().take(3).collect()
}

/// Plan entry with metadata
struct PlanEntry {
    id: Option<u32>,
    path: PathBuf,
    modified: f64,
    summary: String,
}

fn find_recent_plans(hours: f64) -> Vec<PlanEntry> {
    let dir = plans_dir();
    if !dir.exists() {
        return Vec::new();
    }

    let mut plans: Vec<_> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .filter(|e| {
            // Skip archived directory
            !e.path()
                .parent()
                .is_some_and(|p| p.file_name().is_some_and(|n| n == "archived"))
        })
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
            let id = extract_tracking_id(filename);
            let content = fs::read_to_string(&path).ok()?;
            let summary = extract_summary(&content);

            Some(PlanEntry {
                id,
                path,
                modified,
                summary,
            })
        })
        .collect();

    // Sort by: tracked plans first (by ID desc), then untracked by modified time
    plans.sort_by(|a, b| match (a.id, b.id) {
        (Some(id_a), Some(id_b)) => id_b.cmp(&id_a),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => b
            .modified
            .partial_cmp(&a.modified)
            .unwrap_or(std::cmp::Ordering::Equal),
    });

    plans
}

fn main() {
    let _input = match read_input() {
        Some(i) => i,
        None => exit_skip_prompt(),
    };

    let recent_plans = find_recent_plans(24.0);

    if recent_plans.is_empty() {
        exit_skip_prompt();
    }

    let mut context = String::new();
    context.push_str("📋 **EXISTING PLANS DETECTED**\n\n");

    if let Some(plan) = recent_plans.first() {
        context.push_str("**Most Recent Plan:**\n");

        if let Some(id) = plan.id {
            context.push_str(&format!("📄 #{:05} `{}`\n", id, plan.path.display()));
        } else {
            context.push_str(&format!("📄 `{}`\n", plan.path.display()));
        }
        context.push_str(&format!("   {}\n\n", plan.summary));

        if let Ok(content) = fs::read_to_string(&plan.path) {
            let objectives = extract_objectives(&content);
            if !objectives.is_empty() {
                context.push_str("**Key Objectives:**\n");
                for obj in &objectives {
                    context.push_str(&format!("  • {}\n", obj));
                }
                context.push('\n');
            }
        }

        context.push_str("**Action:** Read this plan to continue previous work.\n");
    }

    if recent_plans.len() > 1 {
        context.push_str("\n**Other Recent Plans:**\n");
        for plan in recent_plans.iter().skip(1).take(3) {
            let filename = plan
                .path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if let Some(id) = plan.id {
                context.push_str(&format!("  • #{:05} {} - {}\n", id, filename, plan.summary));
            } else {
                context.push_str(&format!("  • {} - {}\n", filename, plan.summary));
            }
        }
    }

    exit_with_context(&context);
}
