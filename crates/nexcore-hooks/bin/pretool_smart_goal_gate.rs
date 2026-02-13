//! PreToolUse:Skill SMART Goal Gate
//!
//! Fires on PreToolUse:Skill matcher. If no active SMART goal exists for the
//! invoked skill, injects a prompt requesting goal definition.
//!
//! Exit codes:
//! - 0: Allow skill execution (with optional context injection)

use chrono::{DateTime, Utc};
use nexcore_hooks::{HookOutput, HookSpecificOutput, read_input};
use nexcore_id::NexId;
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

/// Create a new goal from user input in transcript
fn create_goal_from_prompt(skill: &str, prompt: &str) -> ActiveGoal {
    // Extract SMART goal components from prompt if present
    // Format: SMART: S=... M=... A=... R=... T=...
    let (statement, criteria) = parse_smart_goal(prompt, skill);

    ActiveGoal {
        version: 1,
        goal_id: NexId::v4().to_string(),
        skill: skill.to_string(),
        statement,
        smart_criteria: criteria,
        status: GoalStatus::InProgress,
        progress_notes: vec!["Goal created from skill invocation".to_string()],
        created_at: Utc::now(),
        completed_at: None,
        aspiration: None,
    }
}

/// Parse SMART goal from prompt text
fn parse_smart_goal(prompt: &str, skill: &str) -> (String, SmartCriteria) {
    // Look for structured SMART format
    // SMART: S=Create auth module M=Tests pass A=true R=Security needed T=This session
    if prompt.contains("SMART:") || prompt.contains("GOAL:") {
        let lines: Vec<&str> = prompt.lines().collect();
        for line in &lines {
            if line.contains("SMART:") || line.contains("GOAL:") {
                return parse_smart_line(line, skill);
            }
        }
    }

    // Default: Use prompt as statement with placeholder criteria
    let statement = prompt
        .lines()
        .next()
        .unwrap_or("Complete skill task")
        .trim()
        .to_string();

    (
        statement.clone(),
        SmartCriteria {
            specific: statement,
            measurable: "Task completed successfully".to_string(),
            achievable: true,
            relevant: format!("Required for {} skill execution", skill),
            time_bound: "This session".to_string(),
        },
    )
}

/// Parse a structured SMART line
fn parse_smart_line(line: &str, skill: &str) -> (String, SmartCriteria) {
    let mut specific = String::new();
    let mut measurable = String::new();
    let mut achievable = true;
    let mut relevant = String::new();
    let mut time_bound = String::new();

    // Parse S=... M=... A=... R=... T=...
    for part in line.split_whitespace() {
        if let Some(value) = part.strip_prefix("S=") {
            specific = value.replace('_', " ");
        } else if let Some(value) = part.strip_prefix("M=") {
            measurable = value.replace('_', " ");
        } else if let Some(value) = part.strip_prefix("A=") {
            achievable = value != "false";
        } else if let Some(value) = part.strip_prefix("R=") {
            relevant = value.replace('_', " ");
        } else if let Some(value) = part.strip_prefix("T=") {
            time_bound = value.replace('_', " ");
        }
    }

    // Fill defaults
    if specific.is_empty() {
        specific = format!("Complete {} task", skill);
    }
    if measurable.is_empty() {
        measurable = "Task completed successfully".to_string();
    }
    if relevant.is_empty() {
        relevant = format!("Required for {} skill", skill);
    }
    if time_bound.is_empty() {
        time_bound = "This session".to_string();
    }

    (
        specific.clone(),
        SmartCriteria {
            specific,
            measurable,
            achievable,
            relevant,
            time_bound,
        },
    )
}

/// Generate SMART goal prompt injection
fn smart_goal_prompt(skill: &str) -> String {
    format!(
        r#"🎯 **Aspiration + SMART Goal Required for `{skill}`**

## Step 1: Primitive Aspiration (Foundation)

First, define your aspiration using the **Value Equation**:

```
VALUE = ASYMMETRY × TIME × EXCHANGE
```

| Term | Question |
|------|----------|
| **ASYMMETRY** | What do you know that others don't? |
| **TIME** | When do you know it relative to others? |
| **EXCHANGE** | How does knowing convert to value? |

**Quick format:** `ASPIRATION: A=your_edge T=timing E=exchange_mechanism`

## Step 2: SMART Goal (Execution)

Then derive a specific goal:

| Criterion | Question |
|-----------|----------|
| **S**pecific | What exactly will you accomplish? |
| **M**easurable | How will you verify completion? |
| **A**chievable | Is this realistic given context? |
| **R**elevant | Why does this matter? |
| **T**ime-bound | When must this be done? |

**Quick format:** `SMART: S=goal M=metric A=true R=reason T=timeframe`

Or just describe what you want to achieve—I'll help structure it.

Once set, the Stop hook will remind you to complete the goal before ending."#
    )
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => {
            // No input - allow
            println!(r#"{{}}"#);
            process::exit(0);
        }
    };

    // Only fire on Skill tool
    if input.tool_name.as_deref() != Some("Skill") {
        println!(r#"{{}}"#);
        process::exit(0);
    }

    // Extract skill name from tool_input
    let skill_name = input
        .tool_input
        .as_ref()
        .and_then(|v| v.get("skill"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    // Skip SMART-dev itself (would be recursive)
    if skill_name.to_lowercase().contains("smart") {
        println!(r#"{{}}"#);
        process::exit(0);
    }

    // Check for existing active goal
    if let Some(goal) = load_active_goal() {
        if goal.status == GoalStatus::InProgress {
            // Already have an active goal - show reminder
            let context = format!(
                "🎯 **Active SMART Goal** for `{}`:\n\n**Goal:** {}\n**Measurable:** {}\n\nSay \"GOAL ACHIEVED\" when complete.",
                goal.skill, goal.statement, goal.smart_criteria.measurable
            );

            let output = HookOutput {
                hook_specific_output: Some(HookSpecificOutput::pre_tool_use_with_context(context)),
                ..Default::default()
            };
            output.emit();
            process::exit(0);
        }
    }

    // Check if user provided SMART goal in prompt/transcript
    let prompt = input.prompt.as_deref().unwrap_or("");
    let has_smart_goal = prompt.contains("SMART:") || prompt.contains("GOAL:");

    if has_smart_goal {
        // User provided goal - create and save it
        let goal = create_goal_from_prompt(skill_name, prompt);
        if let Err(e) = save_active_goal(&goal) {
            eprintln!("Warning: Failed to save goal: {e}");
        }

        let context = format!(
            "🎯 **SMART Goal Set** for `{}`:\n\n**Goal:** {}\n**Measurable:** {}\n\nSay \"GOAL ACHIEVED\" when complete.",
            goal.skill, goal.statement, goal.smart_criteria.measurable
        );

        let output = HookOutput {
            hook_specific_output: Some(HookSpecificOutput::pre_tool_use_with_context(context)),
            ..Default::default()
        };
        output.emit();
        process::exit(0);
    }

    // No goal - inject SMART goal prompt
    let context = smart_goal_prompt(skill_name);

    let output = HookOutput {
        hook_specific_output: Some(HookSpecificOutput::pre_tool_use_with_context(context)),
        ..Default::default()
    };
    output.emit();
    process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_smart_line() {
        let line = "SMART: S=Create_auth M=Tests_pass A=true R=Security T=Today";
        let (statement, criteria) = parse_smart_line(line, "test-skill");

        assert_eq!(statement, "Create auth");
        assert_eq!(criteria.measurable, "Tests pass");
        assert!(criteria.achievable);
        assert_eq!(criteria.relevant, "Security");
        assert_eq!(criteria.time_bound, "Today");
    }

    #[test]
    fn test_parse_smart_goal_default() {
        let prompt = "Create a new authentication module";
        let (statement, criteria) = parse_smart_goal(prompt, "skill-dev");

        assert_eq!(statement, "Create a new authentication module");
        assert!(criteria.achievable);
    }

    #[test]
    fn test_smart_goal_prompt_generation() {
        let prompt = smart_goal_prompt("skill-dev");
        assert!(prompt.contains("SMART Goal Required"));
        assert!(prompt.contains("skill-dev"));
    }
}
