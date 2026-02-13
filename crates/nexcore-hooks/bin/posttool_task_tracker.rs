//! Task Progress Tracker - PostToolUse Event
//!
//! Tracks task state changes and progress after TaskUpdate completes.
//! Persists task metrics to enable completion auditing.
//!
//! Tracks:
//! - Tasks transitioned to in_progress
//! - Tasks marked completed
//! - Task lifecycle events
//!
//! Output: Update ~/.claude/task_state.json with task metrics

use nexcore_hooks::state::now;
use nexcore_hooks::{exit_success_auto, read_input};
use std::fs;
use std::path::PathBuf;

fn task_state_file() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
        .join("task_state.json")
}

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
struct TaskState {
    tasks_created: u32,
    tasks_completed: u32,
    tasks_in_progress: u32,
    tasks_deleted: u32,
    last_updated: f64,
    /// Task IDs that are currently in progress
    active_task_ids: Vec<String>,
}

impl TaskState {
    fn load() -> Self {
        let path = task_state_file();
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(state) = serde_json::from_str(&content) {
                    return state;
                }
            }
        }
        Self::default()
    }

    fn save(&mut self) {
        self.last_updated = now();
        let path = task_state_file();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, content);
        }
    }

    fn record_created(&mut self, task_id: Option<&str>) {
        self.tasks_created += 1;
        if let Some(id) = task_id {
            if !self.active_task_ids.contains(&id.to_string()) {
                // New task starts as pending, not in active list yet
            }
        }
    }

    fn record_in_progress(&mut self, task_id: Option<&str>) {
        self.tasks_in_progress += 1;
        if let Some(id) = task_id {
            if !self.active_task_ids.contains(&id.to_string()) {
                self.active_task_ids.push(id.to_string());
            }
        }
    }

    fn record_completed(&mut self, task_id: Option<&str>) {
        self.tasks_completed += 1;
        if let Some(id) = task_id {
            self.active_task_ids.retain(|i| i != id);
        }
    }

    fn record_deleted(&mut self, task_id: Option<&str>) {
        self.tasks_deleted += 1;
        if let Some(id) = task_id {
            self.active_task_ids.retain(|i| i != id);
        }
    }
}

fn main() {
    let input = match read_input() {
        Some(i) => i,
        None => exit_success_auto(),
    };

    // Only process TaskUpdate and TaskCreate
    let tool_name = input.tool_name.as_deref().unwrap_or("");
    if tool_name != "TaskUpdate" && tool_name != "TaskCreate" {
        exit_success_auto();
    }

    let mut state = TaskState::load();

    // Get tool input for task details
    let tool_input = match &input.tool_input {
        Some(i) => i,
        None => {
            state.save();
            exit_success_auto();
        }
    };

    let task_id = tool_input.get("taskId").and_then(|v| v.as_str());

    if tool_name == "TaskCreate" {
        state.record_created(task_id);
    } else if tool_name == "TaskUpdate" {
        // Check what status transition happened
        if let Some(status) = tool_input.get("status").and_then(|v| v.as_str()) {
            match status {
                "in_progress" => state.record_in_progress(task_id),
                "completed" => state.record_completed(task_id),
                "deleted" => state.record_deleted(task_id),
                _ => {}
            }
        }
    }

    state.save();
    exit_success_auto();
}
