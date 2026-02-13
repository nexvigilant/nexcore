//! Stop hook: Creates atomic improvement actions via molecular bonding
//!
//! Event: Stop
//!
//! Instead of blocking session end, this hook creates actionable molecular bonds
//! that represent capability improvements. Each atomic action has:
//!
//! - **Cause**: Current state (missing feature, outdated version, etc.)
//! - **Effect**: Improved state (what the capability gains)
//! - **Bond Type**: Ionic (one-way improvement transfer)
//! - **Catalyst**: This hook (stop_capability_improver)
//! - **Activation Energy**: Effort required (1-100)
//!
//! Actions are written to `~/.claude/bonds/pending_actions.json` for pickup
//! by SessionStart or manual execution.
//!
//! Exit codes:
//! - 0: Allow (contribution marker exists)
//! - 2: Block until contribution is made (atomic action queued)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Always allow session end — never block with exit 2.
    // Suggestions are advisory (exit 1 = warn).

    // Check if contribution marker exists
    let marker_path = contribution_marker_path();
    if marker_path.exists() {
        std::process::exit(0);
    }

    // Find ONE suggestion (if any) and warn — never block
    if let Some(suggestion) = find_improvement_opportunity() {
        let action = AtomicAction::from_suggestion(&suggestion);

        // Dedup: skip if this target already has a pending action
        if is_already_pending(&action.cause.target) {
            std::process::exit(0);
        }

        // Best-effort queue and document
        if let Err(e) = queue_atomic_action(&action) {
            eprintln!("⚠️  Failed to queue action: {e}");
        }
        if let Err(e) = write_action_reference(&action) {
            eprintln!("⚠️  Failed to write reference: {e}");
        }

        // Create contribution marker so subsequent invocations exit 0
        if let Err(e) = create_contribution_marker() {
            eprintln!("⚠️  Failed to create marker: {e}");
        }

        eprintln!("\n💡 **Improvement suggestion** ─────────────────────────────────");
        eprintln!("   {}: {}", action.cause.target, action.cause.description);
        eprintln!("   Queued to: ~/.claude/bonds/pending_actions.json");
        eprintln!("───────────────────────────────────────────────────────────────\n");

        // Warn only — never block session end
        std::process::exit(1);
    }

    // No suggestions — all good
    std::process::exit(0);
}

fn contribution_marker_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home)
        .join(".claude")
        .join(".contribution_marker")
}

/// Create the contribution marker so this hook exits 0 on subsequent calls.
fn create_contribution_marker() -> io::Result<()> {
    let path = contribution_marker_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, "marked")
}

/// An atomic action with cause-effect molecular bond
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AtomicAction {
    /// Unique bond identifier
    bond_id: String,
    /// Timestamp of creation
    created_at: u64,
    /// Bond type (always ionic for improvements)
    bond_type: String,
    /// Catalyst hook that created this action
    catalyst: String,
    /// Activation energy required (1-100)
    activation_energy: u8,
    /// Cause (reactant state)
    cause: CauseState,
    /// Effect (product state)
    effect: EffectState,
    /// Context transferred through the bond
    context_transfer: Vec<String>,
    /// Status
    status: ActionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CauseState {
    /// Type of capability (Skill, Hook, MCP, Subagent)
    capability_type: String,
    /// Target name
    target: String,
    /// Description of current state
    description: String,
    /// Path to the target
    path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EffectState {
    /// What improvement this creates
    improvement: String,
    /// Detailed description
    description: String,
    /// Quick action command
    quick_action: String,
    /// MCP tool to invoke (if applicable)
    mcp_tool: Option<String>,
    /// MCP tool arguments
    mcp_args: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum ActionStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
}

impl AtomicAction {
    fn from_suggestion(suggestion: &ImprovementSuggestion) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let bond_id = format!(
            "imp-{}-{}-{}",
            suggestion.capability_type.to_lowercase(),
            suggestion.target.replace('/', "-").replace(' ', "-"),
            timestamp % 10000
        );

        // Calculate activation energy based on type and complexity
        let activation_energy = match suggestion.capability_type.as_str() {
            "Skill" => 15,    // Skills are easy to update
            "Hook" => 35,     // Hooks require more care
            "MCP" => 50,      // MCP tools need integration work
            "Subagent" => 25, // Subagents are moderate
            _ => 30,
        };

        // Determine MCP tool for automated execution
        let (mcp_tool, mcp_args) = match suggestion.capability_type.as_str() {
            "Skill" => (
                Some("mcp__nexcore__skill_validate".to_string()),
                Some({
                    let mut args = HashMap::new();
                    args.insert(
                        "path".to_string(),
                        suggestion.path.clone().unwrap_or_default(),
                    );
                    args
                }),
            ),
            _ => (None, None),
        };

        Self {
            bond_id,
            created_at: timestamp,
            bond_type: "ionic".to_string(),
            catalyst: "stop_capability_improver".to_string(),
            activation_energy,
            cause: CauseState {
                capability_type: suggestion.capability_type.clone(),
                target: suggestion.target.clone(),
                description: suggestion.description.clone(),
                path: suggestion.path.clone(),
            },
            effect: EffectState {
                improvement: format!("{} enhancement", suggestion.capability_type),
                description: format!(
                    "Improve {} by: {}",
                    suggestion.target, suggestion.description
                ),
                quick_action: suggestion.quick_action.clone(),
                mcp_tool,
                mcp_args,
            },
            context_transfer: vec![
                "capability_path".to_string(),
                "improvement_type".to_string(),
                "validation_context".to_string(),
            ],
            status: ActionStatus::Pending,
        }
    }
}

/// Queue the atomic action to pending_actions.json
fn queue_atomic_action(action: &AtomicAction) -> io::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let bonds_dir = PathBuf::from(&home).join(".claude/bonds");
    let pending_path = bonds_dir.join("pending_actions.json");

    // Ensure directory exists
    fs::create_dir_all(&bonds_dir)?;

    // Load existing actions or create new
    let mut actions: PendingActions = if pending_path.exists() {
        let content = fs::read_to_string(&pending_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        PendingActions::default()
    };

    // Add the new action
    actions.actions.push(action.clone());
    actions.last_updated = action.created_at;

    // Write back
    let json = serde_json::to_string_pretty(&actions)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(&pending_path, json)?;

    Ok(())
}

/// Check if a target already has a pending action (dedup guard).
fn is_already_pending(target: &str) -> bool {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let pending_path = PathBuf::from(&home).join(".claude/bonds/pending_actions.json");

    if !pending_path.exists() {
        return false;
    }

    let content = match fs::read_to_string(&pending_path) {
        Ok(c) => c,
        Err(_) => return false,
    };

    let actions: PendingActions = match serde_json::from_str(&content) {
        Ok(a) => a,
        Err(_) => return false,
    };

    actions
        .actions
        .iter()
        .any(|a| matches!(a.status, ActionStatus::Pending) && a.cause.target == target)
}

/// Write reference documentation for the action
fn write_action_reference(action: &AtomicAction) -> io::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let refs_dir = PathBuf::from(&home).join(".claude/bonds/references");

    // Ensure directory exists
    fs::create_dir_all(&refs_dir)?;

    let ref_path = refs_dir.join(format!("{}.md", action.bond_id));

    let content = format!(
        r#"# Atomic Action: {}

## Bond Properties

| Property | Value |
|----------|-------|
| **Bond ID** | `{}` |
| **Type** | {} (one-way improvement transfer) |
| **Catalyst** | `{}` |
| **Activation Energy** | {}/100 |
| **Created** | {} |
| **Status** | {:?} |

## Cause (Reactant State)

**{}**: `{}`

{}

Path: `{}`

## Effect (Product State)

**Improvement**: {}

{}

### Quick Action

```
{}
```

{}

## Context Transfer

The following context passes through this bond:

{}

## Execution

To execute this atomic action:

1. **Manual**: Follow the quick action above
2. **Automated**: This action will be picked up by `SessionStart` hooks
3. **MCP**: {}

## Molecular Equation

```
[{} (current)] + catalyst:{} → [{} (improved)] + validation_result
```

---
*Generated by stop_capability_improver at {}*
"#,
        action.bond_id,
        action.bond_id,
        action.bond_type,
        action.catalyst,
        action.activation_energy,
        action.created_at,
        action.status,
        action.cause.capability_type,
        action.cause.target,
        action.cause.description,
        action.cause.path.as_deref().unwrap_or("N/A"),
        action.effect.improvement,
        action.effect.description,
        action.effect.quick_action,
        action
            .effect
            .mcp_tool
            .as_ref()
            .map(|t| format!("\n### MCP Tool\n\n`{}`", t))
            .unwrap_or_default(),
        action
            .context_transfer
            .iter()
            .map(|c| format!("- `{}`", c))
            .collect::<Vec<_>>()
            .join("\n"),
        action
            .effect
            .mcp_tool
            .as_ref()
            .map(|t| format!("Invoke `{}` with appropriate args", t))
            .unwrap_or_else(|| "No automated MCP tool available".to_string()),
        action.cause.target,
        action.catalyst,
        action.cause.target,
        action.created_at,
    );

    fs::write(&ref_path, content)?;

    Ok(())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PendingActions {
    version: String,
    last_updated: u64,
    actions: Vec<AtomicAction>,
}

impl Default for ActionStatus {
    fn default() -> Self {
        ActionStatus::Pending
    }
}

struct ImprovementSuggestion {
    capability_type: String,
    target: String,
    description: String,
    quick_action: String,
    path: Option<String>,
}

/// Minimum doc line threshold for hooks (avoids false positives for well-documented hooks)
const MIN_HOOK_DOC_LINES: usize = 15;

/// Path to the capability queue file (round-robin order)
fn queue_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude/bonds/.capability_queue.json")
}

/// Capability queue for fair round-robin selection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CapabilityQueue {
    /// All capabilities in priority order (first = next to suggest)
    queue: Vec<CapabilityEntry>,
    /// Last update timestamp
    last_updated: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CapabilityEntry {
    /// Capability type: Skill, Hook, MCP, Subagent
    cap_type: String,
    /// Target name
    name: String,
    /// Path to the capability
    path: String,
    /// Times suggested (for fairness tracking)
    times_suggested: u32,
}

/// Load or build the capability queue
fn load_or_build_queue() -> CapabilityQueue {
    let path = queue_path();

    // Try to load existing queue
    if path.exists() {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(queue) = serde_json::from_str::<CapabilityQueue>(&content) {
                // Refresh queue if older than 1 hour (3600 seconds)
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                if now - queue.last_updated < 3600 {
                    return queue;
                }
            }
        }
    }

    // Build fresh queue by scanning all capabilities
    build_capability_queue()
}

/// Scan all capabilities and build a queue sorted by times_suggested (fairness)
fn build_capability_queue() -> CapabilityQueue {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let mut entries = Vec::new();

    // Scan skills
    let skills_dir = PathBuf::from(&home).join(".claude/skills");
    if let Ok(dir) = fs::read_dir(&skills_dir) {
        for entry in dir.flatten() {
            let skill_md = entry.path().join("SKILL.md");
            if skill_md.exists() {
                entries.push(CapabilityEntry {
                    cap_type: "Skill".to_string(),
                    name: entry.file_name().to_string_lossy().to_string(),
                    path: entry.path().to_string_lossy().to_string(),
                    times_suggested: 0,
                });
            }
        }
    }

    // Scan hooks
    let hooks_dir = PathBuf::from(&home).join("nexcore/crates/nexcore-hooks/bin");
    if let Ok(dir) = fs::read_dir(&hooks_dir) {
        for entry in dir.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "rs").unwrap_or(false) {
                entries.push(CapabilityEntry {
                    cap_type: "Hook".to_string(),
                    name: path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    path: path.to_string_lossy().to_string(),
                    times_suggested: 0,
                });
            }
        }
    }

    // Scan subagents
    let agents_dir = PathBuf::from(&home).join(".config/agents");
    if let Ok(dir) = fs::read_dir(&agents_dir) {
        for entry in dir.flatten() {
            let path = entry.path();
            if path
                .extension()
                .map(|e| e == "yaml" || e == "json")
                .unwrap_or(false)
            {
                entries.push(CapabilityEntry {
                    cap_type: "Subagent".to_string(),
                    name: path
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string(),
                    path: path.to_string_lossy().to_string(),
                    times_suggested: 0,
                });
            }
        }
    }

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    CapabilityQueue {
        queue: entries,
        last_updated: now,
    }
}

/// Save the queue after modification (best-effort, non-blocking)
fn save_queue(queue: &CapabilityQueue) {
    let path = queue_path();
    if let Some(parent) = path.parent() {
        // Best-effort: queue save is non-critical for hook operation
        if fs::create_dir_all(parent).is_err() {
            return;
        }
    }
    if let Ok(json) = serde_json::to_string_pretty(queue) {
        // Best-effort: failing to persist queue just means we re-scan next time
        if fs::write(&path, json).is_err() {
            eprintln!("⚠️  Failed to save capability queue (will rebuild on next run)");
        }
    }
}

fn find_improvement_opportunity() -> Option<ImprovementSuggestion> {
    let mut queue = load_or_build_queue();

    if queue.queue.is_empty() {
        return None;
    }

    // Sort by times_suggested (ascending) to ensure fairness
    queue.queue.sort_by_key(|e| e.times_suggested);

    // Find first capability that needs improvement (by index to avoid borrow issues)
    let mut found_idx: Option<usize> = None;
    let mut suggestion: Option<ImprovementSuggestion> = None;

    for (idx, entry) in queue.queue.iter().enumerate() {
        if let Some(s) = check_capability_needs_improvement(entry) {
            found_idx = Some(idx);
            suggestion = Some(s);
            break;
        }
    }

    // If found, update count and save
    if let (Some(idx), Some(s)) = (found_idx, suggestion) {
        queue.queue[idx].times_suggested += 1;
        queue.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        save_queue(&queue);
        return Some(s);
    }

    // All capabilities are good — no suggestion needed
    None
}

/// Check if a specific capability needs improvement
fn check_capability_needs_improvement(entry: &CapabilityEntry) -> Option<ImprovementSuggestion> {
    match entry.cap_type.as_str() {
        "Skill" => check_skill_improvement(&entry.name, &entry.path),
        "Hook" => check_hook_improvement(&entry.name, &entry.path),
        "Subagent" => check_subagent_improvement(&entry.name, &entry.path),
        _ => None,
    }
}

/// Check a specific skill for improvement opportunities
/// Requires BOTH documentation AND functional tools
fn check_skill_improvement(name: &str, path: &str) -> Option<ImprovementSuggestion> {
    let skill_path = PathBuf::from(path);
    let skill_md = skill_path.join("SKILL.md");
    let scripts_dir = skill_path.join("scripts");
    let content = fs::read_to_string(&skill_md).ok()?;

    // Check for missing scripts directory (no functional tools)
    let has_scripts = scripts_dir.exists()
        && fs::read_dir(&scripts_dir)
            .map(|dir| dir.count() > 0)
            .unwrap_or(false);

    // Check for missing Quick Start
    let has_quick_start =
        content.contains("## Quick Start") || content.contains("## Quick Reference");

    // Priority 1: Missing both tool AND documentation
    if !has_scripts && !has_quick_start {
        return Some(ImprovementSuggestion {
            capability_type: "Skill".to_string(),
            target: name.to_string(),
            description: "Missing functional tools AND Quick Start documentation".to_string(),
            quick_action: format!(
                "1. Create {}/scripts/ with executable tool(s)\n2. Add '## Quick Start' section to SKILL.md",
                name
            ),
            path: Some(path.to_string()),
        });
    }

    // Priority 2: Has docs but missing tools
    if !has_scripts {
        return Some(ImprovementSuggestion {
            capability_type: "Skill".to_string(),
            target: name.to_string(),
            description: "Missing functional tools in scripts/ directory".to_string(),
            quick_action: format!(
                "Create {}/scripts/ with executable tool(s) - documentation-only skills need implementation",
                name
            ),
            path: Some(path.to_string()),
        });
    }

    // Priority 3: Has tools but missing Quick Start
    if !has_quick_start {
        return Some(ImprovementSuggestion {
            capability_type: "Skill".to_string(),
            target: name.to_string(),
            description: "Has tools but missing Quick Start documentation".to_string(),
            quick_action: format!("Add '## Quick Start' section to {}/SKILL.md", name),
            path: Some(path.to_string()),
        });
    }

    // Priority 4: Version bump (only if both tool and docs exist)
    if content.contains("version: 0.") {
        return Some(ImprovementSuggestion {
            capability_type: "Skill".to_string(),
            target: name.to_string(),
            description: "Skill version could be bumped with improvements".to_string(),
            quick_action: format!("Bump version and enhance {}/SKILL.md", name),
            path: Some(path.to_string()),
        });
    }

    None
}

/// Check a specific hook for improvement opportunities
fn check_hook_improvement(name: &str, path: &str) -> Option<ImprovementSuggestion> {
    let content = fs::read_to_string(path).ok()?;

    // Count doc lines
    let doc_lines = content
        .lines()
        .filter(|l| l.trim_start().starts_with("//!"))
        .count();

    if doc_lines < MIN_HOOK_DOC_LINES {
        return Some(ImprovementSuggestion {
            capability_type: "Hook".to_string(),
            target: name.to_string(),
            description: format!(
                "Hook has {} doc lines (minimum: {})",
                doc_lines, MIN_HOOK_DOC_LINES
            ),
            quick_action: format!("Add detailed //! doc comments to {}.rs", name),
            path: Some(path.to_string()),
        });
    }

    None
}

/// Check a specific subagent for improvement opportunities
fn check_subagent_improvement(name: &str, path: &str) -> Option<ImprovementSuggestion> {
    let content = fs::read_to_string(path).ok()?;

    // Check for minimal tools list
    if !content.contains("tools:") || content.lines().filter(|l| l.contains("mcp__")).count() < 3 {
        return Some(ImprovementSuggestion {
            capability_type: "Subagent".to_string(),
            target: name.to_string(),
            description: "Subagent has minimal tool configuration".to_string(),
            quick_action: format!("Add more MCP tools to {} agent", name),
            path: Some(path.to_string()),
        });
    }

    // Check for missing description
    if content.len() < 500 {
        return Some(ImprovementSuggestion {
            capability_type: "Subagent".to_string(),
            target: name.to_string(),
            description: "Subagent prompt could be more detailed".to_string(),
            quick_action: format!("Enhance {} agent prompt with more guidance", name),
            path: Some(path.to_string()),
        });
    }

    None
}
