//! Filesystem scanner for skills and agents.
//!
//! Walks `~/.claude/skills/` and `~/.claude/agents/`, parses YAML frontmatter,
//! and upserts into the database. Replaces the zsh `scan-skills.sh` script.

use std::path::{Path, PathBuf};

use chrono::Utc;
use nexcore_fs::walk::WalkDir;
use rusqlite::Connection;
use serde_json::Value as JsonValue;

use crate::agents::{self, AgentRow};
use crate::audit::{self, AuditRow};
use crate::error::{RegistryError, Result};
use crate::skills::{self, SkillRow};

/// Results of a full scan.
#[derive(Debug, Clone)]
pub struct ScanResult {
    /// Number of skills scanned
    pub skills_scanned: usize,
    /// Number of agents scanned
    pub agents_scanned: usize,
    /// Non-fatal errors encountered
    pub errors: Vec<String>,
}

/// Default skills directory.
fn default_skills_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude/skills")
}

/// Default agents directory.
fn default_agents_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude/agents")
}

/// Scan skills and agents using default directories.
///
/// # Errors
///
/// Returns an error if the database operations fail.
pub fn scan_all(conn: &Connection) -> Result<ScanResult> {
    let skills_dir = default_skills_dir();
    let agents_dir = default_agents_dir();

    let mut errors = Vec::new();

    let skills_scanned = match scan_skills(conn, &skills_dir) {
        Ok(n) => n,
        Err(e) => {
            errors.push(format!("Skills scan error: {e}"));
            0
        }
    };

    let agents_scanned = match scan_agents(conn, &agents_dir) {
        Ok(n) => n,
        Err(e) => {
            errors.push(format!("Agents scan error: {e}"));
            0
        }
    };

    Ok(ScanResult {
        skills_scanned,
        agents_scanned,
        errors,
    })
}

/// Scan a skills directory and upsert all found skills.
///
/// Walks the directory looking for `SKILL.md` files, parses YAML frontmatter,
/// and upserts each skill into the database.
///
/// # Errors
///
/// Returns an error if directory traversal or database operations fail.
pub fn scan_skills(conn: &Connection, skills_dir: &Path) -> Result<usize> {
    if !skills_dir.exists() {
        return Err(RegistryError::Scan(format!(
            "Skills directory not found: {}",
            skills_dir.display()
        )));
    }

    let now = Utc::now();
    let mut count = 0;

    for entry in WalkDir::new(skills_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let filename = entry.file_name().to_string_lossy();

        if filename != "SKILL.md" {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let line_count = i32::try_from(content.lines().count()).unwrap_or(i32::MAX);

        // Determine skill name from directory structure
        let rel = path.strip_prefix(skills_dir).unwrap_or(path);
        let components: Vec<&str> = rel
            .parent()
            .map(|p| {
                p.components()
                    .map(|c| c.as_os_str().to_str().unwrap_or(""))
                    .collect()
            })
            .unwrap_or_default();

        if components.is_empty() {
            continue;
        }

        let skill_name = components.join("/");
        let (parent_skill, is_sub) = if components.len() > 1 {
            (Some(components[0].to_string()), true)
        } else {
            (None, false)
        };

        // Parse frontmatter
        let fm = parse_frontmatter(&content);

        // Check for paired agent file
        let agent_path = skills_dir
            .parent()
            .unwrap_or(skills_dir)
            .join("agents")
            .join(format!("{}.md", components[0]));
        let has_agent = agent_path.exists();

        // Count sub-skills
        let sub_count = if !is_sub {
            count_sub_skills(skills_dir, components[0])
        } else {
            0
        };

        // --- Anthropic official frontmatter fields ---
        let argument_hint = fm
            .get("argument-hint")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Default: true per Anthropic spec
        let user_invocable = fm
            .get("user-invocable")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        // Default: false per Anthropic spec
        let disable_model_invocation = fm
            .get("disable-model-invocation")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let allowed_tools = fm.get("allowed-tools").map(|v| v.to_string());
        let model = fm.get("model").and_then(|v| v.as_str()).map(String::from);
        let context = fm.get("context").and_then(|v| v.as_str()).map(String::from);
        let agent = fm.get("agent").and_then(|v| v.as_str()).map(String::from);
        let hooks = fm.get("hooks").map(|v| v.to_string());

        // --- Runtime feature detection (scanned from content body) ---
        let content_chars = i32::try_from(content.len()).unwrap_or(i32::MAX);
        let uses_arguments =
            content.contains("$ARGUMENTS") || content.contains("$0") || content.contains("$1");
        let uses_dynamic_context = content.contains("!`");
        let uses_session_id = content.contains("${CLAUDE_SESSION_ID}");

        // --- NexVigilant KPI tracking fields ---
        let chain_position = fm
            .get("chain_position")
            .and_then(|v| v.as_str())
            .map(String::from);

        let pipeline = fm
            .get("pipeline")
            .and_then(|v| v.as_str())
            .map(String::from);

        // Tags: from frontmatter, or auto-derive from directory path
        let tags = fm.get("tags").map(|v| v.to_string()).or_else(|| {
            let parts: Vec<&str> = components[0].split('-').collect();
            let tag_list: Vec<String> = parts.iter().map(|p| format!("\"{p}\"")).collect();
            Some(format!("[{}]", tag_list.join(", ")))
        });

        // Version: from frontmatter, or default to "1.0.0"
        let version = fm
            .get("version")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| Some("1.0.0".to_string()));

        let row = SkillRow {
            // Anthropic official
            name: skill_name,
            path: path.to_string_lossy().to_string(),
            description: fm
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            argument_hint,
            disable_model_invocation,
            user_invocable,
            allowed_tools,
            model,
            context,
            agent,
            hooks,
            // Computed / operational
            line_count: Some(line_count),
            has_agent,
            sub_skill_count: sub_count,
            parent_skill,
            // Runtime feature detection
            uses_arguments,
            uses_dynamic_context,
            uses_session_id,
            content_chars: Some(content_chars),
            // SMST v2 component breakdown (populated by assess engine, not scanner)
            smst_input: None,
            smst_output: None,
            smst_logic: None,
            smst_error_handling: None,
            smst_examples: None,
            smst_references: None,
            last_assessed_at: None,
            assessed_by: None,
            // NexVigilant KPI tracking
            version,
            compliance: None,
            smst_v1: None,
            smst_v2: None,
            tags,
            chain_position,
            pipeline,
            // Timestamps
            scanned_at: now,
            updated_at: now,
        };

        if let Err(e) = skills::upsert(conn, &row) {
            tracing::warn!(skill = %row.name, error = %e, "Failed to upsert skill");
            continue;
        }

        // Audit trail
        let _ = audit::record(
            conn,
            &AuditRow {
                id: None,
                skill_name: row.name.clone(),
                action: "scanned".to_string(),
                details: None,
                actor: Some("scan".to_string()),
                created_at: now,
            },
        );

        count += 1;
    }

    Ok(count)
}

/// Scan an agents directory and upsert all found agents.
///
/// # Errors
///
/// Returns an error if directory traversal or database operations fail.
pub fn scan_agents(conn: &Connection, agents_dir: &Path) -> Result<usize> {
    if !agents_dir.exists() {
        return Err(RegistryError::Scan(format!(
            "Agents directory not found: {}",
            agents_dir.display()
        )));
    }

    let now = Utc::now();
    let mut count = 0;

    for entry in WalkDir::new(agents_dir)
        .max_depth(1)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("md") {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let agent_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        if agent_name.is_empty() {
            continue;
        }

        let fm = parse_frontmatter(&content);

        let row = AgentRow {
            name: agent_name,
            path: path.to_string_lossy().to_string(),
            model: fm.get("model").and_then(|v| v.as_str()).map(String::from),
            tools: fm.get("tools").map(|v| v.to_string()),
            description: fm
                .get("description")
                .and_then(|v| v.as_str())
                .map(String::from),
            paired_skill: fm
                .get("paired_skill")
                .or_else(|| fm.get("skill"))
                .and_then(|v| v.as_str())
                .map(String::from),
            scanned_at: now,
        };

        if let Err(e) = agents::upsert(conn, &row) {
            tracing::warn!(agent = %row.name, error = %e, "Failed to upsert agent");
            continue;
        }

        count += 1;
    }

    Ok(count)
}

/// Parse YAML frontmatter from a markdown file.
///
/// Returns an empty map if no valid frontmatter is found.
fn parse_frontmatter(content: &str) -> serde_json::Map<String, JsonValue> {
    if !content.starts_with("---") {
        return serde_json::Map::new();
    }

    let rest = &content[3..];
    let end_idx = match rest.find("\n---") {
        Some(i) => i,
        None => return serde_json::Map::new(),
    };

    let yaml_str = &rest[..end_idx];

    match serde_yaml::from_str::<JsonValue>(yaml_str) {
        Ok(JsonValue::Object(map)) => map,
        _ => serde_json::Map::new(),
    }
}

/// Count sub-skill directories for a top-level skill.
fn count_sub_skills(skills_dir: &Path, parent: &str) -> i32 {
    let parent_dir = skills_dir.join(parent);
    if !parent_dir.is_dir() {
        return 0;
    }

    let mut count = 0i32;
    for entry in WalkDir::new(&parent_dir)
        .min_depth(1)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name().to_string_lossy() == "SKILL.md" {
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter_valid() {
        let content = "---\nname: test\ndescription: A test skill\nversion: 1.0.0\nuser-invocable: true\n---\n# Content here";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.get("name").and_then(|v| v.as_str()), Some("test"));
        assert_eq!(
            fm.get("description").and_then(|v| v.as_str()),
            Some("A test skill")
        );
        assert_eq!(
            fm.get("user-invocable").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[test]
    fn test_parse_frontmatter_missing() {
        let content = "# No frontmatter\nJust content.";
        let fm = parse_frontmatter(content);
        assert!(fm.is_empty());
    }

    #[test]
    fn test_parse_frontmatter_invalid_yaml() {
        let content = "---\n: invalid: yaml: [[\n---\n# Content";
        let fm = parse_frontmatter(content);
        assert!(fm.is_empty());
    }

    #[test]
    fn test_parse_frontmatter_with_tools_array() {
        let content =
            "---\nname: agent\nmodel: sonnet\ntools:\n  - Read\n  - Grep\n  - Bash\n---\n# Agent";
        let fm = parse_frontmatter(content);
        assert_eq!(fm.get("model").and_then(|v| v.as_str()), Some("sonnet"));
        assert!(fm.get("tools").and_then(|v| v.as_array()).is_some());
    }
}
