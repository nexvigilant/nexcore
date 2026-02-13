//! FORGE Activator Hook
//!
//! Event: UserPromptSubmit

use nexcore_hooks::protocol::HookOutput;
use nexcore_hooks::{exit_success_auto, read_input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// FORGE state persisted between interactions
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ForgeState {
    /// active
    pub active: bool,
    /// paused
    pub paused: bool,
    /// goal
    pub goal: Option<String>,
    /// cycle
    pub cycle_count: u32,
    /// mined
    pub primitives_mined: Vec<String>,
    /// artifacts
    pub artifacts_generated: Vec<String>,
    /// verification
    pub last_verification: String,
    /// activity
    pub last_activity: Option<String>,
}

impl ForgeState {
    /// state_path
    fn state_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".claude")
            .join("brain")
            .join("forge")
            .join("state.json")
    }

    /// load
    fn load() -> Self {
        let p = Self::state_path();
        if p.exists() {
            fs::read_to_string(&p)
                .ok()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default()
        } else {
            Self::default()
        }
    }

    /// save
    fn save(&self) -> Result<(), std::io::Error> {
        let p = Self::state_path();
        if let Some(par) = p.parent() {
            fs::create_dir_all(par)?;
        }
        fs::write(p, serde_json::to_string_pretty(self)?)
    }
}

/// Commands
#[derive(Debug)]
enum ForgeCommand {
    /// Start
    Start(Option<String>),
    /// Status
    Status,
    /// Pause
    Pause,
    /// None
    None,
}

/// extract_goal
fn extract_goal(prompt: &str, trigger: &str) -> Option<String> {
    let idx = prompt.to_uppercase().find(trigger)?;
    let trimmed = prompt[idx + trigger.len()..].trim_start_matches(':').trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// parse_command
fn parse_command(prompt: &str) -> ForgeCommand {
    let upper = prompt.to_uppercase();
    if upper.contains("START FORGE") {
        return ForgeCommand::Start(extract_goal(prompt, "START FORGE"));
    }
    if upper.contains("RUN FORGE") {
        return ForgeCommand::Start(extract_goal(prompt, "RUN FORGE"));
    }
    if upper.contains("FORGE STATUS") {
        return ForgeCommand::Status;
    }
    if upper.contains("PAUSE FORGE") {
        return ForgeCommand::Pause;
    }
    ForgeCommand::None
}

/// handle_start
fn handle_start(goal: Option<String>) {
    let mut state = ForgeState::load();
    state.active = true;
    state.paused = false;
    state.goal = goal.clone();
    state.cycle_count = 0;
    state.primitives_mined.clear();
    state.artifacts_generated.clear();
    state.last_verification = "pending".to_string();
    state.last_activity = Some(chrono::Utc::now().to_rfc3339());
    let _ = state.save();

    let g = goal.as_deref().unwrap_or("(general Rust development)");
    let msg = format!("🔨 FORGE ACTIVATED\nGoal: {g}\n\nREFINEMENT LOOP INITIATED.");
    HookOutput {
        decision: None,
        reason: Some(msg),
        ..Default::default()
    }
    .emit();
    std::process::exit(1);
}

/// handle_none
fn handle_none() {
    let s = ForgeState::load();
    if s.active && !s.paused {
        let msg = format!("[FORGE active: cycle {}]", s.cycle_count);
        HookOutput {
            decision: None,
            reason: Some(msg),
            ..Default::default()
        }
        .emit();
        std::process::exit(0);
    }
    exit_success_auto();
}

/// main
fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };
    let prompt = input.prompt.as_deref().unwrap_or("");
    match parse_command(prompt) {
        ForgeCommand::Start(g) => handle_start(g),
        ForgeCommand::Status => {
            let s = ForgeState::load();
            let msg = format!("FORGE STATUS: {:?}", s);
            HookOutput {
                decision: None,
                reason: Some(msg),
                ..Default::default()
            }
            .emit();
            std::process::exit(1);
        }
        ForgeCommand::Pause => {
            let mut s = ForgeState::load();
            s.paused = true;
            let _ = s.save();
            std::process::exit(1);
        }
        ForgeCommand::None => handle_none(),
    }
}
