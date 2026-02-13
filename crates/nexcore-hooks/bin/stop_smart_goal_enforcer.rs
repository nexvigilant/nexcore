//! Stop SMART Goal Enforcer
//!
//! Blocks stop until SMART goal is achieved, abandoned, or completed.
//! Reads active goal from ~/.claude/brain/goals/active_goal.json.
//!
//! Exit codes:
//! - 0: Allow stop (goal completed/abandoned/none)
//!
//! Escape hatches (user can say):
//! - "GOAL ACHIEVED" or "GOAL: COMPLETE" → mark complete, allow stop
//! - "/abandon" → mark abandoned, allow stop

use chrono::{DateTime, Utc};
use nexcore_hooks::{HookDecision, HookOutput, read_input};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process;

/// Primitive Aspiration (prerequisite to SMART goals)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimitiveAspiration {
    pub statement: String,
    pub value_equation: ValueEquation,
    pub primitive_form: String,
    pub source_mode: SourceMode,
}

/// Source mode for primitive extraction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceMode {
    Full,
    Partial,
    Expert,
    Hybrid,
}

/// VALUE = ASYMMETRY × TIME × EXCHANGE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueEquation {
    pub asymmetry: String,
    pub time: String,
    pub exchange: String,
}

/// SMART goal state stored in Brain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveGoal {
    pub version: u32,
    pub goal_id: String,
    pub skill: String,
    pub statement: String,
    pub smart_criteria: SmartCriteria,
    pub status: GoalStatus,
    pub progress_notes: Vec<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aspiration: Option<PrimitiveAspiration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartCriteria {
    pub specific: String,
    pub measurable: String,
    pub achievable: bool,
    pub relevant: String,
    pub time_bound: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GoalStatus {
    InProgress,
    Completed,
    Abandoned,
}

/// Get path to active goal file
fn goal_file_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(".claude")
        .join("brain")
        .join("goals")
        .join("active_goal.json")
}

/// Load active goal if it exists
fn load_active_goal() -> Option<ActiveGoal> {
    let path = goal_file_path();
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

/// Save active goal
fn save_active_goal(goal: &ActiveGoal) -> std::io::Result<()> {
    let path = goal_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(goal)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(path, content)
}

/// Mark goal as completed
fn mark_completed(goal: &mut ActiveGoal) {
    goal.status = GoalStatus::Completed;
    goal.completed_at = Some(Utc::now());
    goal.progress_notes.push("Goal marked complete".to_string());
    if let Err(e) = save_active_goal(goal) {
        eprintln!("Warning: Failed to save goal state: {e}");
    }
}

/// Mark goal as abandoned
fn mark_abandoned(goal: &mut ActiveGoal) {
    goal.status = GoalStatus::Abandoned;
    goal.completed_at = Some(Utc::now());
    goal.progress_notes
        .push("Goal abandoned by user".to_string());
    if let Err(e) = save_active_goal(goal) {
        eprintln!("Warning: Failed to save goal state: {e}");
    }
}

/// Completion keywords that mark goal as achieved
const COMPLETION_KEYWORDS: &[&str] = &[
    "GOAL ACHIEVED",
    "GOAL: COMPLETE",
    "GOAL:COMPLETE",
    "GOAL_ACHIEVED",
    "GOAL_COMPLETE",
];

/// Abandon keywords that mark goal as abandoned (all lowercase for comparison)
const ABANDON_KEYWORDS: &[&str] = &["/abandon", "abandon goal", "abandon_goal"];

/// Check if transcript contains completion keyword
fn has_completion_keyword(transcript: &str) -> bool {
    let upper = transcript.to_uppercase();
    COMPLETION_KEYWORDS.iter().any(|kw| upper.contains(kw))
}

/// Check if transcript contains abandon keyword
fn has_abandon_keyword(transcript: &str) -> bool {
    let lower = transcript.to_lowercase();
    ABANDON_KEYWORDS.iter().any(|kw| lower.contains(kw))
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => {
            // No input - allow stop
            println!(r#"{{"decision":"approve"}}"#);
            process::exit(0);
        }
    };

    // Check if stop hook is already active (prevent infinite loop)
    if input.is_stop_hook_active() {
        println!(r#"{{"decision":"approve"}}"#);
        process::exit(0);
    }

    // Load active goal
    let mut goal = match load_active_goal() {
        Some(g) => g,
        None => {
            // No active goal - allow stop
            println!(r#"{{"decision":"approve"}}"#);
            process::exit(0);
        }
    };

    // If goal already completed or abandoned, allow stop
    if goal.status != GoalStatus::InProgress {
        println!(r#"{{"decision":"approve"}}"#);
        process::exit(0);
    }

    // Read transcript to check for escape hatches
    let transcript = input
        .transcript_path
        .as_ref()
        .and_then(|p| fs::read_to_string(p).ok())
        .unwrap_or_default();

    // Get last ~100 lines for recent context
    let lines: Vec<&str> = transcript.lines().collect();
    let recent_lines = if lines.len() > 100 {
        &lines[lines.len() - 100..]
    } else {
        &lines[..]
    };
    let recent_content = recent_lines.join("\n");

    // Check for completion keyword
    if has_completion_keyword(&recent_content) {
        mark_completed(&mut goal);
        eprintln!("🎯 SMART Goal completed: {}", goal.statement);
        println!(r#"{{"decision":"approve"}}"#);
        process::exit(0);
    }

    // Check for abandon keyword
    if has_abandon_keyword(&recent_content) {
        mark_abandoned(&mut goal);
        eprintln!("⚠️ SMART Goal abandoned: {}", goal.statement);
        println!(r#"{{"decision":"approve"}}"#);
        process::exit(0);
    }

    // Goal is in progress - block stop
    let aspiration_info = goal
        .aspiration
        .as_ref()
        .map(|a| format!("\n\n**Aspiration:** {}", a.statement))
        .unwrap_or_default();

    let reason = format!(
        r#"🎯 SMART Goal in progress for skill `{}`:

**Goal:** {}

**Measurable:** {}{}

**Options:**
1. Complete the goal, then say "GOAL ACHIEVED"
2. Say "/abandon" to abandon the goal
3. Continue working toward the goal

Cannot stop until goal is achieved or explicitly abandoned."#,
        goal.skill, goal.statement, goal.smart_criteria.measurable, aspiration_info
    );

    let output = HookOutput {
        decision: Some(HookDecision::Block),
        reason: Some(reason.clone()),
        ..Default::default()
    };

    output.emit();
    eprintln!("{reason}");
    process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_completion_keywords() {
        assert!(has_completion_keyword("I've finished, GOAL ACHIEVED!"));
        assert!(has_completion_keyword("GOAL: COMPLETE"));
        assert!(!has_completion_keyword("I'm still working on this"));
    }

    #[test]
    fn test_abandon_keywords() {
        assert!(has_abandon_keyword("/abandon"));
        assert!(has_abandon_keyword("ABANDON GOAL"));
        assert!(!has_abandon_keyword("I'm still working"));
    }

    #[test]
    fn test_goal_status_serialization() {
        let status = GoalStatus::InProgress;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""in_progress""#);

        let status = GoalStatus::Completed;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, r#""completed""#);
    }
}
