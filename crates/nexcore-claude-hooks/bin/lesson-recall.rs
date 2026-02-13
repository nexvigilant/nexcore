//! Lesson Recall — UserPromptSubmit Hook
//!
//! Reads the lessons database, scores each lesson by keyword overlap
//! with the user's prompt, and injects the top matches as stderr context.
//! Closes the learn→apply loop automatically.
//!
//! Tier: T2-C (cross-domain composite)
//! Grounding: T1(String, Vec, bool) via keyword extraction + scoring
//!
//! Action: Context injection (no block)
//! Exit: 0 = pass (with optional context on stderr)

use nexcore_hook_lib::cytokine::emit_hook_completed;
use nexcore_hook_lib::pass;
use std::env;

const HOOK_NAME: &str = "lesson-recall";
use std::fs;
use std::io::{self, Read};

/// UserPromptSubmit input structure.
/// Tier: T2-C
#[derive(Debug, serde::Deserialize)]
struct PromptInput {
    prompt: Option<String>,
}

/// Lesson from the lessons-mcp database.
/// Tier: T3 (domain-specific)
#[derive(Debug, serde::Deserialize)]
struct Lesson {
    title: String,
    content: String,
    context: String,
    #[serde(default)]
    tags: Vec<String>,
}

/// Lessons database container.
/// Tier: T2-C
#[derive(Debug, serde::Deserialize)]
struct LessonsDb {
    lessons: Vec<Lesson>,
}

/// Scored lesson for ranking.
/// Tier: T2-C
struct ScoredLesson<'a> {
    lesson: &'a Lesson,
    score: usize,
}

/// Stopwords to exclude from keyword extraction.
/// Tier: T1 (static sequence)
const STOPWORDS: &[&str] = &[
    "the", "a", "an", "is", "are", "was", "were", "in", "on", "to", "for", "of", "and", "or", "it",
    "this", "that", "how", "what", "do", "can", "i", "my", "me", "we", "you", "with", "from", "at",
    "by", "be", "not", "but", "have", "has", "had", "will", "would", "should", "could", "if",
    "then", "so", "up", "out", "about", "when", "which", "there", "all", "no", "just", "get",
    "set", "use", "make", "like",
];

/// Minimum keyword matches required to surface a lesson.
const MIN_SCORE: usize = 2;
/// Maximum lessons to surface.
const MAX_RESULTS: usize = 3;
/// Maximum content preview length per lesson.
const MAX_PREVIEW_LEN: usize = 100;

fn main() {
    let prompt = read_prompt();
    let keywords = extract_keywords(&prompt);

    if keywords.len() < 2 {
        pass();
    }

    let lessons_db = match load_lessons() {
        Some(db) => db,
        None => pass(),
    };

    let scored = rank_lessons(&lessons_db, &keywords);

    if !scored.is_empty() {
        format_output(&scored);
        // Emit cytokine signal (TGF-beta = regulation, lessons surfaced)
        emit_hook_completed(HOOK_NAME, 0, &format!("recalled_{}_lessons", scored.len()));
    }

    pass();
}

/// Read stdin, parse prompt, and validate. Calls pass() on skip conditions.
/// Tier: T1 (Sequence: read → parse → validate)
fn read_prompt() -> String {
    let mut buffer = String::new();
    if io::stdin().read_to_string(&mut buffer).is_err() {
        pass();
    }
    if buffer.trim().is_empty() {
        pass();
    }

    let input: PromptInput = match serde_json::from_str(&buffer) {
        Ok(i) => i,
        Err(_) => pass(),
    };

    let prompt = match input.prompt {
        Some(p) => p,
        None => pass(),
    };

    // Skip slash commands
    if prompt.trim_start().starts_with('/') {
        pass();
    }

    prompt
}

/// Score and rank lessons by keyword overlap, returning top matches.
/// Tier: T1 (Mapping + Sequence: score → sort → truncate)
fn rank_lessons<'a>(db: &'a LessonsDb, keywords: &[String]) -> Vec<ScoredLesson<'a>> {
    let mut scored: Vec<ScoredLesson<'a>> = db
        .lessons
        .iter()
        .map(|lesson| {
            let score = score_lesson(lesson, keywords);
            ScoredLesson { lesson, score }
        })
        .filter(|sl| sl.score >= MIN_SCORE)
        .collect();

    scored.sort_by(|a, b| b.score.cmp(&a.score));
    scored.truncate(MAX_RESULTS);
    scored
}

/// Extract significant keywords from the prompt.
/// Tier: T1 (Sequence + Mapping)
fn extract_keywords(prompt: &str) -> Vec<String> {
    prompt
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
        .filter(|w| w.len() >= 2)
        .filter(|w| !STOPWORDS.contains(w))
        .map(String::from)
        .collect()
}

/// Load the lessons database from disk.
/// Tier: T2-C (file I/O + deserialization)
fn load_lessons() -> Option<LessonsDb> {
    let home = env::var("HOME").ok()?;
    let path = format!("{home}/.local/share/lessons-mcp/lessons.json");
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Score a lesson by keyword overlap across all text fields.
/// Tier: T1 (Mapping: keyword → count)
fn score_lesson(lesson: &Lesson, keywords: &[String]) -> usize {
    let haystack = build_haystack(lesson);
    keywords
        .iter()
        .filter(|kw| haystack.contains(kw.as_str()))
        .count()
}

/// Build a searchable text from all lesson fields.
/// Tier: T1 (Sequence concatenation)
fn build_haystack(lesson: &Lesson) -> String {
    let mut hay = String::with_capacity(
        lesson.title.len() + lesson.content.len() + lesson.context.len() + 64,
    );
    hay.push_str(&lesson.title.to_lowercase());
    hay.push(' ');
    hay.push_str(&lesson.content.to_lowercase());
    hay.push(' ');
    hay.push_str(&lesson.context.to_lowercase());
    for tag in &lesson.tags {
        hay.push(' ');
        hay.push_str(&tag.to_lowercase());
    }
    hay
}

/// Truncate content to a preview, breaking at word boundary.
/// Tier: T1 (String slicing)
fn truncate_preview(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        return content.to_string();
    }
    let truncated = &content[..max_len];
    match truncated.rfind(' ') {
        Some(pos) => format!("{}...", &content[..pos]),
        None => format!("{truncated}..."),
    }
}

/// Format scored lessons as stderr context block.
/// Tier: T2-C (formatted output)
fn format_output(scored: &[ScoredLesson<'_>]) {
    let mut out = String::with_capacity(512);
    out.push_str("\n\u{1f4da} **LESSON RECALL** ");
    out.push_str(&"\u{2500}".repeat(33));
    out.push('\n');
    for sl in scored {
        let preview = truncate_preview(&sl.lesson.content, MAX_PREVIEW_LEN);
        let preview_clean = preview.replace('\n', " ");
        out.push_str(&format!(
            "  \u{2022} [{}]\n    \u{2192} {}\n",
            sl.lesson.title, preview_clean
        ));
    }
    out.push_str(&"\u{2500}".repeat(51));
    out.push('\n');
    eprint!("{out}");
}
