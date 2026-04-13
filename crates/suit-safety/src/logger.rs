//! Safety event logging for audit trail.

/// A safety event record.
#[derive(Debug, Clone)]
pub struct SafetyEvent {
    /// Event timestamp (Unix millis).
    pub timestamp_ms: u64,
    /// Event severity level.
    pub severity: Severity,
    /// Event description.
    pub message: String,
}

/// Severity levels for safety events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// Informational.
    Info,
    /// Warning — action may be needed.
    Warning,
    /// Critical — immediate action required.
    Critical,
}
