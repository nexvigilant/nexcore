//! Prompt Kinetics Analyzer
//!
//! Analyzes prompt patterns to detect skill synthesis opportunities.
//! "Kinetics" = the study of rates and patterns of change.
//!
//! Detects:
//! - Repeated prompt patterns that could become skills
//! - Skill invocation sequences that could be chained
//! - Vocabulary expansion candidates
//!
//! ```text
//!   Prompt Stream                    Pattern Detection
//!   ─────────────                    ─────────────────
//!   "fix rust error"  ─┐
//!   "fix rust panic"  ─┼─────────▶  rust-fix pattern (3x)
//!   "fix rust lifetime"─┘              ↓
//!                                   Skill candidate!
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process;

/// Pattern detected in prompt stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptPattern {
    pub pattern: String,
    pub count: u32,
    pub last_seen: String,
    pub skill_candidate: bool,
}

/// Kinetics state persisted between sessions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KineticsState {
    pub patterns: HashMap<String, PromptPattern>,
    pub total_prompts: u32,
    pub skill_candidates: Vec<String>,
}

impl KineticsState {
    fn state_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".claude")
            .join("metrics")
            .join("prompt_kinetics.json")
    }

    pub fn load() -> Self {
        let path = Self::state_path();
        if path.exists() {
            fs::read_to_string(&path)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> io::Result<()> {
        let path = Self::state_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)
    }

    /// Extract pattern keywords from prompt
    fn extract_pattern(prompt: &str) -> Option<String> {
        let lower = prompt.to_lowercase();

        // Common action patterns
        let patterns = [
            ("fix", "rust"),
            ("create", "skill"),
            ("add", "hook"),
            ("build", "mcp"),
            ("validate", "test"),
            ("audit", "skill"),
            ("upgrade", "compliance"),
        ];

        for (verb, noun) in patterns {
            if lower.contains(verb) && lower.contains(noun) {
                return Some(format!("{}-{}", verb, noun));
            }
        }

        None
    }

    /// Record a prompt and detect patterns
    pub fn record_prompt(&mut self, prompt: &str) {
        self.total_prompts += 1;

        if let Some(pattern) = Self::extract_pattern(prompt) {
            let entry = self
                .patterns
                .entry(pattern.clone())
                .or_insert(PromptPattern {
                    pattern: pattern.clone(),
                    count: 0,
                    last_seen: String::new(),
                    skill_candidate: false,
                });

            entry.count += 1;
            entry.last_seen = chrono::Utc::now().to_rfc3339();

            // Mark as skill candidate if seen 3+ times
            if entry.count >= 3 && !entry.skill_candidate {
                entry.skill_candidate = true;
                self.skill_candidates.push(pattern);
            }
        }
    }
}

/// Hook input from Claude Code
#[derive(Debug, Deserialize)]
struct HookInput {
    prompt: Option<String>,
    #[allow(dead_code)]
    session_id: Option<String>,
}

fn main() {
    // Read input from stdin
    let stdin = io::stdin();
    let input_line = match stdin.lock().lines().next() {
        Some(Ok(line)) => line,
        _ => {
            // No input - display current state
            let state = KineticsState::load();
            eprintln!(
                "📊 Prompt Kinetics: {} prompts analyzed",
                state.total_prompts
            );
            eprintln!("🔬 Patterns detected: {}", state.patterns.len());
            eprintln!("💡 Skill candidates: {}", state.skill_candidates.len());
            for candidate in &state.skill_candidates {
                eprintln!("   • {}", candidate);
            }
            process::exit(0);
        }
    };

    let input: HookInput = match serde_json::from_str(&input_line) {
        Ok(i) => i,
        Err(_) => {
            process::exit(0);
        }
    };

    // Load state and record prompt
    let mut state = KineticsState::load();

    if let Some(prompt) = input.prompt {
        let old_candidates = state.skill_candidates.len();
        state.record_prompt(&prompt);

        // Save state
        if let Err(e) = state.save() {
            eprintln!("⚠️ Failed to save kinetics state: {}", e);
        }

        // Report new skill candidates (using safe access with if-let)
        if state.skill_candidates.len() > old_candidates {
            if let Some(new_candidate) = state.skill_candidates.last() {
                let count = state.patterns.get(new_candidate).map_or(0, |p| p.count);
                let stdout = io::stdout();
                let mut handle = stdout.lock();
                writeln!(
                    handle,
                    "💡 Skill synthesis candidate detected: {} (seen {}x)",
                    new_candidate, count
                )
                .ok();
            }
        }
    }

    process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_extraction() {
        assert_eq!(
            KineticsState::extract_pattern("fix the rust error"),
            Some("fix-rust".to_string())
        );
        assert_eq!(
            KineticsState::extract_pattern("create a new skill"),
            Some("create-skill".to_string())
        );
        assert_eq!(KineticsState::extract_pattern("random prompt"), None);
    }

    #[test]
    fn test_skill_candidate_threshold() {
        let mut state = KineticsState::default();

        state.record_prompt("fix rust error");
        assert!(state.skill_candidates.is_empty());

        state.record_prompt("fix rust panic");
        assert!(state.skill_candidates.is_empty());

        state.record_prompt("fix rust lifetime");
        assert_eq!(state.skill_candidates.len(), 1);
        assert_eq!(state.skill_candidates[0], "fix-rust");
    }
}
