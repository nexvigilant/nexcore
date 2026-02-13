use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Task/event processing urgency. Ascending: Low=0 < Critical=3.
///
/// Previously named `Priority` with **inverted** ordinals (Critical=0, Low=3).
/// F2 equivocation fix: renamed to `Urgency` with ascending encoding.
/// Heap consumers should wrap in `std::cmp::Reverse<Urgency>` for min-heap.
///
/// Tier: T2-P (κ + N — comparison/ordering)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum Urgency {
    /// Background work, no urgency.
    Low = 0,
    /// Standard processing (default).
    Normal = 1,
    /// Elevated urgency.
    High = 2,
    /// Immediate attention required.
    Critical = 3,
}

/// Backward-compatible alias.
#[deprecated(note = "use Urgency — ascending ordinal, F2 equivocation fix")]
pub type Priority = Urgency;

/// FRIDAY decision engine response action.
///
/// Previously named `Action` — renamed to `DecisionAction` to disambiguate
/// from Action types in other crates (PvAction, FirewallRule, FileOp, ResponseAction).
///
/// Tier: T2-P (ς + ∂ — state transition with boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DecisionAction {
    /// Send context to LLM for conversational response.
    InvokeClaude,
    /// Return a predefined quick response.
    QuickResponse,
    /// Log the event without visible action.
    SilentLog,
    /// Perform an autonomous task defined in authority.
    AutonomousAct,
    /// Escalate directly to the user.
    Escalate,
}

/// Backward-compatible alias.
#[deprecated(note = "use DecisionAction — F2 equivocation fix")]
pub type Action = DecisionAction;

/// Types of executors available to handle actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutorType {
    Claude,
    Shell,
    Speech,
    Notify,
    Mcp,
    Http,
    Browser,
    Maestro,
}

/// A discrete event flowing through the FRIDAY system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique identifier for the event.
    pub id: NexId,
    /// The component that originated the event (e.g., "voice", "git").
    pub source: String,
    /// Specific category of event (e.g., "user_spoke", "file_changed").
    pub event_type: String,
    /// Structured data associated with the event.
    pub payload: serde_json::Value,
    /// Processing urgency of the event.
    pub priority: Urgency,
    /// Time when the event was generated.
    pub timestamp: DateTime<Utc>,
    /// Optional ID to track related events.
    pub correlation_id: Option<String>,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            id: NexId::v4(),
            source: String::new(),
            event_type: String::new(),
            payload: serde_json::Value::Object(serde_json::Map::new()),
            priority: Urgency::Normal,
            timestamp: Utc::now(),
            correlation_id: None,
        }
    }
}

// Implement ordering for PriorityQueue (BinaryHeap is a max-heap; reverse urgency for FIFO-by-urgency)
impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.timestamp == other.timestamp
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher urgency (higher value) should come first in max-heap.
        // Reverse comparison so Critical(3) > Low(0) yields earlier processing.
        self.priority
            .cmp(&other.priority)
            .then_with(|| other.timestamp.cmp(&self.timestamp))
    }
}

/// Record of an interaction with the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interaction {
    pub id: String,
    pub event: Event,
    pub prompt: String,
    pub response: String,
    pub timestamp: DateTime<Utc>,
    pub tokens_used: i32,
    pub contains_learning: bool,
    pub actions_taken: Vec<String>,
}

/// Result returned after an executor processes an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorResult {
    pub executor: ExecutorType,
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}
