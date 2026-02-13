//! Skill Auto-Invoker Hook
//!
//! Event: UserPromptSubmit
//!
//! Detects "run <keywords>" patterns and auto-invokes matching skills.
//! Uses fuzzy matching to find the best skill match.
//!
//! Safety Axiom: A2 (Information Integrity) - ensures correct skill routing.

use nexcore_hooks::{exit_skip_prompt, exit_with_context, read_input};
use std::fs;
use std::path::Path;

/// Skill entry for matching
struct SkillEntry {
    name: String,
    slug: String,
    tags: Vec<String>,
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_skip_prompt(),
    };

    let prompt = match input.get_prompt() {
        Some(p) => p.trim(),
        None => exit_skip_prompt(),
    };

    // Check for "run" prefix (case-insensitive)
    let lower = prompt.to_lowercase();
    if !lower.starts_with("run ") && !lower.starts_with("run\t") {
        exit_skip_prompt();
    }

    // Extract keywords after "run"
    let keywords = prompt[4..].trim();
    if keywords.is_empty() {
        exit_skip_prompt();
    }

    // Load available skills
    let skills = load_skills();
    if skills.is_empty() {
        exit_skip_prompt();
    }

    // Find best match
    if let Some((skill, score)) = find_best_match(keywords, &skills) {
        if score > 0.3 {
            let context = format!(
                "🎯 **SKILL AUTO-INVOKE** ─────────────────────────────\n\
                 Matched: `{}` (score: {:.0}%)\n\
                 \n\
                 Invoke with: `/{}`\n\
                 ───────────────────────────────────────────────────────\n",
                skill.name,
                score * 100.0,
                skill.slug
            );
            exit_with_context(&context);
        }
    }

    exit_skip_prompt();
}

/// Load skills from ~/.claude/skills/
fn load_skills() -> Vec<SkillEntry> {
    let skills_dir = dirs::home_dir()
        .map(|h| h.join(".claude/skills"))
        .unwrap_or_default();

    if !skills_dir.exists() {
        return Vec::new();
    }

    let mut skills = Vec::new();

    if let Ok(entries) = fs::read_dir(&skills_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() || path.is_symlink() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                if name.starts_with('.') {
                    continue;
                }

                // Try to read SKILL.md for tags
                let tags = read_skill_tags(&path);

                skills.push(SkillEntry {
                    slug: name.clone(),
                    name: name.replace('-', " "),
                    tags,
                });
            }
        }
    }

    skills
}

/// Read tags from SKILL.md frontmatter
fn read_skill_tags(skill_dir: &Path) -> Vec<String> {
    // Try multiple possible locations
    let candidates = [
        skill_dir.join("SKILL.md"),
        skill_dir
            .join("skills")
            .join(skill_dir.file_name().unwrap_or_default())
            .join("SKILL.md"),
    ];

    for candidate in &candidates {
        if let Ok(content) = fs::read_to_string(candidate) {
            return extract_tags(&content);
        }
    }

    Vec::new()
}

/// Extract tags from YAML frontmatter
fn extract_tags(content: &str) -> Vec<String> {
    // Simple extraction: look for "tags: [...]" line
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("tags:") {
            let rest = trimmed.strip_prefix("tags:").unwrap_or("").trim();
            if rest.starts_with('[') && rest.ends_with(']') {
                let inner = &rest[1..rest.len() - 1];
                return inner
                    .split(',')
                    .map(|t| t.trim().trim_matches('"').trim_matches('\'').to_string())
                    .filter(|t| !t.is_empty())
                    .collect();
            }
        }
    }
    Vec::new()
}

/// Find best matching skill using simple word overlap scoring
fn find_best_match<'a>(query: &str, skills: &'a [SkillEntry]) -> Option<(&'a SkillEntry, f64)> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();

    let mut best_match: Option<(&SkillEntry, f64)> = None;

    for skill in skills {
        let score = calculate_score(&query_words, skill);
        if score > best_match.as_ref().map(|(_, s)| *s).unwrap_or(0.0) {
            best_match = Some((skill, score));
        }
    }

    best_match
}

/// Calculate match score between query words and skill
fn calculate_score(query_words: &[&str], skill: &SkillEntry) -> f64 {
    let skill_words: Vec<String> = skill
        .name
        .to_lowercase()
        .split_whitespace()
        .map(String::from)
        .chain(skill.tags.iter().map(|t| t.to_lowercase()))
        .collect();

    if query_words.is_empty() || skill_words.is_empty() {
        return 0.0;
    }

    let mut matches = 0;
    for qw in query_words {
        for sw in &skill_words {
            if sw.contains(qw) || qw.contains(sw.as_str()) {
                matches += 1;
                break;
            }
        }
    }

    matches as f64 / query_words.len() as f64
}
