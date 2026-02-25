//! Network request collector with size-based eviction
//!
//! Collects network requests/responses with bounded total size.
//! When size limit is exceeded, oldest entries are evicted.

use nexcore_chrono::DateTime;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, OnceLock};

/// Maximum total size of network data to retain (10MB)
pub const MAX_NETWORK_SIZE: usize = 10 * 1024 * 1024;

/// Maximum number of network entries (regardless of size)
pub const MAX_NETWORK_ENTRIES: usize = 5000;

/// Global network collector instance
static NETWORK_COLLECTOR: OnceLock<Arc<NetworkCollector>> = OnceLock::new();

/// Get the global network collector
#[must_use]
pub fn get_network_collector() -> Arc<NetworkCollector> {
    NETWORK_COLLECTOR
        .get_or_init(|| Arc::new(NetworkCollector::new(MAX_NETWORK_SIZE, MAX_NETWORK_ENTRIES)))
        .clone()
}

/// Network request status
///
/// Tier: T2-P (Cross-domain primitive - HTTP status categories)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NetworkStatus {
    /// Request in progress
    Pending,
    /// Request completed successfully (2xx)
    Success,
    /// Request redirected (3xx)
    Redirect,
    /// Client error (4xx)
    ClientError,
    /// Server error (5xx)
    ServerError,
    /// Request failed (network error, timeout, etc.)
    Failed,
    /// Request cancelled
    Cancelled,
}

impl NetworkStatus {
    /// Create from HTTP status code
    #[must_use]
    pub fn from_status_code(code: u16) -> Self {
        match code {
            100..=199 => Self::Pending,
            200..=299 => Self::Success,
            300..=399 => Self::Redirect,
            400..=499 => Self::ClientError,
            500..=599 => Self::ServerError,
            _ => Self::Failed,
        }
    }

    /// Check if this is an error status
    #[must_use]
    pub fn is_error(&self) -> bool {
        matches!(self, Self::ClientError | Self::ServerError | Self::Failed)
    }

    /// Check if this is a failure (network-level)
    #[must_use]
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failed | Self::Cancelled)
    }
}

/// A collected network entry
///
/// Tier: T3 (Domain-specific network entry)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEntry {
    /// Request ID for correlation
    pub request_id: String,
    /// Request URL
    pub url: String,
    /// HTTP method
    pub method: String,
    /// Request status
    pub status: NetworkStatus,
    /// HTTP status code (if completed)
    pub status_code: Option<u16>,
    /// Response size in bytes
    pub response_size: u64,
    /// Request duration in milliseconds
    pub duration_ms: Option<u64>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Resource type (document, script, image, etc.)
    pub resource_type: Option<String>,
    /// Request start timestamp
    pub started_at: DateTime,
    /// Request completion timestamp
    pub completed_at: Option<DateTime>,
    /// Page ID
    pub page_id: String,
}

impl NetworkEntry {
    /// Estimate memory size of this entry
    #[must_use]
    pub fn estimated_size(&self) -> usize {
        self.request_id.len()
            + self.url.len()
            + self.method.len()
            + self.error.as_ref().map_or(0, String::len)
            + self.resource_type.as_ref().map_or(0, String::len)
            + self.page_id.len()
            + 64 // Fixed overhead for other fields
    }
}

/// Thread-safe network request collector with size-based eviction
pub struct NetworkCollector {
    /// Bounded entry queue
    entries: RwLock<VecDeque<NetworkEntry>>,
    /// Maximum total size in bytes
    max_size: usize,
    /// Maximum entry count
    max_entries: usize,
    /// Current total size estimate
    current_size: RwLock<usize>,
    /// Total requests received (including evicted)
    total_received: RwLock<u64>,
    /// Total requests evicted
    total_evicted: RwLock<u64>,
}

impl NetworkCollector {
    /// Create a new collector with specified limits
    #[must_use]
    pub fn new(max_size: usize, max_entries: usize) -> Self {
        Self {
            entries: RwLock::new(VecDeque::with_capacity(max_entries.min(1000))),
            max_size,
            max_entries,
            current_size: RwLock::new(0),
            total_received: RwLock::new(0),
            total_evicted: RwLock::new(0),
        }
    }

    /// Add a network entry (evicts oldest if at capacity)
    pub fn push(&self, entry: NetworkEntry) {
        let entry_size = entry.estimated_size();
        let mut entries = self.entries.write();
        let mut current_size = self.current_size.write();
        let mut received = self.total_received.write();
        *received += 1;

        // Evict entries until we have space
        while (*current_size + entry_size > self.max_size || entries.len() >= self.max_entries)
            && !entries.is_empty()
        {
            if let Some(removed) = entries.pop_front() {
                *current_size = current_size.saturating_sub(removed.estimated_size());
                let mut evicted = self.total_evicted.write();
                *evicted += 1;
            }
        }

        *current_size += entry_size;
        entries.push_back(entry);
    }

    /// Update an existing entry by request ID (for completion/failure)
    pub fn update(&self, request_id: &str, update_fn: impl FnOnce(&mut NetworkEntry)) {
        let mut entries = self.entries.write();
        if let Some(entry) = entries.iter_mut().find(|e| e.request_id == request_id) {
            let old_size = entry.estimated_size();
            update_fn(entry);
            let new_size = entry.estimated_size();

            // Adjust current size
            let mut current_size = self.current_size.write();
            *current_size = current_size.saturating_sub(old_size) + new_size;
        }
    }

    /// Get all entries (cloned)
    #[must_use]
    pub fn get_all(&self) -> Vec<NetworkEntry> {
        self.entries.read().iter().cloned().collect()
    }

    /// Get failures only
    #[must_use]
    pub fn get_failures(&self) -> Vec<NetworkEntry> {
        self.entries
            .read()
            .iter()
            .filter(|e| e.status.is_failure())
            .cloned()
            .collect()
    }

    /// Get errors (4xx, 5xx, failures)
    #[must_use]
    pub fn get_errors(&self) -> Vec<NetworkEntry> {
        self.entries
            .read()
            .iter()
            .filter(|e| e.status.is_error())
            .cloned()
            .collect()
    }

    /// Get entries for a specific page
    #[must_use]
    pub fn get_by_page(&self, page_id: &str) -> Vec<NetworkEntry> {
        self.entries
            .read()
            .iter()
            .filter(|e| e.page_id == page_id)
            .cloned()
            .collect()
    }

    /// Get the most recent N entries
    #[must_use]
    pub fn get_recent(&self, limit: usize) -> Vec<NetworkEntry> {
        let entries = self.entries.read();
        entries.iter().rev().take(limit).cloned().collect()
    }

    /// Get entry by request ID
    #[must_use]
    pub fn get_by_id(&self, request_id: &str) -> Option<NetworkEntry> {
        self.entries
            .read()
            .iter()
            .find(|e| e.request_id == request_id)
            .cloned()
    }

    /// Get current entry count
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    /// Check if collector is empty
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }

    /// Get failure count
    #[must_use]
    pub fn failure_count(&self) -> usize {
        self.entries
            .read()
            .iter()
            .filter(|e| e.status.is_failure())
            .count()
    }

    /// Get error count (including failures)
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.entries
            .read()
            .iter()
            .filter(|e| e.status.is_error())
            .count()
    }

    /// Calculate failure rate (failures / total)
    #[must_use]
    pub fn failure_rate(&self) -> f64 {
        let entries = self.entries.read();
        let total = entries.len();
        if total == 0 {
            return 0.0;
        }
        let failures = entries.iter().filter(|e| e.status.is_failure()).count();
        failures as f64 / total as f64
    }

    /// Get statistics
    #[must_use]
    pub fn stats(&self) -> NetworkStats {
        let entries = self.entries.read();
        NetworkStats {
            current_count: entries.len(),
            max_entries: self.max_entries,
            current_size: *self.current_size.read(),
            max_size: self.max_size,
            total_received: *self.total_received.read(),
            total_evicted: *self.total_evicted.read(),
            failure_count: entries.iter().filter(|e| e.status.is_failure()).count(),
            error_count: entries.iter().filter(|e| e.status.is_error()).count(),
            success_count: entries
                .iter()
                .filter(|e| e.status == NetworkStatus::Success)
                .count(),
        }
    }

    /// Clear all entries
    pub fn clear(&self) {
        self.entries.write().clear();
        *self.current_size.write() = 0;
    }
}

/// Network collector statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    /// Current entry count
    pub current_count: usize,
    /// Maximum entry count
    pub max_entries: usize,
    /// Current total size estimate (bytes)
    pub current_size: usize,
    /// Maximum size (bytes)
    pub max_size: usize,
    /// Total requests received (including evicted)
    pub total_received: u64,
    /// Total requests evicted
    pub total_evicted: u64,
    /// Current failure count
    pub failure_count: usize,
    /// Current error count (4xx, 5xx, failures)
    pub error_count: usize,
    /// Current success count
    pub success_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_status_from_code() {
        assert_eq!(NetworkStatus::from_status_code(200), NetworkStatus::Success);
        assert_eq!(
            NetworkStatus::from_status_code(404),
            NetworkStatus::ClientError
        );
        assert_eq!(
            NetworkStatus::from_status_code(500),
            NetworkStatus::ServerError
        );
        assert_eq!(
            NetworkStatus::from_status_code(301),
            NetworkStatus::Redirect
        );
    }

    #[test]
    fn test_collector_size_eviction() {
        // Small max size to trigger eviction
        let collector = NetworkCollector::new(500, 100);

        for i in 0..10 {
            collector.push(NetworkEntry {
                request_id: format!("req_{i}"),
                url: format!("https://example.com/{i}"),
                method: "GET".to_string(),
                status: NetworkStatus::Success,
                status_code: Some(200),
                response_size: 100,
                duration_ms: Some(50),
                error: None,
                resource_type: Some("document".to_string()),
                started_at: DateTime::now(),
                completed_at: Some(DateTime::now()),
                page_id: "page_1".to_string(),
            });
        }

        let stats = collector.stats();
        assert!(stats.current_size <= 500);
        assert!(stats.total_evicted > 0);
    }

    #[test]
    fn test_collector_update() {
        let collector = NetworkCollector::new(10000, 100);

        collector.push(NetworkEntry {
            request_id: "req_1".to_string(),
            url: "https://example.com".to_string(),
            method: "GET".to_string(),
            status: NetworkStatus::Pending,
            status_code: None,
            response_size: 0,
            duration_ms: None,
            error: None,
            resource_type: None,
            started_at: DateTime::now(),
            completed_at: None,
            page_id: "page_1".to_string(),
        });

        collector.update("req_1", |entry| {
            entry.status = NetworkStatus::Success;
            entry.status_code = Some(200);
            entry.response_size = 1024;
            entry.duration_ms = Some(100);
            entry.completed_at = Some(DateTime::now());
        });

        let entry = collector.get_by_id("req_1").expect("Entry should exist");
        assert_eq!(entry.status, NetworkStatus::Success);
        assert_eq!(entry.status_code, Some(200));
    }

    #[test]
    fn test_failure_rate() {
        let collector = NetworkCollector::new(10000, 100);

        // 3 successes, 2 failures
        for i in 0..3 {
            collector.push(NetworkEntry {
                request_id: format!("success_{i}"),
                url: "https://example.com".to_string(),
                method: "GET".to_string(),
                status: NetworkStatus::Success,
                status_code: Some(200),
                response_size: 100,
                duration_ms: Some(50),
                error: None,
                resource_type: None,
                started_at: DateTime::now(),
                completed_at: Some(DateTime::now()),
                page_id: "page_1".to_string(),
            });
        }

        for i in 0..2 {
            collector.push(NetworkEntry {
                request_id: format!("failure_{i}"),
                url: "https://example.com".to_string(),
                method: "GET".to_string(),
                status: NetworkStatus::Failed,
                status_code: None,
                response_size: 0,
                duration_ms: None,
                error: Some("Connection refused".to_string()),
                resource_type: None,
                started_at: DateTime::now(),
                completed_at: None,
                page_id: "page_1".to_string(),
            });
        }

        let rate = collector.failure_rate();
        assert!((rate - 0.4).abs() < 0.001); // 2/5 = 0.4
    }
}
