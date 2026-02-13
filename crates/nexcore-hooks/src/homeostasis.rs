//! Homeostasis Module - Guardian-Inspired Control Loop for Hooks
//!
//! Provides reusable components for implementing the SENSING → DECISION → RESPONSE
//! feedback loop pattern in hooks, based on the Guardian immune-system model.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
//! │   SENSOR    │───▶│  DECISION   │───▶│  RESPONDER  │
//! │ (Detect     │    │  ENGINE     │    │ (Generate   │
//! │  signals)   │    │ (Evaluate)  │    │  output)    │
//! └─────────────┘    └─────────────┘    └─────────────┘
//!       ▲                                      │
//!       │           FEEDBACK LOOP              │
//!       └──────────────────────────────────────┘
//!              (State persistence)
//! ```
//!
//! # Signal Types (PAMP/DAMP Classification)
//!
//! - **PAMP (Pathogen-Associated)**: External quality gaps (missing docs, old versions)
//! - **DAMP (Damage-Associated)**: Internal deficiencies (no tests, poor coverage)
//!
//! # Usage
//!
//! ```rust,ignore
//! use nexcore_hooks::homeostasis::{Signal, SignalType, StateTracker, Severity};
//!
//! // Create a signal for missing test coverage
//! let signal = Signal::new(
//!     "missing_test",
//!     SignalType::Damp,
//!     Severity::Warning,
//!     "No test coverage for src/foo.rs",
//! );
//!
//! // Use state tracker to avoid duplicate warnings
//! let mut tracker = StateTracker::load("test_enforcer");
//! if !tracker.has_seen("src/foo.rs") {
//!     tracker.mark_seen("src/foo.rs");
//!     // Emit warning
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Signal types based on Guardian's PAMP/DAMP classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalType {
    /// Pathogen-Associated Molecular Pattern - External threats/gaps
    Pamp,
    /// Damage-Associated Molecular Pattern - Internal deficiencies
    Damp,
    /// Hybrid - Both external and internal factors
    Hybrid,
}

/// Severity levels for signals
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// Informational - no action needed
    Info,
    /// Warning - proceed with caution
    Warning,
    /// Error - should be addressed
    Error,
    /// Critical - must be addressed before proceeding
    Critical,
}

impl Severity {
    /// Convert to exit code
    pub fn exit_code(&self) -> i32 {
        match self {
            Severity::Info => 0,
            Severity::Warning => 1,
            Severity::Error => 1,
            Severity::Critical => 2,
        }
    }
}

/// A detected signal from a sensor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    /// Signal identifier
    pub id: String,
    /// Signal type (PAMP/DAMP/Hybrid)
    pub signal_type: SignalType,
    /// Severity level
    pub severity: Severity,
    /// Human-readable description
    pub description: String,
    /// Target (file, capability, etc.) that triggered the signal
    pub target: Option<String>,
    /// Timestamp when signal was detected
    pub detected_at: u64,
    /// Suggested action to resolve
    pub suggested_action: Option<String>,
}

impl Signal {
    /// Create a new signal
    pub fn new(id: &str, signal_type: SignalType, severity: Severity, description: &str) -> Self {
        Self {
            id: id.to_string(),
            signal_type,
            severity,
            description: description.to_string(),
            target: None,
            detected_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            suggested_action: None,
        }
    }

    /// Set the target that triggered this signal
    pub fn with_target(mut self, target: &str) -> Self {
        self.target = Some(target.to_string());
        self
    }

    /// Set a suggested action
    pub fn with_action(mut self, action: &str) -> Self {
        self.suggested_action = Some(action.to_string());
        self
    }

    /// Check if this signal should block execution
    pub fn is_blocking(&self) -> bool {
        self.severity == Severity::Critical
    }
}

/// Persistent state tracker for feedback loop
///
/// Tracks which targets have been seen/warned about to avoid
/// duplicate notifications within a session.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct StateTracker {
    /// Tracker name (used for state file naming)
    #[serde(skip)]
    name: String,
    /// Targets that have been seen/warned
    seen_targets: HashSet<String>,
    /// Session ID for resetting between sessions
    session_id: Option<String>,
    /// Last update timestamp
    last_updated: u64,
}

impl StateTracker {
    /// Create a new state tracker with the given name
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Load state from persistent storage
    pub fn load(name: &str) -> Self {
        let path = Self::state_path(name);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut state) = serde_json::from_str::<StateTracker>(&content) {
                    state.name = name.to_string();
                    return state;
                }
            }
        }
        Self::new(name)
    }

    /// Get the state file path for a tracker
    fn state_path(name: &str) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join(".claude/state")
            .join(format!("{}_state.json", name))
    }

    /// Save state to persistent storage (best-effort)
    pub fn save(&self) {
        let path = Self::state_path(&self.name);
        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                return;
            }
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if fs::write(&path, json).is_err() {
                eprintln!("⚠️  Failed to save {} state", self.name);
            }
        }
    }

    /// Check if a target has been seen
    pub fn has_seen(&self, target: &str) -> bool {
        self.seen_targets.contains(target)
    }

    /// Mark a target as seen and persist
    pub fn mark_seen(&mut self, target: &str) {
        self.seen_targets.insert(target.to_string());
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.save();
    }

    /// Clear all seen targets (useful for new sessions)
    pub fn reset(&mut self) {
        self.seen_targets.clear();
        self.save();
    }

    /// Number of targets seen
    pub fn seen_count(&self) -> usize {
        self.seen_targets.len()
    }
}

/// Priority queue for fair round-robin selection
///
/// Maintains a queue of items sorted by selection count to ensure
/// even distribution of selections over time.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FairQueue<T: Clone + Serialize> {
    /// Queue name for persistence
    #[serde(skip)]
    name: String,
    /// Items with their selection counts
    items: Vec<QueueEntry<T>>,
    /// Last update timestamp
    last_updated: u64,
    /// TTL in seconds for queue refresh
    ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEntry<T> {
    /// The item
    pub item: T,
    /// Number of times selected
    pub times_selected: u32,
}

impl<T: Clone + Serialize + for<'de> Deserialize<'de>> FairQueue<T> {
    /// Create a new fair queue
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            items: Vec::new(),
            last_updated: 0,
            ttl_seconds: 3600, // 1 hour default
        }
    }

    /// Set TTL for queue refresh
    pub fn with_ttl(mut self, seconds: u64) -> Self {
        self.ttl_seconds = seconds;
        self
    }

    /// Load queue from persistent storage
    pub fn load(name: &str) -> Self {
        let path = Self::queue_path(name);
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(mut queue) = serde_json::from_str::<FairQueue<T>>(&content) {
                    queue.name = name.to_string();

                    // Check TTL
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0);
                    if now - queue.last_updated < queue.ttl_seconds {
                        return queue;
                    }
                }
            }
        }
        Self::new(name)
    }

    fn queue_path(name: &str) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        PathBuf::from(home)
            .join(".claude/state")
            .join(format!("{}_queue.json", name))
    }

    /// Save queue to persistent storage
    pub fn save(&self) {
        let path = Self::queue_path(&self.name);
        if let Some(parent) = path.parent() {
            if fs::create_dir_all(parent).is_err() {
                return;
            }
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            if fs::write(&path, json).is_err() {
                eprintln!("⚠️  Failed to save {} queue", self.name);
            }
        }
    }

    /// Add an item to the queue
    pub fn push(&mut self, item: T) {
        self.items.push(QueueEntry {
            item,
            times_selected: 0,
        });
    }

    /// Clear and rebuild queue with new items
    pub fn rebuild(&mut self, items: Vec<T>) {
        self.items = items
            .into_iter()
            .map(|item| QueueEntry {
                item,
                times_selected: 0,
            })
            .collect();
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    /// Get the next item with the lowest selection count (fair round-robin)
    pub fn next(&mut self) -> Option<&T> {
        if self.items.is_empty() {
            return None;
        }

        // Sort by times_selected (ascending)
        self.items.sort_by_key(|e| e.times_selected);

        // Increment count for the first item
        self.items[0].times_selected += 1;
        self.last_updated = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.save();

        Some(&self.items[0].item)
    }

    /// Get number of items in queue
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Check if queue is empty
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signal_creation() {
        let signal = Signal::new("test", SignalType::Pamp, Severity::Warning, "Test signal")
            .with_target("src/foo.rs")
            .with_action("Add tests");

        assert_eq!(signal.id, "test");
        assert_eq!(signal.signal_type, SignalType::Pamp);
        assert_eq!(signal.severity, Severity::Warning);
        assert_eq!(signal.target, Some("src/foo.rs".to_string()));
        assert!(!signal.is_blocking());
    }

    #[test]
    fn test_severity_exit_codes() {
        assert_eq!(Severity::Info.exit_code(), 0);
        assert_eq!(Severity::Warning.exit_code(), 1);
        assert_eq!(Severity::Error.exit_code(), 1);
        assert_eq!(Severity::Critical.exit_code(), 2);
    }

    #[test]
    fn test_state_tracker() {
        let mut tracker = StateTracker::new("test");
        assert!(!tracker.has_seen("foo"));
        tracker.mark_seen("foo");
        assert!(tracker.has_seen("foo"));
        assert!(!tracker.has_seen("bar"));
        assert_eq!(tracker.seen_count(), 1);
    }

    #[test]
    fn test_fair_queue() {
        let mut queue: FairQueue<String> = FairQueue::new("test");
        queue.rebuild(vec!["a".into(), "b".into(), "c".into()]);

        // First selection should get lowest count item
        let first = queue.next().cloned();
        assert!(first.is_some());

        // Select two more items to complete a round
        let second = queue.next().cloned();
        let third = queue.next().cloned();
        assert!(second.is_some());
        assert!(third.is_some());

        // Verify fairness: max difference in selection counts should be <= 1
        let counts: Vec<u32> = queue.items.iter().map(|e| e.times_selected).collect();
        let min = *counts.iter().min().unwrap();
        let max = *counts.iter().max().unwrap();
        assert!(max - min <= 1, "Fair queue should maintain balanced counts");
    }
}
