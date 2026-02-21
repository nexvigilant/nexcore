//! Common types shared across all domains

use serde::{Deserialize, Serialize};

/// Pagination parameters for list endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u32,
    pub total_pages: u32,
}

/// Generic API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pagination: Option<Pagination>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Firestore-compatible timestamp (seconds + nanoseconds)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timestamp {
    pub seconds: i64,
    pub nanoseconds: u32,
}

impl Timestamp {
    /// Convert to milliseconds since epoch
    pub fn to_millis(&self) -> i64 {
        self.seconds * 1000 + (self.nanoseconds / 1_000_000) as i64
    }

    /// Create from chrono UTC datetime
    pub fn now() -> Self {
        let now = chrono::Utc::now();
        Self {
            seconds: now.timestamp(),
            nanoseconds: now.timestamp_subsec_nanos(),
        }
    }
}

/// Sort direction for queries
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    Asc,
    Desc,
}

/// Content status used across multiple domains
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ContentStatus {
    Draft,
    Review,
    Published,
    Archived,
}
