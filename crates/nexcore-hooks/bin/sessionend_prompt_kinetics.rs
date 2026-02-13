//! Session End Prompt Kinetics Reporter
//!
//! Generates a kinetics report at session end showing:
//! - Prompt patterns detected this session
//! - Skill synthesis candidates
//! - Recommendations for new skills
//!
//! Fires on SessionEnd event.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
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
}

fn main() {
    let state = KineticsState::load();

    // Only report if we have meaningful data
    if state.total_prompts == 0 {
        process::exit(0);
    }

    let stdout = io::stdout();
    let mut handle = stdout.lock();

    // Header
    writeln!(handle).ok();
    writeln!(
        handle,
        "╔══════════════════════════════════════════════════════════╗"
    )
    .ok();
    writeln!(
        handle,
        "║           📊 PROMPT KINETICS SESSION REPORT              ║"
    )
    .ok();
    writeln!(
        handle,
        "╠══════════════════════════════════════════════════════════╣"
    )
    .ok();

    // Summary stats
    writeln!(
        handle,
        "║  Total prompts analyzed: {:>6}                         ║",
        state.total_prompts
    )
    .ok();
    writeln!(
        handle,
        "║  Patterns detected:      {:>6}                         ║",
        state.patterns.len()
    )
    .ok();
    writeln!(
        handle,
        "║  Skill candidates:       {:>6}                         ║",
        state.skill_candidates.len()
    )
    .ok();

    // Top patterns
    if !state.patterns.is_empty() {
        writeln!(
            handle,
            "╠══════════════════════════════════════════════════════════╣"
        )
        .ok();
        writeln!(
            handle,
            "║  Top Patterns:                                           ║"
        )
        .ok();

        let mut patterns: Vec<_> = state.patterns.values().collect();
        patterns.sort_by(|a, b| b.count.cmp(&a.count));

        for pattern in patterns.iter().take(5) {
            let candidate_marker = if pattern.skill_candidate {
                "💡"
            } else {
                "  "
            };
            writeln!(
                handle,
                "║  {} {:20} {:>3}x                          ║",
                candidate_marker, pattern.pattern, pattern.count
            )
            .ok();
        }
    }

    // Skill candidates with recommendations
    if !state.skill_candidates.is_empty() {
        writeln!(
            handle,
            "╠══════════════════════════════════════════════════════════╣"
        )
        .ok();
        writeln!(
            handle,
            "║  💡 SKILL SYNTHESIS RECOMMENDATIONS:                     ║"
        )
        .ok();

        for candidate in &state.skill_candidates {
            writeln!(
                handle,
                "║     • Consider creating /{} skill              ║",
                candidate
            )
            .ok();
        }
    }

    writeln!(
        handle,
        "╚══════════════════════════════════════════════════════════╝"
    )
    .ok();

    process::exit(0);
}
