//! Bulk frontmatter writer for SKILL.md files.
//!
//! Writes missing frontmatter fields back to SKILL.md files on disk,
//! enabling automated metadata backfill across the skill ecosystem.

use std::path::Path;

use rusqlite::Connection;

use crate::error::{RegistryError, Result};

/// Add or update a single frontmatter field in a SKILL.md file.
///
/// If the file has no frontmatter (no `---` delimiters), one is created.
/// If the key already exists, it is updated in place.
///
/// # Errors
///
/// Returns an error if the file cannot be read or written.
pub fn set_frontmatter_field(path: &Path, key: &str, value: &str) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| RegistryError::Scan(format!("Cannot read {}: {e}", path.display())))?;

    let new_content = if let Some(rest) = content.strip_prefix("---") {
        // Has frontmatter — find the closing ---
        match rest.find("\n---") {
            Some(end_idx) => {
                let fm_block = &rest[..end_idx];
                let after = &rest[end_idx + 4..]; // skip \n---

                // Check if key already exists
                let key_prefix = format!("{key}:");
                let updated_fm = if fm_block.lines().any(|l| l.starts_with(&key_prefix)) {
                    // Replace existing line
                    fm_block
                        .lines()
                        .map(|line| {
                            if line.starts_with(&key_prefix) {
                                format!("{key}: {value}")
                            } else {
                                line.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                } else {
                    // Append new field
                    format!("{fm_block}\n{key}: {value}")
                };

                format!("---{updated_fm}\n---{after}")
            }
            None => {
                // Malformed frontmatter — add field at the top
                format!("---\n{key}: {value}\n---\n{content}")
            }
        }
    } else {
        // No frontmatter — create one
        format!("---\n{key}: {value}\n---\n{content}")
    };

    std::fs::write(path, new_content)
        .map_err(|e| RegistryError::Scan(format!("Cannot write {}: {e}", path.display())))?;

    Ok(())
}

/// Backfill tags for all skills missing them.
///
/// Derives tags from the skill's directory path (parent directory name).
/// Returns the number of skills updated.
///
/// # Errors
///
/// Returns an error on database or filesystem failure.
pub fn backfill_tags(conn: &Connection, skills_dir: &Path) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT name, path FROM active_skills WHERE tags IS NULL ORDER BY name",
    )?;

    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut updated = 0usize;
    for (name, file_path) in &rows {
        let tags = derive_tags_from_name(name);
        let p = Path::new(file_path);
        if !p.exists() {
            continue;
        }
        if set_frontmatter_field(p, "tags", &tags).is_ok() {
            // Update DB too
            let _ = conn.execute(
                "UPDATE active_skills SET tags = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE name = ?2",
                rusqlite::params![tags, name],
            );
            updated += 1;
        }
    }

    let _ = skills_dir; // used for context; actual paths come from DB
    Ok(updated)
}

/// Backfill version "1.0.0" for all skills missing a version.
///
/// Returns the number of skills updated.
///
/// # Errors
///
/// Returns an error on database or filesystem failure.
pub fn backfill_versions(conn: &Connection, _skills_dir: &Path) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT name, path FROM active_skills WHERE version IS NULL ORDER BY name",
    )?;

    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut updated = 0usize;
    for (name, file_path) in &rows {
        let p = Path::new(file_path);
        if !p.exists() {
            continue;
        }
        if set_frontmatter_field(p, "version", "1.0.0").is_ok() {
            let _ = conn.execute(
                "UPDATE active_skills SET version = '1.0.0', updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE name = ?1",
                rusqlite::params![name],
            );
            updated += 1;
        }
    }

    Ok(updated)
}

/// Backfill argument-hint for user-invocable skills missing it.
///
/// Uses a known mapping table for common skill names; defaults to `[args]`.
/// Returns the number of skills updated.
///
/// # Errors
///
/// Returns an error on database or filesystem failure.
pub fn backfill_argument_hints(conn: &Connection, _skills_dir: &Path) -> Result<usize> {
    let mut stmt = conn.prepare(
        "SELECT name, path FROM active_skills WHERE user_invocable = 1 AND argument_hint IS NULL ORDER BY name",
    )?;

    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<std::result::Result<Vec<_>, _>>()?;

    let mut updated = 0usize;
    for (name, file_path) in &rows {
        let hint = derive_argument_hint(name);
        let p = Path::new(file_path);
        if !p.exists() {
            continue;
        }
        if set_frontmatter_field(p, "argument-hint", &hint).is_ok() {
            let _ = conn.execute(
                "UPDATE active_skills SET argument_hint = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%SZ','now') WHERE name = ?2",
                rusqlite::params![hint, name],
            );
            updated += 1;
        }
    }

    Ok(updated)
}

/// Derive tags from skill name by splitting on hyphens and using the top-level directory.
fn derive_tags_from_name(name: &str) -> String {
    let parts: Vec<&str> = name.split('/').collect();
    let base = parts[0];

    // Split hyphenated name into tag words
    let words: Vec<&str> = base.split('-').collect();
    let tags: Vec<String> = words.iter().map(|w| format!("\"{w}\"")).collect();

    format!("[{}]", tags.join(", "))
}

/// Derive argument-hint from skill name using known patterns.
fn derive_argument_hint(name: &str) -> String {
    // Known skill-to-hint mappings
    let base = name.split('/').next().unwrap_or(name);
    match base {
        "commit" => "[message]".to_string(),
        "review-pr" => "[pr-number]".to_string(),
        "brain-reconcile" => "[--force]".to_string(),
        "brain-auditor" => "[--verbose]".to_string(),
        "team-composer" => "<TEAM-NAME>".to_string(),
        "parallel-fix" => "[target]".to_string(),
        "feature-dev" => "[feature-description]".to_string(),
        "playground" => "[topic]".to_string(),
        "hookify" => "[mistake-description]".to_string(),
        "learn-program" => "[source-url]".to_string(),
        "forge" => "[crate-name]".to_string(),
        "validify" => "[target]".to_string(),
        "scope-program" => "[project-path]".to_string(),
        "trace-program" => "[error-description]".to_string(),
        "pulse-program" => "[--full]".to_string(),
        "craft-program" => "[target]".to_string(),
        "guard-program" => "[crate-name]".to_string(),
        "domain-translator" => "<source-domain> <target-domain> <concept>".to_string(),
        _ => "[args]".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_derive_tags() {
        assert_eq!(derive_tags_from_name("rust-dev"), "[\"rust\", \"dev\"]");
        assert_eq!(derive_tags_from_name("brain-reconcile"), "[\"brain\", \"reconcile\"]");
        assert_eq!(
            derive_tags_from_name("rust-dev/cargo"),
            "[\"rust\", \"dev\"]"
        );
    }

    #[test]
    fn test_derive_argument_hint() {
        assert_eq!(derive_argument_hint("commit"), "[message]");
        assert_eq!(derive_argument_hint("review-pr"), "[pr-number]");
        assert_eq!(derive_argument_hint("unknown-skill"), "[args]");
    }

    #[test]
    fn test_set_frontmatter_field_new() {
        let dir = TempDir::new().ok();
        assert!(dir.is_some());
        let dir = dir.unwrap_or_else(|| unreachable!());
        let file = dir.path().join("SKILL.md");
        fs::write(&file, "# My Skill\nContent here.").ok();

        set_frontmatter_field(&file, "version", "1.0.0").ok();
        let content = fs::read_to_string(&file).unwrap_or_default();
        assert!(content.starts_with("---\nversion: 1.0.0\n---\n"));
        assert!(content.contains("# My Skill"));
    }

    #[test]
    fn test_set_frontmatter_field_existing() {
        let dir = TempDir::new().ok();
        assert!(dir.is_some());
        let dir = dir.unwrap_or_else(|| unreachable!());
        let file = dir.path().join("SKILL.md");
        fs::write(&file, "---\nname: test\ndescription: old\n---\n# Content").ok();

        set_frontmatter_field(&file, "version", "2.0.0").ok();
        let content = fs::read_to_string(&file).unwrap_or_default();
        assert!(content.contains("version: 2.0.0"));
        assert!(content.contains("name: test"));
    }

    #[test]
    fn test_set_frontmatter_field_update() {
        let dir = TempDir::new().ok();
        assert!(dir.is_some());
        let dir = dir.unwrap_or_else(|| unreachable!());
        let file = dir.path().join("SKILL.md");
        fs::write(&file, "---\nversion: 1.0.0\n---\n# Content").ok();

        set_frontmatter_field(&file, "version", "2.0.0").ok();
        let content = fs::read_to_string(&file).unwrap_or_default();
        assert!(content.contains("version: 2.0.0"));
        assert!(!content.contains("version: 1.0.0"));
    }
}
