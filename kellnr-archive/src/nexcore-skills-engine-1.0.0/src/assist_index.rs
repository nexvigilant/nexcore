//! # Skill Knowledge Index
//!
//! Pre-populated index of SKILL.md files for intent-based search.
//! Mirrors `nexcore-knowledge::KsbIndex` pattern: rayon parallel scoring,
//! regex-per-term matching, weighted field scoring.

use std::collections::HashMap;
use std::path::Path;

use rayon::prelude::*;
use regex::RegexBuilder;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::foundation::skill_metadata::parse_frontmatter;

/// Single skill's compiled knowledge entry.
///
/// Tier: T3 (Domain-specific knowledge representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillKnowledgeEntry {
    /// Skill name (from frontmatter `name`)
    pub name: String,
    /// Human-readable intent/description
    pub intent: String,
    /// Categorical tags
    pub tags: Vec<String>,
    /// Trigger phrases
    pub triggers: Vec<String>,
    /// MCP tools this skill references
    pub mcp_tools: Vec<String>,
    /// Related skills (see-also)
    pub see_also: Vec<String>,
    /// Upstream dependencies
    pub upstream: Vec<String>,
    /// Downstream consumers
    pub downstream: Vec<String>,
    /// Domain (e.g., "pharmacovigilance")
    pub domain: Option<String>,
    /// Pipeline membership
    pub pipeline: Option<String>,
    /// Full markdown body after frontmatter
    pub body: String,
    /// Extracted `## Section` headings
    pub sections: Vec<String>,
    /// File path
    pub path: String,
}

/// Pre-populated index with inverted lookups.
///
/// Tier: T3 (Domain-specific index structure)
#[derive(Debug, Default)]
pub struct SkillKnowledgeIndex {
    /// Skills by name
    by_name: HashMap<String, SkillKnowledgeEntry>,
    /// Inverted: tag → skill names
    by_tag: HashMap<String, Vec<String>>,
    /// Inverted: MCP tool → skill names
    by_mcp_tool: HashMap<String, Vec<String>>,
    /// Total entry count
    count: usize,
}

/// Search result with relevance score.
///
/// Tier: T3 (Domain-specific search output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSearchResult {
    /// Skill name
    pub name: String,
    /// Skill intent
    pub intent: String,
    /// Relevance score (0-100)
    pub score: u32,
    /// Matched field:term pairs
    pub matches: Vec<String>,
    /// MCP tools for chaining
    pub mcp_tools: Vec<String>,
    /// Best-matching section content
    pub relevant_section: Option<String>,
    /// Related skills
    pub related_skills: Vec<String>,
}

// ============================================================================
// Index construction
// ============================================================================

impl SkillKnowledgeIndex {
    /// Create empty index.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Scan directory for SKILL.md files and build index.
    ///
    /// Uses rayon for parallel frontmatter parsing.
    ///
    /// # Errors
    ///
    /// Returns error if directory doesn't exist.
    pub fn scan(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Err(format!("Directory does not exist: {}", path.display()));
        }

        let skill_paths = collect_skill_paths(path);
        let entries = parse_entries_parallel(&skill_paths);
        Ok(build_index(entries))
    }

    /// Get entry by name.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&SkillKnowledgeEntry> {
        self.by_name.get(name)
    }

    /// Get all entries.
    #[must_use]
    pub fn entries(&self) -> Vec<&SkillKnowledgeEntry> {
        self.by_name.values().collect()
    }

    /// Get skills by tag.
    #[must_use]
    pub fn get_by_tag(&self, tag: &str) -> Vec<&str> {
        self.by_tag
            .get(&tag.to_lowercase())
            .map(|names| names.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// Get skills by MCP tool.
    #[must_use]
    pub fn get_by_mcp_tool(&self, tool: &str) -> Vec<&str> {
        self.by_mcp_tool
            .get(tool)
            .map(|names| names.iter().map(String::as_str).collect())
            .unwrap_or_default()
    }

    /// Total entry count.
    #[must_use]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Check if index is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Insert an entry directly (for testing or programmatic construction).
    pub fn insert(&mut self, entry: SkillKnowledgeEntry) {
        let name = entry.name.clone();
        for tag in &entry.tags {
            self.by_tag
                .entry(tag.to_lowercase())
                .or_default()
                .push(name.clone());
        }
        for tool in &entry.mcp_tools {
            self.by_mcp_tool
                .entry(tool.clone())
                .or_default()
                .push(name.clone());
        }
        self.by_name.insert(name, entry);
        self.count += 1;
    }
}

/// Walk directory tree and collect SKILL.md paths.
fn collect_skill_paths(path: &Path) -> Vec<std::path::PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_name() == "SKILL.md")
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Parse SKILL.md files into entries using rayon.
fn parse_entries_parallel(paths: &[std::path::PathBuf]) -> Vec<SkillKnowledgeEntry> {
    paths
        .par_iter()
        .filter_map(|p| parse_single_entry(p))
        .collect()
}

/// Parse a single SKILL.md file into an entry.
fn parse_single_entry(path: &Path) -> Option<SkillKnowledgeEntry> {
    let content = std::fs::read_to_string(path).ok()?;
    let meta = parse_frontmatter(&content).ok()?;
    if meta.name.is_empty() {
        return None;
    }
    let body = extract_body(&content);
    let sections = extract_sections(&body);

    Some(SkillKnowledgeEntry {
        name: meta.name,
        intent: meta.intent.unwrap_or_default(),
        tags: meta.tags,
        triggers: meta.triggers,
        mcp_tools: meta.mcp_tools,
        see_also: meta.see_also,
        upstream: meta.upstream,
        downstream: meta.downstream,
        domain: meta.domain,
        pipeline: meta.pipeline,
        body,
        sections,
        path: path.to_string_lossy().to_string(),
    })
}

/// Build index from parsed entries.
fn build_index(entries: Vec<SkillKnowledgeEntry>) -> SkillKnowledgeIndex {
    let mut index = SkillKnowledgeIndex::new();
    for entry in entries {
        index.insert(entry);
    }
    index
}

// ============================================================================
// Search
// ============================================================================

/// Search skills by intent query.
///
/// Scoring weights (tuned for skill discovery):
/// - Name exact: +60, Name contains: +30
/// - Trigger match: +50
/// - Intent/desc: +35
/// - Tag match: +25
/// - Section heading: +15
/// - Body content: +2/hit (max 10)
/// - All-terms bonus: +20
///
/// Returns results sorted by score descending, capped at `limit`.
#[must_use]
pub fn search_skills(
    index: &SkillKnowledgeIndex,
    query: &str,
    tag_filter: Option<&str>,
    limit: usize,
) -> Vec<SkillSearchResult> {
    let query_lower = query.to_lowercase();
    let query_terms: Vec<&str> = query_lower.split_whitespace().collect();

    if query_terms.is_empty() {
        return Vec::new();
    }

    let patterns = build_patterns(&query_terms);
    let entries = filter_entries(index, tag_filter);

    let mut results: Vec<SkillSearchResult> = entries
        .par_iter()
        .filter_map(|entry| score_entry(entry, &patterns, &query_terms))
        .collect();

    results.sort_by(|a, b| b.score.cmp(&a.score));
    results.truncate(limit);
    results
}

/// Build case-insensitive regex patterns from query terms.
fn build_patterns(terms: &[&str]) -> Vec<regex::Regex> {
    terms
        .iter()
        .filter_map(|term| {
            RegexBuilder::new(&regex::escape(term))
                .case_insensitive(true)
                .build()
                .ok()
        })
        .collect()
}

/// Filter entries by optional tag.
fn filter_entries<'a>(
    index: &'a SkillKnowledgeIndex,
    tag_filter: Option<&str>,
) -> Vec<&'a SkillKnowledgeEntry> {
    match tag_filter {
        Some(tag) => index
            .get_by_tag(tag)
            .iter()
            .filter_map(|n| index.get(n))
            .collect(),
        None => index.entries(),
    }
}

/// Score a single entry against query patterns.
fn score_entry(
    entry: &SkillKnowledgeEntry,
    patterns: &[regex::Regex],
    terms: &[&str],
) -> Option<SkillSearchResult> {
    let mut score: u32 = 0;
    let mut matches = Vec::new();
    let mut best_section: Option<&str> = None;

    for (i, pattern) in patterns.iter().enumerate() {
        let term = terms.get(i).copied().unwrap_or_default();
        let field_score = score_fields(entry, pattern, term);
        score += field_score.total;
        matches.extend(field_score.matches);
        if let Some(s) = field_score.best_section {
            best_section = Some(s);
        }
    }

    // All-terms bonus
    if !terms.is_empty() && matches.len() >= terms.len() {
        score += 20;
    }

    if score == 0 {
        return None;
    }

    score = score.min(100);

    let relevant_section = resolve_section(entry, best_section, terms);
    let related = collect_related(entry);

    Some(SkillSearchResult {
        name: entry.name.clone(),
        intent: truncate(&entry.intent, 200),
        score,
        matches,
        mcp_tools: entry.mcp_tools.clone(),
        relevant_section,
        related_skills: related,
    })
}

/// Per-field scoring result for a single term.
struct FieldScore<'a> {
    total: u32,
    matches: Vec<String>,
    best_section: Option<&'a str>,
}

/// Score all fields for a single pattern/term.
fn score_fields<'a>(
    entry: &'a SkillKnowledgeEntry,
    pattern: &regex::Regex,
    term: &str,
) -> FieldScore<'a> {
    let mut total: u32 = 0;
    let mut matches = Vec::new();
    let mut best_section: Option<&str> = None;

    let name_lower = entry.name.to_lowercase();

    // Name match
    if name_lower == term || name_lower.contains(term) {
        total += if name_lower == term { 60 } else { 30 };
        matches.push(format!("name:{term}"));
    }

    // Trigger match
    if entry.triggers.iter().any(|t| pattern.is_match(t)) {
        total += 50;
        push_unique(&mut matches, term, "trigger");
    }

    // Intent match
    if pattern.is_match(&entry.intent) {
        total += 35;
        push_unique(&mut matches, term, "intent");
    }

    // Tag match
    if entry.tags.iter().any(|t| t.to_lowercase().contains(term)) {
        total += 25;
        push_unique(&mut matches, term, "tag");
    }

    // Section heading match
    for section in &entry.sections {
        if pattern.is_match(section) {
            total += 15;
            best_section = Some(section.as_str());
            push_unique(&mut matches, term, "section");
            break;
        }
    }

    // Body content (2 per hit, max 10)
    let body_lower = entry.body.to_lowercase();
    let body_count = pattern.find_iter(&body_lower).count();
    if body_count > 0 {
        total += (body_count.min(5) * 2) as u32;
        push_unique(&mut matches, term, "body");
    }

    FieldScore {
        total,
        matches,
        best_section,
    }
}

/// Push field:term if term not already tracked.
fn push_unique(matches: &mut Vec<String>, term: &str, field: &str) {
    if !matches.iter().any(|m| m.ends_with(term)) {
        matches.push(format!("{field}:{term}"));
    }
}

/// Resolve best relevant section content.
fn resolve_section(
    entry: &SkillKnowledgeEntry,
    best_section: Option<&str>,
    terms: &[&str],
) -> Option<String> {
    // Try best-scoring section first
    if let Some(heading) = best_section {
        if let Some(content) = extract_section_content(&entry.body, heading) {
            return Some(content);
        }
    }

    // Fallback: first section mentioning any term
    entry.sections.iter().find_map(|s| {
        let content = extract_section_content(&entry.body, s)?;
        let lower = content.to_lowercase();
        if terms.iter().any(|t| lower.contains(t)) {
            Some(content)
        } else {
            None
        }
    })
}

/// Collect related skills (see_also + upstream + downstream), deduplicated.
fn collect_related(entry: &SkillKnowledgeEntry) -> Vec<String> {
    let mut related = entry.see_also.clone();
    related.extend(entry.upstream.iter().cloned());
    related.extend(entry.downstream.iter().cloned());
    related.dedup();
    related
}

// ============================================================================
// Text extraction helpers
// ============================================================================

/// Extract body content after frontmatter delimiters.
fn extract_body(content: &str) -> String {
    if !content.starts_with("---") {
        return content.to_string();
    }
    let rest = &content[3..];
    rest.find("\n---")
        .map(|idx| rest[idx + 4..].trim().to_string())
        .unwrap_or_default()
}

/// Extract `## Section` headings from markdown body.
fn extract_sections(body: &str) -> Vec<String> {
    body.lines()
        .filter(|line| line.starts_with("## "))
        .map(|line| line[3..].trim().to_string())
        .collect()
}

/// Extract content under a `## Section` heading (max 500 chars).
fn extract_section_content(body: &str, heading: &str) -> Option<String> {
    let target = format!("## {heading}");
    let start = body.find(&target)?;
    let after = &body[start + target.len()..];
    let end = after.find("\n## ").unwrap_or(after.len());
    let content = after[..end].trim();
    if content.is_empty() {
        return None;
    }
    Some(truncate(content, 500))
}

/// Truncate string to max length with ellipsis.
fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}

/// Default skills directory path.
#[must_use]
pub fn default_skills_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(std::path::PathBuf::from(format!("{home}/.claude/skills")))
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_entry(name: &str, intent: &str) -> SkillKnowledgeEntry {
        SkillKnowledgeEntry {
            name: name.to_string(),
            intent: intent.to_string(),
            tags: vec![],
            triggers: vec![],
            mcp_tools: vec![],
            see_also: vec![],
            upstream: vec![],
            downstream: vec![],
            domain: None,
            pipeline: None,
            body: String::new(),
            sections: vec![],
            path: "/test".to_string(),
        }
    }

    #[test]
    fn test_empty_index() {
        let index = SkillKnowledgeIndex::new();
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
    }

    #[test]
    fn test_extract_body() {
        let content = "---\nname: test\n---\n# Body\nContent here";
        assert_eq!(extract_body(content), "# Body\nContent here");
    }

    #[test]
    fn test_extract_body_no_frontmatter() {
        assert_eq!(extract_body("# Just markdown"), "# Just markdown");
    }

    #[test]
    fn test_extract_sections() {
        let body = "# Title\n\n## One\nA\n\n## Two\nB";
        assert_eq!(extract_sections(body), vec!["One", "Two"]);
    }

    #[test]
    fn test_extract_section_content() {
        let body = "## First\nA\n\n## Second\nB\n\n## Third\nC";
        assert_eq!(
            extract_section_content(body, "Second"),
            Some("B".to_string())
        );
    }

    #[test]
    fn test_extract_section_content_last() {
        let body = "## Only\nThe only section";
        assert_eq!(
            extract_section_content(body, "Only"),
            Some("The only section".to_string())
        );
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is longer", 10), "this is...");
    }

    #[test]
    fn test_search_empty_query() {
        let index = SkillKnowledgeIndex::new();
        assert!(search_skills(&index, "", None, 5).is_empty());
    }

    #[test]
    fn test_search_empty_index() {
        let index = SkillKnowledgeIndex::new();
        assert!(search_skills(&index, "anything", None, 5).is_empty());
    }

    #[test]
    fn test_search_by_intent() {
        let mut index = SkillKnowledgeIndex::new();
        let mut entry = make_test_entry("ctvp-validator", "Validate code for production");
        entry.triggers = vec!["validate code".to_string()];
        entry.mcp_tools = vec!["validation_run".to_string()];
        entry.tags = vec!["validation".to_string()];
        entry.see_also = vec!["code-inspector".to_string()];
        index.insert(entry);

        let results = search_skills(&index, "validate production", None, 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "ctvp-validator");
        assert!(results[0].score > 0);
    }

    #[test]
    fn test_search_by_trigger() {
        let mut index = SkillKnowledgeIndex::new();
        let mut entry = make_test_entry("ctvp-validator", "Validate code");
        entry.triggers = vec!["production ready".to_string()];
        index.insert(entry);

        let results = search_skills(&index, "production ready", None, 5);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_tag_filter() {
        let mut index = SkillKnowledgeIndex::new();
        let mut entry = make_test_entry("ctvp-validator", "Validate code");
        entry.tags = vec!["testing".to_string()];
        index.insert(entry);

        assert!(!search_skills(&index, "validate", Some("testing"), 5).is_empty());
        assert!(search_skills(&index, "validate", Some("nonexistent"), 5).is_empty());
    }

    #[test]
    fn test_inverted_lookups() {
        let mut index = SkillKnowledgeIndex::new();
        let mut entry = make_test_entry("ctvp-validator", "Validate");
        entry.tags = vec!["validation".to_string()];
        entry.mcp_tools = vec!["validation_run".to_string()];
        index.insert(entry);

        assert_eq!(index.get_by_tag("validation"), vec!["ctvp-validator"]);
        assert_eq!(
            index.get_by_mcp_tool("validation_run"),
            vec!["ctvp-validator"]
        );
    }

    #[test]
    fn test_score_caps_at_100() {
        let mut index = SkillKnowledgeIndex::new();
        let mut entry = make_test_entry("signal", "Signal detection for signal analysis");
        entry.tags = vec!["signal".to_string()];
        entry.triggers = vec!["signal detection".to_string()];
        entry.body = "## Signal\nSignal signal signal signal".to_string();
        entry.sections = vec!["Signal".to_string()];
        index.insert(entry);

        let results = search_skills(&index, "signal", None, 5);
        assert!(results[0].score <= 100);
    }

    #[test]
    fn test_relevant_section() {
        let mut index = SkillKnowledgeIndex::new();
        let mut entry = make_test_entry("test-skill", "Test skill");
        entry.body = "## Setup\nGeneral.\n\n## Auth\nOAuth flow details.".to_string();
        entry.sections = vec!["Setup".to_string(), "Auth".to_string()];
        index.insert(entry);

        let results = search_skills(&index, "auth", None, 5);
        assert!(!results.is_empty());
        let section = results[0].relevant_section.as_deref().unwrap_or("");
        assert!(section.contains("OAuth"));
    }

    #[test]
    fn test_default_skills_path() {
        if std::env::var("HOME").is_ok() {
            assert!(default_skills_path().is_some());
        }
    }
}
