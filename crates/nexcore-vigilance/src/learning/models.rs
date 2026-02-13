//! Core domain models for teaching ecosystem.

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of events that can be observed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationEventType {
    SessionStarted,
    SessionCompleted,
    SessionPaused,
    SessionResumed,
    TodoAdded,
    TodoUpdated,
    ToolCalled,
    ToolResult,
    SubagentStarted,
    SubagentCompleted,
    ResponseGenerated,
    ErrorOccurred,
}

/// Flags for categorizing observations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationFlag {
    DecisionPoint,
    ToolSelection,
    ErrorRecovery,
    SuccessIndicator,
    PatternCandidate,
    Deviation,
    Excellent,
    NeedsAttention,
}

/// An observation event captured during agent execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Observation {
    pub id: NexId,
    pub timestamp: DateTime<Utc>,
    pub event_type: ObservationEventType,
    pub agent_id: String,
    pub session_id: String,
    pub data: serde_json::Value,
    pub flags: Vec<ObservationFlag>,
}

/// Type of coaching intervention.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InterventionType {
    /// Provide guidance/suggestion
    Guidance,
    /// Correct an error
    Correction,
    /// Encourage good behavior
    Encouragement,
    /// Stop execution (critical issue)
    Halt,
}

/// Andon signal for quality control (Toyota Production System).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AndonSignal {
    /// All good
    Green,
    /// Warning
    Yellow,
    /// Stop
    Red,
}

/// A coaching intervention.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intervention {
    pub id: NexId,
    pub timestamp: DateTime<Utc>,
    pub trigger: String,
    #[serde(rename = "type")]
    pub intervention_type: InterventionType,
    pub signal: AndonSignal,
    pub message: String,
    pub context: serde_json::Value,
    pub student_response: Option<String>,
    pub effective: Option<bool>,
}

/// A todo/task item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoItem {
    pub id: String,
    pub content: String,
    pub active_form: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// An agent learning session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub agent_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
    pub todos: Vec<TodoItem>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Configuration for teaching behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeachingConfig {
    pub skills_dir: String,
    pub teaching_dir: String,
    pub min_observations_for_extraction: u32,
    pub confidence_threshold: f64,
    pub enable_interventions: bool,
    pub intervention_threshold: f64,
}

impl Default for TeachingConfig {
    fn default() -> Self {
        Self {
            skills_dir: "~/.claude/skills".into(),
            teaching_dir: "~/.claude/teaching".into(),
            min_observations_for_extraction: 3,
            confidence_threshold: 0.8,
            enable_interventions: true,
            intervention_threshold: 0.5,
        }
    }
}
