//! Session Handoff Generator - Stop Event
//!
//! Generates the final session handoff document with full details.
//! Uses tracking registry for consistent ID numbering (00001.md, 00002.md, etc.)
//!
//! This runs at session end and produces a comprehensive handoff that includes:
//! - Session summary and metrics
//! - Active plan reference
//! - All commits made during session
//! - Uncommitted changes
//! - Unverified assumptions / blockers
//! - Requirements and scope
//! - Next steps

use nexcore_hooks::paths::{handoffs_dir, plans_dir};
use nexcore_hooks::state::{SessionState, now};
use nexcore_hooks::tracking::TrackingRegistry;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn get_uncommitted_changes() -> Vec<String> {
    Command::new("git")
        .args(["status", "--porcelain"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn get_recent_commits(since_timestamp: f64) -> Vec<String> {
    let since = chrono::DateTime::from_timestamp(since_timestamp as i64, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
        .unwrap_or_else(|| "1970-01-01".to_string());

    Command::new("git")
        .args(["log", "--oneline", "--since", &since])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn find_active_plan(session_start: f64) -> Option<(PathBuf, String)> {
    let dir = plans_dir();
    if !dir.exists() {
        return None;
    }

    let mut plans: Vec<_> = fs::read_dir(&dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "md"))
        .filter_map(|e| {
            let modified = e
                .metadata()
                .ok()?
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs_f64();
            if modified >= session_start {
                Some((e.path(), modified))
            } else {
                None
            }
        })
        .collect();

    plans.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    plans.first().and_then(|(path, _)| {
        let content = fs::read_to_string(path).ok()?;
        let summary = content
            .lines()
            .find(|l| l.starts_with('#'))
            .map(|l| l.trim_start_matches('#').trim().to_string())
            .unwrap_or_else(|| content.chars().take(100).collect::<String>() + "...");
        Some((path.clone(), summary))
    })
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

fn main() {
    let state = SessionState::load();

    // Read stdin for hook input
    let mut buffer = String::new();
    let _ = std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer);
    let input: serde_json::Value = serde_json::from_str(&buffer).unwrap_or_default();

    let session_id = input
        .get("session_id")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Calculate duration with sanity check (if session_start is 0 or invalid, use 0)
    let duration = if state.session_start > 0.0 && state.session_start < now() {
        now() - state.session_start
    } else {
        0.0 // Invalid session_start, show as 0
    };

    // Load registry and get/create handoff ID
    let mut registry = TrackingRegistry::load();
    let handoff_id = registry.get_or_create_handoff_id(session_id);

    // Gather session data
    let uncommitted = get_uncommitted_changes();
    let recent_commits = get_recent_commits(state.session_start);
    let active_plan = find_active_plan(state.session_start);
    // Deduplicate assumptions using BTreeSet (preserves order)
    let unverified: Vec<_> = state
        .assumptions
        .iter()
        .filter(|a| a.status == "assumed")
        .map(|a| format!("{} ({})", a.assumption, a.confidence))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();

    // Build comprehensive handoff document
    let mut handoff = String::new();

    handoff.push_str(&format!(
        "# Handoff #{:05} [FINAL]\n\
         Session: {}\n\
         Generated: {}\n\
         Status: **Session Complete**\n\n",
        handoff_id,
        session_id,
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    ));

    // Session Summary
    handoff.push_str("## Session Summary\n");
    handoff.push_str(&format!("| Metric | Value |\n"));
    handoff.push_str(&format!("|--------|-------|\n"));
    handoff.push_str(&format!("| Duration | {} |\n", format_duration(duration)));
    handoff.push_str(&format!(
        "| Files modified | {} |\n",
        state.files_since_verification
    ));
    handoff.push_str(&format!(
        "| Lines written | {} |\n",
        state.lines_since_verification
    ));
    handoff.push_str(&format!("| Commits | {} |\n", recent_commits.len()));
    handoff.push_str(&format!("| Uncommitted | {} |\n", uncommitted.len()));
    handoff.push_str(&format!(
        "| Requirements verified | {} |\n\n",
        if state.requirements_verified {
            "✅"
        } else {
            "❌"
        }
    ));

    // Active Plan
    handoff.push_str("## Active Plan\n");
    if let Some((path, summary)) = &active_plan {
        handoff.push_str(&format!("📄 **{}**\n", path.display()));
        handoff.push_str(&format!("   {}\n\n", summary));
    } else {
        handoff.push_str("⚠️ No active plan found for this session\n\n");
    }

    // Commits this session
    handoff.push_str("## Commits This Session\n");
    if recent_commits.is_empty() {
        handoff.push_str("- None\n\n");
    } else {
        for commit in &recent_commits {
            handoff.push_str(&format!("- `{}`\n", commit));
        }
        handoff.push_str("\n");
    }

    // Uncommitted Changes
    handoff.push_str("## Uncommitted Changes\n");
    if uncommitted.is_empty() {
        handoff.push_str("✅ Working tree clean\n\n");
    } else {
        for (i, change) in uncommitted.iter().enumerate() {
            if i < 30 {
                handoff.push_str(&format!("- `{}`\n", change));
            }
        }
        if uncommitted.len() > 30 {
            handoff.push_str(&format!("- ...and {} more\n", uncommitted.len() - 30));
        }
        handoff.push_str("\n");
    }

    // Unverified Assumptions / Blockers
    handoff.push_str("## Unverified Assumptions / Blockers\n");
    if unverified.is_empty() {
        handoff.push_str("✅ No unverified assumptions\n\n");
    } else {
        for assumption in &unverified {
            handoff.push_str(&format!("- [ ] {}\n", assumption));
        }
        handoff.push_str("\n");
    }

    // Requirements
    if !state.explicit_requirements.is_empty() || !state.implicit_requirements.is_empty() {
        handoff.push_str("## Requirements\n");
        if !state.explicit_requirements.is_empty() {
            handoff.push_str("### Explicit\n");
            for req in &state.explicit_requirements {
                handoff.push_str(&format!("- {}\n", req));
            }
        }
        if !state.implicit_requirements.is_empty() {
            handoff.push_str("### Implicit\n");
            for req in &state.implicit_requirements {
                handoff.push_str(&format!("- {}\n", req));
            }
        }
        handoff.push_str("\n");
    }

    // Scope
    if !state.scope_in.is_empty() || !state.scope_out.is_empty() {
        handoff.push_str("## Scope\n");
        if !state.scope_in.is_empty() {
            handoff.push_str("### In Scope\n");
            for item in &state.scope_in {
                handoff.push_str(&format!("- {}\n", item));
            }
        }
        if !state.scope_out.is_empty() {
            handoff.push_str("### Out of Scope\n");
            for item in &state.scope_out {
                handoff.push_str(&format!("- {}\n", item));
            }
        }
        handoff.push_str("\n");
    }

    // Next Steps
    handoff.push_str("## Next Steps\n");
    if !uncommitted.is_empty() {
        handoff.push_str("1. ⚠️ Review and commit uncommitted changes\n");
    }
    if !unverified.is_empty() {
        handoff.push_str("2. 🔍 Verify unresolved assumptions\n");
    }
    if active_plan.is_some() {
        handoff.push_str("3. 📋 Continue with active plan\n");
    }
    handoff.push_str("\n---\n");
    handoff.push_str(&format!(
        "*Handoff #{:05} - Generated by session_handoff_generator*\n",
        handoff_id
    ));

    // Save handoff document
    let dir = handoffs_dir();
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(format!("{:05}.md", handoff_id));
    let saved = fs::write(&path, &handoff).is_ok();

    // Update registry
    registry.update_artifact(handoff_id);
    let _ = registry.save();

    let output = serde_json::json!({
        "continue": true,
        "decision": "approve",
        "stopReason": format!("Handoff #{:05} generated: {} commits, {} uncommitted",
            handoff_id, recent_commits.len(), uncommitted.len()),
        "systemMessage": if saved {
            format!("📄 Session handoff #{:05} saved to {}", handoff_id, path.display())
        } else {
            format!("⚠️ Failed to save handoff #{:05}", handoff_id)
        }
    });
    println!("{}", output);
}
