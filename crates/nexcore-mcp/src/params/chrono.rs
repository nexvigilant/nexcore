//! Chrono MCP Parameters
//!
//! Parameter structs for the sovereign datetime engine tools.

use rmcp::schemars::{self, JsonSchema};
use rmcp::serde::{Deserialize, Serialize};

/// Parameters for `chrono_parse` — parse a datetime string.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChronoParseParams {
    /// Datetime string to parse. Accepts RFC 3339 (e.g. "2026-02-25T14:30:00Z"),
    /// ISO 8601 date (e.g. "2026-02-25"), or naive datetime with an explicit format.
    pub input: String,
    /// Optional strftime format string for naive datetimes (e.g. "%Y-%m-%d %H:%M:%S").
    /// When omitted, RFC 3339 and ISO 8601 date are tried automatically.
    pub format: Option<String>,
}

/// Parameters for `chrono_format` — format a Unix timestamp using a strftime pattern.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChronoFormatParams {
    /// Unix timestamp in seconds. Defaults to the current UTC time when omitted.
    pub timestamp: Option<i64>,
    /// strftime format string.
    /// Supported specifiers: `%Y` (year), `%m` (month), `%d` (day),
    /// `%H` (hour), `%M` (minute), `%S` (second), `%F` (YYYY-MM-DD),
    /// `%T` (HH:MM:SS), `%%` (literal %).
    pub format: String,
}

/// Parameters for `chrono_duration` — create and inspect a duration.
#[derive(Debug, Deserialize, Serialize, JsonSchema)]
#[serde(crate = "rmcp::serde")]
pub struct ChronoDurationParams {
    /// Numeric value of the duration.
    pub value: i64,
    /// Unit of the duration value.
    /// Accepted values: "seconds", "minutes", "hours", "days", "weeks".
    pub unit: String,
}
