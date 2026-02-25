//! Console message collector with FIFO eviction
//!
//! Collects console messages (log, warn, error, etc.) with a bounded size.
//! When capacity is exceeded, oldest messages are evicted.

use nexcore_chrono::DateTime;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, OnceLock};

/// Maximum number of console messages to retain
pub const MAX_CONSOLE_MESSAGES: usize = 1000;

/// Global console collector instance
static CONSOLE_COLLECTOR: OnceLock<Arc<ConsoleCollector>> = OnceLock::new();

/// Get the global console collector
#[must_use]
pub fn get_console_collector() -> Arc<ConsoleCollector> {
    CONSOLE_COLLECTOR
        .get_or_init(|| Arc::new(ConsoleCollector::new(MAX_CONSOLE_MESSAGES)))
        .clone()
}

/// Console message level
///
/// Tier: T2-P (Cross-domain primitive - log levels are universal)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConsoleLevel {
    /// Standard log output.
    Log,
    /// Debug-level output.
    Debug,
    /// Informational message.
    Info,
    /// Warning message.
    Warning,
    /// Error message.
    Error,
    /// Assertion failure.
    Assert,
    /// `console.dir` output.
    Dir,
    /// `console.dirxml` output.
    DirXml,
    /// Tabular data.
    Table,
    /// Stack trace.
    Trace,
    /// Console cleared.
    Clear,
    /// Group start.
    StartGroup,
    /// Collapsed group start.
    StartGroupCollapsed,
    /// Group end.
    EndGroup,
    /// Counter output.
    Count,
    /// Timer end.
    TimeEnd,
    /// CPU profile start.
    Profile,
    /// CPU profile end.
    ProfileEnd,
}

impl ConsoleLevel {
    /// Parse from CDP string format.
    #[must_use]
    pub fn parse_cdp(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "log" => Self::Log,
            "debug" => Self::Debug,
            "info" => Self::Info,
            "warning" | "warn" => Self::Warning,
            "error" => Self::Error,
            "assert" => Self::Assert,
            "dir" => Self::Dir,
            "dirxml" => Self::DirXml,
            "table" => Self::Table,
            "trace" => Self::Trace,
            "clear" => Self::Clear,
            "startgroup" => Self::StartGroup,
            "startgroupcollapsed" => Self::StartGroupCollapsed,
            "endgroup" => Self::EndGroup,
            "count" => Self::Count,
            "timeend" => Self::TimeEnd,
            "profile" => Self::Profile,
            "profileend" => Self::ProfileEnd,
            _ => Self::Log,
        }
    }

    /// Check if this is an error-level message
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error | Self::Assert)
    }

    /// Check if this is a warning-level message
    #[must_use]
    pub fn is_warning(&self) -> bool {
        matches!(self, Self::Warning)
    }
}

/// A collected console entry
///
/// Tier: T3 (Domain-specific console entry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEntry {
    /// Message level
    pub level: ConsoleLevel,
    /// Message text
    pub text: String,
    /// Source URL
    pub url: Option<String>,
    /// Line number
    pub line: Option<u32>,
    /// Column number
    pub column: Option<u32>,
    /// Collection timestamp
    pub timestamp: DateTime,
    /// Page ID that emitted this message
    pub page_id: String,
}

/// Thread-safe console message collector with FIFO eviction
pub struct ConsoleCollector {
    /// Bounded message queue
    messages: RwLock<VecDeque<ConsoleEntry>>,
    /// Maximum capacity
    capacity: usize,
    /// Total messages received (including evicted)
    total_received: RwLock<u64>,
    /// Total messages evicted
    total_evicted: RwLock<u64>,
}

impl ConsoleCollector {
    /// Create a new collector with specified capacity
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            messages: RwLock::new(VecDeque::with_capacity(capacity)),
            capacity,
            total_received: RwLock::new(0),
            total_evicted: RwLock::new(0),
        }
    }

    /// Add a console entry (evicts oldest if at capacity)
    pub fn push(&self, entry: ConsoleEntry) {
        let mut messages = self.messages.write();
        let mut received = self.total_received.write();
        *received += 1;

        if messages.len() >= self.capacity {
            messages.pop_front();
            let mut evicted = self.total_evicted.write();
            *evicted += 1;
        }

        messages.push_back(entry);
    }

    /// Get all messages (cloned)
    #[must_use]
    pub fn get_all(&self) -> Vec<ConsoleEntry> {
        self.messages.read().iter().cloned().collect()
    }

    /// Get messages filtered by level
    #[must_use]
    pub fn get_by_level(&self, level: ConsoleLevel) -> Vec<ConsoleEntry> {
        self.messages
            .read()
            .iter()
            .filter(|e| e.level == level)
            .cloned()
            .collect()
    }

    /// Get error messages only
    #[must_use]
    pub fn get_errors(&self) -> Vec<ConsoleEntry> {
        self.messages
            .read()
            .iter()
            .filter(|e| e.level.is_error())
            .cloned()
            .collect()
    }

    /// Get warning messages only
    #[must_use]
    pub fn get_warnings(&self) -> Vec<ConsoleEntry> {
        self.messages
            .read()
            .iter()
            .filter(|e| e.level.is_warning())
            .cloned()
            .collect()
    }

    /// Get messages for a specific page
    #[must_use]
    pub fn get_by_page(&self, page_id: &str) -> Vec<ConsoleEntry> {
        self.messages
            .read()
            .iter()
            .filter(|e| e.page_id == page_id)
            .cloned()
            .collect()
    }

    /// Get the most recent N messages
    #[must_use]
    pub fn get_recent(&self, limit: usize) -> Vec<ConsoleEntry> {
        let messages = self.messages.read();
        messages.iter().rev().take(limit).cloned().collect()
    }

    /// Get current message count
    #[must_use]
    pub fn len(&self) -> usize {
        self.messages.read().len()
    }

    /// Check if collector is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.messages.read().is_empty()
    }

    /// Get error count
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.messages
            .read()
            .iter()
            .filter(|e| e.level.is_error())
            .count()
    }

    /// Get warning count
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.messages
            .read()
            .iter()
            .filter(|e| e.level.is_warning())
            .count()
    }

    /// Get statistics
    #[must_use]
    pub fn stats(&self) -> ConsoleStats {
        let messages = self.messages.read();
        ConsoleStats {
            current_count: messages.len(),
            capacity: self.capacity,
            total_received: *self.total_received.read(),
            total_evicted: *self.total_evicted.read(),
            error_count: messages.iter().filter(|e| e.level.is_error()).count(),
            warning_count: messages.iter().filter(|e| e.level.is_warning()).count(),
        }
    }

    /// Clear all messages
    pub fn clear(&self) {
        self.messages.write().clear();
    }
}

/// Console collector statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleStats {
    /// Current message count
    pub current_count: usize,
    /// Maximum capacity
    pub capacity: usize,
    /// Total messages received (including evicted)
    pub total_received: u64,
    /// Total messages evicted due to capacity
    pub total_evicted: u64,
    /// Current error count
    pub error_count: usize,
    /// Current warning count
    pub warning_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_level_from_str() {
        assert_eq!(ConsoleLevel::parse_cdp("error"), ConsoleLevel::Error);
        assert_eq!(ConsoleLevel::parse_cdp("WARNING"), ConsoleLevel::Warning);
        assert_eq!(ConsoleLevel::parse_cdp("warn"), ConsoleLevel::Warning);
        assert_eq!(ConsoleLevel::parse_cdp("unknown"), ConsoleLevel::Log);
    }

    #[test]
    fn test_collector_fifo_eviction() {
        let collector = ConsoleCollector::new(3);

        for i in 0..5 {
            collector.push(ConsoleEntry {
                level: ConsoleLevel::Log,
                text: format!("Message {i}"),
                url: None,
                line: None,
                column: None,
                timestamp: DateTime::now(),
                page_id: "page_1".to_string(),
            });
        }

        let messages = collector.get_all();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].text, "Message 2"); // Oldest remaining
        assert_eq!(messages[2].text, "Message 4"); // Newest

        let stats = collector.stats();
        assert_eq!(stats.total_received, 5);
        assert_eq!(stats.total_evicted, 2);
    }

    #[test]
    fn test_collector_filter_by_level() {
        let collector = ConsoleCollector::new(10);

        collector.push(ConsoleEntry {
            level: ConsoleLevel::Error,
            text: "Error 1".to_string(),
            url: None,
            line: None,
            column: None,
            timestamp: DateTime::now(),
            page_id: "page_1".to_string(),
        });

        collector.push(ConsoleEntry {
            level: ConsoleLevel::Warning,
            text: "Warning 1".to_string(),
            url: None,
            line: None,
            column: None,
            timestamp: DateTime::now(),
            page_id: "page_1".to_string(),
        });

        collector.push(ConsoleEntry {
            level: ConsoleLevel::Log,
            text: "Log 1".to_string(),
            url: None,
            line: None,
            column: None,
            timestamp: DateTime::now(),
            page_id: "page_1".to_string(),
        });

        assert_eq!(collector.get_errors().len(), 1);
        assert_eq!(collector.get_warnings().len(), 1);
        assert_eq!(collector.error_count(), 1);
        assert_eq!(collector.warning_count(), 1);
    }
}
