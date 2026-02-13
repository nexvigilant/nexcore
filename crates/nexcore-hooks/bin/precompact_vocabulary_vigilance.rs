//! PreCompact hook: Vocabulary Vigilance - Pattern Detection
//!
//! Scans conversation transcript for repeated constraint clusters and
//! proposes new vocabulary shorthands when patterns exceed threshold.
//!
//! Vigilance lifecycle phase: SURVEILLANCE (continuous monitoring)
//!
//! Exit codes:
//! - 0: Always (detection is advisory)

use nexcore_hooks::read_input;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

/// Minimum occurrences before suggesting a shorthand
const PATTERN_THRESHOLD: usize = 3;

/// Maximum patterns to track per session
const MAX_PATTERNS: usize = 50;

/// Minimum words in a constraint cluster
const MIN_CLUSTER_SIZE: usize = 3;

/// Constraint indicators that suggest a pattern worth tracking
const CONSTRAINT_MARKERS: &[&str] = &[
    "must",
    "always",
    "never",
    "require",
    "ensure",
    "mandatory",
    "policy",
    "enforce",
    "standard",
    "pattern",
    "approach",
    "methodology",
    "framework",
    "architecture",
    "design",
];

#[derive(Serialize, Deserialize, Default)]
struct PatternTracker {
    /// Pattern -> occurrence count
    patterns: HashMap<String, usize>,
    /// Patterns already converted to vocabulary
    adopted: Vec<String>,
    /// Session count
    sessions_analyzed: usize,
}

#[derive(Serialize)]
struct ProposedShorthand {
    pattern: String,
    occurrences: usize,
    suggested_name: String,
    constraints: Vec<String>,
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => std::process::exit(0),
    };

    // Read transcript to extract patterns
    if let Some(transcript_path) = &input.transcript_path {
        if let Ok(content) = fs::read_to_string(transcript_path) {
            let mut tracker = load_tracker();
            let new_patterns = extract_constraint_patterns(&content);

            // Update pattern counts
            for pattern in new_patterns {
                *tracker.patterns.entry(pattern).or_insert(0) += 1;
            }

            // Prune to max patterns (keep highest counts)
            if tracker.patterns.len() > MAX_PATTERNS {
                let mut entries: Vec<_> = tracker.patterns.into_iter().collect();
                entries.sort_by(|a, b| b.1.cmp(&a.1));
                entries.truncate(MAX_PATTERNS);
                tracker.patterns = entries.into_iter().collect();
            }

            tracker.sessions_analyzed += 1;

            // Check for patterns exceeding threshold
            let proposals = propose_shorthands(&tracker);

            if !proposals.is_empty() {
                if let Err(e) = save_proposals(&proposals) {
                    eprintln!("Warning: Could not save proposals: {e}");
                }
                eprintln!(
                    "📚 Vocabulary Vigilance: {} candidate shorthand(s) detected",
                    proposals.len()
                );
                for p in &proposals {
                    eprintln!(
                        "   → \"{}\" (seen {}x) - suggested: {}",
                        truncate(&p.pattern, 50),
                        p.occurrences,
                        p.suggested_name
                    );
                }
            }

            if let Err(e) = save_tracker(&tracker) {
                eprintln!("Warning: Could not save tracker: {e}");
            }
        }
    }

    std::process::exit(0);
}

fn extract_constraint_patterns(content: &str) -> Vec<String> {
    let mut patterns = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    for (i, line) in lines.iter().enumerate() {
        let lower = line.to_lowercase();

        // Check if line contains constraint markers
        let has_marker = CONSTRAINT_MARKERS.iter().any(|m| lower.contains(m));
        if !has_marker {
            continue;
        }

        // Extract the constraint clause
        let words: Vec<&str> = line.split_whitespace().collect();
        if words.len() < MIN_CLUSTER_SIZE {
            continue;
        }

        // Build a normalized pattern (lowercase, limited length)
        let pattern: String = words
            .iter()
            .take(12) // Cap at 12 words
            .map(|w| w.to_lowercase())
            .filter(|w| w.len() > 2) // Skip short words
            .collect::<Vec<_>>()
            .join(" ");

        if pattern.len() >= 20 && pattern.len() <= 200 {
            // Check for multi-line constraint lists (look ahead)
            let mut extended_pattern = pattern.clone();
            for j in 1..=3 {
                if i + j < lines.len() {
                    let next = lines[i + j].trim();
                    if next.starts_with('-') || next.starts_with('•') || next.starts_with('*') {
                        let item: String = next
                            .chars()
                            .skip(1)
                            .take(50)
                            .collect::<String>()
                            .trim()
                            .to_lowercase();
                        if !item.is_empty() {
                            extended_pattern.push_str(" + ");
                            extended_pattern.push_str(&item);
                        }
                    }
                }
            }
            patterns.push(extended_pattern);
        }
    }

    patterns
}

fn propose_shorthands(tracker: &PatternTracker) -> Vec<ProposedShorthand> {
    let vocabulary = load_vocabulary();
    let existing_expansions: Vec<String> = vocabulary
        .get("shorthands")
        .and_then(|s| s.as_object())
        .map(|obj| {
            obj.values()
                .filter_map(|v| v.get("expansion"))
                .filter_map(|e| e.as_array())
                .flat_map(|arr| {
                    arr.iter()
                        .filter_map(|s| s.as_str().map(|s| s.to_lowercase()))
                })
                .collect()
        })
        .unwrap_or_default();

    tracker
        .patterns
        .iter()
        .filter(|(pattern, count)| {
            **count >= PATTERN_THRESHOLD
                && !tracker.adopted.contains(pattern)
                && !is_already_covered(pattern, &existing_expansions)
        })
        .map(|(pattern, count)| {
            let suggested_name = generate_shorthand_name(pattern);
            let constraints = extract_constraint_list(pattern);
            ProposedShorthand {
                pattern: pattern.clone(),
                occurrences: *count,
                suggested_name,
                constraints,
            }
        })
        .collect()
}

fn is_already_covered(pattern: &str, existing: &[String]) -> bool {
    // Check if pattern is substantially covered by existing vocabulary
    let words: Vec<&str> = pattern.split_whitespace().collect();
    let matched = words
        .iter()
        .filter(|w| existing.iter().any(|e| e.contains(*w)))
        .count();

    // If >60% of words are in existing expansions, consider it covered
    matched as f64 / words.len() as f64 > 0.6
}

fn generate_shorthand_name(pattern: &str) -> String {
    // Extract key nouns/verbs for the shorthand name
    let stop_words = [
        "the", "a", "an", "is", "are", "must", "should", "always", "never", "be", "to", "for",
        "and", "or", "with",
    ];

    let key_words: Vec<&str> = pattern
        .split_whitespace()
        .filter(|w| w.len() > 3 && !stop_words.contains(w))
        .take(3)
        .collect();

    if key_words.is_empty() {
        "unnamed-pattern".to_string()
    } else {
        key_words.join("-")
    }
}

fn extract_constraint_list(pattern: &str) -> Vec<String> {
    pattern
        .split(" + ")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max - 3])
    }
}

// --- Persistence ---

fn implicit_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".claude").join("implicit")
}

fn load_tracker() -> PatternTracker {
    let path = implicit_path().join("pattern_tracker.json");
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_tracker(tracker: &PatternTracker) -> std::io::Result<()> {
    let dir = implicit_path();
    fs::create_dir_all(&dir)?;
    let path = dir.join("pattern_tracker.json");
    let json =
        serde_json::to_string_pretty(tracker).map_err(|e| std::io::Error::other(e.to_string()))?;
    fs::write(path, json)
}

fn load_vocabulary() -> serde_json::Value {
    let path = implicit_path().join("vocabulary.json");
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| serde_json::json!({}))
}

fn save_proposals(proposals: &[ProposedShorthand]) -> std::io::Result<()> {
    let dir = implicit_path();
    fs::create_dir_all(&dir)?;
    let path = dir.join("vocabulary_proposals.jsonl");

    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;

    for proposal in proposals {
        let entry = serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "suggested_name": proposal.suggested_name,
            "pattern": proposal.pattern,
            "occurrences": proposal.occurrences,
            "constraints": proposal.constraints
        });
        writeln!(file, "{}", entry)?;
    }

    Ok(())
}
