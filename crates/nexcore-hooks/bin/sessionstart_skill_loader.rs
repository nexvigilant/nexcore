//! Skill Loader Hook
//!
//! SessionStart hook that syncs skills from marketplace to local directory.
//!
//! # Event
//! SessionStart
//!
//! # Purpose
//! Syncs skills from ~/nexcore-plugins/ to ~/.claude/skills/ at session start.
//!
//! # Behavior
//! - Wipes skills directory clean and loads fresh copies
//! - Creates manifest to track marketplace vs local modifications
//! - Safety Axiom: A1 (Conservation of Intent) - preserves skill definitions
//!
//! # Exit Codes
//! - 0: Skills loaded successfully

use nexcore_hooks::{exit_skip_session, exit_with_session_context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Manifest tracking skill sources
#[derive(Serialize, Deserialize, Default)]
struct SkillManifest {
    version: String,
    loaded_at: String,
    skills: HashMap<String, SkillEntry>,
}

#[derive(Serialize, Deserialize)]
struct SkillEntry {
    source: String,
    source_path: String,
    loaded_at: String,
}

fn main() {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => exit_skip_session(),
    };

    let plugins_dir = home.join(".nexcore-plugins");
    let skills_dir = home.join(".claude/skills");

    // Skip if no plugins directory
    if !plugins_dir.exists() {
        exit_skip_session();
    }

    // Wipe and recreate skills directory
    if let Err(e) = wipe_skills_dir(&skills_dir) {
        let ctx = format!("⚠️ Failed to clean skills directory: {e}");
        exit_with_session_context(&ctx);
    }

    // Load skills from plugins
    let (loaded, manifest) = match load_skills(&plugins_dir, &skills_dir) {
        Ok(result) => result,
        Err(e) => {
            let ctx = format!("⚠️ Failed to load skills: {e}");
            exit_with_session_context(&ctx);
        }
    };

    // Write manifest
    let manifest_path = skills_dir.join(".manifest.json");
    if let Ok(json) = serde_json::to_string_pretty(&manifest) {
        let _ = fs::write(&manifest_path, json);
    }

    if loaded > 0 {
        let ctx = format!(
            "✅ **SKILLS LOADED** ─────────────────────────────────\n\
             Synced {loaded} skills from marketplace to ~/.claude/skills/\n\
             Manifest: ~/.claude/skills/.manifest.json\n\
             ───────────────────────────────────────────────────────\n"
        );
        exit_with_session_context(&ctx);
    }

    exit_skip_session();
}

/// Wipe the skills directory clean
fn wipe_skills_dir(skills_dir: &Path) -> std::io::Result<()> {
    if skills_dir.exists() {
        // Remove all contents
        for entry in fs::read_dir(skills_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip hidden files/dirs
            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with('.'))
            {
                continue;
            }

            if path.is_dir() || path.is_symlink() {
                if path.is_symlink() {
                    fs::remove_file(&path)?;
                } else {
                    fs::remove_dir_all(&path)?;
                }
            } else {
                fs::remove_file(&path)?;
            }
        }
    } else {
        fs::create_dir_all(skills_dir)?;
    }

    Ok(())
}

/// Load skills from plugins directory to skills directory
fn load_skills(plugins_dir: &Path, skills_dir: &Path) -> std::io::Result<(usize, SkillManifest)> {
    let mut count = 0;
    let timestamp = chrono::Utc::now().to_rfc3339();
    let mut manifest = SkillManifest {
        version: "1.0".to_string(),
        loaded_at: timestamp.clone(),
        skills: HashMap::new(),
    };

    for entry in fs::read_dir(plugins_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Only process skill-* directories
        let plugin_name = match path.file_name().and_then(|n| n.to_str()) {
            Some(n) if n.starts_with("skill-") => n,
            _ => continue,
        };

        if !path.is_dir() {
            continue;
        }

        // Find all skills within this plugin
        let skills_found = find_all_skills(&path);

        for (skill_name, skill_path) in skills_found {
            // Skip if already loaded (dedup by skill name)
            if manifest.skills.contains_key(&skill_name) {
                continue;
            }

            let dest = skills_dir.join(&skill_name);
            if let Err(_e) = copy_dir_recursive(&skill_path, &dest) {
                continue;
            }

            manifest.skills.insert(
                skill_name.clone(),
                SkillEntry {
                    source: "marketplace".to_string(),
                    source_path: format!("{} ({})", plugin_name, skill_path.display()),
                    loaded_at: timestamp.clone(),
                },
            );

            count += 1;
        }
    }

    Ok((count, manifest))
}

/// Find all skills within a plugin directory
fn find_all_skills(plugin_dir: &Path) -> Vec<(String, PathBuf)> {
    let mut skills = Vec::new();

    // Pattern 1: plugin_dir/skills/*/SKILL.md (most common)
    let skills_subdir = plugin_dir.join("skills");
    if skills_subdir.is_dir() {
        if let Ok(entries) = fs::read_dir(&skills_subdir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() && path.join("SKILL.md").exists() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        skills.push((name.to_string(), path));
                    }
                }
            }
        }
    }

    // Pattern 2: plugin_dir/SKILL.md (skill at root)
    if plugin_dir.join("SKILL.md").exists() {
        if let Some(name) = plugin_dir.file_name().and_then(|n| n.to_str()) {
            let skill_name = name.strip_prefix("skill-").unwrap_or(name);
            skills.push((skill_name.to_string(), plugin_dir.to_path_buf()));
        }
    }

    skills
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());

        // Skip hidden files, target dirs, and .git
        let name = entry.file_name();
        let name_str = name.to_str().unwrap_or("");
        if name_str.starts_with('.') || name_str == "target" {
            continue;
        }

        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }

    Ok(())
}
