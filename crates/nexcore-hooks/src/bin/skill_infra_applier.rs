//! Applies standardized Infrastructure section to all skills.
//!
//! Usage: skill_infra_applier [--dry-run]
//!
//! This tool ensures every skill has the same hooks and MCP tool references,
//! so prompts operate on consistent "variables" across the ecosystem.

use std::env;
use std::fs;
use std::path::PathBuf;

const INFRASTRUCTURE_SECTION: &str = r#"
---

## Infrastructure

This skill integrates with the Claude Code ecosystem through hooks, MCP tools, and vocabulary shorthands.

### Hooks (Enforcement Layer)

| Event | Hooks | Purpose |
|-------|-------|---------|
| **SessionStart** | `vocabulary_loader`, `brain_session_creator` | Load context, create session |
| **UserPromptSubmit** | `escape_word_detector`, `skill_invoker`, `mcp_suggester` | Security, auto-invoke, tool hints |
| **PreToolUse** | `primitive_pattern_validator`, `secret_scanner`, `panic_enforcer` | Block violations before execution |
| **PostToolUse** | `incremental_verifier`, `test_analyzer`, `skill_bonding_tracker` | Verify after execution |
| **Stop** | `completion_checker`, `handoff_generator` | Verify deliverables, continuity |

### MCP Tools (Computation Layer)

| Category | Tools | Prefix |
|----------|-------|--------|
| **Brain** | session_create, session_load, artifact_save, artifact_resolve, artifact_get, artifact_diff | `mcp__nexcore__brain_*` |
| **Code Tracking** | code_tracker_track, code_tracker_changed, code_tracker_original | `mcp__nexcore__code_tracker_*` |
| **Implicit** | implicit_get, implicit_set | `mcp__nexcore__implicit_*` |
| **Skills** | skill_list, skill_get, skill_validate, skill_scan, skill_search_by_tag | `mcp__nexcore__skill_*` |
| **Foundation** | foundation_levenshtein, foundation_fuzzy_search, foundation_sha256, foundation_yaml_parse, foundation_graph_topsort | `mcp__nexcore__foundation_*` |
| **Validation** | validation_run, validation_check, validation_domains | `mcp__nexcore__validation_*` |
| **Hooks** | hooks_stats, hooks_for_event, hooks_for_tier | `mcp__nexcore__hooks_*` |

### Vocabulary Shorthands

| Shorthand | Meaning | Use |
|-----------|---------|-----|
| `brain-session` | Artifact versioning with .resolved.N snapshots | Memory persistence |
| `hook-enforced` | 5 enforcement constraints active | Quality gates |
| `build-doctrine` | 100% Rust, NexCore MCP first, foundation before features | Policy |
| `ctvp-validated` | 5-phase validation, anti-mock-theater | Testing |
| `diamond-v2` | Full skill compliance (SMST ≥85%) | Quality |

### Integration Points

```
┌─────────────────────────────────────────────────────────────┐
│                     SKILL EXECUTION                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  SessionStart ──► Brain Session Created ──► Skill Loaded    │
│                         │                                    │
│                         ▼                                    │
│  UserPrompt ──► Hooks Validate ──► Skill Executes           │
│                         │                                    │
│                         ▼                                    │
│  PreToolUse ──► MCP Tools Called ──► PostToolUse            │
│                         │                                    │
│                         ▼                                    │
│  Stop ──► Artifacts Resolved ──► Session Persisted          │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Common Patterns

```bash
# Track work in Brain
mcp__nexcore__brain_artifact_save(name="task.md", content="...")
mcp__nexcore__brain_artifact_resolve(name="task.md")

# Track file changes
mcp__nexcore__code_tracker_track(path="src/lib.rs")

# Validate skill compliance
mcp__nexcore__skill_validate(path="~/.claude/skills/this-skill")

# Query hook status
mcp__nexcore__hooks_for_event(event="PreToolUse")
```
"#;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let dry_run = args.iter().any(|a| a == "--dry-run");

    let home = env::var("HOME")?;
    let skills_dir = PathBuf::from(&home).join(".claude/skills");

    if !skills_dir.exists() {
        return Err(format!("Skills directory not found: {:?}", skills_dir).into());
    }

    let mut updated = 0;
    let mut skipped = 0;
    let mut errors = 0;

    // Find all SKILL.md files
    for entry in fs::read_dir(&skills_dir)? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let skill_md = path.join("SKILL.md");
        if !skill_md.exists() {
            continue;
        }

        let content = match fs::read_to_string(&skill_md) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {:?}: {}", skill_md, e);
                errors += 1;
                continue;
            }
        };

        // Check if Infrastructure section already exists
        if content.contains("## Infrastructure") {
            println!("SKIP: {:?} (already has Infrastructure section)", skill_md);
            skipped += 1;
            continue;
        }

        // Find where to insert (before "## See Also" or at end)
        let new_content = if let Some(pos) = content.find("## See Also") {
            let (before, after) = content.split_at(pos);
            format!("{}{}\n{}", before.trim_end(), INFRASTRUCTURE_SECTION, after)
        } else {
            format!("{}\n{}", content.trim_end(), INFRASTRUCTURE_SECTION)
        };

        if dry_run {
            println!("WOULD UPDATE: {:?}", skill_md);
            updated += 1;
        } else {
            match fs::write(&skill_md, &new_content) {
                Ok(_) => {
                    println!("UPDATED: {:?}", skill_md);
                    updated += 1;
                }
                Err(e) => {
                    eprintln!("Error writing {:?}: {}", skill_md, e);
                    errors += 1;
                }
            }
        }
    }

    println!("\n=== Summary ===");
    println!("Updated: {}", updated);
    println!("Skipped: {}", skipped);
    println!("Errors:  {}", errors);
    if dry_run {
        println!("\n(Dry run - no files modified)");
    }

    Ok(())
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
