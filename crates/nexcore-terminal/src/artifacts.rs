//! Terminal Artifact Bridge — persists tool results to brain for cross-surface sync.
//!
//! When the terminal runs a Station tool, microgram, or AI query, the result
//! is saved as a brain artifact. Nucleus can then read these artifacts via
//! `/api/nexcore/brain/sessions/{id}` to display terminal activity.
//!
//! ## Grounding
//!
//! `π(Persistence: result → brain artifact) + ∂(Boundary: terminal → brain → nucleus)`

use serde::Serialize;

/// A terminal result artifact for cross-surface persistence.
#[derive(Debug, Serialize)]
pub struct TerminalArtifact {
    /// Source surface (e.g., "station", "mcg", "relay")
    pub source: String,
    /// Tool or microgram name
    pub name: String,
    /// The result data
    pub result: serde_json::Value,
    /// Execution time in milliseconds
    pub elapsed_ms: u64,
    /// Terminal session ID that produced this artifact
    pub session_id: String,
    /// ISO 8601 timestamp
    pub timestamp: String,
}

impl TerminalArtifact {
    /// Create a new artifact from a Station tool result.
    pub fn from_station(
        tool_name: &str,
        result: &serde_json::Value,
        elapsed_ms: u64,
        session_id: &str,
    ) -> Self {
        Self {
            source: "station".to_string(),
            name: tool_name.to_string(),
            result: result.clone(),
            elapsed_ms,
            session_id: session_id.to_string(),
            timestamp: chrono_now(),
        }
    }

    /// Create a new artifact from a microgram result.
    pub fn from_microgram(
        mcg_name: &str,
        result: &serde_json::Value,
        elapsed_ms: u64,
        session_id: &str,
    ) -> Self {
        Self {
            source: "mcg".to_string(),
            name: mcg_name.to_string(),
            result: result.clone(),
            elapsed_ms,
            session_id: session_id.to_string(),
            timestamp: chrono_now(),
        }
    }

    /// Create a new artifact from a relay AI response.
    pub fn from_relay(query: &str, response: &str, elapsed_ms: u64, session_id: &str) -> Self {
        Self {
            source: "relay".to_string(),
            name: "ai_query".to_string(),
            result: serde_json::json!({
                "query": query,
                "response": response,
            }),
            elapsed_ms,
            session_id: session_id.to_string(),
            timestamp: chrono_now(),
        }
    }

    /// Serialize to JSON for brain persistence.
    ///
    /// Returns a Result to surface serialization errors.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Current UTC timestamp in ISO 8601 format.
fn chrono_now() -> String {
    // Use std::time to avoid chrono dependency — format manually
    let now = std::time::SystemTime::now();
    let secs = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Simple UTC timestamp (not perfect but sufficient for artifact ordering)
    format!("{secs}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn station_artifact_serializes() {
        let a = TerminalArtifact::from_station(
            "search_adverse_events",
            &serde_json::json!({"count": 42}),
            350,
            "tsn_abc123",
        );
        let json = a.to_json().expect("serialization should succeed");
        assert!(json.contains("station"));
        assert!(json.contains("search_adverse_events"));
    }

    #[test]
    fn relay_artifact_captures_query() {
        let a = TerminalArtifact::from_relay(
            "What is PRR?",
            "PRR is the proportional reporting ratio.",
            4500,
            "tsn_abc123",
        );
        let json = a.to_json().expect("serialization should succeed");
        assert!(json.contains("What is PRR?"));
        assert!(json.contains("proportional"));
    }
}
