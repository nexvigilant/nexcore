//! Browser events for Vigil integration
//!
//! Defines the `BrowserEvent` enum that bridges CDP events to Vigil's EventBus.
//! Events are broadcast via `tokio::sync::broadcast` for fan-out to multiple subscribers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Browser events for Vigil integration.
///
/// Tier: T3 (Domain-specific browser events)
/// Grounds to: T1 primitives via String/u64/bool variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BrowserEvent {
    /// Console message from browser DevTools
    ConsoleMessage {
        /// Message level (log, warn, error, etc.)
        level: String,
        /// Message text
        text: String,
        /// Source URL where message originated
        url: Option<String>,
        /// Line number in source
        line: Option<u32>,
        /// Column number in source
        column: Option<u32>,
        /// Timestamp
        timestamp: DateTime<Utc>,
        /// Page ID that emitted this message
        page_id: String,
    },

    /// Network request failure
    NetworkFailure {
        /// Request URL
        url: String,
        /// HTTP method
        method: String,
        /// Error reason
        error: String,
        /// Request ID for correlation
        request_id: String,
        /// Timestamp
        timestamp: DateTime<Utc>,
        /// Page ID
        page_id: String,
    },

    /// Network request completed
    NetworkComplete {
        /// Request URL
        url: String,
        /// HTTP method
        method: String,
        /// HTTP status code
        status: u16,
        /// Response size in bytes
        size: u64,
        /// Request duration in milliseconds
        duration_ms: u64,
        /// Request ID
        request_id: String,
        /// Timestamp
        timestamp: DateTime<Utc>,
        /// Page ID
        page_id: String,
    },

    /// Page loaded event
    PageLoaded {
        /// Page URL
        url: String,
        /// Page title
        title: Option<String>,
        /// Load time in milliseconds
        load_time_ms: Option<u64>,
        /// Page ID
        page_id: String,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },

    /// Performance violation detected
    PerformanceViolation {
        /// Violation type (long-task, layout-shift, etc.)
        violation_type: String,
        /// Duration or magnitude
        value: f64,
        /// Threshold that was exceeded
        threshold: f64,
        /// Page ID
        page_id: String,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },

    /// Browser disconnected
    BrowserDisconnected {
        /// Reason for disconnection
        reason: String,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },

    /// Page crashed
    PageCrashed {
        /// Page ID that crashed
        page_id: String,
        /// Error description
        error: Option<String>,
        /// Timestamp
        timestamp: DateTime<Utc>,
    },
}

impl BrowserEvent {
    /// Get the event type as a string (for Vigil event_type field)
    #[must_use]
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::ConsoleMessage { level, .. } => match level.as_str() {
                "error" => "console_error",
                "warning" | "warn" => "console_warning",
                _ => "console_message",
            },
            Self::NetworkFailure { .. } => "network_failure",
            Self::NetworkComplete { .. } => "network_complete",
            Self::PageLoaded { .. } => "page_loaded",
            Self::PerformanceViolation { .. } => "performance_violation",
            Self::BrowserDisconnected { .. } => "browser_disconnected",
            Self::PageCrashed { .. } => "page_crashed",
        }
    }

    /// Get the page ID associated with this event (if any)
    #[must_use]
    pub fn page_id(&self) -> Option<&str> {
        match self {
            Self::ConsoleMessage { page_id, .. }
            | Self::NetworkFailure { page_id, .. }
            | Self::NetworkComplete { page_id, .. }
            | Self::PageLoaded { page_id, .. }
            | Self::PerformanceViolation { page_id, .. }
            | Self::PageCrashed { page_id, .. } => Some(page_id),
            Self::BrowserDisconnected { .. } => None,
        }
    }

    /// Determine priority for Vigil (Critical=0, High=1, Normal=2, Low=3)
    #[must_use]
    pub fn priority(&self) -> u8 {
        match self {
            Self::ConsoleMessage { level, .. } if level == "error" => 1, // High
            Self::ConsoleMessage { level, .. } if level == "warning" || level == "warn" => 2, // Normal
            Self::ConsoleMessage { .. } => 3,                                                 // Low
            Self::NetworkFailure { .. } => 1,       // High
            Self::NetworkComplete { .. } => 3,      // Low
            Self::PageLoaded { .. } => 3,           // Low
            Self::PerformanceViolation { .. } => 2, // Normal
            Self::BrowserDisconnected { .. } => 1,  // High
            Self::PageCrashed { .. } => 0,          // Critical
        }
    }

    /// Get timestamp of the event
    #[must_use]
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            Self::ConsoleMessage { timestamp, .. }
            | Self::NetworkFailure { timestamp, .. }
            | Self::NetworkComplete { timestamp, .. }
            | Self::PageLoaded { timestamp, .. }
            | Self::PerformanceViolation { timestamp, .. }
            | Self::BrowserDisconnected { timestamp, .. }
            | Self::PageCrashed { timestamp, .. } => *timestamp,
        }
    }
}
