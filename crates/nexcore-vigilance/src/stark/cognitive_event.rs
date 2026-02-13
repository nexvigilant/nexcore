//! # Cognitive Event Schema
//!
//! Unified system event schema inspired by the Allotrope Simple Model (ASM).
//! Standardizes diverse system outputs into a single 'Cognitive Event' format.
//!
//! ## Purpose
//!
//! While `AtomResult` in `stark/mod.rs` tracks verification operations,
//! `CognitiveEvent` tracks higher-level cognitive operations across
//! the entire system (ingestion, processing, validation, output).
//!
//! ## Example
//!
//! ```
//! use nexcore_vigilance::stark::cognitive_event::CognitiveEvent;
//! use serde_json::json;
//!
//! let event = CognitiveEvent::new(
//!     "orchestrator",
//!     "INGESTION_ACCEPT",
//!     json!({ "message_id": "msg-123" }),
//!     0.95,
//! );
//!
//! assert_eq!(event.source_module, "orchestrator");
//! assert_eq!(event.event_type, "INGESTION_ACCEPT");
//! ```

use chrono::{DateTime, Utc};
use nexcore_id::NexId;
use serde::{Deserialize, Serialize};

/// Unified System Event Schema (Inspired by Allotrope Simple Model - ASM)
///
/// Standardizes diverse system outputs into a single 'Cognitive Event' format.
/// Each event captures:
/// - Source module that generated it
/// - Event type classification
/// - Arbitrary JSON payload
/// - Cognitive power score at time of event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CognitiveEvent {
    /// Unique identifier for this event
    pub event_id: String,
    /// ISO 8601 timestamp
    pub timestamp: DateTime<Utc>,
    /// Module that generated this event
    pub source_module: String,
    /// Event type classification (e.g., "INGESTION", "COGNITION", "VERIFICATION")
    pub event_type: String,
    /// Arbitrary JSON payload
    pub data: serde_json::Value,
    /// Cognitive power score at time of event (0.0 - 1.0)
    pub power_score: f64,
}

impl CognitiveEvent {
    /// Create a new cognitive event.
    ///
    /// # Arguments
    ///
    /// * `source` - The module or component generating the event
    /// * `event_type` - Classification of the event (e.g., "INGESTION_ACCEPT")
    /// * `data` - Arbitrary JSON payload
    /// * `power` - Cognitive power score at time of event
    #[must_use]
    pub fn new(source: &str, event_type: &str, data: serde_json::Value, power: f64) -> Self {
        Self {
            event_id: NexId::v4().to_string(),
            timestamp: Utc::now(),
            source_module: source.to_string(),
            event_type: event_type.to_string(),
            data,
            power_score: power,
        }
    }

    /// Create an ingestion event.
    #[must_use]
    pub fn ingestion(source: &str, accepted: bool, data: serde_json::Value, power: f64) -> Self {
        let event_type = if accepted {
            "INGESTION_ACCEPT"
        } else {
            "INGESTION_SKIP"
        };
        Self::new(source, event_type, data, power)
    }

    /// Create a cognition gate event (pass or fail).
    #[must_use]
    pub fn cognition_gate(passed: bool, data: serde_json::Value, power: f64) -> Self {
        let event_type = if passed {
            "COGNITION_GATE_PASS"
        } else {
            "COGNITION_GATE_FAIL"
        };
        Self::new("power_analyzer", event_type, data, power)
    }

    /// Create a verification event.
    #[must_use]
    pub fn verification(source: &str, passed: bool, data: serde_json::Value, power: f64) -> Self {
        let event_type = if passed {
            "VERIFICATION_PASS"
        } else {
            "VERIFICATION_FAIL"
        };
        Self::new(source, event_type, data, power)
    }

    /// Check if this event represents a successful operation.
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.event_type.contains("ACCEPT")
            || self.event_type.contains("PASS")
            || self.event_type.contains("SUCCESS")
    }

    /// Check if this event represents a failure.
    #[must_use]
    pub fn is_failure(&self) -> bool {
        self.event_type.contains("FAIL")
            || self.event_type.contains("ERROR")
            || self.event_type.contains("REJECT")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_event_creation() {
        let event = CognitiveEvent::new(
            "orchestrator",
            "INGESTION_ACCEPT",
            json!({ "message_id": "msg-123" }),
            0.95,
        );

        assert!(!event.event_id.is_empty());
        assert_eq!(event.source_module, "orchestrator");
        assert_eq!(event.event_type, "INGESTION_ACCEPT");
        assert!((event.power_score - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ingestion_helper() {
        let event = CognitiveEvent::ingestion("processor", true, json!({ "count": 10 }), 0.85);
        assert_eq!(event.event_type, "INGESTION_ACCEPT");
        assert!(event.is_success());
    }

    #[test]
    fn test_cognition_gate() {
        let pass_event = CognitiveEvent::cognition_gate(true, json!({}), 0.9);
        assert_eq!(pass_event.event_type, "COGNITION_GATE_PASS");
        assert!(pass_event.is_success());

        let fail_event = CognitiveEvent::cognition_gate(false, json!({}), 0.5);
        assert_eq!(fail_event.event_type, "COGNITION_GATE_FAIL");
        assert!(fail_event.is_failure());
    }

    #[test]
    fn test_verification_event() {
        let event = CognitiveEvent::verification(
            "critic",
            false,
            json!({ "reason": "hallucination detected" }),
            0.7,
        );
        assert!(event.is_failure());
    }
}
